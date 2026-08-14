//! # mp-toy — the insecure teaching crate
//!
//! Companion exercises for the mental-poker cryptography lesson plan. Each
//! module matches a lesson module and contains functions whose bodies are
//! `todo!()` — your job is to fill them in until `cargo test` passes.
//!
//! Reference solutions ship behind a feature flag so you can verify the tests
//! are sound, or peek when stuck:
//!
//! ```text
//! cargo test                       # runs YOUR implementations
//! cargo test --features solutions  # runs the reference solutions (all pass)
//! cargo test m1_                   # just module 1's tests
//! ```
//!
//! **This crate is deliberately insecure.** Small numbers so every value is
//! inspectable, non-cryptographic hash, no constant time, no real randomness.
//! It exists for X-ray vision into the algebra and retires after module 8 of
//! the plan. Never let any of it near production code.

pub mod m1_groups;
pub mod m2_dh;
pub mod m3_elgamal;
pub mod m4_threshold;
pub mod m5_sigma;
pub mod m6_fiat_shamir;

/// The exercise macro: with `--features solutions` the reference body runs;
/// otherwise you hit a `todo!` telling you what to implement.
#[macro_export]
macro_rules! exercise {
    ($hint:literal, $solution:block) => {{
        #[cfg(feature = "solutions")]
        {
            $solution
        }
        #[cfg(not(feature = "solutions"))]
        {
            todo!($hint)
        }
    }};
}

// ─── Shared toy parameters ───────────────────────────────────────────────────

/// Safe prime: P = 2Q + 1. The multiplicative group mod P has order P−1 = 2Q,
/// so it contains a subgroup of prime order Q — that subgroup is where all our
/// "cards" and keys live. Small enough to check everything by hand.
pub const P: u64 = 467;
/// Prime order of the subgroup we work in. Exponents live mod Q.
pub const Q: u64 = 233;
/// A generator of the order-Q subgroup (you will *verify* this in module 1).
/// 2 generates the full group mod 467; squaring it lands in the subgroup.
pub const G: u64 = 4; // = 2²

/// Deterministic toy RNG (an LCG) so tests are reproducible. Real protocols
/// use an OS CSPRNG; this is exactly the kind of thing you never ship.
pub struct ToyRng(u64);

impl ToyRng {
    pub fn seeded(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    /// Uniform-ish in `1..q` — a valid secret exponent.
    pub fn exponent(&mut self, q: u64) -> u64 {
        1 + self.next_u64() % (q - 1)
    }
}

/// Non-cryptographic 64-bit hash (FNV-1a) used for Fiat–Shamir challenges in
/// module 6. Real implementations use SHA-256/SHA-3; the *shape* of the
/// transform is identical, which is what we're here to learn.
pub fn toy_hash(parts: &[u64]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for p in parts {
        for b in p.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    }
    h
}
