use anchor_lang::prelude::*;
use crate::constants::{FEE_BPS_DENOM, MAX_TICKETS_PER_PLAYER};
use crate::errors::IC42NErrorCode;
use crate::state::config::Config;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    /// Global Config PDA.
    /// Only the `authority` stored in Config is allowed to update it.
    #[account(
        mut,
        seeds = [Config::SEED],
        bump = config.bump,
        has_one = authority @ IC42NErrorCode::Unauthorized
    )]
    pub config: Account<'info, Config>,

    /// Current program authority.
    ///
    /// Must match `config.authority` due to the `has_one` constraint above.
    pub authority: Signer<'info>,
}


/// Arguments for updating one or more fields of a given tier.
///
/// All fields are optional:
/// - If a field is `None`, the existing value is left unchanged.
/// - `tier_id` is used to locate the tier inside `Config.tiers`.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TierUpdateArgs {
    /// Numeric ID of the tier to update (must match an existing TierSettings.tier_id).
    pub tier_id: u8,

    /// If provided, sets the active flag (0 or 1).
    pub active: Option<u8>,

    /// New minimum bet in lamports (optional).
    pub min_bet_lamports: Option<u64>,

    /// New maximum bet in lamports (optional).
    pub max_bet_lamports: Option<u64>,

    /// New curve multiplier for this tier (optional).
    pub curve_factor: Option<f32>,

    /// Ticket reward % (in basis points) for this tier (optional).
    /// 1000 = 10% of losers. 0 disables ticket awards.
    pub ticket_reward_bps: Option<u16>,

    /// Hard cap on ticket recipients for this tier (optional).
    pub ticket_reward_max: Option<u16>,

    /// Ticket reward count for this tier (optional).
    pub tickets_rewarded: Option<u8>,
}


/// Updates one or more global configuration parameters.
///
/// - Only callable by the `authority` stored in `Config`.
/// - Any argument set to `None` is left unchanged.
/// - `tier_updates` may be an empty vector (no tier changes).
pub fn update_config_handler(
    ctx: Context<UpdateConfig>,
    pause_bet: Option<u8>,
    pause_withdraw: Option<u8>,
    new_authority: Option<Pubkey>,
    new_fee_vault: Option<Pubkey>,
    new_fee_bps: Option<u16>,
    new_min_fee_bps: Option<u16>,
    new_rollover_fee_step_bps: Option<u16>,
    new_cutoff_slots: Option<u64>,
    new_primary_roll_over_number: Option<u8>,
    tier_updates: Vec<TierUpdateArgs>,
) -> Result<()> {
    let cfg = &mut ctx.accounts.config;

    // ─────────────────────────────────────────────
    // Pause flags
    // ─────────────────────────────────────────────
    if let Some(pause) = pause_bet {
        cfg.pause_bet = if pause == 1 { 1 } else { 0 };
    }
    if let Some(pause) = pause_withdraw {
        cfg.pause_withdraw = if pause == 1 { 1 } else { 0 };
    }

    // ─────────────────────────────────────────────
    // Authority rotation
    // ─────────────────────────────────────────────
    if let Some(new_auth) = new_authority {
        require!(new_auth != Pubkey::default(), IC42NErrorCode::InvalidAuthorityTarget);
        require!(new_auth != system_program::ID, IC42NErrorCode::InvalidAuthorityTarget);
        require!(new_auth != *ctx.program_id, IC42NErrorCode::InvalidAuthorityTarget);
        require!(new_auth != cfg.key(), IC42NErrorCode::InvalidAuthorityTarget);
        require!(new_auth != cfg.fee_vault, IC42NErrorCode::InvalidAuthorityTarget);
        cfg.authority = new_auth;
    }

    // ─────────────────────────────────────────────
    // Fee vault update
    // ─────────────────────────────────────────────
    if let Some(new_vault) = new_fee_vault {
        require!(new_vault != Pubkey::default(), IC42NErrorCode::InvalidFeeVault);
        require!(new_vault != system_program::ID, IC42NErrorCode::InvalidFeeVault);
        require!(new_vault != *ctx.program_id, IC42NErrorCode::InvalidFeeVault);
        require!(new_vault != cfg.key(), IC42NErrorCode::InvalidFeeVault);
        require!(new_vault != cfg.authority, IC42NErrorCode::InvalidFeeVault);
        cfg.fee_vault = new_vault;
    }

    // ─────────────────────────────────────────────
    // Misc globals
    // ─────────────────────────────────────────────
    if let Some(cutoff_slots) = new_cutoff_slots {
        require!(cutoff_slots > 20, IC42NErrorCode::InvalidCutOffNumber);
        cfg.bet_cutoff_slots = cutoff_slots;
    }

    if let Some(roll_over_number) = new_primary_roll_over_number {
        require!(roll_over_number <= 9, IC42NErrorCode::InvalidRollOverNumber);
        cfg.primary_roll_over_number = roll_over_number;
    }

    // ─────────────────────────────────────────────
    // Tier updates (patch in-place)
    // ─────────────────────────────────────────────
    for update in tier_updates.into_iter() {
        let tier = cfg
            .tiers
            .iter_mut()
            .find(|t| t.tier_id == update.tier_id)
            .ok_or(IC42NErrorCode::UnknownTier)?;

        if let Some(active) = update.active {
            require!(active <= 1, IC42NErrorCode::InvalidTierFlag);
            tier.active = active;
        }

        let mut changed_min_or_max = false;

        if let Some(min_bet) = update.min_bet_lamports {
            tier.min_bet_lamports = min_bet;
            changed_min_or_max = true;
        }
        if let Some(max_bet) = update.max_bet_lamports {
            tier.max_bet_lamports = max_bet;
            changed_min_or_max = true;
        }

        if let Some(curve) = update.curve_factor {
            require!(curve.is_finite(), IC42NErrorCode::InvalidCurveValue);
            if tier.active == 1 {
                require!(curve > 0.0, IC42NErrorCode::InvalidCurveValue);
            }
            tier.curve_factor = curve;
        }

        if tier.active == 1 || changed_min_or_max {
            require!(
                tier.min_bet_lamports < tier.max_bet_lamports,
                IC42NErrorCode::InvalidTierBounds
            );
        }

        // Ticket config (use effective_bps so max validation matches caller intent)
        let mut effective_bps = tier.ticket_reward_bps;
        if let Some(bps) = update.ticket_reward_bps {
            require!(bps <= FEE_BPS_DENOM as u16, IC42NErrorCode::InvalidTicketBps);
            tier.ticket_reward_bps = bps;
            effective_bps = bps;
        }

        if let Some(max) = update.ticket_reward_max {
            if effective_bps > 0 {
                require!(max > 0, IC42NErrorCode::InvalidTicketMax);
            }
            tier.ticket_reward_max = max;
        }

        if let Some(tickets) = update.tickets_rewarded {
            require!(
                tickets as u32 <= MAX_TICKETS_PER_PLAYER,
                IC42NErrorCode::InvalidTicketAmount
            );
            tier.tickets_per_recipient = tickets;
        }
    }

    // ─────────────────────────────────────────────
    // Fees: compute effective -> validate -> apply ONCE
    // ─────────────────────────────────────────────
    let effective_base_fee = new_fee_bps.unwrap_or(cfg.base_fee_bps);
    let effective_min_fee  = new_min_fee_bps.unwrap_or(cfg.min_fee_bps);
    let effective_step_fee = new_rollover_fee_step_bps.unwrap_or(cfg.rollover_fee_step_bps);
    let effective_authority = new_authority.unwrap_or(cfg.authority);
    let effective_fee_vault = new_fee_vault.unwrap_or(cfg.fee_vault);

    require!(effective_authority != effective_fee_vault, IC42NErrorCode::AuthorityCannotEqualFeeVault);
    require!(effective_base_fee <= FEE_BPS_DENOM as u16, IC42NErrorCode::InvalidFee);
    require!(effective_min_fee  <= FEE_BPS_DENOM as u16, IC42NErrorCode::InvalidMinimumFee);
    require!(effective_step_fee <= FEE_BPS_DENOM as u16, IC42NErrorCode::InvalidFeeStep);

    // key invariant
    require!(effective_min_fee <= effective_base_fee, IC42NErrorCode::InvalidFeeConfig);

    // Step should never exceed the base fee
    // (prevents "first rollover drops straight to min" surprises)
    require!(effective_step_fee <= effective_base_fee, IC42NErrorCode::InvalidFeeStep);

    // ----- apply ONLY the fields that were provided -----
    if let Some(v) = new_fee_bps { cfg.base_fee_bps = v; }
    if let Some(v) = new_min_fee_bps { cfg.min_fee_bps = v; }
    if let Some(v) = new_rollover_fee_step_bps { cfg.rollover_fee_step_bps = v; }

    if let Some(v) = new_authority { cfg.authority = v; }
    if let Some(v) = new_fee_vault { cfg.fee_vault = v; }
    
    Ok(())
}