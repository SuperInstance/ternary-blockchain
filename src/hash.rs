/// Balanced ternary hash. Output is 27 trits (3^3 rounds of mixing).
/// Trits are i8 values in {-1, 0, +1}.
pub const HASH_LEN: usize = 27;
pub type TritHash = [i8; HASH_LEN];

/// Add two trits in Z₃ (balanced: result in {-1,0,+1}).
#[inline]
fn trit_add(a: i8, b: i8) -> i8 {
    let s = a + b;
    if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s }
}

/// Multiply two trits in Z₃.
#[inline]
fn trit_mul(a: i8, b: i8) -> i8 {
    let p = a * b;
    if p > 1 { p - 3 } else if p < -1 { p + 3 } else { p }
}

/// Rotate trit slice left by `n` positions.
fn rotate_left(state: &mut [i8; HASH_LEN], n: usize) {
    let n = n % HASH_LEN;
    let tmp: Vec<i8> = state.to_vec();
    for i in 0..HASH_LEN {
        state[i] = tmp[(i + n) % HASH_LEN];
    }
}

/// Convert a byte to 5 balanced trits.
fn byte_to_trits(b: u8) -> [i8; 5] {
    let mut v = b as i16;
    let mut t = [0i8; 5];
    for i in 0..5 {
        let r = ((v % 3) + 3) % 3;
        t[i] = if r == 2 { -1 } else { r as i8 };
        v = (v - if r == 2 { -1 } else { r as i16 }) / 3;
    }
    t
}

/// Core sponge-style absorption of a trit chunk into state.
fn absorb(state: &mut [i8; HASH_LEN], chunk: &[i8]) {
    for (i, &t) in chunk.iter().enumerate() {
        let idx = i % HASH_LEN;
        state[idx] = trit_add(state[idx], t);
    }
    // Mix: three rounds
    for round in 0..3u8 {
        let round_t = (round as i8) - 1; // -1, 0, 1
        // Nonlinear layer: each trit mixed with its neighbours
        let prev = *state;
        for i in 0..HASH_LEN {
            let left = prev[(i + HASH_LEN - 1) % HASH_LEN];
            let right = prev[(i + 1) % HASH_LEN];
            state[i] = trit_add(trit_add(prev[i], trit_mul(left, right)), round_t);
        }
        // Rotation based on round
        rotate_left(state, 3 * (round as usize + 1));
    }
}

/// Hash arbitrary bytes to a 27-trit hash.
pub fn ternary_hash(data: &[u8]) -> TritHash {
    let mut state = [0i8; HASH_LEN];
    // Domain separation: inject length
    let len_trits = byte_to_trits((data.len() & 0xFF) as u8);
    absorb(&mut state, &len_trits);
    // Process each byte
    let mut buf = [0i8; 5];
    for &byte in data {
        buf.copy_from_slice(&byte_to_trits(byte));
        absorb(&mut state, &buf);
    }
    // Finalize: absorb padding sentinel
    absorb(&mut state, &[1i8, -1, 0, 1, -1]);
    state
}

/// Hash two trit hashes together (for Merkle).
pub fn hash_pair(left: &TritHash, right: &TritHash) -> TritHash {
    let mut combined = Vec::with_capacity(HASH_LEN * 2);
    // Convert trits to bytes for hashing
    for &t in left.iter().chain(right.iter()) {
        combined.push((t + 1) as u8); // map {-1,0,1} → {0,1,2}
    }
    ternary_hash(&combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let h1 = ternary_hash(b"hello");
        let h2 = ternary_hash(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_differs_for_different_input() {
        let h1 = ternary_hash(b"hello");
        let h2 = ternary_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_output_is_valid_trits() {
        let h = ternary_hash(b"ternary");
        for &t in &h {
            assert!(t == -1 || t == 0 || t == 1);
        }
    }
}
