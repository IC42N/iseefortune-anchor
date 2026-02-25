use anchor_lang::prelude::*;
use crate::constants::{FEE_BPS_DENOM, RESOLVED_GAME_VERSION};
use crate::errors::IC42NErrorCode;
use crate::state::{Config, GameStatus, LiveFeed, ResolvedGame, RolloverReason};
use crate::state::treasury::Treasury;
use crate::utils::resolve::{get_next_rollover_number, next_fee_bps_on_rollover};

#[derive(Accounts)]
#[instruction(epoch: u64, tier: u8)]
pub struct ResolvedGameRollover<'info> {

    #[account(
        mut,
        seeds = [Config::SEED],
        bump = config.bump,
        has_one = authority @ IC42NErrorCode::Unauthorized
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [LiveFeed::SEED_PREFIX, &[tier]],
        bump = live_feed.bump,
    )]
    pub live_feed: Account<'info, LiveFeed>,

    #[account(
        init,
        payer = authority,
        space = 8 + ResolvedGame::SIZE,
        seeds = [ResolvedGame::SEED_PREFIX, epoch.to_le_bytes().as_ref(), &[tier]],
        bump
    )]
    pub resolved_game: Account<'info, ResolvedGame>,

    #[account(
        mut,
        seeds = [Treasury::SEED],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,


    #[account(mut, address = config.authority @ IC42NErrorCode::Unauthorized)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}


pub fn complete_rollover_game_handler(
    ctx: Context<ResolvedGameRollover>,
    epoch: u64,
    tier: u8,
    winning_number: u8,
    rng_epoch_slot_used: u64,
    rng_blockhash_used: [u8; 32],
) -> Result<()> {
    let config    = &mut ctx.accounts.config;
    let live      = &mut ctx.accounts.live_feed;
    let treasury  = &mut ctx.accounts.treasury;
    let game      = &mut ctx.accounts.resolved_game;


    // Confirm this game is a rollover
    // It can happen if
    // - The winning number is 0 or the secondary rollover number
    // - There are no winners.
    let w = winning_number as usize;
    require!(w < 10, IC42NErrorCode::InvalidWinningNumber);

    let is_rollover_number = winning_number == 0 || winning_number == live.secondary_rollover_number;
    let has_winners = live.bets_per_number[w] > 0;
    require!(
        is_rollover_number || !has_winners,
        IC42NErrorCode::CarryNotAllowed
    );

    let rollover_reason = if is_rollover_number {
        RolloverReason::RolloverNumber
    } else {
        // since require! ensures (is_rollover_number || !has_winners),
        // and we're in the else branch => must be !has_winners
        RolloverReason::NoWinners
    };


    let clock         = Clock::get()?;
    let current_epoch = clock.epoch;
    let resolved_ts   = clock.unix_timestamp;


    // Must have something to roll over
    require!(live.total_bets > 0, IC42NErrorCode::NoBetsToResolve);
    require!(live.total_lamports > 0, IC42NErrorCode::NoBetsToResolve);

    // Epoch/tier alignment
    require_eq!(live.epoch, epoch, IC42NErrorCode::EpochMismatch);
    require!(live.epoch < current_epoch, IC42NErrorCode::EpochNotComplete);
    require_eq!(live.tier, tier, IC42NErrorCode::TierMismatch);

    let tier_cfg = config.get_tier_settings(tier)?;
    require!(tier_cfg.is_active(), IC42NErrorCode::InactiveTier);


    // Gross pot is everything in live.total_lamports.
    // In rollover: no fee, full pot carries forward.
    let gross_pot = live.total_lamports;
    let fee_bps   = live.current_fee_bps as u64;

    let fee = gross_pot
        .checked_mul(fee_bps)
        .ok_or(IC42NErrorCode::MathOverflow)?
        / FEE_BPS_DENOM;

    let expected_net = gross_pot
        .checked_sub(fee)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    // Treasury must be able to cover the full pot.
    let treasury_balance = **treasury.to_account_info().lamports.borrow();
    require!(
        treasury_balance >= expected_net,
        IC42NErrorCode::InsufficientTreasuryBalance
    );

    // Full gross pot gets carried over
    let carry_over_lamports_for_next = gross_pot;
    let carry_over_bets_for_next     = live.total_bets;
    let carry_over_bets_per_number   = live.bets_per_number;
    let carry_over_lamports_per_number = live.lamports_per_number;

    // Populate ResolvedGame snapshot for this epoch (no winners)
    game.epoch = epoch;
    game.first_epoch_in_chain = live.first_epoch_in_chain;
    game.tier = tier;
    game.bump = ctx.bumps.resolved_game;
    
    game.winning_number = winning_number;
    game.rng_epoch_slot_used = rng_epoch_slot_used;
    game.rng_blockhash_used = rng_blockhash_used;
    
    game.status = GameStatus::Resolved as u8;
    game.attempt_count = 1;
    game.last_updated_slot = clock.slot;
    game.last_updated_ts   = resolved_ts;

    game.carry_over_bets       = carry_over_bets_for_next;
    game.total_bets            = live.total_bets;
    game.carry_in_lamports  = live.carried_over_lamports;
    game.carry_out_lamports = carry_over_lamports_for_next;
    game.protocol_fee_lamports = fee;
    game.fee_bps = live.current_fee_bps;
    game.net_prize_pool        = expected_net;
    game.total_winners   = 0;
    game.claimed_winners = 0;
    game.resolved_at      = resolved_ts;
    
    game.merkle_root = [0u8; 32];
    game.results_uri = [0u8; 128]; // no Arweave needed for simple rollover
    
    game.claimed_bitmap = Vec::new();
    game.version = RESOLVED_GAME_VERSION;
    game.claimed_lamports = 0;
    game.rollover_reason = rollover_reason.as_u8();
    game.secondary_rollover_number = live.secondary_rollover_number;
    game._reserved = [0u8; 12];


    // If the winning number is 0 or is the current secondary rollover number,
    // then we keep the same rollover number. Else, we use the winning number as the new rollover number.
    let next_secondary_rollover: u8 = get_next_rollover_number(winning_number,live.secondary_rollover_number);

    // The fee only decreases on rollover-number carry
    let next_fee_bps = if is_rollover_number {
        next_fee_bps_on_rollover(
            live.current_fee_bps,
            config.rollover_fee_step_bps,
            config.min_fee_bps,
        )
    } else {
        // no-winners carry: keep the current fee (but still enforce >= min)
        live.current_fee_bps.max(config.min_fee_bps)
    };

    // Reset LiveFeed for the next epoch using your existing helper.
    let next_epoch = live.epoch + 1;
    live.reset_for_new_epoch(
        next_epoch,
        config.bet_cutoff_slots,
        carry_over_lamports_for_next,
        carry_over_bets_for_next,
        carry_over_lamports_per_number,
        carry_over_bets_per_number,
        next_secondary_rollover,
        next_fee_bps
    );

    Ok(())
}