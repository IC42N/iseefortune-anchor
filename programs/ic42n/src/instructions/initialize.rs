use anchor_lang::prelude::*;
use crate::constants::{TIER1_MAX, TIER1_MIN, TIER2_MAX, TIER2_MIN, TIER3_MAX, TIER3_MIN};
use crate::state::*;
use crate::state::tiers::{TierSettings};
use crate::state::treasury::Treasury;

#[derive(Accounts)]
#[instruction(fee_bps: u16, tier: u8)]
pub struct Initialize<'info> {
    /// Global config PDA.
    #[account(
        init,
        payer = authority,
        space = 8 + Config::SIZE,
        seeds = [Config::SEED],
        bump
    )]
    pub config: Account<'info, Config>,

    /// Live feed PDA for the provided tier.
    #[account(
        init,
        payer = authority,
        space = 8 + LiveFeed::SIZE,
        seeds = [LiveFeed::SEED_PREFIX, &[tier]],
        bump
    )]
    pub live_feed: Account<'info, LiveFeed>,

    /// Treasury PDA holding protocol lamports.
    #[account(
        init,
        payer = authority,
        space = 8 + Treasury::SIZE,
        seeds = [Treasury::SEED],
        bump
    )]
    pub treasury: Account<'info, Treasury>,

    /// CHECK: Fee destination account; validated later via `address = config.fee_vault`.
    pub fee_vault: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_handler(ctx: Context<Initialize>, fee_bps: u16, tier: u8) -> Result<()> {
    let authority_key = ctx.accounts.authority.key();
    let cfg = &mut ctx.accounts.config;
    let live = &mut ctx.accounts.live_feed;

    let cutoff = 300u64;

    // Fetch system epoch info
    let clock = Clock::get()?;
    let current_epoch = clock.epoch;

    // ────────────────────────────────────────────────
    // Initialize config
    // ────────────────────────────────────────────────
    cfg.pause_bet = 0;
    cfg.pause_withdraw = 0;

    cfg.authority = authority_key;
    cfg.fee_vault = ctx.accounts.fee_vault.key();
    cfg.base_fee_bps = fee_bps;
    cfg.bet_cutoff_slots = cutoff;
    cfg.started_at = clock.unix_timestamp;
    cfg.started_epoch = current_epoch;
    cfg.primary_roll_over_number = 0;

    cfg.bump = ctx.bumps.config;
    cfg.min_fee_bps = 200;
    cfg.rollover_fee_step_bps = 100;
    cfg._reserved = [0; 16];

    cfg.tiers = [
        // Tier 1: 0.01 – 1 SOL
        TierSettings {
            tier_id: 1,
            active: 1,
            min_bet_lamports: TIER1_MIN,
            max_bet_lamports: TIER1_MAX,
            curve_factor: 0.9,
            ticket_reward_bps: 1_000,   // 10% of losers
            ticket_reward_max: 100,     // cap 100 recipients
            tickets_per_recipient: 1,
            _reserved: [0; 10],

        },
        // Tier 2: 1 – 10 SOL
        TierSettings {
            tier_id: 2,
            active: 0,
            min_bet_lamports: TIER2_MIN,
            max_bet_lamports: TIER2_MAX,
            curve_factor: 0.9,
            ticket_reward_bps: 1_000,   // 10% of losers
            ticket_reward_max: 100,     // cap 100 recipients
            tickets_per_recipient: 1,
            _reserved: [0; 10],
        },
        // Tier 3: 10 – 100 SOL
        TierSettings {
            tier_id: 3,
            active: 0,
            min_bet_lamports: TIER3_MIN,
            max_bet_lamports: TIER3_MAX,
            curve_factor: 0.9,
            ticket_reward_bps: 1_000,   // 10% of losers
            ticket_reward_max: 100,     // cap 100 recipients
            tickets_per_recipient: 1,
            _reserved: [0; 10],
        },
        // Tier 4: placeholder / inactive tier
        TierSettings {
            tier_id: 4,
            active: 0,
            min_bet_lamports: 0,
            max_bet_lamports: 0,
            curve_factor: 0.0,
            ticket_reward_bps: 0,
            ticket_reward_max: 0,
            tickets_per_recipient: 1,
            _reserved: [0; 10],
        },
        // Tier 5: placeholder / inactive tier
        TierSettings {
            tier_id: 5,
            active: 0,
            min_bet_lamports: 0,
            max_bet_lamports: 0,
            curve_factor: 0.0,
            ticket_reward_bps: 0,
            ticket_reward_max: 0,
            tickets_per_recipient: 1,
            _reserved: [0; 10],
        },
    ];

    // ────────────────────────────────────────────────
    // Initialize live feed
    // ────────────────────────────────────────────────
    live.init_new(
        current_epoch,
        cutoff,
        tier,
        ctx.accounts.treasury.key(),
        ctx.bumps.live_feed,
        fee_bps
    );


    // ────────────────────────────────────────────────
    // Initialize treasury
    // ────────────────────────────────────────────────
    let treasury = &mut ctx.accounts.treasury;
    treasury.authority = authority_key;
    treasury.tier = 0;
    treasury.bump = ctx.bumps.treasury;
    treasury.total_in_lamports = 0;
    treasury.total_out_lamports = 0;
    treasury.total_fees_withdrawn = 0;
    treasury.version = 1;
    treasury._reserved = [0; 32];

    Ok(())
}