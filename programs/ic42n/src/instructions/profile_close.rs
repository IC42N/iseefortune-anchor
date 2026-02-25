use anchor_lang::prelude::*;
use crate::errors::IC42NErrorCode;
use crate::state::player_profile::PlayerProfile;

#[derive(Accounts)]
pub struct ClosePlayerProfile<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [PlayerProfile::SEED_PREFIX, player.key().as_ref()],
        bump = profile.bump,
        constraint = profile.player == player.key() @ IC42NErrorCode::Unauthorized,
        close = player
    )]
    pub profile: Box<Account<'info, PlayerProfile>>,
    pub system_program: Program<'info, System>,
}


pub fn close_player_profile_handler(ctx: Context<ClosePlayerProfile>) -> Result<()> {
    let profile = &ctx.accounts.profile;
    let clock = Clock::get()?;
    require!(
        clock.epoch >= profile.locked_until_epoch,
        IC42NErrorCode::ProfileLockedActiveGame
    );
    Ok(())
}