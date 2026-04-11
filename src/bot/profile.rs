//! Top-level bot personality type.
//!
//! A [`BotProfile`] combines a [`RangeStrategy`] and a [`BettingStrategy`]
//! into a named, serializable playing style. Profiles are typically stored as
//! YAML files and loaded by agent binaries at startup.
//!
//! YAML support requires the **`bot-profiles`** feature flag.

use crate::bot::betting_strategy::BettingStrategy;
use crate::bot::playbook::Playbook;
use crate::bot::range_strategy::RangeStrategy;
use crate::bot::weighted_range::WeightedRange;
use crate::casino::table::position::Position;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(feature = "bot-profiles")]
use std::path::Path;

// ── PlayStyle ─────────────────────────────────────────────────────────────────

/// A free-form playing style label attached to a [`BotProfile`].
///
/// `PlayStyle` is a transparent newtype over [`String`] so any label can be
/// used in YAML profile files without requiring code changes. The named
/// constructors on [`BotProfile`] (`gto()`, `tight_passive()`,
/// `loose_aggressive()`) set conventional labels as examples, but callers are
/// free to supply any name they choose.
///
/// Serializes as a bare string in YAML:
///
/// ```yaml
/// style: tight_passive
/// ```
///
/// # Examples
///
/// ```
/// use pkcore::bot::profile::PlayStyle;
///
/// let style = PlayStyle::new("tight_aggressive");
/// assert_eq!(style.to_string(), "tight_aggressive");
///
/// // Any label works — no code changes needed for new styles
/// let custom = PlayStyle::new("my_custom_style");
/// assert_eq!(custom.to_string(), "my_custom_style");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayStyle(pub String);

impl PlayStyle {
    /// Creates a [`PlayStyle`] with the given label string.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::PlayStyle;
    ///
    /// let style = PlayStyle::new("lag");
    /// assert_eq!(style.to_string(), "lag");
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl fmt::Display for PlayStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
///     PlayStyle::new("tight_passive"),
///     RangeStrategy::tight_passive(),
///     BettingStrategy::tight_passive(),
/// );
/// assert_eq!(profile.style, PlayStyle::new("tight_passive"));
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
    /// Optional position- and table-size-aware strategy overrides.
    ///
    /// When `Some`, [`BotProfile::range_for`] and [`BotProfile::betting_for`]
    /// prefer this over the flat `range_strategy` / `betting_strategy` fields.
    /// Profiles without a playbook serialize identically to before this field
    /// was added.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playbook: Option<Playbook>,
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
    ///     PlayStyle::new("gto"),
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
            playbook: None,
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
    /// assert_eq!(p.style, PlayStyle::new("tight_passive"));
    /// ```
    #[must_use]
    pub fn tight_passive() -> Self {
        Self::new(
            "tight_passive",
            "Plays strong hands only; rarely bluffs or raises without a strong holding.",
            PlayStyle::new("tight_passive"),
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
    /// assert_eq!(p.style, PlayStyle::new("loose_aggressive"));
    /// ```
    #[must_use]
    pub fn loose_aggressive() -> Self {
        Self::new(
            "loose_aggressive",
            "Wide ranges, frequent bets and bluffs — puts maximum pressure on opponents.",
            PlayStyle::new("loose_aggressive"),
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
    /// assert_eq!(p.style, PlayStyle::new("gto"));
    /// ```
    #[must_use]
    pub fn gto() -> Self {
        Self::new(
            "gto",
            "Balanced frequencies informed by GTO solver output; unexploitable at equilibrium.",
            PlayStyle::new("gto"),
            RangeStrategy::gto(),
            BettingStrategy::gto(),
        )
    }

    // ── Playbook builder ──────────────────────────────────────────────────────

    /// Attaches a [`Playbook`] to this profile, enabling position- and
    /// table-size-aware strategy resolution via [`BotProfile::range_for`] and
    /// [`BotProfile::betting_for`].
    ///
    /// Consumes `self` and returns the updated profile, making it easy to
    /// chain onto a named constructor:
    ///
    /// ```rust
    /// use pkcore::bot::playbook::Playbook;
    /// use pkcore::bot::profile::BotProfile;
    ///
    /// let profile = BotProfile::gto().with_playbook(Playbook::gto());
    /// assert!(profile.playbook.is_some());
    /// ```
    #[must_use]
    pub fn with_playbook(mut self, playbook: Playbook) -> Self {
        self.playbook = Some(playbook);
        self
    }

    // ── Playbook resolution helpers ───────────────────────────────────────────

    /// Resolves the [`WeightedRange`] for a given `(seats, position, action)` triple.
    ///
    /// Returns `None` when the playbook is absent, the seat count has no entry,
    /// or the action is not mapped for that position.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let profile = BotProfile::gto();
    /// // No playbook on the base profile → returns None
    /// assert!(profile.range_for(6, Position::BTN, "open_raise").is_none());
    /// ```
    #[must_use]
    pub fn range_for(&self, seats: u8, pos: Position, action: &str) -> Option<&WeightedRange> {
        self.playbook
            .as_ref()
            .and_then(|pb| pb.for_seats(seats))
            .and_then(|entry| entry.position_ranges.for_position(pos).for_action(action))
    }

    /// Resolves the [`WeightedRange`] for `(seats, position, action)`, or
    /// constructs a flat fallback from the profile's `range_strategy` fields.
    ///
    /// Fallback behaviour:
    /// - `"open_raise"` → `WeightedRange::from_flat(&self.range_strategy.open_raise)`
    /// - `"three_bet"`  → `WeightedRange::from_flat(&self.range_strategy.three_bet)`
    /// - anything else  → empty [`WeightedRange`]
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let profile = BotProfile::gto();
    /// // Falls back to range_strategy.open_raise
    /// let wr = profile.range_for_or_default(6, Position::BTN, "open_raise");
    /// assert!(!wr.is_empty());
    /// ```
    #[must_use]
    pub fn range_for_or_default(&self, seats: u8, pos: Position, action: &str) -> WeightedRange {
        self.range_for(seats, pos, action)
            .cloned()
            .unwrap_or_else(|| match action {
                "open_raise" => WeightedRange::from_flat(&self.range_strategy.open_raise),
                "three_bet" => WeightedRange::from_flat(&self.range_strategy.three_bet),
                _ => WeightedRange::new(),
            })
    }

    /// Resolves the [`BettingStrategy`] for `(seats, position)`.
    ///
    /// Falls back to `&self.betting_strategy` when the playbook is absent or
    /// the seat count / position has no specific entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let profile = BotProfile::gto();
    /// // No playbook → returns the flat betting_strategy
    /// assert_eq!(
    ///     profile.betting_for(6, Position::BTN).aggression_factor,
    ///     profile.betting_strategy.aggression_factor,
    /// );
    /// ```
    #[must_use]
    pub fn betting_for(&self, seats: u8, pos: Position) -> &BettingStrategy {
        self.playbook
            .as_ref()
            .and_then(|pb| pb.for_seats(seats))
            .map_or(&self.betting_strategy, |entry| {
                entry.positional_betting.for_position(pos)
            })
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
            PlayStyle::new("custom"),
            RangeStrategy::gto(),
            BettingStrategy::gto(),
        );
        assert_eq!(p.name, "test");
        assert_eq!(p.style, PlayStyle::new("custom"));
    }

    #[test]
    fn test_bot_profile_tight_passive() {
        let p = BotProfile::tight_passive();
        assert_eq!(p.style, PlayStyle::new("tight_passive"));
    }

    #[test]
    fn test_bot_profile_loose_aggressive() {
        let p = BotProfile::loose_aggressive();
        assert_eq!(p.style, PlayStyle::new("loose_aggressive"));
    }

    #[test]
    fn test_bot_profile_gto() {
        let p = BotProfile::gto();
        assert_eq!(p.style, PlayStyle::new("gto"));
    }

    #[test]
    fn test_play_style_display() {
        assert_eq!(PlayStyle::new("tight_passive").to_string(), "tight_passive");
        assert_eq!(PlayStyle::new("loose_aggressive").to_string(), "loose_aggressive");
        assert_eq!(PlayStyle::new("gto").to_string(), "gto");
        assert_eq!(PlayStyle::new("my_custom_style").to_string(), "my_custom_style");
    }

    #[test]
    fn test_bot_profile_display() {
        let p = BotProfile::gto();
        assert_eq!(p.to_string(), "gto (gto)");
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

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn test_bot_profile_yaml_round_trip_with_playbook() {
        use crate::bot::playbook::Playbook;
        let p = BotProfile::gto().with_playbook(Playbook::gto());
        assert!(p.playbook.is_some());
        let yaml = p.to_yaml_string().unwrap();
        let loaded = BotProfile::from_yaml_str(&yaml).unwrap();
        assert_eq!(p, loaded);
        assert!(loaded.playbook.is_some());
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn test_bot_profile_yaml_without_playbook_unchanged() {
        // Profiles without a playbook must not gain a `playbook:` key in YAML,
        // preserving backward compatibility with existing profile files.
        let p = BotProfile::gto();
        let yaml = p.to_yaml_string().unwrap();
        assert!(!yaml.contains("playbook"), "flat profile should not emit playbook key");
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

    /// Each file in `data/bots/` must parse without error.
    #[cfg(all(feature = "bot-profiles", not(target_arch = "wasm32")))]
    #[test]
    fn test_data_bots_all_load() {
        let names = [
            "gto",
            "tight_passive",
            "loose_aggressive",
            "tight_aggressive",
            "loose_passive",
            "maniac",
            "abc",
            "short_stack_ninja",
        ];
        for name in names {
            let path = format!("data/bots/{name}.yaml");
            let loaded = BotProfile::from_file(&path)
                .unwrap_or_else(|e| panic!("failed to load {path}: {e}"));
            assert_eq!(loaded.name, name, "{path}: name field mismatch");
        }
    }

    /// The three constructor-backed profiles must be byte-identical to their
    /// YAML files after a round-trip through deserialization.
    #[cfg(all(feature = "bot-profiles", not(target_arch = "wasm32")))]
    #[test]
    fn test_data_bots_constructors_match_files() {
        for (name, expected) in [
            ("gto", BotProfile::gto()),
            ("tight_passive", BotProfile::tight_passive()),
            ("loose_aggressive", BotProfile::loose_aggressive()),
        ] {
            let path = format!("data/bots/{name}.yaml");
            let from_file = BotProfile::from_file(&path)
                .unwrap_or_else(|e| panic!("failed to load {path}: {e}"));
            assert_eq!(from_file, expected, "{path} does not match constructor output");
        }
    }
}
