//! Parameter encoding: `ExploitConfig ↔ [f64; DIM]`.
//!
//! The 16-element parameter vector maps one-to-one onto the fields of
//! [`ExploitConfig`].  [`decode`] clamps every element to its bounds and
//! enforces `min_hands_heavy >= min_hands_light`, so the optimiser is free
//! to explore without producing invalid configs.

use crate::bot::exploit::ExploitConfig;

/// Number of dimensions in the parameter vector.
pub const DIM: usize = 16;

/// Per-dimension lower bounds (same order as [`encode`]).
pub const LO: [f64; DIM] = [
    // thresholds
    0.30, 0.10, 0.20, 0.05, 0.03, 1.00, 0.20, 0.05, // multipliers
    1.00, 0.20, 0.10, 0.10, 0.30, 0.30, // sample gates (continuous relaxation of u64)
    5.0, 10.0,
];

/// Per-dimension upper bounds (same order as [`encode`]).
pub const HI: [f64; DIM] = [
    // thresholds
    0.90, 0.60, 0.80, 0.30, 0.20, 8.00, 0.60, 0.25, // multipliers
    2.50, 1.00, 1.00, 1.00, 1.00, 1.00, // sample gates
    100.0, 200.0,
];

/// Converts an [`ExploitConfig`] into a fixed-length parameter vector.
///
/// # Examples
///
/// ```
/// use pkcore::bot::exploit::ExploitConfig;
/// use pkcore::bot::training::encoding::{encode, DIM};
///
/// let v = encode(&ExploitConfig::default());
/// assert_eq!(v.len(), DIM);
/// ```
// `min_hands_*` are hand counts, far below f64's 2^53 exact-integer ceiling.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn encode(c: &ExploitConfig) -> [f64; DIM] {
    [
        c.fold_to_cbet_high_threshold,
        c.fold_to_cbet_low_threshold,
        c.vpip_calling_station_threshold,
        c.pfr_passive_threshold,
        c.pfr_nit_threshold,
        c.aggression_factor_threshold,
        c.wtsd_threshold,
        c.three_bet_pct_threshold,
        c.fold_to_cbet_high_multiplier,
        c.fold_to_cbet_low_multiplier,
        c.bluff_vs_station_multiplier,
        c.bluff_vs_wtsd_multiplier,
        c.aggression_vs_nit_multiplier,
        c.aggression_vs_three_bettor_multiplier,
        c.min_hands_light as f64,
        c.min_hands_heavy as f64,
    ]
}

/// Converts a raw parameter vector into an [`ExploitConfig`], clamping each
/// element to its bounds and enforcing `min_hands_heavy >= min_hands_light`.
///
/// # Examples
///
/// ```
/// use pkcore::bot::exploit::ExploitConfig;
/// use pkcore::bot::training::encoding::{encode, decode};
///
/// let original = ExploitConfig::default();
/// let roundtripped = decode(&encode(&original));
/// assert_eq!(roundtripped.fold_to_cbet_high_threshold,
///            original.fold_to_cbet_high_threshold);
/// ```
// Every element is clamped into `[LO, HI]` on the line above the casts, so
// the rounded value is in range and non-negative by construction.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
pub fn decode(p: &[f64; DIM]) -> ExploitConfig {
    let v: [f64; DIM] = std::array::from_fn(|i| p[i].clamp(LO[i], HI[i]));
    let min_hands_light = v[14].round() as u64;
    let min_hands_heavy = (v[15].round() as u64).max(min_hands_light);
    ExploitConfig {
        fold_to_cbet_high_threshold: v[0],
        fold_to_cbet_low_threshold: v[1],
        vpip_calling_station_threshold: v[2],
        pfr_passive_threshold: v[3],
        pfr_nit_threshold: v[4],
        aggression_factor_threshold: v[5],
        wtsd_threshold: v[6],
        three_bet_pct_threshold: v[7],
        fold_to_cbet_high_multiplier: v[8],
        fold_to_cbet_low_multiplier: v[9],
        bluff_vs_station_multiplier: v[10],
        bluff_vs_wtsd_multiplier: v[11],
        aggression_vs_nit_multiplier: v[12],
        aggression_vs_three_bettor_multiplier: v[13],
        min_hands_light,
        min_hands_heavy,
    }
}

/// Returns the per-dimension range (`HI[i] - LO[i]`), used for sigma scaling.
///
/// # Examples
///
/// ```
/// use pkcore::bot::training::encoding::ranges;
///
/// let r = ranges();
/// assert!(r.iter().all(|&x| x > 0.0));
/// ```
#[must_use]
pub fn ranges() -> [f64; DIM] {
    std::array::from_fn(|i| HI[i] - LO[i])
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__training__encoding_tests {
    use super::*;

    #[test]
    fn encode_default_roundtrips() {
        let original = ExploitConfig::default();
        let roundtripped = decode(&encode(&original));
        assert_eq!(
            roundtripped.fold_to_cbet_high_threshold,
            original.fold_to_cbet_high_threshold
        );
        assert_eq!(
            roundtripped.fold_to_cbet_low_threshold,
            original.fold_to_cbet_low_threshold
        );
        assert_eq!(
            roundtripped.vpip_calling_station_threshold,
            original.vpip_calling_station_threshold
        );
        assert_eq!(roundtripped.min_hands_light, original.min_hands_light);
        assert_eq!(roundtripped.min_hands_heavy, original.min_hands_heavy);
    }

    #[test]
    fn decode_clamps_out_of_bounds() {
        let mut out_of_bounds = encode(&ExploitConfig::default());
        out_of_bounds[0] = 99.0; // fold_to_cbet_high_threshold: way above HI[0] = 0.90
        out_of_bounds[8] = -5.0; // fold_to_cbet_high_multiplier: below LO[8] = 1.00
        let decoded = decode(&out_of_bounds);
        assert!(decoded.fold_to_cbet_high_threshold <= HI[0]);
        assert!(decoded.fold_to_cbet_high_multiplier >= LO[8]);
    }

    #[test]
    fn decode_enforces_hands_order() {
        let mut params = encode(&ExploitConfig::default());
        // Set min_hands_heavy below min_hands_light.
        params[14] = 50.0; // min_hands_light
        params[15] = 20.0; // min_hands_heavy < min_hands_light — must be corrected
        let decoded = decode(&params);
        assert!(
            decoded.min_hands_heavy >= decoded.min_hands_light,
            "min_hands_heavy ({}) must be >= min_hands_light ({})",
            decoded.min_hands_heavy,
            decoded.min_hands_light,
        );
    }

    #[test]
    fn bounds_cover_default() {
        let v = encode(&ExploitConfig::default());
        for i in 0..DIM {
            assert!(
                v[i] > LO[i] && v[i] < HI[i],
                "default param[{i}] = {} is not strictly inside ({}, {})",
                v[i],
                LO[i],
                HI[i]
            );
        }
    }

    #[test]
    fn ranges_are_all_positive() {
        assert!(ranges().iter().all(|&r| r > 0.0));
    }
}
