//! Module 3 — ElGamal and the masked card.
//!
//! ElGamal encryption of a card m under public key h:
//!     mask:    (c1, c2) = (g^r, m · h^r)        with fresh random r
//!     remask:  (c1·g^s, c2·h^s)                 same plaintext, new look
//!     unmask:  m = c2 / c1^x                     where h = g^x
//!
//! Remask is the star: it's why a shuffle can hide which card went where.

use crate::exercise;
use crate::m1_groups::{mod_exp, mod_mul};

pub const DECK: usize = 52;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ciphertext {
    pub c1: u64,
    pub c2: u64,
}

/// Card i (0..52) as a subgroup element: g^(i+1). Toy-only encoding — the
/// discrete logs of the card points are *known*, which real encodings avoid;
/// fine here because we're studying the protocol, not hiding from ourselves.
pub fn encode_card(i: usize, g: u64, p: u64) -> u64 {
    exercise!("m3: g^(i+1) mod p", { mod_exp(g, (i as u64) + 1, p) })
}

/// Invert `encode_card` by scanning the 52 encodings. (Real: lookup table.)
pub fn decode_card(m: u64, g: u64, p: u64) -> Option<usize> {
    exercise!("m3: find i in 0..52 with encode_card(i) == m", {
        (0..DECK).find(|&i| encode_card(i, g, p) == m)
    })
}

/// Encrypt (mask) plaintext element m under public key h with randomness r.
pub fn mask(m: u64, h: u64, r: u64, g: u64, p: u64) -> Ciphertext {
    exercise!("m3: (g^r, m * h^r)", {
        Ciphertext { c1: mod_exp(g, r, p), c2: mod_mul(m, mod_exp(h, r, p), p) }
    })
}

/// Re-randomize: same plaintext, fresh randomness s folded in.
pub fn remask(ct: Ciphertext, h: u64, s: u64, g: u64, p: u64) -> Ciphertext {
    exercise!("m3: (c1 * g^s, c2 * h^s)", {
        Ciphertext {
            c1: mod_mul(ct.c1, mod_exp(g, s, p), p),
            c2: mod_mul(ct.c2, mod_exp(h, s, p), p),
        }
    })
}

/// Decrypt with the full secret key x: m = c2 · (c1^x)^(-1).
/// Inverse in the order-q subgroup: a^(-1) = a^(q-1).
pub fn unmask_full(ct: Ciphertext, x: u64, q: u64, p: u64) -> u64 {
    exercise!("m3: c2 * (c1^x)^(q-1) mod p", {
        let c1x = mod_exp(ct.c1, x, p);
        mod_mul(ct.c2, mod_exp(c1x, q - 1, p), p)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{G, P, Q, ToyRng};

    #[test]
    fn m3_encode_decode_roundtrip_all_52() {
        for i in 0..DECK {
            let m = encode_card(i, G, P);
            assert_eq!(decode_card(m, G, P), Some(i));
        }
        // All 52 encodings distinct.
        let mut all: Vec<u64> = (0..DECK).map(|i| encode_card(i, G, P)).collect();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), DECK);
    }

    #[test]
    fn m3_mask_unmask_roundtrip() {
        let mut rng = ToyRng::seeded(3);
        let x = rng.exponent(Q);
        let h = mod_exp(G, x, P);
        for i in [0usize, 13, 51] {
            let m = encode_card(i, G, P);
            let ct = mask(m, h, rng.exponent(Q), G, P);
            assert_eq!(unmask_full(ct, x, Q, P), m);
        }
    }

    #[test]
    fn m3_remask_preserves_plaintext_but_changes_ciphertext() {
        let mut rng = ToyRng::seeded(9);
        let x = rng.exponent(Q);
        let h = mod_exp(G, x, P);
        let m = encode_card(7, G, P);
        let ct0 = mask(m, h, rng.exponent(Q), G, P);

        let mut ct = ct0;
        for _ in 0..100 {
            let next = remask(ct, h, rng.exponent(Q), G, P);
            assert_ne!(next, ct, "remask must change the ciphertext's appearance");
            assert_eq!(unmask_full(next, x, Q, P), m, "…but never the plaintext");
            ct = next;
        }
    }

    #[test]
    fn m3_why_shuffling_without_remask_hides_nothing() {
        // A permuted-but-not-remasked deck: byte-identical ciphertexts, so an
        // observer tracks each card through the "shuffle" by simple equality.
        let mut rng = ToyRng::seeded(11);
        let x = rng.exponent(Q);
        let h = mod_exp(G, x, P);
        let deck: Vec<Ciphertext> =
            (0..5).map(|i| mask(encode_card(i, G, P), h, rng.exponent(Q), G, P)).collect();
        let permuted = vec![deck[3], deck[0], deck[4], deck[1], deck[2]];
        for (orig_pos, ct) in deck.iter().enumerate() {
            let now_at = permuted.iter().position(|c| c == ct).unwrap();
            // We just recovered the whole permutation without any secret:
            assert_eq!(permuted[now_at], deck[orig_pos]);
        }
        // Remask each card and the equality trail goes cold — that's Module 3's
        // punchline and the reason `shuffle` = permute + remask, always.
    }
}
