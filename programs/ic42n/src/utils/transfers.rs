use anchor_lang::prelude::*;
use crate::errors::IC42NErrorCode;

pub fn transfer_lamports<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, IC42NErrorCode::InvalidBetAmount);

    anchor_lang::system_program::transfer(
        CpiContext::new(
            system_program.clone(),
            anchor_lang::system_program::Transfer {
                from: from.clone(),
                to: to.clone(),
            },
        ),
        amount,
    )
}