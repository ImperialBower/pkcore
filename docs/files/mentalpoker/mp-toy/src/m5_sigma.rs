//! Module 5 — Sigma protocols: Schnorr and Chaum–Pedersen.
//!
//! Three moves: commit, challenge, respond.
//!
//! Schnorr (prove knowledge of x in h = g^x):
//!     commit:   t = g^k            for random k
//!     respond:  s = k + e·x mod q  for challenge e
//!     verify:   g^s == t · h^e
//!
//! Chaum–Pedersen (prove log_g(h) == log_u(v), i.e. the SAME x): run Schnorr
//! twice with the same k, e, s against both bases — (g,h) and (u,v). This is
//! what makes a reveal token trustworthy: u = c1, v = d_i, and the proof says
//! "d_i was computed with the same secret as my public key."

use crate::exercise;
use crate::m1_groups::{mod_exp, mod_mul};

#[derive(Clone, Copy, Debug)]
pub struct SchnorrCommit {
    pub t: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct CpCommit {
    pub t1: u64, // g^k
    pub t2: u64, // u^k
}

/// Prover move 1: commit with fresh k.
pub fn schnorr_commit(k: u64, g: u64, p: u64) -> SchnorrCommit {
    exercise!("m5: t = g^k", { SchnorrCommit { t: mod_exp(g, k, p) } })
}

/// Prover move 3: respond to challenge e. s = k + e·x mod q.
pub fn schnorr_respond(k: u64, x: u64, e: u64, q: u64) -> u64 {
    exercise!("m5: (k + e*x) mod q — exponent arithmetic is mod q!", {
        (k + mod_mul(e % q, x % q, q)) % q
    })
}

/// Verifier: g^s == t · h^e.
pub fn schnorr_verify(h: u64, com: SchnorrCommit, e: u64, s: u64, g: u64, p: u64) -> bool {
    exercise!("m5: check g^s == t * h^e", {
        mod_exp(g, s, p) == mod_mul(com.t, mod_exp(h, e, p), p)
    })
}

/// Chaum–Pedersen commit: the same k against both bases.
pub fn cp_commit(k: u64, g: u64, u: u64, p: u64) -> CpCommit {
    exercise!("m5: (g^k, u^k)", { CpCommit { t1: mod_exp(g, k, p), t2: mod_exp(u, k, p) } })
}

/// Chaum–Pedersen response is *identical* to Schnorr's — same s covers both.
pub fn cp_respond(k: u64, x: u64, e: u64, q: u64) -> u64 {
    schnorr_respond(k, x, e, q)
}

/// Verify both equations with one (e, s): g^s == t1·h^e AND u^s == t2·v^e.
pub fn cp_verify(h: u64, u: u64, v: u64, com: CpCommit, e: u64, s: u64, g: u64, p: u64) -> bool {
    exercise!("m5: both Schnorr checks with the same e and s", {
        mod_exp(g, s, p) == mod_mul(com.t1, mod_exp(h, e, p), p)
            && mod_exp(u, s, p) == mod_mul(com.t2, mod_exp(v, e, p), p)
    })
}

/// The cheat: if the prover knows the challenge e *before* committing, they
/// can pass verification for an h whose discrete log they do NOT know.
/// Pick s freely, then t = g^s · h^(−e). Implement it to feel why the
/// challenge must be unpredictable.
pub fn schnorr_forge(h: u64, e: u64, s: u64, g: u64, q: u64, p: u64) -> SchnorrCommit {
    exercise!("m5: t = g^s * (h^e)^(q-1) — commit *after* seeing e", {
        let he = mod_exp(h, e, p);
        SchnorrCommit { t: mod_mul(mod_exp(g, s, p), mod_exp(he, q - 1, p), p) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_elgamal::{encode_card, mask};
    use crate::m4_threshold::reveal_token;
    use crate::{G, P, Q, ToyRng};

    #[test]
    fn m5_schnorr_honest_prover_passes() {
        let mut rng = ToyRng::seeded(21);
        let x = rng.exponent(Q);
        let h = mod_exp(G, x, P);
        for _ in 0..20 {
            let k = rng.exponent(Q);
            let com = schnorr_commit(k, G, P);
            let e = rng.exponent(Q); // verifier's random challenge
            let s = schnorr_respond(k, x, e, Q);
            assert!(schnorr_verify(h, com, e, s, G, P));
        }
    }

    #[test]
    fn m5_schnorr_wrong_secret_fails() {
        let mut rng = ToyRng::seeded(22);
        let x = rng.exponent(Q);
        let h = mod_exp(G, x, P);
        let k = rng.exponent(Q);
        let com = schnorr_commit(k, G, P);
        let e = rng.exponent(Q);
        let s = schnorr_respond(k, (x + 1) % Q, e, Q); // lying about x
        assert!(!schnorr_verify(h, com, e, s, G, P));
    }

    #[test]
    fn m5_fixed_challenge_forgery_works() {
        // h with an unknown discrete log (encode_card gives us one whose log we
        // "forget"). With e known in advance, the forger passes anyway.
        let h = encode_card(30, G, P);
        let e = 77u64; // leaked / predictable challenge
        let s = 123u64; // chosen freely
        let com = schnorr_forge(h, e, s, G, Q, P);
        assert!(
            schnorr_verify(h, com, e, s, G, P),
            "a predictable challenge destroys soundness — hence module 6"
        );
    }

    #[test]
    fn m5_chaum_pedersen_ties_reveal_token_to_public_key() {
        let mut rng = ToyRng::seeded(23);
        let x = rng.exponent(Q);
        let h = mod_exp(G, x, P); // my public key
        let ct = mask(encode_card(9, G, P), h, rng.exponent(Q), G, P);

        // Honest token + proof verifies.
        let d = reveal_token(ct, x, P); // d = c1^x
        let k = rng.exponent(Q);
        let com = cp_commit(k, G, ct.c1, P);
        let e = rng.exponent(Q);
        let s = cp_respond(k, x, e, Q);
        assert!(cp_verify(h, ct.c1, d, com, e, s, G, P));

        // A bogus token (wrong exponent) cannot satisfy both equations.
        let bogus = reveal_token(ct, (x + 5) % Q, P);
        assert!(!cp_verify(h, ct.c1, bogus, com, e, s, G, P),
            "without this check, a player could make you misread your own hand");
    }
}
