use anchor_lang::prelude::*;
use crate::constants::MAX_TICKETS_PER_GRANT;
use crate::state::player_profile::PlayerProfile;
use crate::errors::IC42NErrorCode;
use crate::state::{Config};
use crate::utils::ticket::{ award_tickets_to_profile};

#[derive(Accounts)]
pub struct ManualAwardTicket<'info> {
    #[account(
        seeds = [Config::SEED],
        bump = config.bump,
        has_one = authority @ IC42NErrorCode::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub profile: Account<'info, PlayerProfile>,

    /// Authority must match `config.authority`
    pub authority: Signer<'info>,
}

pub fn award_ticket_manual_handler(ctx: Context<ManualAwardTicket>, tickets: u32) -> Result<()> {

    // Validate input
    require!(tickets > 0, IC42NErrorCode::InvalidTicketAmount);
    require!(tickets <= MAX_TICKETS_PER_GRANT, IC42NErrorCode::InvalidTicketAmount);

    let profile = &mut ctx.accounts.profile;
    
    award_tickets_to_profile(profile, tickets);
    Ok(())
}