//! Module 4 — Threshold ElGamal: the key is everyone's, and no one's.
//!
//! Each player i holds x_i; the aggregate public key is h = Π g^{x_i}. A card
//! masked under h can only be opened with *every* player's help: player i
//! contributes the reveal token d_i = c1^{x_i}. Apply n−1 tokens and one seat
//! finishes privately (a hole card); apply all n and it's public (the board).

use crate::exercise;
use crate::m1_groups::{mod_exp, mod_mul};
use crate::m3_elgamal::Ciphertext;

/// Aggregate public key from per-player public keys: their product mod p.
pub fn aggregate_key(pks: &[u64], p: u64) -> u64 {
    exercise!("m4: fold mod_mul over the public keys", {
        pks.iter().fold(1u64, |acc, &h_i| mod_mul(acc, h_i, p))
    })
}

/// Player i's reveal token for a ciphertext: d_i = c1^{x_i}.
pub fn reveal_token(ct: Ciphertext, x_i: u64, p: u64) -> u64 {
    exercise!("m4: c1 ^ x_i mod p", { mod_exp(ct.c1, x_i, p) })
}

/// Apply a set of reveal tokens: m' = c2 · (Π d_i)^(-1). With every player's
/// token this is the plaintext; with a strict subset it's still garbage —
/// that's the threshold property the test pins down.
pub fn apply_tokens(ct: Ciphertext, tokens: &[u64], q: u64, p: u64) -> u64 {
    exercise!("m4: c2 * (product of tokens)^(q-1) mod p", {
        let prod = tokens.iter().fold(1u64, |acc, &d| mod_mul(acc, d, p));
        mod_mul(ct.c2, mod_exp(prod, q - 1, p), p)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_elgamal::{decode_card, encode_card, mask};
    use crate::{G, P, Q, ToyRng};

    const N: usize = 3;

    fn setup(seed: u64) -> (ToyRng, Vec<u64>, Vec<u64>, u64) {
        let mut rng = ToyRng::seeded(seed);
        let sks: Vec<u64> = (0..N).map(|_| rng.exponent(Q)).collect();
        let pks: Vec<u64> = sks.iter().map(|&x| mod_exp(G, x, P)).collect();
        let h = aggregate_key(&pks, P);
        (rng, sks, pks, h)
    }

    #[test]
    fn m4_aggregate_key_is_g_to_sum_of_secrets() {
        let (_rng, sks, _pks, h) = setup(5);
        let sum: u64 = sks.iter().fold(0, |a, &x| (a + x) % Q);
        assert_eq!(h, mod_exp(G, sum, P), "h = g^(Σ x_i): no one knows Σ x_i");
    }

    #[test]
    fn m4_all_tokens_open_the_card() {
        let (mut rng, sks, _pks, h) = setup(6);
        let m = encode_card(21, G, P);
        let ct = mask(m, h, rng.exponent(Q), G, P);
        let tokens: Vec<u64> = sks.iter().map(|&x| reveal_token(ct, x, P)).collect();
        assert_eq!(decode_card(apply_tokens(ct, &tokens, Q, P), G, P), Some(21));
    }

    #[test]
    fn m4_threshold_any_strict_subset_learns_nothing() {
        let (mut rng, sks, _pks, h) = setup(8);
        let m = encode_card(3, G, P);
        let ct = mask(m, h, rng.exponent(Q), G, P);
        let tokens: Vec<u64> = sks.iter().map(|&x| reveal_token(ct, x, P)).collect();
        // Every proper subset — including n−1 of n — yields a value that is
        // NOT the true card. (Small-group caveat: with only 233 elements and
        // 52 card encodings, a partial value can *collide by accident* with
        // some other card ~22% of the time — so the honest assertion is
        // "not the true plaintext", not "not any card". In a 2^252 group the
        // collision probability is negligible and both statements hold.)
        for skip in 0..N {
            let partial: Vec<u64> =
                tokens.iter().enumerate().filter(|(i, _)| *i != skip).map(|(_, &d)| d).collect();
            let out = apply_tokens(ct, &partial, Q, P);
            assert_ne!(out, m, "n-1 tokens must not yield the true plaintext");
        }
    }

    #[test]
    fn m4_staged_unmask_deals_a_hole_card() {
        // Players 1 and 2 publish tokens; player 0 finishes privately with
        // their own share — the toy version of dealing seat 0 a hole card.
        let (mut rng, sks, _pks, h) = setup(13);
        let m = encode_card(47, G, P);
        let ct = mask(m, h, rng.exponent(Q), G, P);

        let others: Vec<u64> = (1..N).map(|i| reveal_token(ct, sks[i], P)).collect();
        // Not yet the true card (see the small-group caveat in the test above).
        assert_ne!(apply_tokens(ct, &others, Q, P), m,
            "before adding their own share, even the recipient lacks the card");

        let mut all = others;
        all.push(reveal_token(ct, sks[0], P));
        assert_eq!(decode_card(apply_tokens(ct, &all, Q, P), G, P), Some(47));
    }
}
