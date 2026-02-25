use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(epoch: u64, tier: u8)]
pub struct CloseGame<'info> {
    #[account(
        seeds = [Config::SEED],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [ResolvedGame::SEED_PREFIX, epoch.to_le_bytes().as_ref(), &[tier]],
        bump,
        close = authority
    )]
    pub resolved_game: Account<'info, ResolvedGame>,

    #[account(mut, address = config.authority)]
    pub authority: Signer<'info>,
}


pub fn close_resolved_game_handler(
    _ctx: Context<CloseGame>,
    _epoch: u64,
    _tier: u8,
) -> Result<()> {
    Ok(())
}