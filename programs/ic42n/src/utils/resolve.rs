// If the winning number is 0 or is the current secondary rollover number,
// then we keep the same rollover number. Else, we use the winning number as the new rollover number.
pub fn get_next_rollover_number(winning_number: u8,current_rollover: u8) -> u8 {
    let next_secondary_rollover: u8 = if winning_number == 0 || winning_number == current_rollover {
        current_rollover
    } else {
        winning_number
    };
    next_secondary_rollover
}


/// Calculates the next fee BPS after a rollover.
///
/// Rules:
/// - Fee decreases by `rollover_step_bps` on each rollover
/// - Fee will NEVER go below `min_fee_bps`
/// - Safe against underflow
///
/// Example if min is set to 300
///   current = 500, a step = 100, min = 300 → 400
///   current = 400, a step = 100, min = 300 → 300
///   current = 300, a step = 100, min = 300 → 300
pub fn next_fee_bps_on_rollover(
    current_fee_bps: u16,
    rollover_step_bps: u16,
    min_fee_bps: u16,
) -> u16 {
    // Defensive: ensure config makes sense
    if rollover_step_bps == 0 {
        return current_fee_bps.max(min_fee_bps);
    }
    
    let current = current_fee_bps.max(min_fee_bps);
    let decreased = current.saturating_sub(rollover_step_bps);
    decreased.max(min_fee_bps)
}