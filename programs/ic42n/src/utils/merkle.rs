use sha2::{Sha256, Digest};

/// Verify a Merkle proof using SHA-256
///
/// Tree rule:
///     parent = SHA256(left_child || right_child)
///
/// - `leaf` = 32-byte hash of the leaf (from `hash_winner_leaf`)
/// - `proof` = vector of sibling hashes from leaf → root
/// - `root` = expected Merkle root (from on-chain `ResolvedGame`)
/// - `index` = leaf index in the original sorted winner list
pub fn verify_merkle_proof(
    leaf: &[u8; 32],
    proof: &[[u8; 32]],
    root: &[u8; 32],
    mut index: u32,
) -> bool {
    let mut computed = *leaf;

    for sibling in proof {
        let mut hasher = Sha256::new();

        if index % 2 == 0 {
            hasher.update(&computed);
            hasher.update(sibling);
        } else {
            hasher.update(sibling);
            hasher.update(&computed);
        }

        computed = hasher.finalize().into();
        index /= 2;
    }

    computed == *root
}