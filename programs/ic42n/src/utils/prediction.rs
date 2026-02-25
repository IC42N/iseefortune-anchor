use anchor_lang::{err, require};
use crate::errors::IC42NErrorCode;
use crate::state::{LiveFeed, Prediction};

/// Derive the exact selection set and mask from prediction_type and encoded choice.
///
/// New choice encoding:
/// - `choice` is an u32 whose decimal digits represent the selected numbers.
///   Examples: 3 => [3], 37 => [3,7], 356 => [3,5,6], 7895 => [5,7,8,9] (canonicalized)
///
/// Blocked rules:
/// - 0 is never allowed
/// - digits must be 1~9
/// - `blocked_secondary` is excluded (last winning number)
/// - no duplicates allowed
///
/// For some prediction types, `choice` is interpreted as a mode:
/// - HIGH_LOW: choice = 0 (low) or 1 (high), selections are derived from eligible list
/// - EVEN_ODD: choice = 0 (even) or 1 (odd), selections are derived from eligible list
pub fn derive_prediction_selections(
    prediction_type: u8,
    choice: u32,
    blocked_secondary: u8,
) -> anchor_lang::Result<(u8, [u8; 8], u16)> {
    // blocked_secondary must be a real number 1~9
    require!(
        blocked_secondary >= 1 && blocked_secondary <= 9,
        IC42NErrorCode::InvalidBetNumber
    );

    // Build eligible numbers: 1..=9 excluding blocked_secondary
    let mut eligible: Vec<u8> = Vec::with_capacity(8);
    for n in 1u8..=9u8 {
        if n == blocked_secondary {
            continue;
        }
        eligible.push(n);
    }
    // Should always be exactly 8
    require!(eligible.len() == 8, IC42NErrorCode::InvalidBetNumber);

    let mut out = [0u8; 8];
    let count: u8;

    match prediction_type {
        // ------------------------------------------------------------
        // SINGLE_NUMBER
        // choice must encode exactly 1 digit
        // ------------------------------------------------------------
        x if x == Prediction::TYPE_SINGLE_NUMBER => {
            let (c, arr, mask) = decode_choice_digits(choice, blocked_secondary)?;
            require!(c == 1, IC42NErrorCode::InvalidBetNumber);
            out = arr;
            count = c;
            return Ok((count, out, mask));
        }

        // ------------------------------------------------------------
        // TWO_NUMBERS
        // choice must encode exactly 2 digits
        // ------------------------------------------------------------
        x if x == Prediction::TYPE_TWO_NUMBERS => {
            let (c, arr, mask) = decode_choice_digits(choice, blocked_secondary)?;
            require!(c == 2, IC42NErrorCode::InvalidBetNumber);
            out = arr;
            count = c;
            return Ok((count, out, mask));
        }

        // ------------------------------------------------------------
        // HIGH_LOW
        // choice is a mode: 0=low, 1=high
        // selections derived from an eligible list (already sorted asc)
        // ------------------------------------------------------------
        x if x == Prediction::TYPE_HIGH_LOW => {
            require!(choice == 0 || choice == 1, IC42NErrorCode::InvalidBetNumber);

            if choice == 0 {
                // LOW = first 4 eligible numbers
                for (i, v) in eligible.iter().take(4).enumerate() {
                    out[i] = *v;
                }
            } else {
                // HIGH = last 4 eligible numbers
                for (i, v) in eligible.iter().skip(4).take(4).enumerate() {
                    out[i] = *v;
                }
            }
            count = 4;
        }

        // ------------------------------------------------------------
        // EVEN_ODD
        // choice is a mode: 0=even, 1=odd
        // selections derived from eligible list
        // ------------------------------------------------------------
        x if x == Prediction::TYPE_EVEN_ODD => {
            require!(choice == 0 || choice == 1, IC42NErrorCode::InvalidBetNumber);

            let want_odd = choice == 1;
            let mut idx = 0usize;

            for &v in eligible.iter() {
                let is_odd = (v % 2) == 1;
                if is_odd == want_odd {
                    require!(idx < 8, IC42NErrorCode::InvalidBetNumber);
                    out[idx] = v;
                    idx += 1;
                }
            }

            require!(idx > 0, IC42NErrorCode::InvalidBetNumber);
            count = idx as u8;
        }

        // ------------------------------------------------------------
        // Optional future: MULTI_NUMBER
        // choice encodes 3..=8 digits (if/when you add this type)
        // ------------------------------------------------------------
        x if x == Prediction::TYPE_MULTI_NUMBER => {
            let (c, arr, mask) = decode_choice_digits(choice, blocked_secondary)?;
            require!(c >= 3 && c <= 8, IC42NErrorCode::InvalidBetNumber);
            out = arr;
            count = c;
            return Ok((count, out, mask));
        }

        _ => return err!(IC42NErrorCode::InvalidBetNumber),
    }

    // Build mask and validate uniqueness for derived modes (HIGH_LOW / EVEN_ODD)
    require!(count >= 1 && count <= 8, IC42NErrorCode::InvalidBetNumber);

    let mut mask: u16 = 0;
    for i in 0..(count as usize) {
        let v = out[i];
        require!(v >= 1 && v <= 9, IC42NErrorCode::InvalidBetNumber);
        require!(v != blocked_secondary, IC42NErrorCode::InvalidBetNumber);

        let bit = 1u16 << v;
        require!((mask & bit) == 0, IC42NErrorCode::InvalidBetNumber);
        mask |= bit;
    }

    Ok((count, out, mask))
}

/// Decode an u32 "digit-encoded" choice into a canonical selection list + mask.
/// - Digits must be 1~9 (0 forbidden)
/// - No duplicates
/// - blocked_secondary forbidden
/// - Canonicalized: ascending order
fn decode_choice_digits(
    choice: u32,
    blocked_secondary: u8,
) -> anchor_lang::Result<(u8, [u8; 8], u16)> {
    // Must supply something (no empty set)
    require!(choice > 0, IC42NErrorCode::InvalidBetNumber);

    let mut seen = [false; 10]; // indices 0..9; we forbid 0
    let mut tmp = [0u8; 8];
    let mut count: u8 = 0;

    let mut v = choice;
    while v > 0 {
        let d = (v % 10) as u8;
        v /= 10;

        require!(d >= 1 && d <= 9, IC42NErrorCode::InvalidBetNumber);
        require!(d != blocked_secondary, IC42NErrorCode::InvalidBetNumber);
        require!(!seen[d as usize], IC42NErrorCode::InvalidBetNumber); // or DuplicateSelection
        require!(count < 8, IC42NErrorCode::InvalidBetNumber);

        seen[d as usize] = true;
        tmp[count as usize] = d;
        count += 1;
    }

    require!(count >= 1 && count <= 8, IC42NErrorCode::InvalidBetNumber);

    // Canonicalize: sort ascending in-place for the active prefix
    let mut i = 0usize;
    while i < (count as usize) {
        let mut j = i + 1;
        while j < (count as usize) {
            if tmp[j] < tmp[i] {
                let t = tmp[i];
                tmp[i] = tmp[j];
                tmp[j] = t;
            }
            j += 1;
        }
        i += 1;
    }

    // Build out + mask
    let mut out = [0u8; 8];
    let mut mask: u16 = 0;

    for i in 0..(count as usize) {
        out[i] = tmp[i];
        mask |= 1u16 << tmp[i];
    }

    Ok((count, out, mask))
}



pub fn retract_per_number_from_live(
    live: &mut LiveFeed,
    lamports_per_number: u64,
    selections: &[u8; 8],
    selection_count: u8,
) -> anchor_lang::Result<()> {
    require!(lamports_per_number > 0, IC42NErrorCode::InvalidBetAmount);

    let k = selection_count as usize;
    require!(k >= 1 && k <= 8, IC42NErrorCode::InvalidBetNumber);

    for i in 0..k {
        let v = selections[i];
        require!(v >= 1 && v <= 9, IC42NErrorCode::InvalidBetNumber);
        let n = v as usize;

        require!(n < live.lamports_per_number.len(), IC42NErrorCode::InvalidBetNumber);
        require!(live.lamports_per_number[n] >= lamports_per_number, IC42NErrorCode::InvalidLiveFeedState);

        live.lamports_per_number[n] = live.lamports_per_number[n]
            .checked_sub(lamports_per_number)
            .ok_or(IC42NErrorCode::MathOverflow)?;
    }

    Ok(())
}

pub fn apply_per_number_to_live(
    live: &mut LiveFeed,
    lamports_per_number: u64,
    selections: &[u8; 8],
    selection_count: u8,
) -> anchor_lang::Result<()> {
    require!(lamports_per_number > 0, IC42NErrorCode::InvalidBetAmount);

    let k = selection_count as usize;
    require!(k >= 1 && k <= 8, IC42NErrorCode::InvalidBetNumber);

    for i in 0..k {
        let v = selections[i];
        require!(v >= 1 && v <= 9, IC42NErrorCode::InvalidBetNumber);
        let n = v as usize;

        require!(n < live.lamports_per_number.len(), IC42NErrorCode::InvalidBetNumber);

        live.lamports_per_number[n] = live.lamports_per_number[n]
            .checked_add(lamports_per_number)
            .ok_or(IC42NErrorCode::MathOverflow)?;
    }

    Ok(())
}
