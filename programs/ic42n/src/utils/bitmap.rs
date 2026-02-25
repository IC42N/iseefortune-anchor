/// ---------------------------------------------------------------------------
/// Check whether a given claim index has already been claimed.
///
/// Bitmap encoding:
///   - `bitmap` is a Vec<u8>, where each byte stores 8 claim bits.
///   - Index N is stored at:
///         byte_index = N / 8
///         bit_index  = N % 8
///
/// Safety rule:
///   - If the index is OUT OF RANGE of the bitmap, we return **true**
///     (meaning “already claimed”).
///     This prevents invalid indices from ever being claimable.
/// ---------------------------------------------------------------------------
pub fn is_claimed(bitmap: &Vec<u8>, index: u32) -> bool {
    let byte_index = (index / 8) as usize;
    let bit_index = (index % 8) as u8;

    // If the index is out of bounds → treat as already claimed (safe default)
    if byte_index >= bitmap.len() {
        return true;
    }

    // Check the relevant bit
    let mask = 1 << bit_index;
    (bitmap[byte_index] & mask) != 0
}

/// ---------------------------------------------------------------------------
/// Mark a given index as claimed in the bitmap.
///
/// This sets the single bit corresponding to the index:
///     byte_index = index / 8
///     bit_index  = index % 8
///
/// Out-of-range writes are ignored safely.
/// ---------------------------------------------------------------------------
pub fn set_claimed(bitmap: &mut Vec<u8>, index: u32) {
    let byte_index = (index / 8) as usize;
    let bit_index = (index % 8) as u8;

    if byte_index < bitmap.len() {
        let mask = 1 << bit_index;
        bitmap[byte_index] |= mask;
    }
}