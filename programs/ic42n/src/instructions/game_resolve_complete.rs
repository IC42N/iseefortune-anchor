use anchor_lang::prelude::*;
use crate::errors::IC42NErrorCode;
use crate::state::*;
use crate::state::treasury::Treasury;
use crate::constants::*;
use crate::utils::resolve::get_next_rollover_number;

///Cannot resolve the same epoch twice:
// ResolvedGame PDA is created once via InitResolvedGame,
// and this instruction requires status == Resolving and then sets it to Resolved.
#[derive(Accounts)]
#[instruction(epoch: u64, tier: u8)]
pub struct CompleteResolveGame<'info> {
    /// Global config (for authority + fee_bps etc.)
    #[account(
        mut,
        seeds = [Config::SEED],
        bump = config.bump,
        has_one = authority @ IC42NErrorCode::Unauthorized
    )]
    pub config: Account<'info, Config>,

    /// Live feed for this tier (must match epoch & tier)
    #[account(
        mut,
        seeds = [LiveFeed::SEED_PREFIX, &[tier]],
        bump = live_feed.bump,
    )]
    pub live_feed: Account<'info, LiveFeed>,

    /// ResolvedGame PDA for this epoch & tier – MUST already exist
    #[account(
        mut,
        seeds = [ResolvedGame::SEED_PREFIX, epoch.to_le_bytes().as_ref(), &[tier]],
        bump = resolved_game.bump,
        // Belt & suspenders: make sure stored epoch/tier match args
        constraint = resolved_game.epoch == epoch @ IC42NErrorCode::EpochMismatch,
        constraint = resolved_game.tier == tier   @ IC42NErrorCode::TierMismatch,
    )]
    pub resolved_game: Account<'info, ResolvedGame>,

    /// Treasury holding the SOL for all games
    #[account(
        mut,
        seeds = [Treasury::SEED],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    /// Fee vault where protocol fees are accumulated
    #[account(
        mut,
        address = config.fee_vault @ IC42NErrorCode::InvalidFeeVault
    )]
    pub fee_vault: SystemAccount<'info>,

    /// Authority account that is allowed to resolve games
    #[account(mut, address = config.authority @ IC42NErrorCode::Unauthorized)]
    pub authority: Signer<'info>,
    // No system_program needed anymore – we’re not init-ing anything here.
}


/// ---------------------------------------------------------------------------
/// resolve_game_handler
///
/// Called once per (epoch, tier) after an epoch ends and Lambda has computed
/// winners + Merkle root.
///
/// RULES:
///   - If there ARE winners:
///        • Charge protocol fee from the gross pot
///        • Remaining net pot = prize pool for claims
///        • No carry-over (current simple model)
///
///   - If there are NO winners:
///        • **NO protocol fee is taken**
///        • The **entire gross pot** rolls over to the next epoch
///        • All bets + lamports are treated as carry-over
///
/// SECURITY:
///   - Only canonical Config PDA + authority can call this
///   - Cannot resolve same epoch twice (ResolvedGame PDA uses `init` + seeds)
///   - Fee and net pot are recomputed on-chain and must match Lambda’s values
/// ---------------------------------------------------------------------------
pub fn complete_resolve_game_handler(
    ctx: Context<CompleteResolveGame>,
    epoch: u64,
    tier: u8,

    // Proposed by Lambda, but NOT trusted — we recompute on-chain.
    protocol_fee_lamports: u64,
    net_prize_pool: u64,

    // Winners + Merkle data
    total_winners: u32,
    merkle_root: [u8; 32],
    results_uri: [u8; 128],
) -> Result<()> {
    // Shorthand for accounts
    let config    = &mut ctx.accounts.config;
    let live      = &mut ctx.accounts.live_feed;
    let treasury  = &mut ctx.accounts.treasury;
    let fee_vault = &mut ctx.accounts.fee_vault;
    let game      = &mut ctx.accounts.resolved_game;

    // -----------------------------------------------------------------------
    // 0) Must have real action this epoch
    // -----------------------------------------------------------------------
    require!(live.total_bets > 0, IC42NErrorCode::NoBetsToResolve);
    require!(live.total_lamports > 0, IC42NErrorCode::NoBetsToResolve);

    // -----------------------------------------------------------------------
    // 1) Validate epoch + tier alignment and status
    // -----------------------------------------------------------------------
    let clock        = Clock::get()?;
    let current_epoch = clock.epoch;
    let resolved_ts   = clock.unix_timestamp;

    // The epoch (argument passed) must match the LiveFeed epoch
    require_eq!(live.epoch, epoch, IC42NErrorCode::EpochMismatch);

    // Epoch must already be completed
    require!(live.epoch < current_epoch, IC42NErrorCode::EpochNotComplete);

    // Tier consistency with value passed
    require_eq!(live.tier, tier, IC42NErrorCode::TierMismatch);

    // Tier must be valid + active in Config
    let tier_cfg = config.get_tier_settings(tier)?;
    require!(tier_cfg.is_active(), IC42NErrorCode::InactiveTier);

    // Results URI must not be empty (all zero bytes)
    let has_nonzero_uri_byte = results_uri.iter().any(|b| *b != 0);
    require!(has_nonzero_uri_byte, IC42NErrorCode::EmptyResultsUri);

    // ResolvedGame must be in a RESOLVING state (single-writer lock)
    require!(
        game.status == GameStatus::Processing as u8,
        IC42NErrorCode::GameNotInResolvingState
    );

    // -----------------------------------------------------------------------
    // 2) Recompute fee + net pot on-chain
    // -----------------------------------------------------------------------
    let gross_pot = live.total_lamports;
    let fee_bps   = live.current_fee_bps as u64;

    let (expected_fee, expected_net) = if total_winners == 0 {
        // No winners → the protocol taker fee = 0, full pot carries over.
        (0u64, gross_pot)
    } else {
        // Winners exist → normal fee logic applies.
        let fee = gross_pot
            .checked_mul(fee_bps)
            .ok_or(IC42NErrorCode::MathOverflow)?
            / FEE_BPS_DENOM;

        let net = gross_pot
            .checked_sub(fee)
            .ok_or(IC42NErrorCode::MathOverflow)?;

        (fee, net)
    };

    // Lambda inputs must match canonical on-chain computation
    require_eq!(
        expected_fee,
        protocol_fee_lamports,
        IC42NErrorCode::InvalidFee
    );
    require_eq!(
        expected_net,
        net_prize_pool,
        IC42NErrorCode::InvalidPotBreakdown
    );

    // Sanity: fee + net should never exceed gross
    let combined = expected_fee
        .checked_add(expected_net)
        .ok_or(IC42NErrorCode::MathOverflow)?;
    require!(combined <= gross_pot, IC42NErrorCode::InvalidNetPoolPlusNet);

    // -----------------------------------------------------------------------
    // 3) Compute carry-over lamports + bets
    // If there are NO winners, then we carry over the pot and bets
    // If there are winners, carry-over is 0. (reset)
    // -----------------------------------------------------------------------
    let carry_over_lamports_for_next: u64 = if total_winners == 0 {
        expected_net // == gross_pot in this branch
    } else {
        0
    };

    require!(
        carry_over_lamports_for_next <= expected_net,
        IC42NErrorCode::InvalidCarryOver
    );

    let carry_over_bets_for_next: u32 = if total_winners == 0 {
        live.total_bets
    } else {
        0
    };

    let carry_over_bets_per_number: [u32; 10] = if total_winners == 0 {
        live.bets_per_number
    } else {
        [0u32; 10]
    };

    let carry_over_lamports_per_number: [u64; 10] = if total_winners == 0 {
        live.lamports_per_number
    } else {
        [0u64; 10]
    };


    // -----------------------------------------------------------------------
    // 4) Move protocol fee (ONLY if there are winners)
    // -----------------------------------------------------------------------
    let treasury_balance = **treasury.to_account_info().lamports.borrow();

    let balance_after_fee = treasury_balance
        .checked_sub(expected_fee)
        .ok_or(IC42NErrorCode::MathOverflow)?;

    // After fee (if any), treasury must still hold at least the net pot.
    require!(
        balance_after_fee >= expected_net,
        IC42NErrorCode::InsufficientTreasuryBalance
    );

    if expected_fee > 0 {
        **treasury
            .to_account_info()
            .try_borrow_mut_lamports()? -= expected_fee;
        **fee_vault
            .to_account_info()
            .try_borrow_mut_lamports()? += expected_fee;

        treasury.total_fees_withdrawn = treasury
            .total_fees_withdrawn
            .checked_add(expected_fee)
            .ok_or(IC42NErrorCode::MathOverflow)?;
    }

    // -----------------------------------------------------------------------
    // 5) Populate the ResolvedGame snapshot PDA (final state)
    // -----------------------------------------------------------------------


    // These are genuinely "final result" fields – it's correct to set them here.
    game.total_bets          = live.total_bets;
    game.carry_over_bets     = carry_over_bets_for_next;

    game.protocol_fee_lamports = expected_fee;
    game.fee_bps                = live.current_fee_bps;
    game.net_prize_pool        = expected_net;

    // Inbound from previous epoch(s)
    game.carry_in_lamports  = live.carried_over_lamports;
    // Outbound to next epoch
    game.carry_out_lamports = carry_over_lamports_for_next;

    game.total_winners   = total_winners;
    game.claimed_winners = 0;

    let bitmap_bytes = ((total_winners as usize) + 7) / 8;
    require!(
        bitmap_bytes <= ResolvedGame::MAX_BITMAP_LEN,
        IC42NErrorCode::TooManyWinners
    );
    game.claimed_bitmap  = vec![0u8; bitmap_bytes];

    game.merkle_root = merkle_root;
    game.results_uri = results_uri;
    game.resolved_at = resolved_ts;

    // Update processing metadata / state machine fields
    game.status            = GameStatus::Resolved as u8;
    game.last_updated_slot = clock.slot;
    game.last_updated_ts   = resolved_ts;

    // -----------------------------------------------------------------------
    // 7) Reset LiveFeed for the next epoch
    // -----------------------------------------------------------------------
    let next_epoch = live.epoch + 1;


    // If the winning number is 0 or is the current secondary rollover number,
    // then we keep the same rollover number. Else, we use the winning number as the new rollover number.
    let next_secondary_rollover: u8 = get_next_rollover_number(game.winning_number,live.secondary_rollover_number);

    live.reset_for_new_epoch(
        next_epoch,
        config.bet_cutoff_slots,
        carry_over_lamports_for_next,
        carry_over_bets_for_next,
        carry_over_lamports_per_number,
        carry_over_bets_per_number,
        next_secondary_rollover,
        config.base_fee_bps
    );

    Ok(())
}