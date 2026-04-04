//! Top-level bot personality type.
//!
//! A [`BotProfile`] combines a [`RangeStrategy`] and a [`BettingStrategy`]
//! into a named, serializable playing style. Profiles are typically stored as
//! YAML files and loaded by agent binaries at startup.
//!
//! YAML support requires the **`bot-profiles`** feature flag.

use crate::bot::betting_strategy::BettingStrategy;
use crate::bot::range_strategy::RangeStrategy;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(feature = "bot-profiles")]
use std::path::Path;

// ── PlayStyle ─────────────────────────────────────────────────────────────────

/// The broad playing archetype a bot profile represents.
///
/// Used for display and filtering; does not affect game logic directly —
/// that is controlled by the [`RangeStrategy`] and [`BettingStrategy`] fields.
///
/// # Examples
///
/// ```
/// use pkcore::bot::profile::PlayStyle;
///
/// let style = PlayStyle::TightPassive;
/// assert_eq!(style.to_string(), "Tight Passive");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayStyle {
    /// Strong hands only; rarely bluffs or raises without the nuts.
    TightPassive,
    /// Wide ranges; frequent bets, raises, and bluffs.
    LooseAggressive,
    /// Balanced frequencies informed by GTO solver output.
    Gto,
    /// A hand-crafted style not matching a standard archetype.
    Custom,
}

impl fmt::Display for PlayStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayStyle::TightPassive => write!(f, "Tight Passive"),
            PlayStyle::LooseAggressive => write!(f, "Loose Aggressive"),
            PlayStyle::Gto => write!(f, "GTO"),
            PlayStyle::Custom => write!(f, "Custom"),
        }
    }
}

// ── BotError ─────────────────────────────────────────────────────────────────

/// Errors that can occur when loading or saving a [`BotProfile`].
///
/// # Examples
///
/// ```
/// use pkcore::bot::profile::BotError;
///
/// let e = BotError::InvalidProfile("missing field".into());
/// assert!(e.to_string().contains("missing field"));
/// ```
#[derive(Debug)]
pub enum BotError {
    /// The profile data is structurally invalid.
    InvalidProfile(String),
    /// A YAML parse or serialization error (requires `bot-profiles` feature).
    #[cfg(feature = "bot-profiles")]
    Yaml(serde_yaml_bw::Error),
    /// An I/O error reading or writing a profile file.
    #[cfg(not(target_arch = "wasm32"))]
    Io(std::io::Error),
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BotError::InvalidProfile(msg) => write!(f, "invalid profile: {msg}"),
            #[cfg(feature = "bot-profiles")]
            BotError::Yaml(e) => write!(f, "YAML error: {e}"),
            #[cfg(not(target_arch = "wasm32"))]
            BotError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for BotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BotError::InvalidProfile(_) => None,
            #[cfg(feature = "bot-profiles")]
            BotError::Yaml(e) => Some(e),
            #[cfg(not(target_arch = "wasm32"))]
            BotError::Io(e) => Some(e),
        }
    }
}

#[cfg(feature = "bot-profiles")]
impl From<serde_yaml_bw::Error> for BotError {
    fn from(e: serde_yaml_bw::Error) -> Self {
        BotError::Yaml(e)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<std::io::Error> for BotError {
    fn from(e: std::io::Error) -> Self {
        BotError::Io(e)
    }
}

// ── BotProfile ────────────────────────────────────────────────────────────────

/// A fully serializable poker bot personality.
///
/// Combines a [`RangeStrategy`] and a [`BettingStrategy`] under a named
/// playing style. Profiles are typically stored as YAML and loaded at
/// agent startup.
///
/// YAML I/O requires the **`bot-profiles`** crate feature. File I/O is
/// additionally gated to non-WASM targets.
///
/// # Examples
///
/// ```
/// use pkcore::bot::betting_strategy::BettingStrategy;
/// use pkcore::bot::profile::{BotProfile, PlayStyle};
/// use pkcore::bot::range_strategy::RangeStrategy;
///
/// let profile = BotProfile::new(
///     "tight_passive",
///     "Plays strong hands only.",
///     PlayStyle::TightPassive,
///     RangeStrategy::tight_passive(),
///     BettingStrategy::tight_passive(),
/// );
/// assert_eq!(profile.style, PlayStyle::TightPassive);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BotProfile {
    /// Short identifier used as a filename stem (e.g. `"tight_passive"`).
    pub name: String,
    /// Human-readable description of this playing style.
    pub description: String,
    /// Broad archetype classification.
    pub style: PlayStyle,
    /// Preflop ranges and postflop c-bet frequency.
    pub range_strategy: RangeStrategy,
    /// Aggression, bluff frequency, and preferred bet sizing.
    pub betting_strategy: BettingStrategy,
}

impl BotProfile {
    /// Creates a new `BotProfile`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    /// use pkcore::bot::range_strategy::RangeStrategy;
    ///
    /// let profile = BotProfile::new(
    ///     "gto",
    ///     "Balanced GTO strategy.",
    ///     PlayStyle::Gto,
    ///     RangeStrategy::gto(),
    ///     BettingStrategy::gto(),
    /// );
    /// assert_eq!(profile.name, "gto");
    /// ```
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        style: PlayStyle,
        range_strategy: RangeStrategy,
        betting_strategy: BettingStrategy,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            style,
            range_strategy,
            betting_strategy,
        }
    }

    /// Returns the `TightPassive` reference profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    ///
    /// let p = BotProfile::tight_passive();
    /// assert_eq!(p.style, PlayStyle::TightPassive);
    /// ```
    #[must_use]
    pub fn tight_passive() -> Self {
        Self::new(
            "tight_passive",
            "Plays strong hands only; rarely bluffs or raises without a strong holding.",
            PlayStyle::TightPassive,
            RangeStrategy::tight_passive(),
            BettingStrategy::tight_passive(),
        )
    }

    /// Returns the `LooseAggressive` reference profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    ///
    /// let p = BotProfile::loose_aggressive();
    /// assert_eq!(p.style, PlayStyle::LooseAggressive);
    /// ```
    #[must_use]
    pub fn loose_aggressive() -> Self {
        Self::new(
            "loose_aggressive",
            "Wide ranges, frequent bets and bluffs — puts maximum pressure on opponents.",
            PlayStyle::LooseAggressive,
            RangeStrategy::loose_aggressive(),
            BettingStrategy::loose_aggressive(),
        )
    }

    /// Returns the `Gto` reference profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    ///
    /// let p = BotProfile::gto();
    /// assert_eq!(p.style, PlayStyle::Gto);
    /// ```
    #[must_use]
    pub fn gto() -> Self {
        Self::new(
            "gto",
            "Balanced frequencies informed by GTO solver output; unexploitable at equilibrium.",
            PlayStyle::Gto,
            RangeStrategy::gto(),
            BettingStrategy::gto(),
        )
    }

    // ── YAML serialization (requires bot-profiles feature) ────────────────────

    /// Serializes this profile to a YAML string.
    ///
    /// # Errors
    ///
    /// Returns [`BotError::Yaml`] if serialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::bot::profile::BotProfile;
    ///
    /// let yaml = BotProfile::gto().to_yaml_string().unwrap();
    /// assert!(yaml.contains("gto"));
    /// # }
    /// ```
    #[cfg(feature = "bot-profiles")]
    pub fn to_yaml_string(&self) -> Result<String, BotError> {
        Ok(serde_yaml_bw::to_string(self)?)
    }

    /// Deserializes a `BotProfile` from a YAML string.
    ///
    /// # Errors
    ///
    /// Returns [`BotError::Yaml`] if the string is not valid YAML or does
    /// not match the expected schema.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::bot::profile::BotProfile;
    ///
    /// let yaml = BotProfile::gto().to_yaml_string().unwrap();
    /// let loaded = BotProfile::from_yaml_str(&yaml).unwrap();
    /// assert_eq!(loaded.name, "gto");
    /// # }
    /// ```
    #[cfg(feature = "bot-profiles")]
    pub fn from_yaml_str(s: &str) -> Result<Self, BotError> {
        Ok(serde_yaml_bw::from_str(s)?)
    }

    /// Saves this profile to a YAML file.
    ///
    /// *Not available on `wasm32`.*
    ///
    /// # Errors
    ///
    /// Returns [`BotError::Yaml`] or [`BotError::Io`] on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "bot-profiles", not(target_arch = "wasm32")))]
    /// # {
    /// use pkcore::bot::profile::BotProfile;
    ///
    /// BotProfile::gto().to_file("/tmp/gto.yaml").unwrap();
    /// # }
    /// ```
    #[cfg(all(feature = "bot-profiles", not(target_arch = "wasm32")))]
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), BotError> {
        let yaml = self.to_yaml_string()?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Loads a `BotProfile` from a YAML file.
    ///
    /// *Not available on `wasm32`.*
    ///
    /// # Errors
    ///
    /// Returns [`BotError::Yaml`] or [`BotError::Io`] on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "bot-profiles", not(target_arch = "wasm32")))]
    /// # {
    /// use pkcore::bot::profile::BotProfile;
    ///
    /// let profile = BotProfile::from_file("/tmp/gto.yaml").unwrap();
    /// assert_eq!(profile.name, "gto");
    /// # }
    /// ```
    #[cfg(all(feature = "bot-profiles", not(target_arch = "wasm32")))]
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, BotError> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_yaml_str(&contents)
    }
}

impl fmt::Display for BotProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.style)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_profile_new_fields() {
        let p = BotProfile::new(
            "test",
            "A test profile.",
            PlayStyle::Custom,
            RangeStrategy::gto(),
            BettingStrategy::gto(),
        );
        assert_eq!(p.name, "test");
        assert_eq!(p.style, PlayStyle::Custom);
    }

    #[test]
    fn test_bot_profile_tight_passive() {
        let p = BotProfile::tight_passive();
        assert_eq!(p.style, PlayStyle::TightPassive);
    }

    #[test]
    fn test_bot_profile_loose_aggressive() {
        let p = BotProfile::loose_aggressive();
        assert_eq!(p.style, PlayStyle::LooseAggressive);
    }

    #[test]
    fn test_bot_profile_gto() {
        let p = BotProfile::gto();
        assert_eq!(p.style, PlayStyle::Gto);
    }

    #[test]
    fn test_play_style_display() {
        assert_eq!(PlayStyle::TightPassive.to_string(), "Tight Passive");
        assert_eq!(PlayStyle::LooseAggressive.to_string(), "Loose Aggressive");
        assert_eq!(PlayStyle::Gto.to_string(), "GTO");
        assert_eq!(PlayStyle::Custom.to_string(), "Custom");
    }

    #[test]
    fn test_bot_profile_display() {
        let p = BotProfile::gto();
        assert_eq!(p.to_string(), "gto (GTO)");
    }

    #[test]
    fn test_bot_error_display() {
        let e = BotError::InvalidProfile("bad data".into());
        assert!(e.to_string().contains("bad data"));
    }

    #[test]
    fn test_bot_profile_serde_json_round_trip() {
        let p = BotProfile::gto();
        let json = serde_json::to_string(&p).unwrap();
        let loaded: BotProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(p, loaded);
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn test_bot_profile_yaml_round_trip() {
        let p = BotProfile::tight_passive();
        let yaml = p.to_yaml_string().unwrap();
        let loaded = BotProfile::from_yaml_str(&yaml).unwrap();
        assert_eq!(p, loaded);
    }

    #[cfg(all(feature = "bot-profiles", not(target_arch = "wasm32")))]
    #[test]
    fn test_bot_profile_file_round_trip() {
        let p = BotProfile::loose_aggressive();
        let path = std::env::temp_dir().join("pkcore_test_bot_profile.yaml");
        p.to_file(&path).unwrap();
        let loaded = BotProfile::from_file(&path).unwrap();
        assert_eq!(p, loaded);
        let _ = std::fs::remove_file(&path);
    }
}
