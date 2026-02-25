use anchor_lang::prelude::*;

use crate::errors::IC42NErrorCode;
use crate::state::tiers::TierSettings;

/// Global configuration PDA.
///
/// Stores protocol-wide controls (authority, fee routing, pause flags),
/// fee parameters, and tier settings. This account holds no lamports.
#[account]
pub struct Config {
    /// 1 = betting paused, 0 = enabled.
    pub pause_bet: u8,

    /// 1 = withdrawals/claims paused, 0 = enabled.
    pub pause_withdraw: u8,

    /// Program admin authority.
    pub authority: Pubkey,

    /// Destination for collected protocol fees.
    pub fee_vault: Pubkey,

    /// Base protocol fee in basis points (1 bps = 0.01%).
    pub base_fee_bps: u16,

    /// Minimum slots remaining in the epoch required to allow betting.
    pub bet_cutoff_slots: u64,

    /// Unix timestamp when protocol was initialized.
    pub started_at: i64,

    /// Solana epoch when protocol was initialized.
    pub started_epoch: u64,

    /// Primary number that triggers rollover behavior.
    pub primary_roll_over_number: u8,

    /// Tier configurations (fixed-size array).
    pub tiers: [TierSettings; 5],

    /// PDA bump for Config.
    pub bump: u8,

    /// Minimum allowed fee in basis points.
    pub min_fee_bps: u16,

    /// Fee step applied to rollover scenarios (basis points).
    pub rollover_fee_step_bps: u16,

    /// Reserved space for future upgrades.
    pub _reserved: [u8; 16],
}

impl Config {
    pub const SEED: &'static [u8] = b"config";

    /// Serialized size excluding the 8-byte Anchor discriminator.
    pub const SIZE: usize =
        1 +  // pause_bet
            1 +  // pause_withdraw
            32 + // authority
            32 + // fee_vault
            2 +  // base_fee_bps
            8 +  // bet_cutoff_slots
            8 +  // started_at
            8 +  // started_epoch
            1 +  // primary_roll_over_number
            (TierSettings::SIZE * 5) + // tiers
            1 +  // bump
            2 +  // min_fee_bps
            2 +  // rollover_fee_step_bps
            16;  // reserved

    /// Returns tier settings by tier id (1..=5).
    pub fn get_tier_settings(&self, tier_id: u8) -> Result<TierSettings> {
        self.tiers
            .iter()
            .find(|t| t.tier_id == tier_id)
            .copied()
            .ok_or_else(|| error!(IC42NErrorCode::UnknownTier))
    }

    /// Sets the `active` flag for a tier.
    pub fn set_tier_active(&mut self, tier_id: u8, active: u8) -> Result<()> {
        let settings = self
            .tiers
            .iter_mut()
            .find(|t| t.tier_id == tier_id)
            .ok_or_else(|| error!(IC42NErrorCode::UnknownTier))?;

        if active == 1 {
            require!(
                settings.max_bet_lamports > 0 && settings.curve_factor > 0.0,
                IC42NErrorCode::InactiveTier
            );
        }

        settings.active = active;
        Ok(())
    }

    pub fn is_betting_paused(&self) -> bool {
        self.pause_bet != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSerialize;

    fn default_tier(tier_id: u8) -> TierSettings {
        TierSettings {
            tier_id,
            active: 0,
            min_bet_lamports: 0,
            max_bet_lamports: 0,
            curve_factor: 0.0,
            ticket_reward_bps: 0,
            ticket_reward_max: 0,
            tickets_per_recipient: 1,
            _reserved: [0; 10],
        }
    }

    #[test]
    fn config_size_matches_serialization() {
        let tiers = [
            default_tier(1),
            default_tier(2),
            default_tier(3),
            default_tier(4),
            default_tier(5),
        ];

        let cfg = Config {
            pause_bet: 0,
            pause_withdraw: 0,
            authority: Pubkey::default(),
            fee_vault: Pubkey::default(),
            base_fee_bps: 0,
            bet_cutoff_slots: 0,
            started_at: 0,
            started_epoch: 0,
            primary_roll_over_number: 0,
            tiers,
            bump: 0,
            min_fee_bps: 300,
            rollover_fee_step_bps: 100,
            _reserved: [0; 16],
        };

        let bytes = cfg.try_to_vec().unwrap();
        assert_eq!(bytes.len(), Config::SIZE);
    }
}