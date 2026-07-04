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
use crate::games::betting_structure::BettingStructure;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(all(feature = "bot-profiles", not(target_arch = "wasm32")))]
use std::path::Path;

// ── PlayStyle ─────────────────────────────────────────────────────────────────

/// A typed playing style label attached to a [`BotProfile`].
///
/// Known archetypes map to named variants; any other label maps to
/// [`PlayStyle::Custom`]. YAML serialization uses `snake_case` strings
/// (`"tight_passive"`, `"gto"`, etc.) so existing profile files need no changes.
///
/// # Examples
///
/// ```
/// use pkcore::bot::profile::PlayStyle;
///
/// // Known archetypes use named variants
/// assert_eq!(PlayStyle::TightPassive.to_string(), "tight_passive");
/// assert_eq!(PlayStyle::Gto.to_string(), "gto");
///
/// // Unknown labels become Custom — PlayStyle::new() handles both
/// let lag = PlayStyle::new("lag");
/// assert_eq!(lag.to_string(), "lag");
/// assert_eq!(PlayStyle::new("tight_passive"), PlayStyle::TightPassive);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayStyle {
    TightPassive,
    LooseAggressive,
    Gto,
    TightAggressive,
    LoosePassive,
    Maniac,
    Abc,
    ShortStackNinja,
    #[serde(untagged)]
    Custom(String),
}

impl PlayStyle {
    /// Returns the named variant for known archetype strings, otherwise
    /// `Custom(name)`.
    ///
    /// This allows existing code that passes string literals to keep working
    /// while also accepting the enum variant syntax.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::PlayStyle;
    ///
    /// assert_eq!(PlayStyle::new("tight_passive"), PlayStyle::TightPassive);
    /// assert_eq!(PlayStyle::new("gto"),           PlayStyle::Gto);
    /// assert_eq!(PlayStyle::new("my_style"),      PlayStyle::Custom("my_style".into()));
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        match name.into().as_str() {
            "tight_passive" => Self::TightPassive,
            "loose_aggressive" => Self::LooseAggressive,
            "gto" => Self::Gto,
            "tight_aggressive" => Self::TightAggressive,
            "loose_passive" => Self::LoosePassive,
            "maniac" => Self::Maniac,
            "abc" => Self::Abc,
            "short_stack_ninja" => Self::ShortStackNinja,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl fmt::Display for PlayStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TightPassive => write!(f, "tight_passive"),
            Self::LooseAggressive => write!(f, "loose_aggressive"),
            Self::Gto => write!(f, "gto"),
            Self::TightAggressive => write!(f, "tight_aggressive"),
            Self::LoosePassive => write!(f, "loose_passive"),
            Self::Maniac => write!(f, "maniac"),
            Self::Abc => write!(f, "abc"),
            Self::ShortStackNinja => write!(f, "short_stack_ninja"),
            Self::Custom(s) => write!(f, "{s}"),
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
    ///
    /// The `serde_yaml_bw` cause is stringified so the format crate does not leak
    /// into pkcore's public API; the `From<serde_yaml_bw::Error>` impl is the
    /// blessed conversion seam (`docs/AUDIT_Fable_5.md` III.2).
    #[cfg(feature = "bot-profiles")]
    Yaml(String),
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
            BotError::Yaml(_) => None,
            #[cfg(not(target_arch = "wasm32"))]
            BotError::Io(e) => Some(e),
        }
    }
}

#[cfg(feature = "bot-profiles")]
#[allow(clippy::disallowed_types)] // blessed seam: format error stringified, never re-exposed
impl From<serde_yaml_bw::Error> for BotError {
    fn from(e: serde_yaml_bw::Error) -> Self {
        BotError::Yaml(e.to_string())
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
    /// Optional betting structure marker (EPIC-30 Phase 7).
    ///
    /// Carries a marker `BettingStructure` for profiles authored for a
    /// specific variant (e.g. `BettingStructure::FixedLimit { .. }` for
    /// FLHE-tuned profiles). The runtime decider does *not* consult this
    /// field — the authoritative betting structure comes from the
    /// [`crate::bot::table_snapshot::TableSnapshot`] supplied by the
    /// table. This field is for serde clarity ("this profile was tuned
    /// for FLHE") and for the [`BotProfile::for_limit_holdem`] factory
    /// to encode its provenance. Defaults to `None` for variant-agnostic
    /// profiles; existing NLHE YAML round-trips unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub betting_structure: Option<BettingStructure>,
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
            betting_structure: None,
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
            PlayStyle::TightPassive,
            RangeStrategy::tight_passive(),
            BettingStrategy::tight_passive(),
        )
        .with_playbook(Playbook::tight_passive())
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
            PlayStyle::LooseAggressive,
            RangeStrategy::loose_aggressive(),
            BettingStrategy::loose_aggressive(),
        )
        .with_playbook(Playbook::loose_aggressive())
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
            PlayStyle::Gto,
            RangeStrategy::gto(),
            BettingStrategy::gto(),
        )
        .with_playbook(Playbook::gto())
    }

    /// Returns the `TightAggressive` reference profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    ///
    /// let p = BotProfile::tight_aggressive();
    /// assert_eq!(p.style, PlayStyle::new("tight_aggressive"));
    /// ```
    #[must_use]
    pub fn tight_aggressive() -> Self {
        Self::new(
            "tight_aggressive",
            "Selective hand selection with maximum postflop aggression — the baseline winning style.",
            PlayStyle::TightAggressive,
            RangeStrategy::tight_aggressive(),
            BettingStrategy::tight_aggressive(),
        )
    }

    /// Returns the `LoosePassive` reference profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    ///
    /// let p = BotProfile::loose_passive();
    /// assert_eq!(p.style, PlayStyle::new("loose_passive"));
    /// ```
    #[must_use]
    pub fn loose_passive() -> Self {
        Self::new(
            "loose_passive",
            "Wide hand selection with passive betting — the classic calling station archetype.",
            PlayStyle::LoosePassive,
            RangeStrategy::loose_passive(),
            BettingStrategy::loose_passive(),
        )
    }

    /// Returns the `Maniac` reference profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    ///
    /// let p = BotProfile::maniac();
    /// assert_eq!(p.style, PlayStyle::new("maniac"));
    /// ```
    #[must_use]
    pub fn maniac() -> Self {
        Self::new(
            "maniac",
            "Extreme aggressor — bets and raises relentlessly with a very high bluff frequency.",
            PlayStyle::Maniac,
            RangeStrategy::maniac(),
            BettingStrategy::maniac(),
        )
    }

    /// Returns the `Abc` reference profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    ///
    /// let p = BotProfile::abc();
    /// assert_eq!(p.style, PlayStyle::new("abc"));
    /// ```
    #[must_use]
    pub fn abc() -> Self {
        Self::new(
            "abc",
            "By-the-book play — bets strong hands and folds weak ones with no deception or bluffing.",
            PlayStyle::Abc,
            RangeStrategy::abc(),
            BettingStrategy::abc(),
        )
    }

    /// Returns the `ShortStackNinja` reference profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    ///
    /// let p = BotProfile::short_stack_ninja();
    /// assert_eq!(p.style, PlayStyle::new("short_stack_ninja"));
    /// ```
    #[must_use]
    pub fn short_stack_ninja() -> Self {
        Self::new(
            "short_stack_ninja",
            "Push-or-fold strategy optimized for short stack situations — all-in or nothing.",
            PlayStyle::ShortStackNinja,
            RangeStrategy::short_stack_ninja(),
            BettingStrategy::short_stack_ninja(),
        )
    }

    /// Returns the `Joker` placeholder profile.
    ///
    /// The joker profile acts as a seat-entry placeholder when pairing a seat
    /// with a [`crate::bot::decider::JokerDecider`].  Its `range_strategy`
    /// and `betting_strategy` fields are copied from [`BotProfile::gto`] and
    /// are **never used in practice** — [`JokerDecider`] ignores the passed
    /// profile and instead decides using whichever standard profile it randomly
    /// selected at hand-start time.
    ///
    /// [`JokerDecider`]: crate::bot::decider::JokerDecider
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    ///
    /// let p = BotProfile::joker();
    /// assert_eq!(p.name, "joker");
    /// assert_eq!(p.style, PlayStyle::new("joker"));
    /// ```
    #[must_use]
    pub fn joker() -> Self {
        Self::new(
            "joker",
            "Randomly adopts a different playing style each hand — unpredictable by design.",
            PlayStyle::new("joker"),
            RangeStrategy::gto(),
            BettingStrategy::gto(),
        )
    }

    /// Returns all 8 standard reference profiles in a fixed order.
    ///
    /// This is the WASM-safe alternative to loading profiles from YAML files via
    /// `from_file()`, which is not available on `wasm32`. Use this in web/WASM
    /// contexts to get the full set of bot personalities.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::BotProfile;
    ///
    /// let profiles = BotProfile::default_profiles();
    /// assert_eq!(profiles.len(), 8);
    /// assert_eq!(profiles[0].name, "gto");
    /// ```
    #[must_use]
    pub fn default_profiles() -> Vec<Self> {
        vec![
            Self::gto(),
            Self::tight_passive(),
            Self::loose_aggressive(),
            Self::tight_aggressive(),
            Self::loose_passive(),
            Self::maniac(),
            Self::abc(),
            Self::short_stack_ninja(),
        ]
    }

    // ── EPIC-30 Phase 7: FLHE factory ─────────────────────────────────────────

    /// Returns a Fixed-Limit Hold'em flavored profile built on top of one
    /// of the base reference profiles (EPIC-30 Phase 7).
    ///
    /// The returned profile carries
    /// `betting_structure = Some(BettingStructure::FixedLimit { 0, 0, 3 })`
    /// as a provenance marker — runtime sizing reads concrete amounts
    /// from the table's [`crate::games::betting_structure::BettingStructure`]
    /// via the snapshot, so the placeholder zeros here are intentional.
    /// Name and description are tagged with `_flhe` for clarity.
    ///
    /// Aggression and range tuning beyond the marker are left to
    /// hand-authored YAML in `data/bots/flhe/`. The decider's FLHE-aware
    /// sizing (Phase 6) already clamps any raise to the legal tier
    /// increment regardless of the profile's `preferred_bet_sizes`, so
    /// using `for_limit_holdem` on a vanilla NLHE base produces valid
    /// FLHE play out of the box.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// let p = BotProfile::for_limit_holdem(&PlayStyle::TightAggressive);
    /// assert_eq!("tight_aggressive_flhe", p.name);
    /// assert!(matches!(
    ///     p.betting_structure,
    ///     Some(BettingStructure::FixedLimit { .. })
    /// ));
    /// ```
    #[must_use]
    pub fn for_limit_holdem(style: &PlayStyle) -> Self {
        let base = match style {
            PlayStyle::TightPassive => Self::tight_passive(),
            PlayStyle::LooseAggressive => Self::loose_aggressive(),
            PlayStyle::TightAggressive => Self::tight_aggressive(),
            PlayStyle::LoosePassive => Self::loose_passive(),
            PlayStyle::Maniac => Self::maniac(),
            PlayStyle::Abc => Self::abc(),
            PlayStyle::ShortStackNinja => Self::short_stack_ninja(),
            PlayStyle::Gto | PlayStyle::Custom(_) => Self::gto(),
        };
        Self {
            name: format!("{}_flhe", base.name),
            description: format!("{} (FLHE-tuned)", base.description),
            betting_structure: Some(BettingStructure::FixedLimit {
                small_bet: 0,
                big_bet: 0,
                raise_cap: 3,
            }),
            ..base
        }
    }

    /// Returns a Pot-Limit Omaha flavored profile built on top of one of
    /// the base reference profiles (EPIC-31 Phase 6).
    ///
    /// Sets `betting_structure = Some(BettingStructure::PotLimit)` as a
    /// provenance marker. Runtime sizing reads the actual betting
    /// structure from the table snapshot — `pot_limit` profiles
    /// authored against NLHE 2-card ranges will play valid PLO with
    /// mediocre hand selection because the decider's range lookup uses
    /// the top-2-of-4 hole cards. GTO PLO ranges are v1.1 polish.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// let p = BotProfile::for_plo(&PlayStyle::LooseAggressive);
    /// assert_eq!("loose_aggressive_plo", p.name);
    /// assert_eq!(Some(BettingStructure::PotLimit), p.betting_structure);
    /// ```
    #[must_use]
    pub fn for_plo(style: &PlayStyle) -> Self {
        let base = match style {
            PlayStyle::TightPassive => Self::tight_passive(),
            PlayStyle::LooseAggressive => Self::loose_aggressive(),
            PlayStyle::TightAggressive => Self::tight_aggressive(),
            PlayStyle::LoosePassive => Self::loose_passive(),
            PlayStyle::Maniac => Self::maniac(),
            PlayStyle::Abc => Self::abc(),
            PlayStyle::ShortStackNinja => Self::short_stack_ninja(),
            PlayStyle::Gto | PlayStyle::Custom(_) => Self::gto(),
        };
        Self {
            name: format!("{}_plo", base.name),
            description: format!("{} (PLO-tuned)", base.description),
            betting_structure: Some(BettingStructure::PotLimit),
            ..base
        }
    }

    /// Returns a Stud Hi flavored profile built on top of one of the
    /// base reference profiles (EPIC-32 Phase 10).
    ///
    /// Sets `betting_structure = Some(BettingStructure::FixedLimit { .. })`
    /// as a provenance marker. The decider's range lookup uses the
    /// top-2-of-3 hole cards on 3rd street (NLHE-style range notation as
    /// placeholders); EPIC-32 Phase 8 added a coarse partial-hand
    /// equity heuristic so mid-hand decisions are pair/trips-aware.
    /// Stronger Stud ranges + Monte Carlo equity are v1.1 polish items.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// let p = BotProfile::for_stud_hi(&PlayStyle::TightAggressive);
    /// assert_eq!("tight_aggressive_stud", p.name);
    /// assert!(matches!(
    ///     p.betting_structure,
    ///     Some(BettingStructure::FixedLimit { .. })
    /// ));
    /// ```
    #[must_use]
    pub fn for_stud_hi(style: &PlayStyle) -> Self {
        let base = match style {
            PlayStyle::TightPassive => Self::tight_passive(),
            PlayStyle::LooseAggressive => Self::loose_aggressive(),
            PlayStyle::TightAggressive => Self::tight_aggressive(),
            PlayStyle::LoosePassive => Self::loose_passive(),
            PlayStyle::Maniac => Self::maniac(),
            PlayStyle::Abc => Self::abc(),
            PlayStyle::ShortStackNinja => Self::short_stack_ninja(),
            PlayStyle::Gto | PlayStyle::Custom(_) => Self::gto(),
        };
        Self {
            name: format!("{}_stud", base.name),
            description: format!("{} (Stud Hi-tuned)", base.description),
            betting_structure: Some(BettingStructure::FixedLimit {
                small_bet: 0,
                big_bet: 0,
                raise_cap: 3,
            }),
            ..base
        }
    }

    /// Returns a Razz flavored profile built on top of one of the base
    /// reference profiles (EPIC-33 Phase 4).
    ///
    /// Sets `betting_structure = Some(BettingStructure::FixedLimit { .. })`
    /// as a provenance marker; appends a `_razz` suffix to the base
    /// name. The decider currently reuses the Stud-family mid-hand
    /// equity heuristic (pair detection happens to give the right
    /// signal in both variants on 3rd/4th street, since paired
    /// holdings are bad in both Stud Hi and Razz). True Razz-specific
    /// equity (rewarding pair-free low hands) and Razz GTO ranges are
    /// v1.1 polish items.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::{BotProfile, PlayStyle};
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// let p = BotProfile::for_razz(&PlayStyle::TightAggressive);
    /// assert_eq!("tight_aggressive_razz", p.name);
    /// assert!(matches!(
    ///     p.betting_structure,
    ///     Some(BettingStructure::FixedLimit { .. })
    /// ));
    /// ```
    #[must_use]
    pub fn for_razz(style: &PlayStyle) -> Self {
        let base = match style {
            PlayStyle::TightPassive => Self::tight_passive(),
            PlayStyle::LooseAggressive => Self::loose_aggressive(),
            PlayStyle::TightAggressive => Self::tight_aggressive(),
            PlayStyle::LoosePassive => Self::loose_passive(),
            PlayStyle::Maniac => Self::maniac(),
            PlayStyle::Abc => Self::abc(),
            PlayStyle::ShortStackNinja => Self::short_stack_ninja(),
            PlayStyle::Gto | PlayStyle::Custom(_) => Self::gto(),
        };
        Self {
            name: format!("{}_razz", base.name),
            description: format!("{} (Razz-tuned)", base.description),
            betting_structure: Some(BettingStructure::FixedLimit {
                small_bet: 0,
                big_bet: 0,
                raise_cap: 3,
            }),
            ..base
        }
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
    /// // GTO playbook has a 6-max BTN open_raise entry
    /// assert!(profile.range_for(6, Position::BTN, "open_raise").is_some());
    /// // No entry for 3-max → returns None
    /// assert!(profile.range_for(3, Position::BTN, "open_raise").is_none());
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
    /// // GTO playbook: BTN plays more aggressively than the flat default
    /// assert!(profile.betting_for(6, Position::BTN).aggression_factor >
    ///     profile.betting_strategy.aggression_factor);
    /// // No playbook entry for 3-max → falls back to flat betting_strategy
    /// assert_eq!(
    ///     profile.betting_for(3, Position::BTN).aggression_factor,
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

// ── Decision logic ────────────────────────────────────────────────────────────

#[cfg(feature = "bot-profiles")]
impl BotProfile {
    /// Decide a [`crate::casino::action::PlayerAction`] for the given seat using this profile's
    /// aggression factor and preferred bet sizes.
    ///
    /// The decision is probabilistic:
    /// - When facing a bet (`to_call > 0`), `aggression_factor` controls the
    ///   probability of raising (×0.25) or calling (×1.0) vs folding.
    /// - When the action is checked to the bot (`to_call == 0`),
    ///   `aggression_factor` controls whether to bet or check.
    /// - Bet and raise sizes are sampled uniformly from `preferred_bet_sizes`
    ///   as fractions of the effective pot.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use rand::SeedableRng;
    /// use rand::rngs::SmallRng;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// table.act_forced_bets().unwrap();
    /// table.deal_cards_to_seats().unwrap();
    /// let profile = BotProfile::tight_passive();
    /// let mut rng = SmallRng::seed_from_u64(0);
    /// let utg = table.determine_utg();
    /// let action = profile.decide(&table, utg, &mut rng);
    /// // Tight-passive will fold or call — never raises preflop here
    /// assert!(matches!(action, PlayerAction::Fold | PlayerAction::Call | PlayerAction::Check));
    /// # }
    /// ```
    pub fn decide<R: rand::Rng>(
        &self,
        table: &crate::casino::table_no_cell::TableNoCell,
        seat: u8,
        rng: &mut R,
    ) -> crate::casino::action::PlayerAction {
        use crate::bot::player_action::PlayerAction as BotAction;
        use crate::casino::action::PlayerAction;
        let snapshot = crate::bot::table_snapshot::TableSnapshot::from_table(table, seat);
        match crate::bot::decider::RuleBasedDecider::decide_with_rng(self, &snapshot, rng) {
            BotAction::Fold => PlayerAction::Fold,
            BotAction::Check => PlayerAction::Check,
            BotAction::Call => PlayerAction::Call,
            BotAction::Bet(n) => PlayerAction::Bet(n),
            BotAction::Raise(n) => PlayerAction::Raise(n),
            BotAction::AllIn => PlayerAction::AllIn,
        }
    }
}

impl fmt::Display for BotProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.style)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__profile_tests {
    use super::*;

    #[test]
    fn bot_profile_new_fields() {
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

    // ---- EPIC-30 Phase 7 + 8: FLHE factory + reference profiles ----

    #[test]
    fn for_limit_holdem_marker() {
        let p = BotProfile::for_limit_holdem(&PlayStyle::TightAggressive);
        assert_eq!("tight_aggressive_flhe", p.name);
        assert!(matches!(p.betting_structure, Some(BettingStructure::FixedLimit { .. })));
    }

    #[test]
    fn for_limit_holdem_falls_back_to_gto_for_custom() {
        let p = BotProfile::for_limit_holdem(&PlayStyle::Custom("unknown".into()));
        assert_eq!("gto_flhe", p.name);
    }

    // ---- EPIC-31 Phase 6: for_plo factory ----

    #[test]
    fn for_plo_marker() {
        let p = BotProfile::for_plo(&PlayStyle::LooseAggressive);
        assert_eq!("loose_aggressive_plo", p.name);
        assert_eq!(Some(BettingStructure::PotLimit), p.betting_structure);
    }

    #[test]
    fn for_plo_falls_back_to_gto_for_custom() {
        let p = BotProfile::for_plo(&PlayStyle::Custom("unknown".into()));
        assert_eq!("gto_plo", p.name);
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn plo_loose_aggressive_yaml_loads() {
        let yaml = std::fs::read_to_string("data/bots/plo/loose_aggressive_plo.yaml")
            .expect("PLO LAG profile YAML must exist");
        let p = BotProfile::from_yaml_str(&yaml).expect("PLO LAG must deserialize");
        assert_eq!("loose_aggressive_plo", p.name);
        assert_eq!(Some(BettingStructure::PotLimit), p.betting_structure);
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn plo_tight_aggressive_yaml_loads() {
        let yaml = std::fs::read_to_string("data/bots/plo/tight_aggressive_plo.yaml")
            .expect("PLO TAG profile YAML must exist");
        let p = BotProfile::from_yaml_str(&yaml).expect("PLO TAG must deserialize");
        assert_eq!("tight_aggressive_plo", p.name);
        assert_eq!(Some(BettingStructure::PotLimit), p.betting_structure);
    }

    // ---- EPIC-32 Phase 10: for_stud_hi factory + reference profiles ----

    #[test]
    fn for_stud_hi_marker() {
        let p = BotProfile::for_stud_hi(&PlayStyle::TightAggressive);
        assert_eq!("tight_aggressive_stud", p.name);
        assert!(matches!(
            p.betting_structure,
            Some(BettingStructure::FixedLimit { raise_cap: 3, .. })
        ));
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn stud_hi_tight_aggressive_yaml_loads() {
        let yaml = std::fs::read_to_string("data/bots/stud_hi/tight_aggressive_stud_hi.yaml")
            .expect("Stud Hi TAG profile YAML must exist");
        let p = BotProfile::from_yaml_str(&yaml).expect("Stud Hi TAG must deserialize");
        assert_eq!("tight_aggressive_stud_hi", p.name);
        assert!(matches!(
            p.betting_structure,
            Some(BettingStructure::FixedLimit { raise_cap: 3, .. })
        ));
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn stud_hi_loose_passive_yaml_loads() {
        let yaml = std::fs::read_to_string("data/bots/stud_hi/loose_passive_stud_hi.yaml")
            .expect("Stud Hi LP profile YAML must exist");
        let p = BotProfile::from_yaml_str(&yaml).expect("Stud Hi LP must deserialize");
        assert_eq!("loose_passive_stud_hi", p.name);
        assert!(matches!(
            p.betting_structure,
            Some(BettingStructure::FixedLimit { raise_cap: 3, .. })
        ));
    }

    // ---- EPIC-33 Phase 4: for_razz factory + reference profiles ----

    #[test]
    fn for_razz_marker() {
        let p = BotProfile::for_razz(&PlayStyle::TightAggressive);
        assert_eq!("tight_aggressive_razz", p.name);
        assert!(matches!(
            p.betting_structure,
            Some(BettingStructure::FixedLimit { raise_cap: 3, .. })
        ));
    }

    #[test]
    fn for_razz_falls_back_to_gto_for_custom() {
        let p = BotProfile::for_razz(&PlayStyle::Custom("unknown".into()));
        assert_eq!("gto_razz", p.name);
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn razz_tight_aggressive_yaml_loads() {
        let yaml = std::fs::read_to_string("data/bots/razz/tight_aggressive_razz.yaml")
            .expect("Razz TAG profile YAML must exist");
        let p = BotProfile::from_yaml_str(&yaml).expect("Razz TAG must deserialize");
        assert_eq!("tight_aggressive_razz", p.name);
        assert!(matches!(
            p.betting_structure,
            Some(BettingStructure::FixedLimit { raise_cap: 3, .. })
        ));
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn razz_loose_passive_yaml_loads() {
        let yaml =
            std::fs::read_to_string("data/bots/razz/loose_passive_razz.yaml").expect("Razz LP profile YAML must exist");
        let p = BotProfile::from_yaml_str(&yaml).expect("Razz LP must deserialize");
        assert_eq!("loose_passive_razz", p.name);
        assert!(matches!(
            p.betting_structure,
            Some(BettingStructure::FixedLimit { raise_cap: 3, .. })
        ));
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn flhe_tight_aggressive_yaml_loads() {
        let yaml = std::fs::read_to_string("data/bots/flhe/tight_aggressive_flhe.yaml")
            .expect("FLHE TAG profile YAML must exist");
        let p = BotProfile::from_yaml_str(&yaml).expect("FLHE TAG profile must deserialize");
        assert_eq!("tight_aggressive_flhe", p.name);
        assert!(matches!(
            p.betting_structure,
            Some(BettingStructure::FixedLimit { raise_cap: 3, .. })
        ));
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn flhe_loose_passive_yaml_loads() {
        let yaml =
            std::fs::read_to_string("data/bots/flhe/loose_passive_flhe.yaml").expect("FLHE LP profile YAML must exist");
        let p = BotProfile::from_yaml_str(&yaml).expect("FLHE LP profile must deserialize");
        assert_eq!("loose_passive_flhe", p.name);
        assert!(matches!(
            p.betting_structure,
            Some(BettingStructure::FixedLimit { raise_cap: 3, .. })
        ));
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn existing_nlhe_profile_yaml_round_trips_without_betting_structure() {
        // Backward compatibility: existing NLHE YAML files must continue
        // to deserialize cleanly with `betting_structure = None`.
        let yaml =
            std::fs::read_to_string("data/bots/tight_aggressive.yaml").expect("NLHE TAG profile YAML must exist");
        let p = BotProfile::from_yaml_str(&yaml).expect("NLHE TAG must deserialize");
        assert!(p.betting_structure.is_none());
    }

    #[test]
    fn bot_profile_tight_passive() {
        let p = BotProfile::tight_passive();
        assert_eq!(p.style, PlayStyle::new("tight_passive"));
    }

    #[test]
    fn bot_profile_loose_aggressive() {
        let p = BotProfile::loose_aggressive();
        assert_eq!(p.style, PlayStyle::new("loose_aggressive"));
    }

    #[test]
    fn bot_profile_gto() {
        let p = BotProfile::gto();
        assert_eq!(p.style, PlayStyle::new("gto"));
    }

    #[test]
    fn bot_profile_tight_aggressive() {
        let p = BotProfile::tight_aggressive();
        assert_eq!(p.style, PlayStyle::new("tight_aggressive"));
        assert_eq!(p.name, "tight_aggressive");
    }

    #[test]
    fn bot_profile_loose_passive() {
        let p = BotProfile::loose_passive();
        assert_eq!(p.style, PlayStyle::new("loose_passive"));
        assert_eq!(p.name, "loose_passive");
    }

    #[test]
    fn bot_profile_maniac() {
        let p = BotProfile::maniac();
        assert_eq!(p.style, PlayStyle::new("maniac"));
        assert_eq!(p.name, "maniac");
    }

    #[test]
    fn bot_profile_abc() {
        let p = BotProfile::abc();
        assert_eq!(p.style, PlayStyle::new("abc"));
        assert_eq!(p.betting_strategy.bluff_frequency, 0);
    }

    #[test]
    fn bot_profile_short_stack_ninja() {
        let p = BotProfile::short_stack_ninja();
        assert_eq!(p.style, PlayStyle::new("short_stack_ninja"));
        assert_eq!(p.betting_strategy.aggression_factor, 95);
    }

    #[test]
    fn bot_profile_joker() {
        let p = BotProfile::joker();
        assert_eq!(p.name, "joker");
        assert_eq!(p.style, PlayStyle::new("joker"));
        assert!(p.description.contains("unpredictable"));
    }

    #[test]
    fn bot_profile_default_profiles() {
        let profiles = BotProfile::default_profiles();
        assert_eq!(profiles.len(), 8);
        assert_eq!(profiles[0].name, "gto");
        assert_eq!(profiles[7].name, "short_stack_ninja");
        let names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"tight_aggressive"));
        assert!(names.contains(&"maniac"));
        assert!(names.contains(&"abc"));
    }

    #[test]
    fn play_style_display() {
        assert_eq!(PlayStyle::new("tight_passive").to_string(), "tight_passive");
        assert_eq!(PlayStyle::new("loose_aggressive").to_string(), "loose_aggressive");
        assert_eq!(PlayStyle::new("gto").to_string(), "gto");
        assert_eq!(PlayStyle::new("my_custom_style").to_string(), "my_custom_style");
    }

    #[test]
    fn bot_profile_display() {
        let p = BotProfile::gto();
        assert_eq!(p.to_string(), "gto (gto)");
    }

    #[test]
    fn bot_error_display() {
        let e = BotError::InvalidProfile("bad data".into());
        assert!(e.to_string().contains("bad data"));
    }

    #[test]
    fn bot_profile_serde_json_round_trip() {
        let p = BotProfile::gto();
        let json = serde_json::to_string(&p).unwrap();
        let loaded: BotProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(p, loaded);
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn bot_profile_yaml_round_trip() {
        let p = BotProfile::tight_passive();
        let yaml = p.to_yaml_string().unwrap();
        let loaded = BotProfile::from_yaml_str(&yaml).unwrap();
        assert_eq!(p, loaded);
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn bot_profile_yaml_round_trip_with_playbook() {
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
    fn bot_profile_yaml_without_playbook_unchanged() {
        // Profiles without a playbook must not gain a `playbook:` key in YAML,
        // preserving backward compatibility with existing profile files.
        let p = BotProfile::maniac();
        let yaml = p.to_yaml_string().unwrap();
        assert!(!yaml.contains("playbook"), "flat profile should not emit playbook key");
    }

    #[cfg(all(feature = "bot-profiles", not(target_arch = "wasm32")))]
    #[test]
    fn bot_profile_file_round_trip() {
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
    fn data_bots_all_load() {
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
            let loaded = BotProfile::from_file(&path).unwrap_or_else(|e| panic!("failed to load {path}: {e}"));
            assert_eq!(loaded.name, name, "{path}: name field mismatch");
        }
    }

    /// The three constructor-backed profiles must be byte-identical to their
    /// YAML files after a round-trip through deserialization.
    #[cfg(all(feature = "bot-profiles", not(target_arch = "wasm32")))]
    #[test]
    fn data_bots_constructors_match_files() {
        for (name, expected) in [
            ("gto", BotProfile::gto()),
            ("tight_passive", BotProfile::tight_passive()),
            ("loose_aggressive", BotProfile::loose_aggressive()),
        ] {
            let path = format!("data/bots/{name}.yaml");
            let from_file = BotProfile::from_file(&path).unwrap_or_else(|e| panic!("failed to load {path}: {e}"));
            assert_eq!(from_file, expected, "{path} does not match constructor output");
        }
    }
}
