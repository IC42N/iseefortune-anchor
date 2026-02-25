use anchor_lang::prelude::*;

#[account]
pub struct LiveFeed {
    /// The current Solana epoch being tracked for this tier.
    pub epoch: u64,

    /// First epoch included in the current epoch-chain for this tier.
    pub first_epoch_in_chain: u64,

    /// Total lamports wagered across the current epoch-chain.
    pub total_lamports: u64,

    /// Lamports carried into the current epoch (accounting only).
    pub carried_over_lamports: u64,

    /// Total bet count across the current epoch-chain.
    pub total_bets: u32,

    /// Bet count carried into the current epoch (accounting only).
    pub carried_over_bets: u32,

    /// Slots-before-epoch-end cutoff enforced for betting.
    pub bet_cutoff_slots: u64,

    /// Tier ID for this feed (1..=5).
    pub tier: u8,

    /// Treasury PDA holding lamports for payouts/fees.
    pub treasury: Pubkey,

    /// Number of times this game carried forward due to rollover.
    pub epochs_carried_over: u8,

    /// PDA bump.
    pub bump: u8,

    /// Lamports wagered per number index (0..=9).
    pub lamports_per_number: [u64; 10],

    /// Bet count per number index (0..=9).
    pub bets_per_number: [u32; 10],

    /// Secondary rollover number for this tier’s current game (0 disables).
    pub secondary_rollover_number: u8,

    /// Current fee rate for this tier’s current game.
    pub current_fee_bps: u16,

    /// Reserved for future fields.
    pub _reserved: [u8; 61],
}

impl LiveFeed {
    pub const SEED_PREFIX: &'static [u8] = b"live_feed";

    /// Serialized size excluding the 8-byte Anchor discriminator.
    pub const SIZE: usize =
        8  // epoch
            + 8  // first_epoch_in_chain
            + 8  // total_lamports
            + 8  // carried_over_lamports
            + 4  // total_bets
            + 4  // carried_over_bets
            + 8  // bet_cutoff_slots
            + 1  // tier
            + 32 // treasury
            + 1  // epochs_carried_over
            + 1  // bump
            + (8 * 10)  // lamports_per_number
            + (4 * 10)  // bets_per_number
            + 1  // secondary_rollover_number
            + 2  // current_fee_bps
            + 61; // reserved

    pub fn init_new(
        &mut self,
        epoch: u64,
        cutoff_slots: u64,
        tier: u8,
        treasury: Pubkey,
        bump: u8,
        fee_bps: u16,
    ) {
        self.epoch = epoch;
        self.first_epoch_in_chain = epoch;

        self.total_lamports = 0;
        self.carried_over_lamports = 0;

        self.total_bets = 0;
        self.carried_over_bets = 0;

        self.bet_cutoff_slots = cutoff_slots;
        self.tier = tier;
        self.treasury = treasury;
        self.epochs_carried_over = 0;
        self.bump = bump;

        self.secondary_rollover_number = 0;
        self.current_fee_bps = fee_bps;

        self.clear_per_number_state();
        self._reserved = [0u8; 61];
    }

    /// Advances the feed into `new_epoch`. If carry values are non-zero, the
    /// current epoch-chain continues; otherwise a new chain begins at `new_epoch`.
    pub fn reset_for_new_epoch(
        &mut self,
        new_epoch: u64,
        cutoff_slots: u64,
        carry_over_lamports: u64,
        carry_over_bets: u32,
        lamports_per_number: [u64; 10],
        bets_per_number: [u32; 10],
        next_secondary_rollover: u8,
        next_fee_bps: u16,
    ) {
        self.epoch = new_epoch;
        self.bet_cutoff_slots = cutoff_slots;
        self.current_fee_bps = next_fee_bps;

        let is_carry = carry_over_lamports > 0 || carry_over_bets > 0;

        if is_carry {
            self.total_lamports = carry_over_lamports;
            self.carried_over_lamports = carry_over_lamports;

            self.total_bets = carry_over_bets;
            self.carried_over_bets = carry_over_bets;

            self.lamports_per_number = lamports_per_number;
            self.bets_per_number = bets_per_number;

            self.epochs_carried_over = self.epochs_carried_over.saturating_add(1);
            if self.epochs_carried_over == 0 {
                self.epochs_carried_over = 1;
            }
        } else {
            self.first_epoch_in_chain = new_epoch;
            self.epochs_carried_over = 0;

            self.total_lamports = 0;
            self.carried_over_lamports = 0;

            self.total_bets = 0;
            self.carried_over_bets = 0;

            self.secondary_rollover_number = next_secondary_rollover;
            self.clear_per_number_state();
        }
    }

    fn clear_per_number_state(&mut self) {
        self.lamports_per_number = [0u64; 10];
        self.bets_per_number = [0u32; 10];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSerialize;

    #[test]
    fn test_live_feed_size() {
        let lf = LiveFeed {
            epoch: 0,
            first_epoch_in_chain: 0,
            total_lamports: 0,
            carried_over_lamports: 0,
            total_bets: 0,
            carried_over_bets: 0,
            bet_cutoff_slots: 0,
            tier: 0,
            treasury: Pubkey::default(),
            epochs_carried_over: 0,
            bump: 0,
            lamports_per_number: [0u64; 10],
            bets_per_number: [0u32; 10],
            secondary_rollover_number: 0,
            current_fee_bps: 0,
            _reserved: [0u8; 61],
        };

        let bytes = lf.try_to_vec().unwrap();
        assert_eq!(bytes.len(), LiveFeed::SIZE);
    }
}