use anchor_lang::prelude::*;
use crate::errors::IC42NErrorCode;
use crate::state::*;

#[derive(Accounts)]
#[instruction(tier: u8)]
pub struct CloseFeed<'info> {
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
        close = authority
    )]
    pub live_feed: Account<'info, LiveFeed>,

    #[account(mut)]
    pub authority: Signer<'info>,
}


pub fn close_tier_live_feed_handler(
    ctx: Context<CloseFeed>,
    tier: u8,
) -> Result<()> {

    require_eq!(ctx.accounts.live_feed.tier, tier, IC42NErrorCode::InvalidTier);

    let config = &mut ctx.accounts.config;
    let live = &ctx.accounts.live_feed;

    // Don't allow closing if there are unresolved bets / pot
    require_eq!(
        live.total_bets,
        0,
        IC42NErrorCode::LiveFeedNotEmpty
    );

    // Deactivate tier
    config.set_tier_active(tier, 0)?;

    Ok(())
}