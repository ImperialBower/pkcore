//! Module 6 — Fiat–Shamir, and the whole protocol assembled.
//!
//! Replace the interactive challenge with a hash of the transcript:
//!     e = H(statement ‖ commitment)
//! The prover can no longer pick their commitment after seeing e (module 5's
//! forgery), because e is *determined by* the commitment. What goes into the
//! hash matters: hash the full statement, or you get replay/malleability.
//!
//! The final test runs the complete deal: keys+proofs → aggregate → mask →
//! shuffle(permute+remask) ×N → hole card via staged proven tokens → board.

use crate::exercise;
use crate::m1_groups::{mod_exp, mod_mul};
use crate::m3_elgamal::Ciphertext;
use crate::m5_sigma::{
    cp_commit, cp_respond, cp_verify, schnorr_commit, schnorr_respond, schnorr_verify,
    CpCommit, SchnorrCommit,
};
use crate::toy_hash;

#[derive(Clone, Copy, Debug)]
pub struct NizkSchnorr {
    pub t: u64,
    pub s: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct NizkCp {
    pub t1: u64,
    pub t2: u64,
    pub s: u64,
}

/// Non-interactive Schnorr: e = H(g, h, t). Statement AND commitment hashed.
pub fn nizk_schnorr_prove(x: u64, k: u64, g: u64, q: u64, p: u64) -> (u64, NizkSchnorr) {
    exercise!("m6: commit, derive e = toy_hash([g,h,t]) mod q, respond", {
        let h = mod_exp(g, x, p);
        let com = schnorr_commit(k, g, p);
        let e = toy_hash(&[g, h, com.t]) % q;
        let s = schnorr_respond(k, x, e, q);
        (h, NizkSchnorr { t: com.t, s })
    })
}

pub fn nizk_schnorr_verify(h: u64, pf: NizkSchnorr, g: u64, q: u64, p: u64) -> bool {
    exercise!("m6: recompute e from the transcript, then schnorr_verify", {
        let e = toy_hash(&[g, h, pf.t]) % q;
        schnorr_verify(h, SchnorrCommit { t: pf.t }, e, pf.s, g, p)
    })
}

/// Non-interactive Chaum–Pedersen over (g,h)=(g,g^x) and (u,v)=(c1,c1^x):
/// e = H(g, h, u, v, t1, t2).
pub fn nizk_cp_prove(x: u64, k: u64, u: u64, g: u64, q: u64, p: u64) -> (u64, NizkCp) {
    exercise!("m6: v = u^x; commit both bases; e from full statement; respond", {
        let h = mod_exp(g, x, p);
        let v = mod_exp(u, x, p);
        let com = cp_commit(k, g, u, p);
        let e = toy_hash(&[g, h, u, v, com.t1, com.t2]) % q;
        let s = cp_respond(k, x, e, q);
        (v, NizkCp { t1: com.t1, t2: com.t2, s })
    })
}

pub fn nizk_cp_verify(h: u64, u: u64, v: u64, pf: NizkCp, g: u64, q: u64, p: u64) -> bool {
    exercise!("m6: recompute e, then cp_verify both equations", {
        let e = toy_hash(&[g, h, u, v, pf.t1, pf.t2]) % q;
        cp_verify(h, u, v, CpCommit { t1: pf.t1, t2: pf.t2 }, e, pf.s, g, p)
    })
}

/// Shuffle = permute + remask, the module 3 lesson made executable. (The ZK
/// *argument* that this was done honestly is module 7; here honest-but-visible.)
pub fn shuffle_and_remask(
    deck: &[Ciphertext],
    perm: &[usize],
    randomness: &[u64],
    h: u64,
    g: u64,
    p: u64,
) -> Vec<Ciphertext> {
    exercise!("m6: out[i] = remask(deck[perm[i]], randomness[i])", {
        perm.iter()
            .zip(randomness)
            .map(|(&from, &s)| crate::m3_elgamal::remask(deck[from], h, s, g, p))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_elgamal::{decode_card, encode_card, mask, DECK};
    use crate::m4_threshold::{aggregate_key, apply_tokens, reveal_token};
    use crate::{G, P, Q, ToyRng};

    #[test]
    fn m6_nizk_schnorr_roundtrip_and_tamper() {
        let mut rng = ToyRng::seeded(31);
        let x = rng.exponent(Q);
        let (h, pf) = nizk_schnorr_prove(x, rng.exponent(Q), G, Q, P);
        assert!(nizk_schnorr_verify(h, pf, G, Q, P));
        // Any transcript tampering breaks it — e is bound to (h, t).
        let bad = NizkSchnorr { t: pf.t, s: (pf.s + 1) % Q };
        assert!(!nizk_schnorr_verify(h, bad, G, Q, P));
        assert!(!nizk_schnorr_verify(mod_mul(h, G, P), pf, G, Q, P));
    }

    #[test]
    fn m6_forgery_from_module_5_is_now_dead() {
        // The module-5 cheat required choosing t AFTER learning e. Fiat–Shamir
        // computes e FROM t, so the forger chases their own tail: for their
        // forged t to verify, e must equal H(..., t) — but t was built from a
        // guessed e. Unless the guess collides with the hash, it fails.
        let h = encode_card(30, G, P); // unknown discrete log
        let guessed_e = 77u64;
        let s = 123u64;
        let com = crate::m5_sigma::schnorr_forge(h, guessed_e, s, G, Q, P);
        let actual_e = toy_hash(&[G, h, com.t]) % Q;
        assert_ne!(actual_e, guessed_e, "with a real hash this holds overwhelmingly");
        assert!(!nizk_schnorr_verify(h, NizkSchnorr { t: com.t, s }, G, Q, P));
    }

    #[test]
    fn m6_full_protocol_end_to_end() {
        const N: usize = 3;
        let mut rng = ToyRng::seeded(99);

        // Step 0: keygen with proofs; everyone verifies everyone; aggregate.
        let sks: Vec<u64> = (0..N).map(|_| rng.exponent(Q)).collect();
        let mut pks = Vec::new();
        for &x in &sks {
            let (h_i, pf) = nizk_schnorr_prove(x, rng.exponent(Q), G, Q, P);
            assert!(nizk_schnorr_verify(h_i, pf, G, Q, P), "rogue-key gate");
            pks.push(h_i);
        }
        let h = aggregate_key(&pks, P);

        // Step 1: mask the ordered deck under the aggregate key.
        let mut deck: Vec<Ciphertext> = (0..DECK)
            .map(|i| mask(encode_card(i, G, P), h, rng.exponent(Q), G, P))
            .collect();

        // Step 2: each player shuffles (permute + remask).
        for _ in 0..N {
            let mut perm: Vec<usize> = (0..DECK).collect();
            for i in (1..DECK).rev() {
                let j = (rng.next_u64() as usize) % (i + 1);
                perm.swap(i, j);
            }
            let rand: Vec<u64> = (0..DECK).map(|_| rng.exponent(Q)).collect();
            deck = shuffle_and_remask(&deck, &perm, &rand, h, G, P);
        }

        // Steps 4–5: deal slot 0 to player 0 (staged, with CP proofs), then
        // open slot 10 to everyone (full unmask, with CP proofs).
        let hole = deck[0];
        let mut tokens = Vec::new();
        for i in 1..N {
            let (d_i, pf) = nizk_cp_prove(sks[i], rng.exponent(Q), hole.c1, G, Q, P);
            assert!(nizk_cp_verify(pks[i], hole.c1, d_i, pf, G, Q, P), "proven token");
            tokens.push(d_i);
        }
        // Partial value isn't the plaintext yet (it may accidentally collide
        // with *some* card in this tiny group — see module 4's caveat — but it
        // cannot be predicted or controlled).
        let partial = apply_tokens(hole, &tokens, Q, P);
        tokens.push(reveal_token(hole, sks[0], P)); // recipient finishes privately
        let full = apply_tokens(hole, &tokens, Q, P);
        assert_ne!(partial, full, "the recipient's own share changed the result");
        let hole_card = decode_card(full, G, P).expect("player 0 reads a real card");

        let board = deck[10];
        let mut btokens = Vec::new();
        for i in 0..N {
            let (d_i, pf) = nizk_cp_prove(sks[i], rng.exponent(Q), board.c1, G, Q, P);
            assert!(nizk_cp_verify(pks[i], board.c1, d_i, pf, G, Q, P));
            btokens.push(d_i);
        }
        let board_card = decode_card(apply_tokens(board, &btokens, Q, P), G, P)
            .expect("everyone reads the board card");

        assert_ne!(hole_card, board_card);
        println!("hole card (slot 0) -> card #{hole_card}; board (slot 10) -> card #{board_card}");
        // Remaining hole: nothing yet PROVES the shuffles were permutations.
        // That gap — the verifiable shuffle — is module 7.
    }
}
