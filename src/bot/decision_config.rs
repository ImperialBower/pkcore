//! Graded, data-configured decision capabilities for a [`crate::bot::profile::BotProfile`].
//!
//! EPIC-36 turns each pkcore decision capability into a graded knob that lives
//! in the `BotProfile` YAML. The low end of every knob reproduces the decider's
//! historical behavior, so a profile that omits the `decision:` section — or
//! sets every knob to its default — plays exactly as it did before this module
//! existed. The high end wires in the real pkcore engine (multi-way equity,
//! position-aware ranges, draw outs, opponent-adjusted exploitation).

use serde::{Deserialize, Serialize};

/// Default Monte Carlo sample budget for [`EquityMode::Fast`].
fn default_samples() -> u32 {
    2000
}

/// Graded configuration for every pkcore decision capability.
///
/// Every field carries a `#[serde(default)]` and the struct's [`Default`] is
/// the historical decider behavior, so an absent `decision:` section and a
/// fully-defaulted one both deserialize to a bot that plays exactly as it did
/// before EPIC-36.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DecisionConfig {
    /// Postflop hand-strength source.
    pub equity: EquityMode,
    /// Preflop range source.
    pub ranges: RangeMode,
    /// How strictly equity must beat pot odds before calling.
    pub pot_odds: PotOddsConfig,
    /// Draw/outs equity augmentation on the flop and turn.
    pub outs: Toggle,
    /// Opponent-adjusted exploitation (acts only when stats are attached).
    pub exploit: ExploitMode,
    /// Preflop decision chart source.
    pub preflop_charts: PreflopCharts,
}

impl DecisionConfig {
    /// Returns `true` when this config is byte-identical to [`Default`].
    ///
    /// Used as a serde `skip_serializing_if` predicate so profiles that never
    /// opt into graded capabilities do not gain a `decision:` key in their YAML,
    /// preserving backward compatibility with existing profile files.
    #[must_use]
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

/// Postflop hand-strength source.
///
/// `Off` keeps the historical hand-rank proxy (`1 - hand_rank_value / 7462`).
/// `Fast` and `Exact` route through the real multi-way [`crate::analysis::equity`]
/// engine — seeded Monte Carlo and exact enumeration respectively.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum EquityMode {
    /// Hand-rank proxy — the pre-EPIC-36 behavior.
    #[default]
    Off,
    /// Seeded Monte Carlo with a bounded sample budget.
    Fast {
        /// Monte Carlo sample budget.
        #[serde(default = "default_samples")]
        samples: u32,
    },
    /// Exact enumeration of remaining runouts.
    Exact,
}

/// Preflop range source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RangeMode {
    /// Flat `range_strategy.open_raise` lookup — the historical behavior.
    #[default]
    Flat,
    /// Position-aware lookup via the profile's `playbook`.
    PositionAware,
}

/// Graded pot-odds discipline.
///
/// `discipline` scales how strictly equity must beat pot odds before the
/// decider calls: `1.0` is the strict break-even threshold (historical
/// behavior), `0.0` ignores pot odds entirely (looser, weaker).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PotOddsConfig {
    /// Call-threshold strictness in `[0.0, 1.0]`.
    pub discipline: f64,
}

impl Default for PotOddsConfig {
    fn default() -> Self {
        Self { discipline: 1.0 }
    }
}

/// A simple off/on capability toggle, defaulting to `Off`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Toggle {
    /// Capability disabled — the historical behavior.
    #[default]
    Off,
    /// Capability enabled.
    On,
}

/// Opponent-adjusted exploitation intensity.
///
/// Acts only when the table snapshot carries `opponent_stats`; a no-op
/// otherwise, so the knob is safe on any run path and never depends on
/// opponent identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum ExploitMode {
    /// No opponent adjustment — the historical behavior.
    #[default]
    Off,
    /// Light adjustment (higher sample gate before adjusting).
    Light,
    /// Heavy adjustment (lower sample gate; adjusts sooner).
    Heavy,
}

/// Preflop decision chart source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PreflopCharts {
    /// No chart — the historical range-membership behavior.
    #[default]
    Off,
    /// Heads-up precomputed odds table (EPIC-36 follow-on; see corrigendum).
    Hup,
    /// Offline-generated GTO charts (EPIC-36 follow-on; see corrigendum).
    Solver,
}

#[cfg(test)]
#[allow(non_snake_case)]
mod decision_config_tests {
    use super::*;

    #[test]
    fn default_reproduces_current_behavior() {
        let d = DecisionConfig::default();
        assert_eq!(d.equity, EquityMode::Off);
        assert_eq!(d.ranges, RangeMode::Flat);
        assert!((d.pot_odds.discipline - 1.0).abs() < f64::EPSILON);
        assert_eq!(d.outs, Toggle::Off);
        assert_eq!(d.exploit, ExploitMode::Off);
        assert_eq!(d.preflop_charts, PreflopCharts::Off);
    }

    #[test]
    fn is_default_detects_default_and_non_default() {
        assert!(DecisionConfig::default().is_default());
        let d = DecisionConfig {
            ranges: RangeMode::PositionAware,
            ..DecisionConfig::default()
        };
        assert!(!d.is_default());
    }

    #[test]
    fn equity_mode_json_round_trips() {
        for mode in [EquityMode::Off, EquityMode::Fast { samples: 2000 }, EquityMode::Exact] {
            let json = serde_json::to_string(&mode).unwrap();
            let back: EquityMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, back);
        }
    }

    #[test]
    fn fast_samples_default_applies_when_absent() {
        let m: EquityMode = serde_json::from_str(r#"{"mode":"fast"}"#).unwrap();
        assert_eq!(m, EquityMode::Fast { samples: 2000 });
    }

    #[test]
    fn scalar_enums_round_trip() {
        for r in [RangeMode::Flat, RangeMode::PositionAware] {
            let j = serde_json::to_string(&r).unwrap();
            assert_eq!(r, serde_json::from_str::<RangeMode>(&j).unwrap());
        }
        for t in [Toggle::Off, Toggle::On] {
            let j = serde_json::to_string(&t).unwrap();
            assert_eq!(t, serde_json::from_str::<Toggle>(&j).unwrap());
        }
        for p in [PreflopCharts::Off, PreflopCharts::Hup, PreflopCharts::Solver] {
            let j = serde_json::to_string(&p).unwrap();
            assert_eq!(p, serde_json::from_str::<PreflopCharts>(&j).unwrap());
        }
    }

    #[test]
    fn full_config_json_round_trips() {
        let d = DecisionConfig {
            equity: EquityMode::Fast { samples: 500 },
            ranges: RangeMode::PositionAware,
            pot_odds: PotOddsConfig { discipline: 0.5 },
            outs: Toggle::On,
            exploit: ExploitMode::Heavy,
            preflop_charts: PreflopCharts::Hup,
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: DecisionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }
}
