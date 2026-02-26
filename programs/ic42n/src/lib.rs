use anchor_lang::prelude::*;
use solana_security_txt::security_txt;

// -----------------------------------------------------------------------------
// Program ID
// -----------------------------------------------------------------------------
declare_id!("ic429goRDdS7BXEDYr2nZeAYMxtT6FL3AsB3sneaSu7");

security_txt! {
    name: "ISeeFortune",
    project_url: "https://iseefortune.com",
    source_code: "https://github.com/IC42N/iseefortune-anchor",
    contacts: "mailto:contact@iseefortune.com, https://twitter.com/IcFortune",
    policy: "https://github.com/IC42N/iseefortune-anchor/blob/main/SECURITY.md",
    preferred_languages: "en"
}


// -----------------------------------------------------------------------------
// Modules
// -----------------------------------------------------------------------------
pub mod state;
pub mod instructions;
pub mod utils;
pub mod errors;
pub mod constants;

use instructions::*;

// -----------------------------------------------------------------------------
// Program Entrypoints
// -----------------------------------------------------------------------------
#[program]
pub mod ic42n {
    use super::*;

    use crate::instructions::game_resolve_rollover::complete_rollover_game_handler;
    use crate::instructions::profile_close::close_player_profile_handler;

    // -------------------------------------------------------------------------
    // initialize
    // -------------------------------------------------------------------------
    pub fn initialize(ctx: Context<Initialize>, fee_bps: u16, tier: u8) -> Result<()> {
        initialize_handler(ctx, fee_bps, tier)
    }

    // -------------------------------------------------------------------------
    // init_tier_live_feed
    // -------------------------------------------------------------------------
    pub fn init_tier_live_feed(ctx: Context<InitTierLiveFeed>, tier: u8) -> Result<()> {
        init_tier_live_feed_handler(ctx, tier)
    }

    // -------------------------------------------------------------------------
    // reset_live_feed
    // -------------------------------------------------------------------------
    pub fn reset_live_feed(ctx: Context<ResetLiveFeed>, tier: u8, rollover: u8) -> Result<()> {
        reset_live_feed_handler(ctx, tier, rollover)
    }

    // -------------------------------------------------------------------------
    // close_tier_live_feed
    // -------------------------------------------------------------------------
    pub fn close_tier_live_feed(ctx: Context<CloseFeed>, tier: u8) -> Result<()> {
        close_tier_live_feed_handler(ctx, tier)
    }

    // -------------------------------------------------------------------------
    // update_config
    // -------------------------------------------------------------------------
    pub fn update_config(
        ctx: Context<UpdateConfig>,
        pause_bet: Option<u8>,
        pause_withdraw: Option<u8>,
        new_authority: Option<Pubkey>,
        new_fee_vault: Option<Pubkey>,
        new_fee_bps: Option<u16>,
        new_min_fee_bps: Option<u16>,
        new_rollover_fee_step_bps: Option<u16>,
        new_cutoff_slots: Option<u64>,
        new_roll_over_number: Option<u8>,
        tier_updates: Vec<TierUpdateArgs>,
    ) -> Result<()> {
        update_config_handler(
            ctx,
            pause_bet,
            pause_withdraw,
            new_authority,
            new_fee_vault,
            new_fee_bps,
            new_min_fee_bps,
            new_rollover_fee_step_bps,
            new_cutoff_slots,
            new_roll_over_number,
            tier_updates,
        )
    }

    // -------------------------------------------------------------------------
    // emergency_pause_all
    // -------------------------------------------------------------------------
    pub fn emergency_pause_all(ctx: Context<UpdateConfig>) -> Result<()> {
        update_config_handler(
            ctx,
            Some(1),
            Some(1),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            vec![],
        )
    }

    // -------------------------------------------------------------------------
    // update_tier_active
    // -------------------------------------------------------------------------
    pub fn update_tier_active(ctx: Context<UpdateTierActive>, tier_id: u8, active: u8) -> Result<()> {
        update_tier_active_handler(ctx, tier_id, active)
    }

    // =====================================================================
    // NEW PREDICTION ENDPOINTS
    // =====================================================================

    pub fn place_prediction(
        ctx: Context<PlacePrediction>,
        tier: u8,
        prediction_type: u8,
        choice: u32,
        lamports: u64,
    ) -> Result<()> {
        place_prediction_handler(ctx, tier, prediction_type, choice, lamports)
    }

    pub fn change_prediction_number(
        ctx: Context<ChangePredictionNumber>,
        tier: u8,
        new_prediction_type: u8,
        new_choice: u32,
    ) -> Result<()> {
        change_prediction_number_handler(ctx, tier, new_prediction_type, new_choice)
    }

    pub fn increase_prediction(
        ctx: Context<IncreasePrediction>,
        tier: u8,
        additional_lamports: u64,
        choice: u32
    ) -> Result<()> {
        increase_prediction_handler(ctx, tier, additional_lamports, choice)
    }

    // Prediction claim (Prediction-based, leaf binds to selections_mask)
    pub fn claim_prediction(
        ctx: Context<ClaimPrediction>,
        epoch: u64,
        tier: u8,
        index: u32,
        amount: u64,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        claim_prediction_handler(ctx, epoch, tier, index, amount, proof)
    }

    // =====================================================================
    // GAME RESOLUTION / ROLLOVER / CLOSE
    // =====================================================================

    pub fn init_resolved_game(
        ctx: Context<InitResolvedGame>,
        epoch: u64,
        tier: u8,
        winning_number: u8,
        rng_epoch_slot_used: u64,
        rng_blockhash_used: [u8; 32],
    ) -> Result<()> {
        init_resolved_game_handler(ctx, epoch, tier, winning_number, rng_epoch_slot_used, rng_blockhash_used)
    }

    pub fn begin_resolve_game(ctx: Context<ReprocessResolveGame>, epoch: u64, tier: u8) -> Result<()> {
        reprocessing_resolve_game_handler(ctx, epoch, tier)
    }

    pub fn complete_resolve_game(
        ctx: Context<CompleteResolveGame>,
        epoch: u64,
        tier: u8,
        protocol_fee_lamports: u64,
        net_prize_pool: u64,
        total_winners: u32,
        merkle_root: [u8; 32],
        results_uri: [u8; 128],
    ) -> Result<()> {
        complete_resolve_game_handler(
            ctx,
            epoch,
            tier,
            protocol_fee_lamports,
            net_prize_pool,
            total_winners,
            merkle_root,
            results_uri,
        )
    }

    pub fn complete_rollover_game(
        ctx: Context<ResolvedGameRollover>,
        epoch: u64,
        tier: u8,
        winning_number: u8,
        rng_epoch_slot_used: u64,
        rng_blockhash_used: [u8; 32],
    ) -> Result<()> {
        complete_rollover_game_handler(ctx, epoch, tier, winning_number, rng_epoch_slot_used, rng_blockhash_used)
    }

    pub fn close_resolved_game(ctx: Context<CloseGame>, epoch: u64, tier: u8) -> Result<()> {
        close_resolved_game_handler(ctx, epoch, tier)
    }

    // -------------------------------------------------------------------------
    // award tickets
    // -------------------------------------------------------------------------
    pub fn award_ticket_auto(ctx: Context<AutoAwardTicket>, tier: u8) -> Result<()> {
        award_ticket_auto_handler(ctx, tier)
    }

    pub fn award_ticket_manual(ctx: Context<ManualAwardTicket>, tickets: u32) -> Result<()> {
        award_ticket_manual_handler(ctx, tickets)
    }

    // -------------------------------------------------------------------------
    // close_profile
    // -------------------------------------------------------------------------
    pub fn close_profile(ctx: Context<ClosePlayerProfile>) -> Result<()> {
        close_player_profile_handler(ctx)
    }
}