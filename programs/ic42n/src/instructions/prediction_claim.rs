use anchor_lang::prelude::*;
use sha2::{Digest, Sha256};

use crate::errors::IC42NErrorCode;
use crate::state::{GameStatus, Prediction};
use crate::state::resolved_game::ResolvedGame;
use crate::state::treasury::Treasury;
use crate::utils::bitmap::{is_claimed, set_claimed};
use crate::utils::merkle::verify_merkle_proof;

/// Allows a winner to claim their payout for a resolved (epoch, tier) game.
///
/// Claims are validated using a Merkle proof against the committed
/// `merkle_root`, and double-claims are prevented using a bitmap.
#[derive(Accounts)]
#[instruction(epoch: u64, tier: u8)]
pub struct ClaimPrediction<'info> {
    /// Resolved game account containing Merkle root and claim tracking.
    #[account(
        mut,
        seeds = [ResolvedGame::SEED_PREFIX, epoch.to_le_bytes().as_ref(), &[tier]],
        bump = game.bump,
        constraint = game.resolved_at != 0 @ IC42NErrorCode::GameNotResolved
    )]
    pub game: Account<'info, ResolvedGame>,

    /// Prediction associated with the claiming wallet for this game chain.
    #[account(
        mut,
        seeds = [
            Prediction::SEED_PREFIX,
            claimer.key().as_ref(),
            game.first_epoch_in_chain.to_le_bytes().as_ref(),
            &[tier]
        ],
        bump,
        constraint = prediction.player == claimer.key() @ IC42NErrorCode::Unauthorized,
        constraint = prediction.tier == tier @ IC42NErrorCode::TierMismatch,
        constraint = prediction.game_epoch == game.first_epoch_in_chain @ IC42NErrorCode::EpochMismatch
    )]
    pub prediction: Account<'info, Prediction>,

    /// Treasury holding lamports for all payouts.
    #[account(
        mut,
        seeds = [Treasury::SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, Treasury>,

    /// Wallet receiving the payout.
    #[account(mut)]
    pub claimer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// Claims a winning payout using a Merkle proof.
pub fn claim_prediction_handler(
    ctx: Context<ClaimPrediction>,
    epoch: u64,
    tier: u8,
    index: u32,
    amount: u64,
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    require!(proof.len() <= 40, IC42NErrorCode::ProofTooLong);

    let game = &mut ctx.accounts.game;
    let pred = &mut ctx.accounts.prediction;
    let treasury = &mut ctx.accounts.treasury;
    let claimer = &ctx.accounts.claimer;

    // Make sure values are correct.
    pred.assert_invariant()?;

    // Claim must not have been processed already
    require!(
        !is_claimed(&game.claimed_bitmap, index),
        IC42NErrorCode::AlreadyClaimed
    );
    require!(pred.has_claimed == 0, IC42NErrorCode::AlreadyClaimed);
    require!(
        game.claimed_winners < game.total_winners,
        IC42NErrorCode::TooManyClaims
    );

    // Game and claim parameters must match
    require!(
        game.status == GameStatus::Resolved as u8,
        IC42NErrorCode::GameNotResolved
    );
    require_eq!(game.epoch, epoch, IC42NErrorCode::EpochMismatch);
    require_eq!(game.tier, tier, IC42NErrorCode::TierMismatch);

    require!(amount > 0, IC42NErrorCode::InvalidClaimAmount);
    require!(game.total_winners > 0, IC42NErrorCode::ClaimNotAllowed);

    // Index bounds and bitmap integrity
    require!(index < game.total_winners, IC42NErrorCode::InvalidClaimIndex);

    let byte_index = (index / 8) as usize;
    require!(
        byte_index < game.claimed_bitmap.len(),
        IC42NErrorCode::BitmapOutOfBounds
    );

    let expected_len = ((game.total_winners as usize) + 7) / 8;
    require!(
        game.claimed_bitmap.len() == expected_len,
        IC42NErrorCode::InvalidBitmapLen
    );

    // --- OPTIONAL but recommended: sanity-check the prediction selection data ---
    // Ensures the account isn't corrupted (and helps prevent weird proof binding issues).
    let k = pred.selection_count as usize;
    require!(k >= 1 && k <= 8, IC42NErrorCode::InvalidBetNumber);

    let mut recomputed: u16 = 0;
    for i in 0..k {
        let n = pred.selections[i];
        require!(n >= 1 && n <= 9, IC42NErrorCode::InvalidBetNumber);
        recomputed |= 1u16 << n;
    }
    require!(recomputed == pred.selections_mask, IC42NErrorCode::InvalidBetNumber);

    // Rebuild Merkle leaf
    //
    // IMPORTANT CHOICE:
    // - Minimal leaf (matches your old claim): epoch/tier/index/wallet/amount
    // - Stronger leaf (recommended): also commit to selections_mask (and optionally lamports)
    //
    // If you change the leaf format, your resolver (Merkle builder) must match this exactly.
    let mut hasher = Sha256::new();
    hasher.update(b"IC42N_V2");
    hasher.update(&epoch.to_le_bytes());
    hasher.update(&[tier]);
    hasher.update(&index.to_le_bytes());
    hasher.update(claimer.key().as_ref());
    hasher.update(&amount.to_le_bytes());

    // Bind proof to the exact coverage set the user had for this chain
    hasher.update(&pred.selections_mask.to_le_bytes());

    let leaf_hash: [u8; 32] = hasher.finalize().into();

    // Verify Merkle proof
    require!(
        game.merkle_root != [0u8; 32],
        IC42NErrorCode::EmptyMerkleRoot
    );
    require!(
        verify_merkle_proof(&leaf_hash, &proof, &game.merkle_root, index),
        IC42NErrorCode::InvalidProof
    );

    // Ensure a sufficient prize pool and treasury balance
    let remaining = game
        .net_prize_pool
        .checked_sub(game.claimed_lamports)
        .ok_or(IC42NErrorCode::MathOverflow)?;
    require!(amount <= remaining, IC42NErrorCode::InsufficientPrizePool);

    let treasury_balance = **treasury.to_account_info().lamports.borrow();
    require!(treasury_balance >= amount, IC42NErrorCode::InsufficientTreasuryBalance);

    // Transfer lamports
    **treasury.to_account_info().try_borrow_mut_lamports()? -= amount;
    **claimer.to_account_info().try_borrow_mut_lamports()? += amount;

    // Record claim
    set_claimed(&mut game.claimed_bitmap, index);

    game.claimed_lamports = game
        .claimed_lamports
        .checked_add(amount)
        .ok_or(IC42NErrorCode::MathOverflow)?;
    game.claimed_winners = game
        .claimed_winners
        .checked_add(1)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    pred.has_claimed = 1;
    pred.claimed_at_ts = Clock::get()?.unix_timestamp;

    Ok(())
}