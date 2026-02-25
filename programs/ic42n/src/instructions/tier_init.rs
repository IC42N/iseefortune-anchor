use anchor_lang::prelude::*;
use crate::errors::IC42NErrorCode;
use crate::state::*;
use crate::state::treasury::Treasury; 

#[derive(Accounts)]
#[instruction(tier: u8)]
pub struct InitTierLiveFeed<'info> {

    #[account(
        mut,
        has_one = authority @ IC42NErrorCode::Unauthorized
    )]
    pub config: Account<'info, Config>,

    /// Pass treasury for the live feed.
    #[account(
        seeds = [Treasury::SEED],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + LiveFeed::SIZE,
        seeds = [LiveFeed::SEED_PREFIX, &[tier]],
        bump
    )]
    pub live_feed: Account<'info, LiveFeed>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}


pub fn init_tier_live_feed_handler(
    ctx: Context<InitTierLiveFeed>,
    tier: u8,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let live = &mut ctx.accounts.live_feed;

    // ─────────────────────────────────────────────
    // 1) Epoch
    // ─────────────────────────────────────────────
    let clock = Clock::get()?;
    let current_epoch = clock.epoch;

    // ─────────────────────────────────────────────
    // 2) Ensure this tier is configured, then activate
    // ─────────────────────────────────────────────
    config.set_tier_active(tier, 1)?;

    // ─────────────────────────────────────────────
    // 3) Initialize LiveFeed for this tier
    // ─────────────────────────────────────────────
    live.init_new(
        current_epoch,
        config.bet_cutoff_slots,
        tier,
        ctx.accounts.treasury.key(),
        ctx.bumps.live_feed,
        config.base_fee_bps,
    );
    
    Ok(())
}