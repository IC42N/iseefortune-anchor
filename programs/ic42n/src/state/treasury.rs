use anchor_lang::prelude::*;

/// ---------------------------------------------------------------------------
/// Treasury
/// ---------------------------------------------------------------------------
///
/// Program-owned PDA that holds SOL for the IC42N game.
/// If you use a single global treasury, `tier` is fixed to 0.
#[account]
pub struct Treasury {
    /// Who controls configuration / fee withdrawals.
    pub authority: Pubkey,

    /// Tier this treasury is associated with:
    ///   0 = Global (all tiers)
    ///   1 = Low, 2 = Mid, 3 = High (if you ever decide to split).
    pub tier: u8,

    /// PDA bump for deterministic re-derivation.
    pub bump: u8,

    // ─────────────────────────────
    // Accounting / stats
    // ─────────────────────────────

    /// Total lamports ever received as bets into this treasury
    /// (monotonic counter, for analytics / audit).
    pub total_in_lamports: u64,

    /// Total lamports ever paid out to winners from this treasury.
    pub total_out_lamports: u64,

    /// Total lamports withdrawn as protocol fees (house edge).
    pub total_fees_withdrawn: u64,

    // ─────────────────────────────
    // Control flags
    // ─────────────────────────────

    /// Versioning for future migrations.
    pub version: u8,

    /// Padding / reserved bytes for future use (config, extra flags).
    pub _reserved: [u8; 32],
}

impl Treasury {

    pub const SEED: &'static [u8] = b"treasury";
    pub const SIZE: usize =
        32 + // authority
            1  + // tier
            1  + // bump
            8  + // total_in_lamports
            8  + // total_out_lamports
            8  + // total_fees_withdrawn
            1  + // version
            32;  // reserved
    // When allocating:
    // space = 8 (discriminator) + Treasury::SIZE
}


#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSerialize;

    #[test]
    fn test_treasury_size() {
        // Construct a dummy instance to check Borsh serialization size
        let t = Treasury {
            authority: Pubkey::default(),
            tier: 0,
            bump: 0,
            total_in_lamports: 0,
            total_out_lamports: 0,
            total_fees_withdrawn: 0,
            version: 0,
            _reserved: [0u8; 32],
        };

        let bytes = t.try_to_vec().unwrap();

        assert_eq!(
            bytes.len(),
            Treasury::SIZE,
            "Treasury account size mismatch: expected {}, got {}",
            Treasury::SIZE,
            bytes.len()
        );
    }
}