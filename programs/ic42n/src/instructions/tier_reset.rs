use anchor_lang::prelude::*;
use crate::errors::IC42NErrorCode;
use crate::state::*;

#[derive(Accounts)]
#[instruction(tier: u8)]
pub struct ResetLiveFeed<'info> {
    #[account(
        seeds = [Config::SEED],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [LiveFeed::SEED_PREFIX, &[tier]],
        bump = live_feed.bump,
    )]
    pub live_feed: Account<'info, LiveFeed>,

    /// Only admin program authority that is allowed to reset tiers.
    #[account(mut, address = config.authority @ IC42NErrorCode::Unauthorized)]
    pub authority: Signer<'info>,
}


pub fn reset_live_feed_handler(
    ctx: Context<ResetLiveFeed>,
    tier: u8,
    rollover: u8
) -> Result<()> {
    let live = &mut ctx.accounts.live_feed;
    let config = &ctx.accounts.config;
    
    let clock = Clock::get()?;
    let current_epoch = clock.epoch;

    require!(
        live.tier == tier,
        IC42NErrorCode::TierMismatch
    );

    // 2) Only allow moving *forward* in time
    // This prevents resetting in the past
    require!(
        current_epoch >= live.epoch,
        IC42NErrorCode::EpochNotAdvanced
    );

    // Secondary rollover number must be between 1 and 9
    require!(
        rollover > 0 && rollover < 10,
        IC42NErrorCode::InvalidRollOverNumber
    );


    // 3) Do not allow wiping a pot or bets by mistake.
    // This should always be true for the "no activity" path.
    require!(
        live.total_lamports == 0
            && live.carried_over_lamports == 0
            && live.total_bets == 0
            && live.carried_over_bets == 0,
        IC42NErrorCode::LiveFeedNotEmpty
    );
    
    live.reset_for_new_epoch(
        current_epoch,
        config.bet_cutoff_slots,
        0, 
        0,
        [0u64; 10],
        [0u32; 10],
        rollover,
        config.base_fee_bps
    );
    
    Ok(())
}