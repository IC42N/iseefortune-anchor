use anchor_lang::prelude::*;

use crate::errors::IC42NErrorCode;
use crate::state::*;
use crate::state::player_profile::PlayerProfile;
use crate::state::treasury::Treasury;
use crate::utils::prediction::apply_per_number_to_live;
use crate::utils::betting::{is_amount_in_tier, is_betting_still_open};
use crate::utils::transfers::transfer_lamports;

#[derive(Accounts)]
#[instruction(tier: u8, additional_lamports: u64)]
pub struct IncreasePrediction<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [LiveFeed::SEED_PREFIX, &[tier]],
        bump = live_feed.bump,
    )]
    pub live_feed: Account<'info, LiveFeed>,

    #[account(
        mut,
        seeds = [
            Prediction::SEED_PREFIX,
            player.key().as_ref(),
            &live_feed.first_epoch_in_chain.to_le_bytes(),
            &[tier],
        ],
        bump,
        has_one = player @ IC42NErrorCode::Unauthorized,
    )]
    pub prediction: Account<'info, Prediction>,

    #[account(
        mut,
        seeds = [PlayerProfile::SEED_PREFIX, player.key().as_ref()],
        bump,
        constraint = profile.player == player.key() @ IC42NErrorCode::Unauthorized
    )]
    pub profile: Box<Account<'info, PlayerProfile>>,

    #[account(
      seeds = [Config::SEED],
      bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
      mut,
      seeds = [Treasury::SEED],
      bump = treasury.bump,
      constraint = treasury.key() == live_feed.treasury @ IC42NErrorCode::TreasuryMismatch
    )]
    pub treasury: Account<'info, Treasury>,

    pub system_program: Program<'info, System>,
}

pub fn increase_prediction_handler(
    ctx: Context<IncreasePrediction>,
    tier: u8,
    additional_lamports: u64, // Additional per-number lamports
    choice: u32,
) -> Result<()> {
    let pred = &mut ctx.accounts.prediction;
    let live = &mut ctx.accounts.live_feed;
    let config = &ctx.accounts.config;
    let player = &ctx.accounts.player;
    let treasury = &mut ctx.accounts.treasury;
    
    pred.assert_invariant()?;

    require!(config.pause_bet == 0, IC42NErrorCode::BettingPaused);
    require!(pred.has_claimed == 0, IC42NErrorCode::AlreadyClaimed);
    require!(additional_lamports > 0, IC42NErrorCode::InvalidBetAmount);
    require!(choice > 0, IC42NErrorCode::InvalidBetNumber);

    let clock = Clock::get()?;
    let current_epoch = clock.epoch;

    // ─────────────────────────────
    // Epoch / chain checks
    // ─────────────────────────────
    require!(current_epoch == live.epoch, IC42NErrorCode::EpochMismatch);

    require_eq!(
        pred.game_epoch,
        live.first_epoch_in_chain,
        IC42NErrorCode::EpochMismatch
    );

    require!(
        pred.epoch >= live.first_epoch_in_chain && pred.epoch <= live.epoch,
        IC42NErrorCode::EpochMismatch
    );

    require_eq!(pred.tier, tier, IC42NErrorCode::TierMismatch);
    require_eq!(live.tier, tier, IC42NErrorCode::TierMismatch);

    // ─────────────────────────────
    // Cutoff & limits
    // ─────────────────────────────
    require!(
        is_betting_still_open(live.bet_cutoff_slots),
        IC42NErrorCode::BettingClosed
    );

    require_keys_eq!(live.treasury, treasury.key(), IC42NErrorCode::TreasuryMismatch);

    let tier_settings = config.get_tier_settings(tier)?;
    require!(tier_settings.is_active(), IC42NErrorCode::InactiveTier);

    // ─────────────────────────────
    // Selection invariants
    // ─────────────────────────────
    let k_u8 = pred.selection_count;
    require!(k_u8 >= 1 && k_u8 <= 8, IC42NErrorCode::InvalidBetNumber);
    let k = k_u8 as u64;

    // Sanity: selections_mask must match active selections
    let mut recomputed: u16 = 0;
    for i in 0..(k_u8 as usize) {
        let v = pred.selections[i];
        require!(v >= 1 && v <= 9, IC42NErrorCode::InvalidBetNumber);

        let n = v as usize;
        require!(
            n < live.lamports_per_number.len() && n < live.bets_per_number.len(),
            IC42NErrorCode::InvalidBetNumber
        );

        recomputed |= 1u16 << v;
    }
    require!(recomputed == pred.selections_mask, IC42NErrorCode::InvalidBetNumber);

    // ─────────────────────────────
    // Compute new per-number + totals
    // ─────────────────────────────
    let new_per_number = pred
        .lamports_per_number
        .checked_add(additional_lamports)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    // tier range applies to PER NUMBER
    require!(
        is_amount_in_tier(new_per_number, &tier_settings),
        IC42NErrorCode::BetOutOfTierRange
    );

    let additional_total = additional_lamports
        .checked_mul(k)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    let new_total = pred
        .lamports
        .checked_add(additional_total)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    // ─────────────────────────────
    // Update Prediction
    // ─────────────────────────────
    pred.lamports_per_number = new_per_number;
    pred.lamports = new_total;

    pred.changed_count = pred.changed_count.saturating_add(1);
    pred.last_updated_at_ts = clock.unix_timestamp;
    
    
    // ─────────────────────────────
    // Update Profile
    // ─────────────────────────────
    let profile = &mut ctx.accounts.profile;
    profile.total_lamports_wagered = profile.total_lamports_wagered.saturating_add(additional_total);
    
    // ─────────────────────────────
    // Update live feed stats (deltas)
    // ─────────────────────────────
    apply_per_number_to_live(
        live,
        additional_lamports,
        &pred.selections,
        pred.selection_count,
    )?;

    // Total pot increases by additional_total
    live.total_lamports = live
        .total_lamports
        .checked_add(additional_total)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    // Treasury increases by additional_total
    treasury.total_in_lamports = treasury
        .total_in_lamports
        .checked_add(additional_total)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    // ─────────────────────────────
    // Transfer extra lamports player → treasury (TOTAL delta)
    // ─────────────────────────────
    transfer_lamports(
        &player.to_account_info(),
        &treasury.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        additional_total,
    )?;

    require!(
        pred.lamports == pred.expected_total_lamports(),
        IC42NErrorCode::InvalidBetAmount
    );
    
    Ok(())
}