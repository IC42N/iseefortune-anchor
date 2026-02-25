use anchor_lang::prelude::msg;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::sysvar::epoch_schedule::EpochSchedule;
use anchor_lang::solana_program::sysvar::Sysvar;
use crate::state::tiers::TierSettings;


/// Returns true if `amount` is within the tier's [min, max] bounds.
pub fn is_amount_in_tier(amount: u64, tier: &TierSettings) -> bool {
    amount >= tier.min_bet_lamports && amount <= tier.max_bet_lamports
}

/// Returns true if betting is still open given a minimum remaining-slots cutoff.
///
/// This is used to prevent bets near the end of an epoch, where off-chain
/// resolution may be imminent.
pub fn is_betting_still_open(min_slots_cutoff: u64) -> bool {

    let Ok(clock) = Clock::get() else {
        // If sysvars are unavailable (unexpected), fail closed.
        return false;
    };

    let Ok(schedule) = EpochSchedule::get() else {
        // If sysvars are unavailable (unexpected), fail closed.
        return false;
    };

    let current_slot = clock.slot;
    let epoch_from_slot = schedule.get_epoch(current_slot);
    let slots_per_epoch = schedule.slots_per_epoch;

    // Devnet Configuration: Devnet was deliberately configured with a shorter
    // initial epoch length, reportedly around 8,192 slots, to allow for
    // faster epoch transitions. If they do not match, then we skip the cutoff for devnet.
    // We must test and confirm once on the mainnet.
    if epoch_from_slot != clock.epoch {
        msg!(
            "Epoch mismatch: clock.epoch={} vs schedule.get_epoch(slot)={}. \
             slots_per_epoch={}.
             Skipping cutoff and allowing bet.",
            clock.epoch,
            epoch_from_slot,
            slots_per_epoch
        );
        return true;
    }
    
    // Compute timing within this epoch (now that we know it's consistent)
    let first_slot = schedule.get_first_slot_in_epoch(epoch_from_slot);
    let slots_in_epoch = schedule.get_slots_in_epoch(epoch_from_slot);
    let last_slot = first_slot + slots_in_epoch - 1;
    let slots_remaining = last_slot.saturating_sub(current_slot);

    // ── Essential logging only ─────────────────────────────
    msg!(
        "Betting cutoff check: epoch={} slot={} slots_in_epoch={} slots_remaining={} cutoff={}",
        epoch_from_slot,
        current_slot,
        slots_in_epoch,
        slots_remaining,
        min_slots_cutoff
    );

    let open = slots_remaining > min_slots_cutoff;
    if open {
        msg!("Betting ALLOWED");
    } else {
        msg!("Betting CLOSED");
    }
    open
}

