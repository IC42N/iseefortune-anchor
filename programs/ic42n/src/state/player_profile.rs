use anchor_lang::prelude::*;
use crate::constants::RECENT_BETS_CAP;

#[account]
pub struct PlayerProfile {
    /// The owner/player wallet
    pub player: Pubkey, // 32

    /// PDA bump
    pub bump: u8, // 1

    /// Number of available tickets
    pub tickets_available: u32, // 4

    // ─────────────────────────────
    // Aggregate stats
    // ─────────────────────────────
    pub total_bets: u64,              // 8
    pub total_lamports_wagered: u64,  // 8
    pub last_played_epoch: u64,       // 8
    pub last_played_tier: u8,         // 1
    pub last_played_timestamp: i64,   // 8
    pub xp_points: u32,               // 4

    // ─────────────────────────────
    // Recent bets ring buffer
    // ─────────────────────────────
    /// Circular buffer of the last N bet pubkeys
    pub recent_bets: [Pubkey; RECENT_BETS_CAP], // 32 * 40 = 1280

    /// Number of valid entries currently stored (0~RECENT_BETS_CAP)
    pub recent_bets_len: u16, // 2

    /// Next index to write (wraps around 0..RECENT_BETS_CAP-1)
    pub recent_bets_head: u16, // 2

    /// Prevent closer if player is in game. 
    pub locked_until_epoch: u64, // 8

    /// first game played
    pub first_played_epoch: u64,
    
    // ─────────────────────────────
    // Reserved for future upgrades
    // ─────────────────────────────
    pub _reserved: [u8; 16],
}

impl PlayerProfile {
    pub const SEED_PREFIX: &'static [u8] = b"profile";

    /// Total serialized size (not including the 8-byte discriminator)
    pub const SIZE: usize =
        32  // player
            + 1   // bump
            + 4   // tickets_available
            + 8   // total_bets
            + 8   // total_lamports_wagered
            + 8   // last_played_epoch
            + 1   // last_played_tier
            + 8   // last_played_timestamp
            + 4   // xp_points
            + (32 * RECENT_BETS_CAP) // recent_bets
            + 2   // recent_bets_len
            + 2   // recent_bets_head
            + 8   // locked_until_epoch
            + 8   // first_played_epoch
            + 16; // reserved

    /// Push a bet pubkey into the ring buffer (keeps only the last N)
    pub fn push_recent_bet(&mut self, bet: Pubkey) {
        let head = self.recent_bets_head as usize;
        self.recent_bets[head] = bet;

        let next = (head + 1) % RECENT_BETS_CAP;
        self.recent_bets_head = next as u16;

        if (self.recent_bets_len as usize) < RECENT_BETS_CAP {
            self.recent_bets_len += 1;
        }
    }
}