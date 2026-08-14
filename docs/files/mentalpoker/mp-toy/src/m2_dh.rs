//! Module 2 — Diffie–Hellman, and breaking it.
//!
//! First build key exchange out of `mod_exp`. Then implement baby-step
//! giant-step and *break your own exchange* — computing a discrete log in
//! ~√Q steps. At Q = 233 that's ~15 steps; at 2^256 it's 2^128. Feel the gap.

use crate::exercise;
use crate::m1_groups::{mod_exp, mod_mul};
use std::collections::HashMap;

/// (secret, public) = (x, g^x mod p) with x drawn from 1..q.
pub fn keypair(rng: &mut crate::ToyRng, g: u64, q: u64, p: u64) -> (u64, u64) {
    exercise!("m2: x = rng.exponent(q); pk = g^x mod p", {
        let x = rng.exponent(q);
        (x, mod_exp(g, x, p))
    })
}

/// The DH shared secret: their_public^my_secret mod p.
pub fn shared_secret(their_public: u64, my_secret: u64, p: u64) -> u64 {
    exercise!("m2: their_public^my_secret mod p", {
        mod_exp(their_public, my_secret, p)
    })
}

/// Baby-step giant-step: recover x from h = g^x in O(√q) time and memory.
/// Sketch: m = ceil(√q). Table g^j for j in 0..m (baby steps). Then walk
/// h · (g^-m)^i for i in 0..m (giant steps) until it hits the table;
/// x = i·m + j. For the inverse, use g^(q-1) = g^-1 in the order-q subgroup.
pub fn baby_step_giant_step(g: u64, h: u64, q: u64, p: u64) -> Option<u64> {
    exercise!("m2: BSGS — table of baby steps, then giant-step walk", {
        let m = (q as f64).sqrt().ceil() as u64;
        let mut table: HashMap<u64, u64> = HashMap::with_capacity(m as usize);
        let mut acc = 1u64;
        for j in 0..m {
            table.entry(acc).or_insert(j);
            acc = mod_mul(acc, g, p);
        }
        // g^(-m) = (g^m)^(q-1) since element order is q.
        let g_neg_m = mod_exp(mod_exp(g, m, p), q - 1, p);
        let mut gamma = h % p;
        for i in 0..m {
            if let Some(&j) = table.get(&gamma) {
                return Some((i * m + j) % q);
            }
            gamma = mod_mul(gamma, g_neg_m, p);
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{G, P, Q, ToyRng};

    #[test]
    fn m2_key_exchange_agrees() {
        let mut rng = ToyRng::seeded(7);
        let (a_sk, a_pk) = keypair(&mut rng, G, Q, P);
        let (b_sk, b_pk) = keypair(&mut rng, G, Q, P);
        assert_eq!(shared_secret(b_pk, a_sk, P), shared_secret(a_pk, b_sk, P));
    }

    #[test]
    fn m2_bsgs_recovers_known_exponent() {
        for x in [1u64, 2, 57, 232] {
            let h = mod_exp(G, x, P);
            assert_eq!(baby_step_giant_step(G, h, Q, P), Some(x));
        }
    }

    #[test]
    fn m2_break_the_exchange() {
        // The attack: given only public values, recover the shared secret.
        let mut rng = ToyRng::seeded(42);
        let (a_sk, a_pk) = keypair(&mut rng, G, Q, P);
        let (_b_sk, b_pk) = keypair(&mut rng, G, Q, P);
        let stolen_a_sk = baby_step_giant_step(G, a_pk, Q, P).expect("toy group is breakable");
        assert_eq!(
            shared_secret(b_pk, stolen_a_sk, P),
            shared_secret(b_pk, a_sk, P),
            "eavesdropper computed the shared secret from public data alone"
        );
        // Moral: √233 ≈ 15 steps here. √(2^252) ≈ 2^126 steps on a real curve.
        // Security is *only* the size of this number.
    }
}
