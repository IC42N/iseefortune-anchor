use anchor_lang::prelude::*;
use crate::errors::IC42NErrorCode;

/// ---------------------------------------------------------------------------
/// Prediction
/// ---------------------------------------------------------------------------
/// Represents a user's prediction for a specific Solana epoch.
/// Supports multiple "selection styles" while keeping the account fixed-size.
#[account]
pub struct Prediction {

    /// First epoch in the chain for this game (stable game ID).
    pub game_epoch: u64,

    /// Epoch in which this prediction was placed (maybe > game_epoch for rollovers).
    pub epoch: u64,

    /// Player wallet.
    pub player: Pubkey,

    /// Tier this prediction is for.
    pub tier: u8,

    /// Prediction type (UI/analytics hint; the selection set is the source of truth).
    /// 0 = single_number
    /// 1 = split (any number of numbers)
    /// 2 = high_low
    /// 3 = even_odd
    pub prediction_type: u8,

    /// How many entries in `selections` are active (1..=8).
    /// - single_number: 1
    /// - two_numbers: 2
    /// - high_low: typically 4 (exact set stored)
    /// - even_odd: typically 3..=5 depending on blocked rollover numbers
    /// - future "cover" styles: up to 8
    pub selection_count: u8,


    /// Bitmask of selected numbers (bit n => number n is selected).
    /// Example: selecting {1,4,7} => (1<<1)|(1<<4)|(1<<7) = 146.
    pub selections_mask: u16,

    /// Selections used by the prediction (exact covered numbers).
    ///
    /// The active set is `selections[0..selection_count]`.
    /// Each entry must be in 1..=9 and must not include blocked rollover numbers
    /// (e.g., 0 and last winning number).
    ///
    /// Examples:
    /// - single_number: [7, 0, 0, 0, 0, 0, 0, 0] (selection_count=1)
    /// - two_numbers:   [2, 7, 0, 0, 0, 0, 0, 0] (selection_count=2)
    /// - high_low:      [1, 2, 3, 4, 0, 0, 0, 0] (selection_count=4)
    /// - even_odd:      [1, 3, 5, 7, 9, 0, 0, 0] (selection_count=5)
    pub selections: [u8; 8],

    /// Total lamports wagered for this prediction.
    /// Invariant: lamports == lamports_per_number * selection_count
    pub lamports: u64,

    /// The number of times this prediction has been changed.
    pub changed_count: u8,

    /// Slot at which the prediction was first placed.
    pub placed_slot: u64,

    /// Timestamp when the prediction was first placed.
    pub placed_at_ts: i64,

    /// Timestamp when the prediction was last updated.
    pub last_updated_at_ts: i64,

    /// Whether the prediction has been claimed.
    pub has_claimed: u8,

    /// Timestamp when the prediction was claimed (0 if unclaimed, or leave as-is).
    pub claimed_at_ts: i64,

    /// PDA bump.
    pub bump: u8,

    /// Version marker for decoding & future migrations.
    pub version: u8, // = 1

    /// Lamports wagered per selected number.
    pub lamports_per_number: u64,
    
    /// Reserved for future use.
    pub _reserved: [u8; 8],
}

impl Prediction {
    pub const SEED_PREFIX: &'static [u8] = b"prediction";
    pub const VERSION: u8 = 2;
    pub const TYPE_SINGLE_NUMBER: u8 = 0;
    pub const TYPE_TWO_NUMBERS: u8 = 1;
    pub const TYPE_HIGH_LOW: u8 = 2;
    pub const TYPE_EVEN_ODD: u8 = 3;
    pub const TYPE_MULTI_NUMBER: u8 = 4;

    /// Space excluding the 8-byte discriminator.
    ///
    /// NOTE: This must match the field order above exactly.
    pub const SIZE: usize =
   
            8 +  // game_epoch
            8 +  // epoch
            32 + // player
            1 +  // tier
            1 +  // prediction_type
            1 +  // selection_count
            2 +  // selections mask u16,
            8 +  // selections ([u8; 8])
            8 +  // total lamports
            1 +  // changed_count
            8 +  // placed_slot
            8 +  // placed_at_ts
            8 +  // last_updated_at_ts
            1 +  // has_claimed (u8)
            8 +  // claimed_at_ts (i64)
            1 +  // bump
            1 +  // version
            8 +  // lamports per number
            8;  // _reserved

    pub fn per_selection_lamports(&self) -> u64 {
        self.lamports_per_number
    }

    pub fn assert_invariant(&self) -> Result<()> {
        require!(
            self.lamports == self.expected_total_lamports(),
            IC42NErrorCode::AssertInvariantFailed
        );
        Ok(())
    }

    pub fn expected_total_lamports(&self) -> u64 {
        self.lamports_per_number
            .saturating_mul(self.selection_count.max(1) as u64)
    }


    pub fn mask_has(&self, n: u8) -> bool {
        if n > 9 { return false; }
        (self.selections_mask & (1u16 << n)) != 0
    }

    pub fn recompute_mask_from_selections(&self) -> u16 {
        let mut m: u16 = 0;
        let n = self.selection_count.min(8) as usize;
        for &v in self.selections[..n].iter() {
            if (1..=9).contains(&v) {
                m |= 1u16 << v;
            }
        }
        m
    }
}