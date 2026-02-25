pub const RESOLVED_GAME_VERSION: u8 = 2;

pub const FEE_BPS_DENOM: u64 = 10_000;

// Max number of tickets a player can receive as reward as one time
pub const MAX_TICKETS_PER_GRANT: u32 = 5; // adjust as needed

// Max number of tickets a player can have at once
pub const MAX_TICKETS_PER_PLAYER: u32 = 100; // adjust as needed

/// How many recent bet pubkeys to keep in the profile
pub const RECENT_BETS_CAP: usize = 40;

pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub const TIER1_MIN: u64 = 10_000_000;         // 0.01 SOL
pub const TIER1_MAX: u64 = 1_000_000_000;      // 1 SOL
pub const TIER2_MIN: u64 = 1_000_000_000;      // 1 SOL
pub const TIER2_MAX: u64 = 10_000_000_000;     // 10 SOL
pub const TIER3_MIN: u64 = 10_000_000_000;      // 10 SOL
pub const TIER3_MAX: u64 = 100_000_000_000;    // 100 SOL