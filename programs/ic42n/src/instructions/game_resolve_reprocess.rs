use anchor_lang::prelude::*;
use crate::errors::IC42NErrorCode;
use crate::state::*;

/// ---------------------------------------------------------------------------
/// BeginResolveGame
///
/// This is called by your worker (Lambda) **before** running the heavy
/// resolution logic (JSON, Merkle, uploads, etc.).
///
/// Responsibilities:
///   - Ensure LiveFeed still matches (epoch, tier) and epoch is over
///   - Ensure tier is still active
///   - Ensure a ResolvedGame PDA already exists for (epoch, tier)
///   - Ensure its status is Pending/Failed (i.e. not already Resolving/Resolved)
///   - Flip state -> Resolving, increment attempt_count, update timestamps
///
/// After this:
///   - Worker can safely do off-chain work
///   - Then call `resolve_game` to finalize + write results.
/// ---------------------------------------------------------------------------
#[derive(Accounts)]
#[instruction(epoch: u64, tier: u8)]
pub struct ReprocessResolveGame<'info> {
    /// Global config (authority, tiers, fee_bps, etc.)
    #[account(
        mut,
        seeds = [Config::SEED],
        bump = config.bump,
        has_one = authority @ IC42NErrorCode::Unauthorized
    )]
    pub config: Account<'info, Config>,

    /// Live feed for this tier (must match epoch & tier)
    #[account(
        mut,
        seeds = [LiveFeed::SEED_PREFIX, &[tier]],
        bump = live_feed.bump,
    )]
    pub live_feed: Account<'info, LiveFeed>,

    /// ResolvedGame PDA for this epoch & tier – must already exist
    #[account(
        mut,
        seeds = [ResolvedGame::SEED_PREFIX, epoch.to_le_bytes().as_ref(), &[tier]],
        bump = resolved_game.bump
    )]
    pub resolved_game: Account<'info, ResolvedGame>,

    /// Authority allowed to kick off resolution for this program
    #[account(mut, address = config.authority @ IC42NErrorCode::Unauthorized)]
    pub authority: Signer<'info>,
}


pub fn reprocessing_resolve_game_handler(
    ctx: Context<ReprocessResolveGame>,
    epoch: u64,
    tier: u8,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let live   = &ctx.accounts.live_feed;
    let game   = &mut ctx.accounts.resolved_game;

    let clock = Clock::get()?;
    let current_epoch = clock.epoch;

    // ─────────────────────────────────────────────
    // 1) Validate epoch + tier + status sanity
    // ─────────────────────────────────────────────

    // LiveFeed must be for the epoch we’re resolving
    require_eq!(live.epoch, epoch, IC42NErrorCode::EpochMismatch);

    // Epoch must already be completed
    require!(live.epoch < current_epoch, IC42NErrorCode::EpochNotComplete);

    // Tier consistency
    require_eq!(live.tier, tier, IC42NErrorCode::TierMismatch);

    // Tier must be valid + active in Config
    let tier_cfg = config.get_tier_settings(tier)?;
    require!(tier_cfg.is_active(), IC42NErrorCode::InactiveTier);

    // ResolvedGame must match the same epoch/tier (belt & suspenders)
    require_eq!(game.epoch, epoch, IC42NErrorCode::EpochMismatch);
    require_eq!(game.tier, tier, IC42NErrorCode::TierMismatch);

    // Only allow transitions if it has been resolved
    require!(
        game.status != GameStatus::Resolved as u8,
        IC42NErrorCode::GameAlreadyResolved
    );

    // ─────────────────────────────────────────────
    // 2) Flip state → Resolving and bump attempt
    // ─────────────────────────────────────────────

    game.attempt_count = game
        .attempt_count
        .saturating_add(1);
    //Must set to processing just in case it was previously failed
    game.status = GameStatus::Processing as u8;
    game.last_updated_slot = clock.slot;
    game.last_updated_ts   = clock.unix_timestamp;

    Ok(())
}