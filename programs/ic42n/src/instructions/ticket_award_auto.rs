use anchor_lang::prelude::*;
use crate::state::player_profile::PlayerProfile;
use crate::errors::IC42NErrorCode;
use crate::state::{Config};
use crate::utils::ticket::{ award_tickets_to_profile};

/// Admin-only ticket award:
/// - Called by backend after computing losers off-chain.
/// - Uses tier config to determine how many tickets to award.
/// - Only updates PlayerProfile; does NOT touch Bet accounts.
#[derive(Accounts)]
#[instruction(tier: u8)]
pub struct AutoAwardTicket<'info> {

    /// CHECK: Player only used to derive the PDA
    pub player: UncheckedAccount<'info>,

    #[account(
        seeds = [Config::SEED],
        bump = config.bump,
        has_one = authority @ IC42NErrorCode::Unauthorized
    )]
    pub config: Account<'info, Config>,

    /// PlayerProfile PDA - must belong to `player`
    #[account(
        mut,
        seeds = [PlayerProfile::SEED_PREFIX, player.key().as_ref()],
        bump,
        constraint = profile.player == player.key() @ IC42NErrorCode::Unauthorized
    )]
    pub profile: Box<Account<'info, PlayerProfile>>,

    /// The program admin (must match `config.authority`)
    pub authority: Signer<'info>,
}

pub fn award_ticket_auto_handler(ctx: Context<AutoAwardTicket>, tier: u8) -> Result<()> {

    let profile = &mut ctx.accounts.profile;
    let config = &ctx.accounts.config;

    // Award ticket count set by the tier settings
    let tier_settings = config.get_tier_settings(tier)?;
    let tickets = tier_settings.tickets_per_recipient as u32;
    if tickets == 0 {
        return Ok(());
    }

    award_tickets_to_profile(profile, tier_settings.tickets_per_recipient as u32);

    Ok(())
}