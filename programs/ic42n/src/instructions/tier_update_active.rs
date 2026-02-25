use anchor_lang::prelude::*;
use crate::errors::IC42NErrorCode;
use crate::state::config::Config;


/*** Update Tier Active State */
#[derive(Accounts)]
pub struct UpdateTierActive<'info> {
    /// Global config (stores tiers, authority, etc.)
    #[account(
        mut,
        has_one = authority @ IC42NErrorCode::Unauthorized
    )]
    pub config: Account<'info, Config>,

    /// Program authority (admin / DAO / multisig)
    pub authority: Signer<'info>,
}

pub fn update_tier_active_handler(
    ctx: Context<UpdateTierActive>,
    tier_id: u8,
    active: u8,
) -> Result<()> {
    let cfg = &mut ctx.accounts.config;

    // Only allow 0 or 1 for now
    require!(active <= 1, IC42NErrorCode::InvalidTierFlag);

    // Use your helper on Config
    cfg.set_tier_active(tier_id, active)?;

    Ok(())
}