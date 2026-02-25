use anchor_lang::prelude::*;

use crate::errors::IC42NErrorCode;
use crate::state::*;
use crate::state::player_profile::PlayerProfile;
use crate::utils::betting::is_betting_still_open;
use crate::utils::prediction::{
    derive_prediction_selections,
    retract_per_number_from_live,
    apply_per_number_to_live,
};

#[derive(Accounts)]
#[instruction(tier: u8, new_prediction_type: u8, new_choice: u32)]
pub struct ChangePredictionNumber<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [LiveFeed::SEED_PREFIX, &[tier]],
        bump = live_feed.bump,
    )]
    pub live_feed: Account<'info, LiveFeed>,

    #[account(
        mut,
        seeds = [
            Prediction::SEED_PREFIX,
            player.key().as_ref(),
            &live_feed.first_epoch_in_chain.to_le_bytes(),
            &[tier],
        ],
        bump,
        has_one = player @ IC42NErrorCode::Unauthorized,
    )]
    pub prediction: Box<Account<'info, Prediction>>,

    #[account(
        mut,
        seeds = [PlayerProfile::SEED_PREFIX, player.key().as_ref()],
        bump = profile.bump,
        has_one = player @ IC42NErrorCode::Unauthorized,
    )]
    pub profile: Box<Account<'info, PlayerProfile>>,
}

pub fn change_prediction_number_handler(
    ctx: Context<ChangePredictionNumber>,
    tier: u8,
    new_prediction_type: u8,
    new_choice: u32,
) -> Result<()> {
    let pred = &mut ctx.accounts.prediction;
    let profile = &mut ctx.accounts.profile;
    let live = &mut ctx.accounts.live_feed;

    let clock = Clock::get()?;
    let current_epoch = clock.epoch;


    pred.assert_invariant()?;


    // ─────────────────────────────
    // Epoch / chain / tier checks
    // ─────────────────────────────
    require!(current_epoch == live.epoch, IC42NErrorCode::EpochMismatch);
    require!(pred.has_claimed == 0, IC42NErrorCode::AlreadyClaimed);

    require_eq!(
        pred.game_epoch,
        live.first_epoch_in_chain,
        IC42NErrorCode::EpochMismatch
    );

    require!(
        pred.epoch >= live.first_epoch_in_chain && pred.epoch <= live.epoch,
        IC42NErrorCode::EpochMismatch
    );

    require_eq!(pred.tier, tier, IC42NErrorCode::TierMismatch);
    require_eq!(live.tier, tier, IC42NErrorCode::TierMismatch);

    // ─────────────────────────────
    // Cutoff + tickets
    // ─────────────────────────────
    require!(
        is_betting_still_open(live.bet_cutoff_slots),
        IC42NErrorCode::BettingClosed
    );

    require!(
        profile.tickets_available > 0,
        IC42NErrorCode::NoChangeTickets
    );

    // ─────────────────────────────
    // Derive NEW selection set
    // ─────────────────────────────
    let blocked = live.secondary_rollover_number;
    let (new_count, new_selections, new_mask) =
        derive_prediction_selections(new_prediction_type, new_choice, blocked)?;
    
    // Avoid no-op change (same coverage set)
    require!(pred.selections_mask != new_mask, IC42NErrorCode::NoOpChange);

    // Do NOT allow changing selection_count here (avoids refunds/extra payments)
    require!(new_count == pred.selection_count,IC42NErrorCode::InvalidChoiceCount);

    // ─────────────────────────────
    // Retract OLD per-number lamports from live feed
    // ─────────────────────────────
    retract_per_number_from_live(
        live,
        pred.lamports_per_number,
        &pred.selections,
        pred.selection_count
    )?;

    // ─────────────────────────────
    // Update bets_per_number based on mask diff (your logic kept)
    // ─────────────────────────────
    let old_mask = pred.selections_mask;
    let removed = old_mask & !new_mask;
    let added = new_mask & !old_mask;

    for n in 1u8..=9u8 {
        let bit = 1u16 << n;
        let idx = n as usize;

        require!(
            idx < live.bets_per_number.len() && idx < live.lamports_per_number.len(),
            IC42NErrorCode::InvalidBetNumber
        );

        if (removed & bit) != 0 {
            require!(live.bets_per_number[idx] >= 1, IC42NErrorCode::InvalidLiveFeedState);
            live.bets_per_number[idx] = live.bets_per_number[idx]
                .checked_sub(1)
                .ok_or(IC42NErrorCode::MathOverflow)?;
        }

        if (added & bit) != 0 {
            live.bets_per_number[idx] = live.bets_per_number[idx]
                .checked_add(1)
                .ok_or(IC42NErrorCode::MathOverflow)?;
        }
    }

    // ─────────────────────────────
    // Update Prediction fields
    // ─────────────────────────────
    pred.prediction_type = new_prediction_type;
    pred.selection_count = new_count;
    pred.selections = new_selections;
    pred.selections_mask = new_mask;

    pred.changed_count = pred.changed_count.saturating_add(1);
    pred.last_updated_at_ts = clock.unix_timestamp;

    // Consume ticket
    profile.tickets_available = profile.tickets_available.saturating_sub(1);

    // ─────────────────────────────
    // Apply NEW per-number lamports to live feed
    // ─────────────────────────────
    apply_per_number_to_live(
        live,
        pred.lamports_per_number,
        &pred.selections,
        pred.selection_count
    )?;

    // total pot does NOT change in a pure "change numbers" action
    // live.total_lamports unchanged

    Ok(())
}