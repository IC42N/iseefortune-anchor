use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct TierSettings {
    pub tier_id: u8,
    pub active: u8,

    pub min_bet_lamports: u64,
    pub max_bet_lamports: u64,

    /// Optional shaping factor used by your payout/odds math.
    pub curve_factor: f32,

    /// Ticket distribution rate in basis points of losers (0 disables).
    pub ticket_reward_bps: u16,

    /// Max number of recipients eligible for tickets per resolved game.
    pub ticket_reward_max: u16,

    /// Number of tickets to award per selected recipient.
    pub tickets_per_recipient: u8,

    pub _reserved: [u8; 10],
}

impl TierSettings {
    pub const SIZE: usize =
        1  // tier_id
            + 1  // active
            + 8  // min_bet_lamports
            + 8  // max_bet_lamports
            + 4  // curve_factor
            + 2  // ticket_reward_bps
            + 2  // ticket_reward_max
            + 1  // tickets_per_recipient
            + 10; // _reserved

    #[inline]
    pub fn is_active(&self) -> bool {
        self.active != 0
    }

    #[inline]
    pub fn is_valid_bet(&self, lamports: u64) -> bool {
        lamports >= self.min_bet_lamports && lamports <= self.max_bet_lamports
    }
}