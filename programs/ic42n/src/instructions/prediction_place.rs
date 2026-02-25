use anchor_lang::prelude::*;

use crate::constants::RECENT_BETS_CAP;
use crate::errors::IC42NErrorCode;
use crate::state::*;
use crate::state::player_profile::PlayerProfile;
use crate::state::treasury::Treasury;
use crate::utils::betting::{is_amount_in_tier, is_betting_still_open};
use crate::utils::prediction::derive_prediction_selections;
use crate::utils::transfers::transfer_lamports;

#[derive(Accounts)]
#[instruction(tier: u8, prediction_type: u8, choice: u32, lamports: u64)]
pub struct PlacePrediction<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [LiveFeed::SEED_PREFIX, &[tier]],
        bump = live_feed.bump,
    )]
    pub live_feed: Box<Account<'info, LiveFeed>>,

    #[account(
        init_if_needed,
        payer = player,
        space = 8 + Prediction::SIZE,
        seeds = [
            Prediction::SEED_PREFIX,
            player.key().as_ref(),
            &live_feed.first_epoch_in_chain.to_le_bytes(),
            &[tier],
        ],
        bump,
    )]
    pub prediction: Box<Account<'info, Prediction>>,

    #[account(
        init_if_needed,
        payer = player,
        space = 8 + PlayerProfile::SIZE,
        seeds = [PlayerProfile::SEED_PREFIX, player.key().as_ref()],
        bump,
        constraint = profile.player == Pubkey::default()
            || profile.player == player.key() @ IC42NErrorCode::Unauthorized
    )]
    pub profile: Box<Account<'info, PlayerProfile>>,

    #[account(
        mut,
        seeds = [Treasury::SEED],
        bump = treasury.bump,
        constraint = treasury.key() == live_feed.treasury @ IC42NErrorCode::TreasuryMismatch
    )]
    pub treasury: Box<Account<'info, Treasury>>,

    #[account(
      seeds = [Config::SEED],
      bump = config.bump,
    )]
    pub config: Box<Account<'info, Config>>,

    pub system_program: Program<'info, System>,
}

pub fn place_prediction_handler(
    ctx: Context<PlacePrediction>,
    tier: u8,
    prediction_type: u8,
    choice: u32,
    lamports: u64, // per-number lamports
) -> Result<()> {
    let pred = &mut ctx.accounts.prediction;
    let live = &mut ctx.accounts.live_feed;
    let config = &ctx.accounts.config;
    let profile = &mut ctx.accounts.profile;
    let treasury = &mut ctx.accounts.treasury;
    let player = &ctx.accounts.player;

    let clock = Clock::get()?;

    // ─────────────────────────────
    // Basic validations
    // ─────────────────────────────
    require!(config.pause_bet == 0, IC42NErrorCode::BettingPaused);
    require!(lamports > 0, IC42NErrorCode::InvalidBetAmount);

    require!(clock.epoch == live.epoch, IC42NErrorCode::EpochMismatch);
    require!(live.tier == tier, IC42NErrorCode::TierMismatch);

    require!(
        is_betting_still_open(live.bet_cutoff_slots),
        IC42NErrorCode::BettingClosed
    );

    // ─────────────────────────────
    // Derive selections internally
    // ─────────────────────────────
    let blocked = live.secondary_rollover_number;

    let (selection_count, selections, selections_mask) =
        derive_prediction_selections(prediction_type, choice, blocked)?;

    let k = selection_count as u64;
    require!(k > 0, IC42NErrorCode::InvalidChoiceCount);

    // ─────────────────────────────
    // Enforce per-tier min/max (per-number)
    // ─────────────────────────────
    let tier_settings = config.get_tier_settings(tier)?;
    require!(tier_settings.is_active(), IC42NErrorCode::InactiveTier);
    require!(
        is_amount_in_tier(lamports, &tier_settings),
        IC42NErrorCode::BetOutOfTierRange
    );

    // total exposure = per-number * selection_count
    let total_lamports = lamports
        .checked_mul(k)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    // ─────────────────────────────
    // Initialize / hydrate PlayerProfile
    // ─────────────────────────────
    if profile.player == Pubkey::default() {
        profile.player = player.key();
        profile.bump = ctx.bumps.profile;

        profile.total_bets = 0;
        profile.total_lamports_wagered = 0;
        profile.last_played_epoch = 0;
        profile.last_played_tier = 0;
        profile.last_played_timestamp = 0;
        profile.xp_points = 0;

        profile.recent_bets = [Pubkey::default(); RECENT_BETS_CAP];
        profile.recent_bets_len = 0;
        profile.recent_bets_head = 0;
        profile.tickets_available = 1;
        profile._reserved = [0u8; 16];
    }

    // ─────────────────────────────
    // Enforce one prediction per game chain
    // ─────────────────────────────
    let game_epoch = live.first_epoch_in_chain;

    if pred.player != Pubkey::default() {
        return err!(IC42NErrorCode::AlreadyBetThisGame);
    }

    // ─────────────────────────────
    // Initialize Prediction
    // ─────────────────────────────
    pred.game_epoch = game_epoch;
    pred.epoch = clock.epoch;
    pred.player = player.key();
    pred.tier = tier;

    pred.prediction_type = prediction_type;
    pred.selection_count = selection_count;
    pred.selections = selections;
    pred.selections_mask = selections_mask;

    // totals + per-number
    pred.lamports = total_lamports;
    pred.lamports_per_number = lamports;

    pred.changed_count = 0;

    pred.placed_slot = clock.slot;
    pred.placed_at_ts = clock.unix_timestamp;
    pred.last_updated_at_ts = clock.unix_timestamp;

    pred.has_claimed = 0;
    pred.claimed_at_ts = 0;

    pred.bump = ctx.bumps.prediction;
    pred.version = Prediction::VERSION;

    pred._reserved = [0u8; 8];

    // Extend the profile deletion lock: always push it forward, never shorten it
    let new_until = clock.epoch.saturating_add(2);
    profile.locked_until_epoch = profile.locked_until_epoch.max(new_until);

    // Store in profile recent bets ring buffer
    let pred_pk = pred.key();
    profile.push_recent_bet(pred_pk);

    // ─────────────────────────────
    // Update live feed stats
    // ─────────────────────────────
    live.total_bets = live
        .total_bets
        .checked_add(1)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    // TOTAL exposure
    live.total_lamports = live
        .total_lamports
        .checked_add(total_lamports)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    // Per-number stats: each selected number gets full per-number lamports (no split)
    for i in 0..(selection_count as usize) {
        let n = selections[i] as usize;

        require!(
            n < live.bets_per_number.len() && n < live.lamports_per_number.len(),
            IC42NErrorCode::InvalidBetNumber
        );

        live.bets_per_number[n] = live.bets_per_number[n]
            .checked_add(1)
            .ok_or(IC42NErrorCode::MathOverflow)?;

        live.lamports_per_number[n] = live.lamports_per_number[n]
            .checked_add(lamports)
            .ok_or(IC42NErrorCode::MathOverflow)?;
    }

    // ─────────────────────────────
    // Update treasury stats (TOTAL)
    // ─────────────────────────────
    treasury.total_in_lamports = treasury
        .total_in_lamports
        .checked_add(total_lamports)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    // ─────────────────────────────
    // Transfer lamports player → treasury (TOTAL)
    // ─────────────────────────────
    transfer_lamports(
        &player.to_account_info(),
        &treasury.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        total_lamports,
    )?;

    // ─────────────────────────────
    // Update PlayerProfile stats (TOTAL)
    // ─────────────────────────────
    profile.total_bets = profile.total_bets.saturating_add(1);
    profile.total_lamports_wagered = profile
        .total_lamports_wagered
        .saturating_add(total_lamports);

    profile.last_played_epoch = clock.epoch;
    profile.last_played_tier = tier;
    profile.last_played_timestamp = clock.unix_timestamp;

    profile.xp_points = profile.xp_points.saturating_add(1);

    if profile.first_played_epoch == 0 {
        profile.first_played_epoch = live.first_epoch_in_chain;
    }
    
    pred.assert_invariant()?;

    Ok(())
}