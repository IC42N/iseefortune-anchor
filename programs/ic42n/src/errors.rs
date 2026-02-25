use anchor_lang::prelude::*;

#[error_code]
pub enum IC42NErrorCode {
    // ─────────────────────────────
    // Setup and configuration
    // ─────────────────────────────
    EpochMismatch,
    TierMismatch,
    InvalidTierBounds,
    InvalidAuthorityTarget,
    InvalidTierFlag,
    InvalidRollOverNumber,
    InvalidCutOffNumber,

    // ─────────────────────────────
    // General / Access Control
    // ─────────────────────────────
    #[msg("Unauthorized")]
    Unauthorized,

    AuthorityCannotEqualFeeVault,
    InvalidFeeConfig,
    InvalidLiveFeedState,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Invalid input")]
    InvalidInput,

    InvalidFee,
    InvalidMinimumFee,
    InvalidFeeStep,
    InvalidCurveValue,

    GameNotFound,
    UnknownTier,

    #[msg("Invalid percentage")]
    InvalidTicketBps,

    InvalidTicketMax,

    // ─────────────────────────────
    // Initialize new tiers
    // ─────────────────────────────
    #[msg("Inactive tier")]
    InactiveTier,

    #[msg("Invalid tier")]
    InvalidTier,

    EpochNotAdvanced,

    // ─────────────────────────────
    // Close tier feed
    // ─────────────────────────────
    LiveFeedNotEmpty,

    // ─────────────────────────────
    // Game / Epoch Lifecycle
    // ─────────────────────────────
    GameAlreadyResolved,
    GameNotResolved,
    EpochPotNotInitialized,

    // ─────────────────────────────
    // Betting Validation
    // ─────────────────────────────
    #[msg("Already bet")]
    AlreadyBetThisGame,

    #[msg("Betting closed")]
    BettingClosed,

    #[msg("Betting paused")]
    BettingPaused,

    NoOpChange,
    TreasuryMismatch,
    BetOutOfTierRange,
    InvalidChoiceCount,
    AssertInvariantFailed,

    #[msg("Invalid number selection")]
    InvalidBetNumber,

    #[msg("Invalid amount")]
    InvalidBetAmount,

    #[msg("No change tickets")]
    NoChangeTickets,

    // ─────────────────────────────
    // Ticket Awarding
    // ─────────────────────────────
    InvalidTicketAmount,

    // ─────────────────────────────
    // Game Resolution
    // ─────────────────────────────
    CarryNotAllowed,
    GameAlreadyResolvingOrResolved,
    GameNotInResolvingState,
    NoBetsToResolve,

    #[msg("Invalid URI")]
    EmptyResultsUri,

    InvalidFeeVault,
    EpochNotComplete,
    InvalidWinningNumber,
    TooManyWinners,
    InvalidNetPoolPlusNet,
    InvalidPotBreakdown,
    InvalidCarryOver,
    InsufficientTreasuryBalance,
    BitmapTooLarge,

    // ─────────────────────────────
    // Merkle / Claim System
    // ─────────────────────────────
    InvalidBitmapLen,
    InsufficientPrizePool,
    ProofTooLong,

    #[msg("Invalid claim amount")]
    InvalidClaimAmount,

    EmptyMerkleRoot,

    #[msg("Invalid Merkle proof")]
    InvalidProof,

    #[msg("Already claimed")]
    AlreadyClaimed,

    InvalidIndex,

    #[msg("Claim not allowed")]
    ClaimNotAllowed,

    BitmapOutOfBounds,
    InvalidClaimIndex,
    TooManyClaims,
    ProfileLockedActiveGame,
}