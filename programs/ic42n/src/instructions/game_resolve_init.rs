use anchor_lang::prelude::*;
use crate::errors::IC42NErrorCode;
use crate::state::*;
use crate::constants::*;

// -----------------------------------------------------------------------------
// InitResolvedGame
//
// First step in the resolution pipeline. This:
//   - Ensures the epoch is over
//   - Ensures LiveFeed matches (epoch, tier)
//   - Ensures tier is active
//   - Creates the ResolvedGame PDA and sets status = Processing
//
// Called once per (epoch, tier) after the epoch ends, typically by your
// cron/worker when it detects a new epoch that needs resolution.
// -----------------------------------------------------------------------------
#[derive(Accounts)]
#[instruction(epoch: u64, tier: u8)]
pub struct InitResolvedGame<'info> {

    #[account(
        seeds = [Config::SEED],
        bump = config.bump,
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

    #[account(mut, address = config.authority @ IC42NErrorCode::Unauthorized)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}


pub fn init_resolved_game_handler(
    ctx: Context<InitResolvedGame>,
    epoch: u64,
    tier: u8,
    winning_number: u8,
    rng_epoch_slot_used: u64,
    rng_blockhash_used: [u8; 32],
) -> Result<()> {
    let config = &ctx.accounts.config;
    let live   = &ctx.accounts.live_feed;
    let game   = &mut ctx.accounts.resolved_game;

    let clock = Clock::get()?;
    let current_epoch = clock.epoch;

    // ─────────────────────────────────────────────────────────────
    // 1) Basic validation: epoch/tier match and epoch is complete
    // ─────────────────────────────────────────────────────────────

    // LiveFeed must be for the epoch we’re initializing
    require_eq!(live.epoch, epoch, IC42NErrorCode::EpochMismatch);

    // Epoch must already be completed
    require!(live.epoch < current_epoch, IC42NErrorCode::EpochNotComplete);

    // Tier consistency
    require_eq!(live.tier, tier, IC42NErrorCode::TierMismatch);

    // Tier must be valid + active in Config
    let tier_cfg = config.get_tier_settings(tier)?;
    require!(tier_cfg.is_active(), IC42NErrorCode::InactiveTier);

    // The winning number must be valid
    require!(winning_number <= 9, IC42NErrorCode::InvalidWinningNumber);

    // There must be bets to init this game
    require!(live.total_bets > 0 && live.total_lamports > 0, IC42NErrorCode::NoBetsToResolve);

    // ─────────────────────────────────────────────────────────────
    // 2) Initialize ResolvedGame identity + state-machine fields
    // ─────────────────────────────────────────────────────────────

    game.epoch = epoch;
    game.tier  = tier;
    game.bump  = ctx.bumps.resolved_game;

    game.winning_number = winning_number;
    game.rng_epoch_slot_used = rng_epoch_slot_used;
    game.rng_blockhash_used = rng_blockhash_used;

    // Start in Processing – locked by a worker
    game.status            = GameStatus::Processing as u8;
    game.attempt_count     = 1;
    game.last_updated_slot = clock.slot;
    game.last_updated_ts   = clock.unix_timestamp;

    // Pot/accounting fields will be filled in `resolve_game_handler,
    // but we can already record what came *into* this epoch.
    game.carry_over_bets      = 0; // or live.carried_over_bets if you track that
    game.total_bets           = 0; // the final total will be set during resolve
    game.carry_in_lamports    = live.carried_over_lamports;
    game.carry_out_lamports   = 0;
    game.protocol_fee_lamports = 0;
    game.fee_bps               = 0;
    game.net_prize_pool       = 0;
    game.total_winners        = 0;
    game.claimed_winners      = 0;
    game.resolved_at          = 0;

    // Merkle / results – unknown at init
    game.merkle_root  = [0u8; 32];
    game.results_uri  = [0u8; 128];

    game.claimed_bitmap = Vec::new();

    game.version  = RESOLVED_GAME_VERSION;
    game.claimed_lamports = 0;
    game.first_epoch_in_chain = live.first_epoch_in_chain;
    game.rollover_reason = RolloverReason::None.as_u8();
    game.secondary_rollover_number = live.secondary_rollover_number;
    game._reserved = [0u8; 12];
    Ok(())
}