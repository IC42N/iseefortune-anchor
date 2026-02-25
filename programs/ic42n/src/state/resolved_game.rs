use anchor_lang::prelude::*;

#[repr(u8)]
pub enum GameStatus {
    Failed    = 0, // Something went wrong, can be retried
    Processing = 1, // Worker is currently processing (JSON, Merkle, uploads, etc.)
    Resolved  = 2, // Fully finalized on-chain
}

#[repr(u8)]
pub enum RolloverReason {
    None = 0,
    NoWinners = 1,
    RolloverNumber = 2,
}
impl RolloverReason {
    pub fn as_u8(self) -> u8 { self as u8 }
}

/// ---------------------------------------------------------------------------
/// ResolvedGame
/// ---------------------------------------------------------------------------
///
/// Finalized record of a single IC42N game, created after a Solana epoch ends.
/// Stores immutable results, Merkle root, winners, and carry-over data.
///
/// This account is created **after an epoch ends**, once results are known
/// and the Merkle tree of winners has been computed.
///
/// This struct acts as the **ledger entry** for that epoch and tier:
/// - records total bets and pot size
/// - tracks carry-over amounts
/// - stores the Merkle root for claim verification
/// - exposes an immutable URI with full result data
///
/// ⚠️ Lamports themselves are **never held here** — they remain in the
/// central Treasury PDA. This account is purely an accounting and claims record.
#[account]
pub struct ResolvedGame {
    // Identification
    pub epoch: u64,
    pub tier: u8,
    pub status: u8,
    pub bump: u8,
    pub winning_number: u8,

    // RNG provenance
    pub rng_epoch_slot_used: u64,
    pub rng_blockhash_used: [u8; 32],

    // Processing metadata
    pub attempt_count: u8,
    pub last_updated_slot: u64,
    pub last_updated_ts: i64,

    // Accounting
    pub carry_over_bets: u32,
    pub total_bets: u32,
    pub carry_in_lamports: u64,
    pub carry_out_lamports: u64,
    pub protocol_fee_lamports: u64,
    pub net_prize_pool: u64,
    pub total_winners: u32,
    pub claimed_winners: u32,
    pub resolved_at: i64,

    // Claims
    pub merkle_root: [u8; 32],
    pub results_uri: [u8; 128],
    pub claimed_bitmap: Vec<u8>,

    // Versioning / extensions
    pub version: u8,
    pub claimed_lamports: u64,
    pub first_epoch_in_chain: u64,

    pub rollover_reason: u8,
    pub secondary_rollover_number: u8,
    pub fee_bps: u16,
    pub _reserved: [u8; 12],
}

impl ResolvedGame {
    pub const SEED_PREFIX: &'static [u8] = b"resolved_game";
    pub const MAX_WINNERS_PER_GAME: usize = 50_000;
    pub const MAX_BITMAP_LEN: usize = (Self::MAX_WINNERS_PER_GAME + 7) / 8;

    // Fixed fields + Vec length prefix (u32). Excludes bitmap bytes themselves.
    pub const BASE_SIZE: usize =
        8   + // epoch
            1   + // tier
            1   + // status
            1   + // bump
            1   + // winning_number
            8   + // rng_epoch_slot_used
            32  + // rng_blockhash_used
            1   + // attempt_count
            8   + // last_updated_slot
            8   + // last_updated_ts
            4   + // carry_over_bets
            4   + // total_bets
            8   + // carry_in_lamports
            8   + // carry_out_lamports
            8   + // protocol_fee_lamports
            8   + // net_prize_pool
            4   + // total_winners
            4   + // claimed_winners
            8   + // resolved_at
            32  + // merkle_root
            128 + // results_uri
            4   + // claimed_bitmap length prefix
            1   + // version
            8   + // claimed_lamports
            8   + // first_epoch_in_chain
            1   + // rollover_reason
            1   + // secondary_rollover_number
            2   + // feeBps
            12;   // reserved

    pub const SIZE: usize = Self::BASE_SIZE + Self::MAX_BITMAP_LEN;

}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSerialize;

    #[test]
    fn test_resolved_game_max_size() {
        let bitmap_len = ResolvedGame::MAX_BITMAP_LEN;

        let game = ResolvedGame {
            // core + status
            epoch: 0,
            tier: 0,
            status: 0,
            bump: 0,
            winning_number: 0,

            // rng
            rng_epoch_slot_used: 0,
            rng_blockhash_used: [0u8; 32],

            // processing
            attempt_count: 0,
            last_updated_slot: 0,
            last_updated_ts: 0,

            // pot + accounting
            carry_over_bets: 0,
            total_bets: 0,
            carry_in_lamports: 0,
            carry_out_lamports: 0,
            protocol_fee_lamports: 0,
            net_prize_pool: 0,
            total_winners: 0,
            claimed_winners: 0,
            resolved_at: 0,

            // merkle + uri + bitmap
            merkle_root: [0u8; 32],
            results_uri: [0u8; 128],
            claimed_bitmap: vec![0u8; bitmap_len],

            // misc
            version: 0,
            claimed_lamports: 0,
            first_epoch_in_chain: 0,
            rollover_reason: 0,
            secondary_rollover_number: 0,
            fee_bps: 0,
            _reserved: [0u8; 12],
        };

        let bytes = game.try_to_vec().unwrap();
        assert_eq!(bytes.len(), ResolvedGame::SIZE);
    }
}