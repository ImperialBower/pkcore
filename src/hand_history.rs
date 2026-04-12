//! Human-readable YAML hand history format for pkcore.
//!
//! This module provides types for serializing and deserializing poker hand
//! histories as YAML. All card fields are stored as human-readable strings
//! using pkcore's forgiving card notation (e.g., `"A♠"`, `"Kd"`, `"Ts"`)
//! and converted to native pkcore types via bridge methods.
//!
//! YAML I/O (`HandHistory::from_yaml` / `HandHistory::to_yaml`) requires
//! the **`hand-histories`** feature flag. All struct definitions and bridge
//! methods are available without any feature flag.
//!
//! # Examples
//!
//! ```
//! use pkcore::hand_history::{HandHistory, HandVariant, Outcome};
//!
//! let yaml = r#"
//! hand:
//!   id: "ex-001"
//!   game: holdem
//! table:
//!   stakes:
//!     small_blind: 1.0
//!     big_blind: 2.0
//! players:
//!   - seat: 1
//!     name: "Alice"
//!     stack: 200.0
//!   - seat: 2
//!     name: "Bob"
//!     stack: 200.0
//! streets:
//!   preflop:
//!     actions:
//!       - { seat: 1, action: fold }
//! results:
//!   - seat: 2
//!     outcome: win
//!     net: 1.0
//! "#;
//!
//! # #[cfg(feature = "hand-histories")]
//! # {
//! let hh = HandHistory::from_yaml(yaml).unwrap();
//! assert_eq!(hh.hand.id, "ex-001");
//! assert_eq!(hh.hand.game, HandVariant::Holdem);
//! # }
//! ```

use crate::PKError;
use crate::analysis::gto::combos::Combos;
use crate::analysis::hand_rank::HandRank;
use crate::arrays::five::Five;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::cards::Cards;
use crate::casino::table::event::TableAction;
use crate::play::board::Board;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// ─────────────────────────────────────────────────────────────────────────────
// Format version constant
// ─────────────────────────────────────────────────────────────────────────────

/// Current schema version for the pkcore YAML hand history format.
///
/// Increment this when making breaking changes to the format so consumers can
/// detect and reject files written for an incompatible schema.
pub const FORMAT_VERSION: u32 = 1;

// ─────────────────────────────────────────────────────────────────────────────
// Top-level hand history
// ─────────────────────────────────────────────────────────────────────────────

/// Root type for a pkcore YAML hand history.
///
/// All card fields (e.g., [`board`], hole cards, street cards) are stored as
/// strings in pkcore's card notation and converted to native types via bridge
/// methods such as [`HandHistory::to_board`].
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{HandHistory, HandVariant};
///
/// let yaml = r#"
/// hand:
///   id: "test-001"
///   game: kuhn
/// table:
///   stakes:
///     small_blind: 1.0
///     big_blind: 1.0
/// players:
///   - seat: 1
///     name: "A"
///     stack: 100.0
///   - seat: 2
///     name: "B"
///     stack: 100.0
/// "#;
///
/// # #[cfg(feature = "hand-histories")]
/// # {
/// let hh = HandHistory::from_yaml(yaml).unwrap();
/// assert_eq!(hh.hand.game, HandVariant::Kuhn);
/// # }
/// ```
///
/// [`board`]: HandHistory::board
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HandHistory {
    /// The pkcore crate version that produced this file.
    ///
    /// Present in standalone YAML files; `None` when the hand is embedded inside
    /// a [`HandCollection`] (the collection carries the single authoritative
    /// version at its root).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pkcore_version: Option<String>,

    /// Schema version for forward-compatibility checks.
    #[serde(default = "default_format_version")]
    pub format_version: u32,

    /// Hand-level metadata: id, game type, timestamp, provenance.
    pub hand: HandMeta,

    /// Table configuration: name, seat count, button, blinds/antes.
    pub table: TableInfo,

    /// Participating players with seat, stack, and optional hole cards.
    pub players: Vec<PlayerEntry>,

    /// The full community board as a card string, e.g. `"9♣ 6♦ 5♥ 5♠ 8♠"`.
    ///
    /// Partial boards are accepted (flop-only, flop+turn, etc.).
    /// Convert with [`HandHistory::to_board`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<String>,

    /// Per-street breakdown of community cards and player actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub streets: Option<Streets>,

    /// Showdown results with best hands, pkcore hand ranks, and net P&L.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<ResultEntry>>,

    /// Optional GTO/analysis metadata: ranges, equity, solver notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<AnalysisContext>,
}

fn default_pkcore_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_format_version() -> u32 {
    FORMAT_VERSION
}

impl HandHistory {
    /// Convert the [`board`] string field to a pkcore [`Board`].
    ///
    /// Returns [`PKError::NotEnoughCards`] if the board field is absent.
    ///
    /// # Errors
    ///
    /// Returns [`PKError`] if the board string cannot be parsed or the board
    /// field is `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{HandHistory, HandVariant};
    ///
    /// let yaml = r#"
    /// hand:
    ///   id: "b"
    ///   game: holdem
    /// table:
    ///   stakes:
    ///     small_blind: 1.0
    ///     big_blind: 2.0
    /// players: []
    /// board: "9♣ 6♦ 5♥ 5♠ 8♠"
    /// "#;
    ///
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// let hh = HandHistory::from_yaml(yaml).unwrap();
    /// let board = hh.to_board();
    /// assert!(board.is_ok());
    /// # }
    /// ```
    ///
    /// [`board`]: HandHistory::board
    pub fn to_board(&self) -> Result<Board, PKError> {
        match &self.board {
            Some(s) => Board::from_str(s),
            None => Err(PKError::NotEnoughCards),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// YAML I/O (feature-gated)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "hand-histories")]
impl HandHistory {
    /// Deserialize a [`HandHistory`] from a YAML string.
    ///
    /// # Errors
    ///
    /// Returns [`serde_yaml_bw::Error`] if the YAML is malformed or required
    /// fields are missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{HandHistory, HandVariant};
    ///
    /// let yaml = r#"
    /// hand:
    ///   id: "ex-002"
    ///   game: holdem
    /// table:
    ///   stakes:
    ///     small_blind: 1.0
    ///     big_blind: 2.0
    /// players: []
    /// "#;
    ///
    /// let hh = HandHistory::from_yaml(yaml).unwrap();
    /// assert_eq!(hh.hand.id, "ex-002");
    /// ```
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml_bw::Error> {
        serde_yaml_bw::from_str(yaml)
    }

    /// Serialize this [`HandHistory`] to a YAML string.
    ///
    /// # Errors
    ///
    /// Returns [`serde_yaml_bw::Error`] if serialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{HandHistory, HandVariant};
    ///
    /// let yaml = r#"
    /// hand:
    ///   id: "rt-001"
    ///   game: holdem
    /// table:
    ///   stakes:
    ///     small_blind: 1.0
    ///     big_blind: 2.0
    /// players: []
    /// "#;
    ///
    /// let hh = HandHistory::from_yaml(yaml).unwrap();
    /// let out = hh.to_yaml().unwrap();
    /// assert!(out.contains("rt-001"));
    /// ```
    pub fn to_yaml(&self) -> Result<String, serde_yaml_bw::Error> {
        serde_yaml_bw::to_string(self)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Hand collection
// ─────────────────────────────────────────────────────────────────────────────

/// A versioned collection of [`HandHistory`] records, serializable as a
/// single YAML file.
///
/// The top-level `pkcore_version` and `format_version` fields apply to the
/// whole file. [`HandCollection::push`] automatically clears the per-hand
/// `pkcore_version` so it is never repeated in the serialized output.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::HandCollection;
///
/// let collection = HandCollection::new();
/// assert!(collection.is_empty());
/// assert_eq!(collection.len(), 0);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HandCollection {
    /// The pkcore crate version that produced this file.
    #[serde(default = "default_pkcore_version")]
    pub pkcore_version: String,

    /// Schema version for forward-compatibility checks.
    #[serde(default = "default_format_version")]
    pub format_version: u32,

    /// The hands contained in this collection.
    pub hands: Vec<HandHistory>,
}

impl HandCollection {
    /// Creates a new empty [`HandCollection`] with the current crate version
    /// and format version.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::new();
    /// assert!(collection.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            pkcore_version: default_pkcore_version(),
            format_version: default_format_version(),
            hands: Vec::new(),
        }
    }

    /// Appends a [`HandHistory`] to the end of this collection.
    ///
    /// The hand's `pkcore_version` is cleared on insertion; the collection-level
    /// version is authoritative and repeating it on every entry is noise.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{
    ///     HandCollection, HandHistory, HandMeta, HandVariant, TableInfo, Stakes,
    /// };
    ///
    /// let mut collection = HandCollection::new();
    /// collection.push(HandHistory {
    ///     pkcore_version: None,
    ///     format_version: 1,
    ///     hand: HandMeta {
    ///         id: "hand-001".to_string(),
    ///         game: HandVariant::Holdem,
    ///         timestamp: None,
    ///         source: None,
    ///         description: None,
    ///     },
    ///     table: TableInfo {
    ///         name: None,
    ///         seats: None,
    ///         button: None,
    ///         stakes: Stakes { small_blind: 1.0, big_blind: 2.0, ante: None, straddle: None },
    ///     },
    ///     players: vec![],
    ///     board: None,
    ///     streets: None,
    ///     results: None,
    ///     analysis: None,
    /// });
    /// assert_eq!(collection.len(), 1);
    /// ```
    pub fn push(&mut self, mut hand: HandHistory) {
        hand.pkcore_version = None;
        self.hands.push(hand);
    }

    /// Returns the number of hands in this collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::new();
    /// assert_eq!(collection.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.hands.len()
    }

    /// Returns `true` if the collection contains no hands.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::new();
    /// assert!(collection.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hands.is_empty()
    }

    /// Returns an iterator over the [`HandHistory`] records in this collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::new();
    /// assert_eq!(collection.iter().count(), 0);
    /// ```
    pub fn iter(&self) -> std::slice::Iter<'_, HandHistory> {
        self.hands.iter()
    }

    /// Returns a slice of all [`HandHistory`] records in this collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::new();
    /// assert!(collection.hands().is_empty());
    /// ```
    #[must_use]
    pub fn hands(&self) -> &[HandHistory] {
        &self.hands
    }
}

impl Default for HandCollection {
    /// Returns an empty [`HandCollection`] via [`HandCollection::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::default();
    /// assert!(collection.is_empty());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "hand-histories")]
impl HandCollection {
    /// Deserialize a [`HandCollection`] from a YAML string.
    ///
    /// The top-level `pkcore_version` and `format_version` fields are optional;
    /// they default to the current crate version and [`FORMAT_VERSION`]
    /// respectively. Per-hand `pkcore_version` fields are accepted for
    /// compatibility but will be `None` in newly written files.
    ///
    /// # Errors
    ///
    /// Returns a [`serde_yaml_bw::Error`] if the YAML is malformed or does not
    /// match the expected schema.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let yaml = r#"
    /// hands:
    ///   - hand:
    ///       id: "hand-001"
    ///       game: holdem
    ///     table:
    ///       stakes:
    ///         small_blind: 1.0
    ///         big_blind: 2.0
    ///     players: []
    /// "#;
    ///
    /// let collection = HandCollection::from_yaml(yaml).unwrap();
    /// assert_eq!(collection.len(), 1);
    /// # }
    /// ```
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml_bw::Error> {
        serde_yaml_bw::from_str(yaml)
    }

    /// Serialize this [`HandCollection`] to a YAML string.
    ///
    /// Embedded [`HandHistory`] records will not repeat `pkcore_version`;
    /// the single authoritative version lives at the collection root.
    ///
    /// # Errors
    ///
    /// Returns a [`serde_yaml_bw::Error`] if serialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::new();
    /// let yaml = collection.to_yaml().unwrap();
    /// assert!(yaml.contains("hands:"));
    /// # }
    /// ```
    pub fn to_yaml(&self) -> Result<String, serde_yaml_bw::Error> {
        serde_yaml_bw::to_string(self)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Hand metadata
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata about a hand: unique id, game variant, and provenance.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{HandMeta, HandVariant};
///
/// let meta = HandMeta {
///     id: "hand-001".to_string(),
///     game: HandVariant::Holdem,
///     timestamp: None,
///     source: Some("Home game".to_string()),
///     description: None,
/// };
/// assert_eq!(meta.game, HandVariant::Holdem);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HandMeta {
    /// Unique identifier for this hand (user-defined or auto-generated).
    pub id: String,

    /// Poker variant played. Maps to pkcore's supported game types.
    pub game: HandVariant,

    /// ISO 8601 timestamp of when the hand was played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Where this hand came from (e.g., `"PokerStars"`, `"High Stakes Poker S5"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Free-form description or title for the hand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Supported poker game types, matching pkcore's current and planned variants.
///
/// Serializes as `snake_case` in YAML (e.g., `holdem`, `omaha_hi_lo`, `kuhn`).
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::HandVariant;
///
/// assert_eq!(HandVariant::default(), HandVariant::Holdem);
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HandVariant {
    /// Texas Hold'em.
    #[default]
    Holdem,
    /// Omaha High.
    Omaha,
    /// Omaha Hi-Lo (8 or better).
    OmahaHiLo,
    /// Kuhn poker (3-card training game).
    Kuhn,
    /// Razz (7-card stud low).
    Razz,
    /// 7-card stud.
    Stud,
    /// 7-card stud Hi-Lo.
    StudHiLo,
    /// 5-card draw high.
    DrawHigh,
    /// Catch-all for variants pkcore does not yet model.
    Other(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Table info
// ─────────────────────────────────────────────────────────────────────────────

/// Table-level configuration for a hand.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{TableInfo, Stakes};
///
/// let table = TableInfo {
///     name: Some("Main Event".to_string()),
///     seats: Some(9),
///     button: Some(1),
///     stakes: Stakes { small_blind: 100.0, big_blind: 200.0, ante: None, straddle: None },
/// };
/// assert_eq!(table.stakes.big_blind, 200.0);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TableInfo {
    /// Table name or identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Total seats at the table (2–10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seats: Option<u8>,

    /// Seat number of the dealer button (1-indexed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button: Option<u8>,

    /// Blind and ante structure.
    pub stakes: Stakes,
}

/// Blind and ante structure for a hand.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::Stakes;
///
/// let stakes = Stakes { small_blind: 5.0, big_blind: 10.0, ante: Some(2.0), straddle: None };
/// assert_eq!(stakes.small_blind, 5.0);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Stakes {
    /// Small blind amount.
    pub small_blind: f64,

    /// Big blind amount.
    pub big_blind: f64,

    /// Per-player ante amount, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ante: Option<f64>,

    /// Straddle amount, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub straddle: Option<f64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Player entry
// ─────────────────────────────────────────────────────────────────────────────

/// A single player's state at the start of the hand.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::PlayerEntry;
///
/// let player = PlayerEntry {
///     seat: 1,
///     name: "Alice".to_string(),
///     stack: 500.0,
///     hole_cards: Some("A♠ K♠".to_string()),
///     posted: None,
/// };
/// assert!(player.to_two().is_ok());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlayerEntry {
    /// Seat number (1-indexed).
    pub seat: u8,

    /// Player name or alias.
    pub name: String,

    /// Starting stack in chips/currency units.
    pub stack: f64,

    /// Hole cards as a card string (e.g., `"6♠ 6♥"` or `"6s6h"`).
    ///
    /// Maps to [`Two::from_str`] for Hold'em, [`Four::from_str`] for Omaha.
    /// `None` if the player mucked or the cards are unknown.
    ///
    /// [`Four::from_str`]: crate::arrays::four::Four
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hole_cards: Option<String>,

    /// Which blind or ante this player posted, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posted: Option<PostedBlind>,
}

impl PlayerEntry {
    /// Convert the `hole_cards` string to a pkcore [`Two`] (Hold'em hole cards).
    ///
    /// # Errors
    ///
    /// Returns [`PKError::NotEnoughCards`] if `hole_cards` is `None`, or a
    /// parse error if the string is not a valid two-card hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::PlayerEntry;
    ///
    /// let player = PlayerEntry {
    ///     seat: 1,
    ///     name: "Alice".to_string(),
    ///     stack: 200.0,
    ///     hole_cards: Some("A♠ K♠".to_string()),
    ///     posted: None,
    /// };
    /// assert!(player.to_two().is_ok());
    /// ```
    pub fn to_two(&self) -> Result<Two, PKError> {
        match &self.hole_cards {
            Some(s) => Two::from_str(s),
            None => Err(PKError::NotEnoughCards),
        }
    }
}

/// What blind or ante a player posted at the start of the hand.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::PostedBlind;
///
/// let posted = PostedBlind::BigBlind;
/// assert_eq!(posted, PostedBlind::BigBlind);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PostedBlind {
    /// Small blind.
    SmallBlind,
    /// Big blind.
    BigBlind,
    /// Ante only (no blind).
    Ante,
    /// Live straddle.
    Straddle,
    /// Posted two obligations (e.g., small blind + ante).
    Both,
}

// ─────────────────────────────────────────────────────────────────────────────
// Streets
// ─────────────────────────────────────────────────────────────────────────────

/// Container for all street-level community cards and actions.
///
/// Every street is optional; omit streets that did not occur (e.g., everyone
/// folds preflop, so no flop/turn/river).
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{Streets, PreflopStreet, Action, ActionType};
///
/// let streets = Streets {
///     preflop: Some(PreflopStreet {
///         actions: vec![Action { seat: 1, action: ActionType::Fold, amount: None, all_in: None }],
///         pot: Some(3.0),
///     }),
///     flop: None,
///     turn: None,
///     river: None,
/// };
/// assert!(streets.flop.is_none());
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Streets {
    /// Preflop betting round.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preflop: Option<PreflopStreet>,

    /// Flop: three community cards and actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flop: Option<FlopStreet>,

    /// Turn: one community card and actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn: Option<TurnStreet>,

    /// River: one community card and actions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub river: Option<RiverStreet>,
}

/// Street bucket used while parsing an event log in [`Streets::from_event_log`].
#[derive(PartialEq)]
enum EventStreet {
    Preflop,
    Flop,
    Turn,
    River,
}

impl Streets {
    /// Build a [`Streets`] record by parsing a `TableNoCell` event log.
    ///
    /// Walks `log` in a single forward pass, partitioning player-action events
    /// into preflop / flop / turn / river buckets. [`TableAction::DealtFlop`],
    /// [`TableAction::DealtTurn`], and [`TableAction::DealtRiver`] events act as
    /// bucket boundaries and supply the community-card strings for post-flop
    /// streets. The last [`TableAction::PotSize`] event within each bucket
    /// becomes that street's `pot`.
    ///
    /// All amounts are stored as [`f64`] (the `HandHistory` convention) after
    /// casting from the `usize` chip counts used internally by `TableNoCell`.
    ///
    /// Returns `None` only if `log` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::Streets;
    /// use pkcore::casino::table::event::TableAction;
    ///
    /// let log = vec![
    ///     TableAction::ForcedBetSmallBlind(1, 50),
    ///     TableAction::ForcedBetBigBlind(2, 100),
    ///     TableAction::Fold(1),
    ///     TableAction::Check(2),
    ///     TableAction::PotSize(150),
    /// ];
    /// let streets = Streets::from_event_log(&log).unwrap();
    /// assert_eq!(streets.preflop.as_ref().unwrap().actions.len(), 4);
    /// assert_eq!(streets.preflop.as_ref().unwrap().pot, Some(150.0));
    /// assert!(streets.flop.is_none());
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn from_event_log(log: &[TableAction]) -> Option<Self> {
        if log.is_empty() {
            return None;
        }

        let mut current = EventStreet::Preflop;

        let mut preflop_actions: Vec<Action> = Vec::new();
        let mut flop_actions: Vec<Action> = Vec::new();
        let mut turn_actions: Vec<Action> = Vec::new();
        let mut river_actions: Vec<Action> = Vec::new();

        let mut preflop_pot: Option<f64> = None;
        let mut flop_pot: Option<f64> = None;
        let mut turn_pot: Option<f64> = None;
        let mut river_pot: Option<f64> = None;

        let mut flop_cards: Option<String> = None;
        let mut turn_card: Option<String> = None;
        let mut river_card: Option<String> = None;

        for event in log {
            match event {
                TableAction::DealtFlop(bard) => {
                    flop_cards = Some(Cards::from(*bard).to_string());
                    current = EventStreet::Flop;
                }
                TableAction::DealtTurn(bard) => {
                    turn_card = Some(Cards::from(*bard).to_string());
                    current = EventStreet::Turn;
                }
                TableAction::DealtRiver(bard) => {
                    river_card = Some(Cards::from(*bard).to_string());
                    current = EventStreet::River;
                }
                TableAction::PotSize(amount) => {
                    let pot = Some(*amount as f64);
                    match current {
                        EventStreet::Preflop => preflop_pot = pot,
                        EventStreet::Flop => flop_pot = pot,
                        EventStreet::Turn => turn_pot = pot,
                        EventStreet::River => river_pot = pot,
                    }
                }
                other => {
                    if let Some(action) = table_action_to_hand_action(other) {
                        match current {
                            EventStreet::Preflop => preflop_actions.push(action),
                            EventStreet::Flop => flop_actions.push(action),
                            EventStreet::Turn => turn_actions.push(action),
                            EventStreet::River => river_actions.push(action),
                        }
                    }
                }
            }
        }

        Some(Streets {
            preflop: if preflop_actions.is_empty() && preflop_pot.is_none() {
                None
            } else {
                Some(PreflopStreet {
                    actions: preflop_actions,
                    pot: preflop_pot,
                })
            },
            flop: flop_cards.map(|cards| FlopStreet {
                cards,
                actions: flop_actions,
                pot: flop_pot,
            }),
            turn: turn_card.map(|card| TurnStreet {
                card,
                actions: turn_actions,
                pot: turn_pot,
            }),
            river: river_card.map(|card| RiverStreet {
                card,
                actions: river_actions,
                pot: river_pot,
            }),
        })
    }
}

/// Maps a single [`TableAction`] to a [`Action`] for the hand history.
///
/// Returns `None` for non-player-action events (deals, pot bookkeeping, etc.).
#[allow(clippy::cast_precision_loss)]
fn table_action_to_hand_action(event: &TableAction) -> Option<Action> {
    match event {
        TableAction::ForcedBetSmallBlind(seat, amount)
        | TableAction::ForcedBetBigBlind(seat, amount)
        | TableAction::BetAnteForced(seat, amount)
        | TableAction::ForcedBet(seat, amount) => Some(Action {
            seat: *seat,
            action: ActionType::Post,
            amount: Some(*amount as f64),
            all_in: None,
        }),
        TableAction::Check(seat) => Some(Action {
            seat: *seat,
            action: ActionType::Check,
            amount: None,
            all_in: None,
        }),
        TableAction::Bet(seat, amount) => Some(Action {
            seat: *seat,
            action: ActionType::Bet,
            amount: Some(*amount as f64),
            all_in: None,
        }),
        TableAction::Call(seat, amount) => Some(Action {
            seat: *seat,
            action: ActionType::Call,
            amount: Some(*amount as f64),
            all_in: None,
        }),
        TableAction::Raise(seat, amount) => Some(Action {
            seat: *seat,
            action: ActionType::Raise,
            amount: Some(*amount as f64),
            all_in: None,
        }),
        TableAction::AllIn(seat, amount) => Some(Action {
            seat: *seat,
            action: ActionType::AllIn,
            amount: Some(*amount as f64),
            all_in: Some(true),
        }),
        TableAction::Fold(seat) => Some(Action {
            seat: *seat,
            action: ActionType::Fold,
            amount: None,
            all_in: None,
        }),
        _ => None,
    }
}

/// Preflop betting round (no community cards).
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{PreflopStreet, Action, ActionType};
///
/// let preflop = PreflopStreet {
///     actions: vec![Action { seat: 1, action: ActionType::Check, amount: None, all_in: None }],
///     pot: None,
/// };
/// assert_eq!(preflop.actions.len(), 1);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PreflopStreet {
    /// Ordered list of player actions this street.
    pub actions: Vec<Action>,

    /// Pot size at the end of this street.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pot: Option<f64>,
}

/// Flop: three community cards plus actions.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{FlopStreet, Action, ActionType};
///
/// let flop = FlopStreet {
///     cards: "9♣ 6♦ 5♥".to_string(),
///     actions: vec![Action { seat: 1, action: ActionType::Check, amount: None, all_in: None }],
///     pot: Some(60200.0),
/// };
/// assert!(flop.to_three().is_ok());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FlopStreet {
    /// The three flop cards (e.g., `"9♣ 6♦ 5♥"`). Maps to [`Three::from_str`].
    pub cards: String,

    /// Ordered list of player actions this street.
    pub actions: Vec<Action>,

    /// Pot size at the end of this street.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pot: Option<f64>,
}

impl FlopStreet {
    /// Convert the `cards` string to a pkcore [`Three`] (flop cards).
    ///
    /// # Errors
    ///
    /// Returns [`PKError`] if the string cannot be parsed as three cards.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{FlopStreet, Action, ActionType};
    ///
    /// let flop = FlopStreet {
    ///     cards: "9♣ 6♦ 5♥".to_string(),
    ///     actions: vec![],
    ///     pot: None,
    /// };
    /// assert!(flop.to_three().is_ok());
    /// ```
    pub fn to_three(&self) -> Result<Three, PKError> {
        Three::from_str(&self.cards)
    }
}

/// Turn: one community card plus actions.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{TurnStreet, Action, ActionType};
///
/// let turn = TurnStreet {
///     card: "5♠".to_string(),
///     actions: vec![],
///     pot: None,
/// };
/// assert!(turn.to_card().is_ok());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TurnStreet {
    /// The turn card (e.g., `"5♠"`). Maps to [`Card::from_str`].
    pub card: String,

    /// Ordered list of player actions this street.
    pub actions: Vec<Action>,

    /// Pot size at the end of this street.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pot: Option<f64>,
}

impl TurnStreet {
    /// Convert the `card` string to a pkcore [`Card`] (turn card).
    ///
    /// # Errors
    ///
    /// Returns [`PKError`] if the string cannot be parsed as a single card.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{TurnStreet, Action, ActionType};
    ///
    /// let turn = TurnStreet { card: "5♠".to_string(), actions: vec![], pot: None };
    /// assert!(turn.to_card().is_ok());
    /// ```
    pub fn to_card(&self) -> Result<Card, PKError> {
        Card::from_str(&self.card)
    }
}

/// River: one community card plus actions.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{RiverStreet, Action, ActionType};
///
/// let river = RiverStreet {
///     card: "8♠".to_string(),
///     actions: vec![],
///     pot: None,
/// };
/// assert!(river.to_card().is_ok());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RiverStreet {
    /// The river card (e.g., `"8♠"`). Maps to [`Card::from_str`].
    pub card: String,

    /// Ordered list of player actions this street.
    pub actions: Vec<Action>,

    /// Pot size at the end of this street.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pot: Option<f64>,
}

impl RiverStreet {
    /// Convert the `card` string to a pkcore [`Card`] (river card).
    ///
    /// # Errors
    ///
    /// Returns [`PKError`] if the string cannot be parsed as a single card.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{RiverStreet, Action, ActionType};
    ///
    /// let river = RiverStreet { card: "8♠".to_string(), actions: vec![], pot: None };
    /// assert!(river.to_card().is_ok());
    /// ```
    pub fn to_card(&self) -> Result<Card, PKError> {
        Card::from_str(&self.card)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Actions
// ─────────────────────────────────────────────────────────────────────────────

/// A single player action within a betting round.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{Action, ActionType};
///
/// let action = Action { seat: 3, action: ActionType::Raise, amount: Some(100.0), all_in: None };
/// assert_eq!(action.seat, 3);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Action {
    /// Seat number of the acting player (1-indexed).
    pub seat: u8,

    /// The action taken.
    pub action: ActionType,

    /// Amount wagered (for `bet`, `raise`, `call`). Omit for `check`/`fold`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,

    /// Whether the player is all-in after this action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_in: Option<bool>,
}

/// The set of possible player actions in a betting round.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::ActionType;
///
/// let action = ActionType::Bet;
/// assert_eq!(action, ActionType::Bet);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Discard hand and exit the pot.
    Fold,
    /// Pass action without betting (when no bet is facing).
    Check,
    /// Match the current bet.
    Call,
    /// Open the betting.
    Bet,
    /// Increase the current bet.
    Raise,
    /// Forced bet posted by the structure (blind or ante).
    Post,
    /// Go all-in (used when the action itself is the all-in).
    AllIn,
}

// ─────────────────────────────────────────────────────────────────────────────
// Results
// ─────────────────────────────────────────────────────────────────────────────

/// Showdown result for a single player.
///
/// The `hand_rank` field reuses pkcore's [`HandRank`] type directly, so
/// `name` is a [`HandRankName`] (broad category: `FullHouse`) and `class` is
/// a [`HandRankClass`] (specific hand: `SixesOverFives`).
///
/// [`HandRankName`]: crate::analysis::name::HandRankName
/// [`HandRankClass`]: crate::analysis::class::HandRankClass
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{ResultEntry, Outcome};
///
/// let result = ResultEntry {
///     seat: 2,
///     best_hand: None,
///     hand_rank: None,
///     outcome: Outcome::Win,
///     net: Some(150.0),
///     pot_won: None,
///     mucked: None,
/// };
/// assert_eq!(result.outcome, Outcome::Win);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ResultEntry {
    /// Seat number of this player (1-indexed).
    pub seat: u8,

    /// Best 5-card hand string (e.g., `"6♠ 6♥ 6♦ 5♥ 5♠"`).
    ///
    /// Maps to [`Five::from_str`]. `None` if the player mucked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_hand: Option<String>,

    /// pkcore [`HandRank`] — value, name (broad), and class (specific).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand_rank: Option<HandRank>,

    /// Win/lose/tie/fold outcome.
    pub outcome: Outcome,

    /// Net chips won or lost (positive = profit, negative = loss).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<f64>,

    /// Total chips won from the pot (before rake).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pot_won: Option<f64>,

    /// Whether the player mucked their hand at showdown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mucked: Option<bool>,
}

impl ResultEntry {
    /// Convert the `best_hand` string to a pkcore [`Five`].
    ///
    /// # Errors
    ///
    /// Returns [`PKError::NotEnoughCards`] if `best_hand` is `None`, or a
    /// parse error if the string is not a valid five-card hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{ResultEntry, Outcome};
    ///
    /// let result = ResultEntry {
    ///     seat: 1,
    ///     best_hand: Some("6♠ 6♥ 6♦ 5♥ 5♠".to_string()),
    ///     hand_rank: None,
    ///     outcome: Outcome::Win,
    ///     net: None,
    ///     pot_won: None,
    ///     mucked: None,
    /// };
    /// assert!(result.to_five().is_ok());
    /// ```
    pub fn to_five(&self) -> Result<Five, PKError> {
        match &self.best_hand {
            Some(s) => Five::from_str(s),
            None => Err(PKError::NotEnoughCards),
        }
    }
}

/// Outcome for a player at showdown or hand end.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::Outcome;
///
/// assert_ne!(Outcome::Win, Outcome::Lose);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Player won the pot.
    Win,
    /// Player lost the pot.
    Lose,
    /// Split pot.
    Tie,
    /// Player folded before showdown.
    Fold,
}

// ─────────────────────────────────────────────────────────────────────────────
// Analysis / GTO context
// ─────────────────────────────────────────────────────────────────────────────

/// Optional GTO and solver context for a hand.
///
/// The `villain_range` and `hero_range` fields use pkcore's
/// [`Combos::from_str`] notation (e.g., `"66+,AJs+,KQs,AJo+,KQo"`).
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::AnalysisContext;
///
/// let ctx = AnalysisContext {
///     hero_range: None,
///     villain_range: Some("QQ+,AKs".to_string()),
///     hero_equity_preflop: Some(0.48),
///     hero_equity_flop: None,
///     hero_equity_turn: None,
///     hero_equity_river: None,
///     notes: None,
/// };
/// assert!(ctx.to_villain_combos().is_ok());
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AnalysisContext {
    /// Hero's range or specific hand for analysis, in `Combos` notation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero_range: Option<String>,

    /// Villain's range in pkcore combo notation (e.g., `"66+,AJs+,KQs"`).
    /// Maps to [`Combos::from_str`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub villain_range: Option<String>,

    /// Hero's equity at preflop (0.0–1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero_equity_preflop: Option<f64>,

    /// Hero's equity at the flop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero_equity_flop: Option<f64>,

    /// Hero's equity at the turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero_equity_turn: Option<f64>,

    /// Hero's equity at the river.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero_equity_river: Option<f64>,

    /// Free-form notes for analysis context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl AnalysisContext {
    /// Convert the `villain_range` string to pkcore [`Combos`].
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidRangeIndex`] if `villain_range` is `None`,
    /// or a parse error if the range notation is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::AnalysisContext;
    ///
    /// let ctx = AnalysisContext {
    ///     villain_range: Some("QQ+,AKs".to_string()),
    ///     ..Default::default()
    /// };
    /// assert!(ctx.to_villain_combos().is_ok());
    /// ```
    pub fn to_villain_combos(&self) -> Result<Combos, PKError> {
        match &self.villain_range {
            Some(s) => Combos::from_str(s),
            None => Err(PKError::InvalidRangeIndex),
        }
    }

    /// Convert the `hero_range` string to pkcore [`Combos`].
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidRangeIndex`] if `hero_range` is `None`,
    /// or a parse error if the range notation is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::AnalysisContext;
    ///
    /// let ctx = AnalysisContext {
    ///     hero_range: Some("TT+,AQs+".to_string()),
    ///     ..Default::default()
    /// };
    /// assert!(ctx.to_hero_combos().is_ok());
    /// ```
    pub fn to_hero_combos(&self) -> Result<Combos, PKError> {
        match &self.hero_range {
            Some(s) => Combos::from_str(s),
            None => Err(PKError::InvalidRangeIndex),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Embedded YAML for "The Hand" so tests run without filesystem access.
    #[cfg(feature = "hand-histories")]
    const THE_HAND_YAML: &str = r#"
pkcore_version: "0.0.39"
format_version: 1

hand:
  id: "hsp-s5-the-hand"
  game: holdem
  timestamp: "2008-03-01T21:00:00Z"
  source: "High Stakes Poker S5"
  description: "The Hand - Negreanu vs Hansen, Quads vs Full House"

table:
  name: "HSP Table 1"
  seats: 6
  button: 1
  stakes:
    small_blind: 400.0
    big_blind: 800.0
    ante: 200.0

players:
  - seat: 1
    name: "Gus Hansen"
    stack: 468200.0
    hole_cards: "6s 6h"
  - seat: 4
    name: "Daniel Negreanu"
    stack: 331400.0
    hole_cards: "5d 5c"

board: "9c 6d 5h 5s 8s"

streets:
  preflop:
    actions:
      - { seat: 2, action: fold }
      - { seat: 3, action: fold }
      - { seat: 4, action: raise, amount: 3500.0 }
      - { seat: 5, action: fold }
      - { seat: 6, action: fold }
      - { seat: 1, action: call, amount: 3500.0 }
    pot: 8200.0
  flop:
    cards: "9c 6d 5h"
    actions:
      - { seat: 1, action: check }
      - { seat: 4, action: bet, amount: 8000.0 }
      - { seat: 1, action: raise, amount: 26000.0 }
      - { seat: 4, action: call, amount: 26000.0 }
    pot: 60200.0
  turn:
    card: "5s"
    actions:
      - { seat: 1, action: check }
      - { seat: 4, action: bet, amount: 52000.0 }
      - { seat: 1, action: raise, amount: 100000.0 }
      - { seat: 4, action: call, amount: 100000.0 }
    pot: 260200.0
  river:
    card: "8s"
    actions:
      - { seat: 1, action: bet, amount: 205000.0 }
      - { seat: 4, action: call, amount: 205000.0 }
    pot: 670200.0

results:
  - seat: 1
    best_hand: "6s 6h 6d 5h 5s"
    hand_rank:
      value: 271
      name: FullHouse
      class: SixesOverFives
    outcome: win
    net: 335100.0
  - seat: 4
    best_hand: "5h 5s 5d 5c 9c"
    hand_rank:
      value: 117
      name: FourOfAKind
      class: FourFives
    outcome: lose
    net: -335100.0

analysis:
  villain_range: "66+,AJs+,KQs,AJo+,KQo"
  hero_equity_preflop: 0.5083
  notes: "Hansen full house lost to Negreanu quads"
"#;

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_history_deserialize_the_hand() {
        let hh: HandHistory = HandHistory::from_yaml(THE_HAND_YAML).expect("Failed to parse The Hand YAML");

        assert_eq!(hh.hand.id, "hsp-s5-the-hand");
        assert_eq!(hh.hand.game, HandVariant::Holdem);
        assert_eq!(hh.format_version, FORMAT_VERSION);
        assert_eq!(hh.players.len(), 2);
        assert_eq!(hh.players[0].name, "Gus Hansen");
        assert_eq!(hh.players[0].hole_cards.as_deref(), Some("6s 6h"));
        assert_eq!(hh.players[1].hole_cards.as_deref(), Some("5d 5c"));
        assert_eq!(hh.board.as_deref(), Some("9c 6d 5h 5s 8s"));

        let streets = hh.streets.as_ref().unwrap();
        assert_eq!(streets.flop.as_ref().unwrap().cards, "9c 6d 5h");
        assert_eq!(streets.turn.as_ref().unwrap().card, "5s");
        assert_eq!(streets.river.as_ref().unwrap().card, "8s");

        let results = hh.results.as_ref().unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].outcome, Outcome::Win);
        assert_eq!(results[0].hand_rank.unwrap().value, 271);
        assert_eq!(results[1].hand_rank.unwrap().value, 117);
        assert_eq!(results[1].outcome, Outcome::Lose);

        let analysis = hh.analysis.as_ref().unwrap();
        assert_eq!(analysis.villain_range.as_deref(), Some("66+,AJs+,KQs,AJo+,KQo"));
        assert_eq!(analysis.hero_equity_preflop, Some(0.5083));
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_history_round_trip() {
        let hh: HandHistory = HandHistory::from_yaml(THE_HAND_YAML).expect("Failed to parse YAML");
        let yaml_out = hh.to_yaml().expect("Failed to serialize YAML");
        let hh2: HandHistory = HandHistory::from_yaml(&yaml_out).expect("Failed to re-parse YAML");
        assert_eq!(hh, hh2);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_history_minimal() {
        let yaml = r#"
hand:
  id: "quick-fold"
  game: holdem
table:
  stakes:
    small_blind: 1.0
    big_blind: 2.0
players:
  - seat: 1
    name: "Alice"
    stack: 200.0
  - seat: 2
    name: "Bob"
    stack: 200.0
streets:
  preflop:
    actions:
      - { seat: 1, action: fold }
results:
  - seat: 2
    outcome: win
    net: 1.0
"#;
        let hh: HandHistory = HandHistory::from_yaml(yaml).expect("Failed to parse minimal YAML");

        assert_eq!(hh.hand.id, "quick-fold");
        assert!(hh.board.is_none());
        assert!(hh.players[0].hole_cards.is_none());
        assert_eq!(hh.format_version, FORMAT_VERSION);
        assert_eq!(hh.results.as_ref().unwrap()[0].outcome, Outcome::Win);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_history_kuhn() {
        let yaml = r#"
hand:
  id: "kuhn-001"
  game: kuhn
  description: "Kuhn poker CFR training hand"
table:
  stakes:
    small_blind: 1.0
    big_blind: 1.0
players:
  - seat: 1
    name: "Agent-A"
    stack: 100.0
    hole_cards: "Ks"
  - seat: 2
    name: "Agent-B"
    stack: 100.0
    hole_cards: "Qs"
streets:
  preflop:
    actions:
      - { seat: 1, action: bet, amount: 1.0 }
      - { seat: 2, action: fold }
results:
  - seat: 1
    outcome: win
    net: 1.0
  - seat: 2
    outcome: fold
    net: -1.0
"#;
        let hh: HandHistory = HandHistory::from_yaml(yaml).expect("Failed to parse Kuhn YAML");

        assert_eq!(hh.hand.game, HandVariant::Kuhn);
        assert_eq!(hh.players[0].hole_cards.as_deref(), Some("Ks"));
        assert_eq!(hh.players[1].hole_cards.as_deref(), Some("Qs"));
        assert_eq!(hh.results.as_ref().unwrap()[1].outcome, Outcome::Fold);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_history_bridge_methods() {
        let hh: HandHistory = HandHistory::from_yaml(THE_HAND_YAML).expect("Failed to parse YAML");

        assert!(hh.to_board().is_ok());

        assert!(hh.players[0].to_two().is_ok());
        assert!(hh.players[1].to_two().is_ok());

        let streets = hh.streets.as_ref().unwrap();
        assert!(streets.flop.as_ref().unwrap().to_three().is_ok());
        assert!(streets.turn.as_ref().unwrap().to_card().is_ok());
        assert!(streets.river.as_ref().unwrap().to_card().is_ok());

        let results = hh.results.as_ref().unwrap();
        assert!(results[0].to_five().is_ok());
        assert!(results[1].to_five().is_ok());

        assert!(hh.analysis.as_ref().unwrap().to_villain_combos().is_ok());
    }

    #[test]
    fn test_hand_history_bridge_no_board() {
        let hh = HandHistory {
            pkcore_version: Some("0.0.39".to_string()),
            format_version: FORMAT_VERSION,
            hand: HandMeta {
                id: "no-board".to_string(),
                game: HandVariant::Holdem,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: None,
                button: None,
                stakes: Stakes {
                    small_blind: 1.0,
                    big_blind: 2.0,
                    ante: None,
                    straddle: None,
                },
            },
            players: vec![],
            board: None,
            streets: None,
            results: None,
            analysis: None,
        };
        assert_eq!(hh.to_board(), Err(PKError::NotEnoughCards));
    }

    #[test]
    fn test_player_entry_no_hole_cards() {
        let player = PlayerEntry {
            seat: 1,
            name: "Alice".to_string(),
            stack: 200.0,
            hole_cards: None,
            posted: None,
        };
        assert_eq!(player.to_two(), Err(PKError::NotEnoughCards));
    }

    #[test]
    fn test_result_entry_no_best_hand() {
        let result = ResultEntry {
            seat: 1,
            best_hand: None,
            hand_rank: None,
            outcome: Outcome::Fold,
            net: Some(-100.0),
            pot_won: None,
            mucked: None,
        };
        assert_eq!(result.to_five(), Err(PKError::NotEnoughCards));
    }

    #[test]
    fn test_analysis_context_no_villain_range() {
        let ctx = AnalysisContext::default();
        assert_eq!(ctx.to_villain_combos(), Err(PKError::InvalidRangeIndex));
        assert_eq!(ctx.to_hero_combos(), Err(PKError::InvalidRangeIndex));
    }

    #[test]
    fn test_game_type_default() {
        assert_eq!(HandVariant::default(), HandVariant::Holdem);
    }

    #[test]
    fn test_format_version_constant() {
        assert_eq!(FORMAT_VERSION, 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HandCollection helpers and fixtures
    // ─────────────────────────────────────────────────────────────────────────

    fn make_minimal_hand(id: &str, game: HandVariant) -> HandHistory {
        HandHistory {
            pkcore_version: None,
            format_version: 1,
            hand: HandMeta {
                id: id.to_string(),
                game,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: None,
                button: None,
                stakes: Stakes {
                    small_blind: 1.0,
                    big_blind: 2.0,
                    ante: None,
                    straddle: None,
                },
            },
            players: vec![],
            board: None,
            streets: None,
            results: None,
            analysis: None,
        }
    }

    #[cfg(feature = "hand-histories")]
    const TWO_HAND_COLLECTION_YAML: &str = r#"
pkcore_version: "0.0.39"
format_version: 1
hands:
  - hand:
      id: "hand-001"
      game: holdem
    table:
      stakes:
        small_blind: 1.0
        big_blind: 2.0
    players:
      - seat: 1
        name: "Alice"
        stack: 200.0
  - hand:
      id: "hand-002"
      game: kuhn
    table:
      stakes:
        small_blind: 1.0
        big_blind: 1.0
    players:
      - seat: 1
        name: "Agent-A"
        stack: 100.0
"#;

    // ─────────────────────────────────────────────────────────────────────────
    // HandCollection tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_hand_collection_new() {
        let collection = HandCollection::new();
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
        assert!(!collection.pkcore_version.is_empty());
        assert_eq!(collection.format_version, FORMAT_VERSION);
    }

    #[test]
    fn test_hand_collection_default() {
        let a = HandCollection::new();
        let b = HandCollection::default();
        assert_eq!(a, b);
    }

    #[test]
    fn test_hand_collection_push_and_len() {
        let mut collection = HandCollection::new();
        assert!(collection.is_empty());
        collection.push(make_minimal_hand("h1", HandVariant::Holdem));
        assert_eq!(collection.len(), 1);
        assert!(!collection.is_empty());
        collection.push(make_minimal_hand("h2", HandVariant::Kuhn));
        assert_eq!(collection.len(), 2);
    }

    #[test]
    fn test_hand_collection_iter() {
        let mut collection = HandCollection::new();
        collection.push(make_minimal_hand("h1", HandVariant::Holdem));
        collection.push(make_minimal_hand("h2", HandVariant::Kuhn));
        let ids: Vec<&str> = collection.iter().map(|h| h.hand.id.as_str()).collect();
        assert_eq!(ids, vec!["h1", "h2"]);
    }

    #[test]
    fn test_hand_collection_hands_slice() {
        let mut collection = HandCollection::new();
        collection.push(make_minimal_hand("h1", HandVariant::Holdem));
        let slice = collection.hands();
        assert_eq!(slice.len(), 1);
        assert_eq!(slice[0].hand.id, "h1");
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_collection_deserialize() {
        let collection =
            HandCollection::from_yaml(TWO_HAND_COLLECTION_YAML).expect("Failed to parse two-hand collection");
        assert_eq!(collection.len(), 2);
        assert_eq!(collection.hands[0].hand.id, "hand-001");
        assert_eq!(collection.hands[0].hand.game, HandVariant::Holdem);
        assert_eq!(collection.hands[1].hand.id, "hand-002");
        assert_eq!(collection.hands[1].hand.game, HandVariant::Kuhn);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_collection_round_trip() {
        let original = HandCollection::from_yaml(TWO_HAND_COLLECTION_YAML).expect("Failed to parse YAML");
        let yaml_out = original.to_yaml().expect("Failed to serialize");
        let restored = HandCollection::from_yaml(&yaml_out).expect("Failed to re-parse");
        assert_eq!(original, restored);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_collection_empty_yaml() {
        let yaml = "hands: []\n";
        let collection = HandCollection::from_yaml(yaml).expect("Failed to parse empty collection");
        assert!(collection.is_empty());
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_collection_no_version_fields() {
        let yaml = r#"
hands:
  - hand:
      id: "h1"
      game: holdem
    table:
      stakes:
        small_blind: 1.0
        big_blind: 2.0
    players: []
"#;
        let collection = HandCollection::from_yaml(yaml).expect("Failed to parse");
        assert_eq!(collection.format_version, FORMAT_VERSION);
        assert!(!collection.pkcore_version.is_empty());
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_collection_single_hand() {
        let yaml = r#"
hands:
  - hand:
      id: "solo-001"
      game: holdem
    table:
      stakes:
        small_blind: 5.0
        big_blind: 10.0
    players:
      - seat: 1
        name: "Player"
        stack: 1000.0
"#;
        let collection = HandCollection::from_yaml(yaml).expect("Failed to parse single hand");
        assert_eq!(collection.len(), 1);
        assert_eq!(collection.hands[0].hand.id, "solo-001");
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_collection_to_yaml_contains_hands_key() {
        let mut collection = HandCollection::new();
        collection.push(make_minimal_hand("h1", HandVariant::Holdem));
        let yaml = collection.to_yaml().expect("Failed to serialize");
        assert!(yaml.contains("hands:"));
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_history_standalone_unaffected() {
        let hh: HandHistory =
            HandHistory::from_yaml(THE_HAND_YAML).expect("Existing single-hand YAML should still parse correctly");
        assert_eq!(hh.hand.id, "hsp-s5-the-hand");
        assert_eq!(hh.hand.game, HandVariant::Holdem);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Streets::from_event_log
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_streets_from_event_log_empty() {
        assert!(Streets::from_event_log(&[]).is_none());
    }

    #[test]
    fn test_streets_from_event_log_preflop_only() {
        let log = vec![
            TableAction::ForcedBetSmallBlind(1, 50),
            TableAction::ForcedBetBigBlind(2, 100),
            TableAction::Call(3, 100),
            TableAction::Fold(1),
            TableAction::Check(2),
            TableAction::PotSize(300),
        ];
        let streets = Streets::from_event_log(&log).unwrap();
        let preflop = streets.preflop.as_ref().unwrap();
        assert_eq!(preflop.actions.len(), 5);
        assert_eq!(preflop.actions[0].action, ActionType::Post);
        assert_eq!(preflop.actions[0].amount, Some(50.0));
        assert_eq!(preflop.actions[2].action, ActionType::Call);
        assert_eq!(preflop.actions[3].action, ActionType::Fold);
        assert_eq!(preflop.pot, Some(300.0));
        assert!(streets.flop.is_none());
        assert!(streets.turn.is_none());
        assert!(streets.river.is_none());
    }

    #[test]
    fn test_streets_from_event_log_pot_sizes() {
        use crate::bard::Bard;
        let log = vec![
            TableAction::ForcedBetSmallBlind(1, 50),
            TableAction::ForcedBetBigBlind(2, 100),
            TableAction::Call(3, 100),
            TableAction::PotSize(300),
            TableAction::DealtFlop(Bard::ACE_SPADES | Bard::KING_HEARTS | Bard::QUEEN_CLUBS),
            TableAction::Check(2),
            TableAction::Bet(3, 200),
            TableAction::Call(2, 200),
            TableAction::PotSize(700),
        ];
        let streets = Streets::from_event_log(&log).unwrap();
        assert_eq!(streets.preflop.as_ref().unwrap().pot, Some(300.0));
        assert_eq!(streets.flop.as_ref().unwrap().pot, Some(700.0));
    }

    #[test]
    fn test_streets_from_event_log_full_hand() {
        use crate::bard::Bard;
        let log = vec![
            TableAction::ForcedBetSmallBlind(1, 50),
            TableAction::ForcedBetBigBlind(2, 100),
            TableAction::Raise(3, 300),
            TableAction::Call(1, 300),
            TableAction::Call(2, 200),
            TableAction::PotSize(900),
            TableAction::DealtFlop(Bard::ACE_SPADES | Bard::KING_HEARTS | Bard::QUEEN_CLUBS),
            TableAction::Check(1),
            TableAction::Bet(2, 400),
            TableAction::Call(3, 400),
            TableAction::Fold(1),
            TableAction::PotSize(1700),
            TableAction::DealtTurn(Bard::JACK_SPADES),
            TableAction::Check(2),
            TableAction::Check(3),
            TableAction::PotSize(1700),
            TableAction::DealtRiver(Bard::TEN_HEARTS),
            TableAction::Bet(2, 800),
            TableAction::Fold(3),
            TableAction::PotSize(2500),
        ];
        let streets = Streets::from_event_log(&log).unwrap();

        let preflop = streets.preflop.as_ref().unwrap();
        assert_eq!(preflop.actions.len(), 5);
        assert_eq!(preflop.pot, Some(900.0));

        let flop = streets.flop.as_ref().unwrap();
        assert_eq!(flop.actions.len(), 4);
        assert_eq!(flop.actions[1].action, ActionType::Bet);
        assert_eq!(flop.actions[1].amount, Some(400.0));
        assert_eq!(flop.pot, Some(1700.0));

        let turn = streets.turn.as_ref().unwrap();
        assert_eq!(turn.actions.len(), 2);
        assert_eq!(turn.actions[0].action, ActionType::Check);
        assert_eq!(turn.pot, Some(1700.0));

        let river = streets.river.as_ref().unwrap();
        assert_eq!(river.actions.len(), 2);
        assert_eq!(river.actions[0].action, ActionType::Bet);
        assert_eq!(river.pot, Some(2500.0));
    }

    #[test]
    fn test_streets_from_event_log_all_in() {
        let log = vec![
            TableAction::ForcedBetSmallBlind(1, 50),
            TableAction::ForcedBetBigBlind(2, 100),
            TableAction::AllIn(3, 5000),
            TableAction::Fold(1),
            TableAction::Fold(2),
            TableAction::PotSize(5150),
        ];
        let streets = Streets::from_event_log(&log).unwrap();
        let preflop = streets.preflop.as_ref().unwrap();
        let all_in_action = preflop.actions.iter().find(|a| a.action == ActionType::AllIn).unwrap();
        assert_eq!(all_in_action.amount, Some(5000.0));
        assert_eq!(all_in_action.all_in, Some(true));
    }
}
