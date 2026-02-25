use anchor_lang::prelude::Account;
use crate::constants::MAX_TICKETS_PER_PLAYER;
use crate::state::player_profile::{PlayerProfile};

pub fn award_tickets_to_profile(
    profile: &mut Account<PlayerProfile>,
    tickets: u32,
) {
    let new_total = profile
        .tickets_available
        .saturating_add(tickets)
        .min(MAX_TICKETS_PER_PLAYER); // or whatever cap
    profile.tickets_available = new_total;
}