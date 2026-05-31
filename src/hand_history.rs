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
use crate::analysis::eval::Eval;
use crate::analysis::gto::combos::Combos;
use crate::analysis::hand_rank::HandRank;
use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::cards::Cards;
#[cfg(feature = "bot-profiles")]
use crate::casino::action::PlayerAction;
use crate::casino::game::ForcedBets;
use crate::casino::table::event::TableAction;
use crate::casino::table::position::Position;
use crate::casino::table::winnings::Winnings;
#[cfg(feature = "bot-profiles")]
use crate::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
#[cfg(feature = "bot-profiles")]
use crate::games::GamePhase;
use crate::play::board::Board;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

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

    /// The full 52-card shuffled deck at the start of this hand, as a
    /// space-separated card string (e.g. `"A♠ K♠ Q♠ ..."`).
    ///
    /// When present, the hand can be fully replayed from this deck alone.
    /// Cards are consumed in order: hole cards dealt clockwise from the button,
    /// then burn+flop, burn+turn, burn+river.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shuffled_deck: Option<String>,
}

fn default_pkcore_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_format_version() -> u32 {
    FORMAT_VERSION
}

/// Per-seat snapshot tuple consumed by
/// [`HandHistory::from_table_state_with_ids`].
///
/// `(seat, name, starting_stack, hole_cards, player_id)` — `player_id` is
/// the per-player [`Uuid`] (typically `PlayerNoCell.id`); pass `None` when
/// identity threading is not needed (or use the simpler 4-tuple
/// [`HandHistory::from_table_state`] entry point).
pub type PlayerSnapshot = (u8, String, usize, Option<String>, Option<Uuid>);

impl HandHistory {
    /// Constructs a [`HandHistory`] from live game state captured around a
    /// completed hand.
    ///
    /// This is the canonical way to build a hand history from a live or
    /// simulated game. Capture `player_snapshot` **before** forced bets and
    /// hole cards immediately **after** the deal, then call this function
    /// right after [`TableNoCell::end_hand`](crate::casino::table_no_cell::TableNoCell::end_hand).
    ///
    /// Snapshots produced by this entry point carry no per-player [`Uuid`].
    /// Callers that need identity threading (player-stats aggregation,
    /// cross-session correlation) should use [`Self::from_table_state_with_ids`]
    /// instead.
    ///
    /// # Parameters
    ///
    /// - `hand_num` — sequential hand number within the session (used in the hand ID).
    /// - `ts_secs` — Unix timestamp in seconds.
    /// - `button` — 0-based seat index of the dealer button.
    /// - `forced` — blinds/ante structure.
    /// - `player_snapshot` — `(seat, name, starting_stack, hole_cards)` tuples.
    /// - `board_str` — full community board string, or `""` when no board was dealt.
    /// - `winnings` — post-`end_hand` pot distribution.
    /// - `event_log` — per-hand slice of `table.event_log` for deriving per-street
    ///   actions.
    /// - `ending_stacks` — `(seat, chips)` captured after `end_hand()`.
    /// - `source` — provenance label (e.g. `"interactive_play"`).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandHistory;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::winnings::Winnings;
    ///
    /// let hh = HandHistory::from_table_state(
    ///     1, 0, 0,
    ///     &ForcedBets::new(50, 100),
    ///     &[(0, "Alice".to_string(), 1000, Some("A♠ K♠".to_string()))],
    ///     "", &Winnings::default(), &[], &[(0, 1000)], "test", None,
    /// );
    /// assert_eq!(hh.hand.id, "test-hand-001");
    /// assert!(hh.results.is_some());
    /// ```
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn from_table_state(
        hand_num: usize,
        ts_secs: u64,
        button: u8,
        forced: &ForcedBets,
        player_snapshot: &[(u8, String, usize, Option<String>)],
        board_str: &str,
        winnings: &Winnings,
        event_log: &[TableAction],
        ending_stacks: &[(u8, usize)],
        source: &str,
        shuffled_deck: Option<String>,
    ) -> Self {
        let lifted: Vec<PlayerSnapshot> = player_snapshot
            .iter()
            .map(|(seat, name, stack, hole)| (*seat, name.clone(), *stack, hole.clone(), None))
            .collect();
        Self::from_table_state_with_ids(
            hand_num,
            ts_secs,
            button,
            forced,
            &lifted,
            board_str,
            winnings,
            event_log,
            ending_stacks,
            source,
            shuffled_deck,
        )
    }

    /// Variant of [`Self::from_table_state`] that threads a per-player [`Uuid`]
    /// through each seat's `PlayerEntry` and every emitted `Action`.
    ///
    /// Use this when downstream analysis (e.g.
    /// [`crate::analysis::player_stats::StatsRegistry`]) needs to correlate
    /// the same player across multiple hands or sessions. The 5-tuple form
    /// of `player_snapshot` is the canonical [`PlayerSnapshot`] type alias.
    ///
    /// # Parameters
    ///
    /// Same as [`Self::from_table_state`] except `player_snapshot` carries an
    /// extra `Option<Uuid>` element per seat (typically `Some(player.id)`
    /// where `player.id` is the [`Uuid`] from `PlayerNoCell`).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandHistory;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::winnings::Winnings;
    /// use uuid::Uuid;
    ///
    /// let alice = Uuid::new_v4();
    /// let hh = HandHistory::from_table_state_with_ids(
    ///     1, 0, 0,
    ///     &ForcedBets::new(50, 100),
    ///     &[(0, "Alice".to_string(), 1000, Some("A♠ K♠".to_string()), Some(alice))],
    ///     "", &Winnings::default(), &[], &[(0, 1000)], "test", None,
    /// );
    /// assert_eq!(hh.players[0].player_id, Some(alice));
    /// ```
    #[allow(
        clippy::cast_precision_loss,
        clippy::too_many_arguments,
        clippy::cast_possible_truncation
    )]
    #[must_use]
    pub fn from_table_state_with_ids(
        hand_num: usize,
        ts_secs: u64,
        button: u8,
        forced: &ForcedBets,
        player_snapshot: &[PlayerSnapshot],
        board_str: &str,
        winnings: &Winnings,
        event_log: &[TableAction],
        ending_stacks: &[(u8, usize)],
        source: &str,
        shuffled_deck: Option<String>,
    ) -> Self {
        // Folded seats — used to emit `Outcome::Fold` instead of conflating
        // "folded" with "lost at showdown".
        let folded_seats: std::collections::HashSet<u8> = event_log
            .iter()
            .filter_map(|e| match e {
                TableAction::Fold(seat) => Some(*seat),
                _ => None,
            })
            .collect();

        let results: Vec<ResultEntry> = player_snapshot
            .iter()
            .map(|(seat, _, starting_stack, hole_cards, _player_id)| {
                let pot_won: f64 = winnings
                    .vec()
                    .iter()
                    .filter(|pw| pw.equity.seats.contains(*seat))
                    .map(|pw| pw.equity.chips as f64)
                    .sum();

                let ending = ending_stacks.iter().find(|(s, _)| s == seat).map(|(_, c)| *c as f64);
                let net = ending.map(|e| e - *starting_stack as f64);

                let ranked = hole_cards.as_deref().and_then(|h| rank_seven(h, board_str));

                let outcome = if folded_seats.contains(seat) {
                    Outcome::Fold
                } else if pot_won > 0.0 {
                    Outcome::Win
                } else {
                    Outcome::Lose
                };

                ResultEntry {
                    seat: *seat,
                    best_hand: ranked.as_ref().map(|r| r.hand.to_string()),
                    hand_rank: ranked.as_ref().map(|r| r.hand_rank),
                    outcome,
                    net,
                    pot_won: if pot_won > 0.0 { Some(pot_won) } else { None },
                    mucked: None,
                }
            })
            .collect();

        HandHistory {
            pkcore_version: None,
            format_version: FORMAT_VERSION,
            hand: HandMeta {
                id: format!("{source}-hand-{hand_num:03}"),
                game: HandVariant::Holdem,
                timestamp: Some(ts_secs.to_string()),
                source: Some(source.to_string()),
                description: None,
            },
            table: TableInfo {
                name: Some(source.to_string()),
                seats: Some(player_snapshot.len() as u8),
                button: Some(button),
                stakes: Stakes {
                    small_blind: forced.small_blind as f64,
                    big_blind: forced.big_blind as f64,
                    ante: if forced.ante > 0 {
                        Some(forced.ante as f64)
                    } else {
                        None
                    },
                    straddle: None,
                    bring_in: if forced.bring_in > 0 {
                        Some(forced.bring_in as f64)
                    } else {
                        None
                    },
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: player_snapshot
                .iter()
                .map(|(seat, name, stack, hole_cards, player_id)| PlayerEntry {
                    seat: *seat,
                    name: name.clone(),
                    stack: *stack as f64,
                    player_id: *player_id,
                    hole_cards: hole_cards.clone(),
                    posted: None,
                    hole_cards_visibility: None,
                    withdrawn: None,
                })
                .collect(),
            board: if board_str.is_empty() {
                None
            } else {
                Some(board_str.to_string())
            },
            streets: {
                let seat_to_id: HashMap<u8, Uuid> = player_snapshot
                    .iter()
                    .filter_map(|(seat, _, _, _, id)| id.map(|id| (*seat, id)))
                    .collect();
                Streets::from_event_log_with_seat_ids(event_log, &seat_to_id)
            },
            results: Some(results),
            analysis: None,
            shuffled_deck,
        }
    }

    /// Fluent setter for the table's
    /// [`BettingStructure`][crate::games::betting_structure::BettingStructure]
    /// (EPIC-30 Phase 9).
    ///
    /// `HandHistory::from_table_state` and friends construct a `TableInfo`
    /// whose `betting_structure` defaults to `NoLimit`. Callers recording
    /// a Fixed-Limit or Pot-Limit hand chain this after `from_table_state`
    /// to record the variant:
    ///
    /// ```ignore
    /// let hh = HandHistory::from_table_state(...)
    ///     .with_betting_structure(table.betting);
    /// ```
    #[must_use]
    pub fn with_betting_structure(mut self, betting: crate::games::betting_structure::BettingStructure) -> Self {
        self.table.betting_structure = betting;
        self
    }

    /// Fluent setter for the hand's [`HandVariant`] (EPIC-31 Phase 4).
    ///
    /// `from_table_state` hardcodes `HandVariant::Holdem`; callers
    /// recording a non-Holdem variant chain this to override:
    ///
    /// ```ignore
    /// let hh = HandHistory::from_table_state(...).with_variant(HandVariant::Omaha);
    /// ```
    #[must_use]
    pub fn with_variant(mut self, variant: HandVariant) -> Self {
        self.hand.game = variant;
        self
    }

    /// Replays all recorded actions from `streets` through a fresh [`TableNoCell`]
    /// and verifies the final chip counts match the recorded `results`.
    ///
    /// This is useful for testing hand-history consistency: generate a session,
    /// serialize to YAML, deserialize, then call `replay()` on each hand to
    /// confirm the recorded actions reproduce the same outcomes.
    ///
    /// Returns [`PKError::InvalidAction`] if the recorded action sequence is
    /// inconsistent with the game rules (e.g., a player acts out of turn or
    /// makes an illegal move).
    ///
    /// # Errors
    ///
    /// Returns [`PKError`] if table construction, card injection, action
    /// application, or hand-end processing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{HandHistory, HandVariant, Outcome, ResultEntry,
    ///     Action, ActionType, Streets, PreflopStreet, PlayerEntry, HandMeta,
    ///     TableInfo, Stakes};
    ///
    /// let hh = HandHistory {
    ///     pkcore_version: None,
    ///     format_version: 1,
    ///     hand: HandMeta {
    ///         id: "test-001".to_string(),
    ///         game: HandVariant::Holdem,
    ///         timestamp: None,
    ///         source: None,
    ///         description: None,
    ///     },
    ///     table: TableInfo {
    ///         name: None,
    ///         seats: Some(2),
    ///         button: Some(0),
    ///         stakes: Stakes { small_blind: 50.0, big_blind: 100.0, ante: None, straddle: None, bring_in: None },
    ///         betting_structure: Default::default(),
    ///     },
    ///     players: vec![
    ///         PlayerEntry { seat: 0, name: "A".to_string(), stack: 1000.0,
    ///             player_id: None, hole_cards: Some("A♠ K♠".to_string()),
    ///             posted: None, hole_cards_visibility: None, withdrawn: None },
    ///         PlayerEntry { seat: 1, name: "B".to_string(), stack: 1000.0,
    ///             player_id: None, hole_cards: Some("7♦ 2♣".to_string()),
    ///             posted: None, hole_cards_visibility: None, withdrawn: None },
    ///     ],
    ///     board: None,
    ///     streets: Some(Streets {
    ///         preflop: Some(PreflopStreet {
    ///             actions: vec![
    ///                 Action { seat: 0, player_id: None, action: ActionType::Post, amount: Some(50.0), all_in: None, agent: None },
    ///                 Action { seat: 1, player_id: None, action: ActionType::Post, amount: Some(100.0), all_in: None, agent: None },
    ///                 Action { seat: 0, player_id: None, action: ActionType::Fold, amount: None, all_in: None, agent: None },
    ///             ],
    ///             pot: Some(150.0),
    ///         }),
    ///         flop: None,
    ///         turn: None,
    ///         river: None,
    ///     }),
    ///     results: Some(vec![
    ///         ResultEntry { seat: 0, best_hand: None, hand_rank: None,
    ///             outcome: Outcome::Fold, net: Some(-50.0), pot_won: None, mucked: None },
    ///         ResultEntry { seat: 1, best_hand: None, hand_rank: None,
    ///             outcome: Outcome::Win, net: Some(50.0), pot_won: Some(150.0), mucked: None },
    ///     ]),
    ///     analysis: None,
    ///     shuffled_deck: None,
    /// };
    ///
    /// let result = hh.replay();
    /// assert!(result.is_ok());
    /// assert!(result.unwrap().is_consistent);
    /// ```
    #[cfg(feature = "bot-profiles")]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::too_many_lines
    )]
    pub fn replay(&self) -> Result<ReplayResult, PKError> {
        // ── Build table ──────────────────────────────────────────────────────
        let sb = self.table.stakes.small_blind as usize;
        let bb = self.table.stakes.big_blind as usize;
        let button = self.table.button.unwrap_or(0);

        // Build a sparse seats array so that each player is placed at their
        // physical seat index.  This ensures seat number == array index — the
        // invariant the engine assumes throughout.  Empty slots (e.g. a seat
        // that was vacated between hands) are filled with default (empty) seats.
        //
        // The array must also be large enough to hold the button seat, which
        // can point past the last occupied seat when the button advanced to an
        // eliminated player's position (dead-button scenario).  Without this,
        // `act_forced_bets()` receives an out-of-range button and computes the
        // wrong action order for the street.
        let max_seat = self.players.iter().map(|p| p.seat as usize).max().unwrap_or(0);
        let button_seat = self.table.button.unwrap_or(0) as usize;
        let table_size = max_seat.max(button_seat) + 1;
        let mut seats_vec: Vec<SeatNoCell> = (0..table_size)
            .map(|_| SeatNoCell::new(PlayerNoCell::default()))
            .collect();
        for p in &self.players {
            seats_vec[p.seat as usize] =
                SeatNoCell::new(PlayerNoCell::new_with_chips(p.name.clone(), p.stack as usize));
        }
        let seats = SeatsNoCell::new(seats_vec);
        // EPIC-30 Phase 9 / EPIC-31 Phase 5: dispatch on recorded
        // variant + structure so PLO replays through `plo_from_seats`,
        // FLHE replays through `limit_holdem_from_seats`, and everything
        // else falls back to NLHE. PLO takes precedence over
        // `betting_structure` dispatch because the variant determines the
        // showdown evaluator (the betting structure only affects sizing).
        // EPIC-32 Phase 9 / EPIC-33 Phase 6: route Stud and Razz
        // through their respective constructors with recorded
        // ante/bring_in + FixedLimit bet sizes. Falls back to sensible
        // defaults when fields are missing from older records. Stud and
        // Razz share the same hand-loop / visibility / bring-in
        // semantics — the only difference at replay time is which
        // `*_from_seats` constructor sets the `GameType` tag, which
        // drives the showdown evaluator dispatch.
        let is_stud_family = self.hand.game == HandVariant::Stud || self.hand.game == HandVariant::Razz;
        let mut table = if is_stud_family {
            let ante = self.table.stakes.ante.map_or(0, |x| x as usize);
            let bring_in = self.table.stakes.bring_in.map_or(0, |x| x as usize);
            let (small_bet, big_bet) = match self.table.betting_structure {
                crate::games::betting_structure::BettingStructure::FixedLimit { small_bet, big_bet, .. } => {
                    (small_bet, big_bet)
                }
                _ => (sb.max(20), bb.max(40)),
            };
            if self.hand.game == HandVariant::Razz {
                TableNoCell::razz_from_seats(seats, ante, bring_in, small_bet, big_bet)
            } else {
                TableNoCell::stud_hi_from_seats(seats, ante, bring_in, small_bet, big_bet)
            }
        } else if self.hand.game == HandVariant::Omaha {
            TableNoCell::plo_from_seats(seats, (sb, bb))
        } else {
            match self.table.betting_structure {
                crate::games::betting_structure::BettingStructure::FixedLimit {
                    small_bet,
                    big_bet,
                    raise_cap,
                } => TableNoCell::limit_holdem_from_seats(seats, small_bet, big_bet, raise_cap),
                _ => TableNoCell::nlh_from_seats(seats, ForcedBets::new(sb, bb)),
            }
        };
        table.button = button;

        // ── Forced bets & hole cards ─────────────────────────────────────────
        table.act_forced_bets()?;

        let hole_entries: Vec<(u8, String)> = self
            .players
            .iter()
            .filter_map(|p| p.hole_cards.as_ref().map(|h| (p.seat, h.clone())))
            .collect();
        let hole_refs: Vec<(u8, &str)> = hole_entries.iter().map(|(s, h)| (*s, h.as_str())).collect();
        table.inject_hole_cards(&hole_refs)?;

        // EPIC-32 Phase 9 / EPIC-33 Phase 6: for Stud-family replays
        // (Stud Hi + Razz), restore per-card visibility from
        // `hole_cards_visibility` (if recorded) and post the bring-in.
        if is_stud_family {
            for p in &self.players {
                if let Some(vis_tokens) = &p.hole_cards_visibility
                    && let Some(seat) = table.seats.get_seat_mut(p.seat)
                {
                    let visibilities: Vec<crate::play::visibility::Visibility> = vis_tokens
                        .iter()
                        .map(|s| {
                            if s.eq_ignore_ascii_case("up") {
                                crate::play::visibility::Visibility::Up
                            } else {
                                crate::play::visibility::Visibility::Down
                            }
                        })
                        .collect();
                    let cards: Vec<crate::card::Card> =
                        seat.hand.iter().map(crate::play::hole_card::HoleCard::card).collect();
                    seat.hand.clear();
                    for (i, card) in cards.iter().enumerate() {
                        let v = visibilities
                            .get(i)
                            .copied()
                            .unwrap_or(crate::play::visibility::Visibility::Down);
                        seat.hand.push(*card, v);
                    }
                }
            }
            // Bring-in is posted after 3rd-street dealing in Stud Hi /
            // Razz. inject_hole_cards represents the full hand, so we
            // post bring-in here before betting actions are replayed.
            table.act_bring_in()?;
        }

        // ── Street replay helper ─────────────────────────────────────────────
        let replay_actions = |table: &mut TableNoCell, actions: &[Action]| -> Result<(), PKError> {
            for action in actions {
                if let Some(pa) = action_to_player_action(action) {
                    table.apply_action(action.seat, pa)?;
                }
            }
            Ok(())
        };

        // ── Preflop ──────────────────────────────────────────────────────────
        if let Some(ref streets) = self.streets
            && let Some(ref pre) = streets.preflop
        {
            replay_actions(&mut table, &pre.actions)?;
        }

        if table.is_game_over() {
            return build_replay_result(table, &self.players, self.results.as_deref());
        }

        table.bring_it_in()?;

        // ── Flop ─────────────────────────────────────────────────────────────
        if let Some(ref streets) = self.streets
            && let Some(ref flop) = streets.flop
        {
            table.board = Cards::from_str(&flop.cards)?;
            table.phase = GamePhase::DealFlop;
            // Backward-compat: pre-0.0.42 YAMLs recorded actions on all-in
            // run-out streets (frozen bring_it_in didn't exist yet). Reset
            // non-all-in players to YetToAct only when there are actions to
            // apply; an empty list means the current frozen behavior applies,
            // and the frozen state must be preserved so the next bring_it_in()
            // sees is_betting_complete() == true.
            if !flop.actions.is_empty() {
                table.seats.reset_non_allin_to_yet_to_act();
            }
            replay_actions(&mut table, &flop.actions)?;

            if table.is_game_over() {
                return build_replay_result(table, &self.players, self.results.as_deref());
            }

            table.bring_it_in()?;

            // ── Turn ─────────────────────────────────────────────────────
            if let Some(ref turn) = streets.turn {
                let card = Card::from_str(&turn.card)?;
                table.board.insert(card);
                table.phase = GamePhase::DealTurn;
                if !turn.actions.is_empty() {
                    table.seats.reset_non_allin_to_yet_to_act();
                }
                replay_actions(&mut table, &turn.actions)?;

                if table.is_game_over() {
                    return build_replay_result(table, &self.players, self.results.as_deref());
                }

                table.bring_it_in()?;

                // ── River ─────────────────────────────────────────────────
                if let Some(ref river) = streets.river {
                    let card = Card::from_str(&river.card)?;
                    table.board.insert(card);
                    table.phase = GamePhase::DealRiver;
                    if !river.actions.is_empty() {
                        table.seats.reset_non_allin_to_yet_to_act();
                    }
                    replay_actions(&mut table, &river.actions)?;
                }
            }
        }

        build_replay_result(table, &self.players, self.results.as_deref())
    }

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
    ///         stakes: Stakes { small_blind: 1.0, big_blind: 2.0, ante: None, straddle: None, bring_in: None },
    ///         betting_structure: Default::default(),
    ///     },
    ///     players: vec![],
    ///     board: None,
    ///     streets: None,
    ///     results: None,
    ///     analysis: None,
    ///     shuffled_deck: None,
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

    /// Returns an iterator over hands in which the given player participated.
    ///
    /// A "participated" hand is one whose [`HandHistory::players`] list contains
    /// a [`PlayerEntry`] with `player_id == Some(id)`. Hands written by older
    /// pkcore versions that did not stamp `player_id` are silently skipped
    /// (their entries carry `player_id: None`).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandCollection;
    /// use uuid::Uuid;
    ///
    /// let collection = HandCollection::new();
    /// let count = collection.hands_by_player(Uuid::nil()).count();
    /// assert_eq!(0, count);
    /// ```
    pub fn hands_by_player(&self, id: Uuid) -> impl Iterator<Item = &HandHistory> {
        self.hands
            .iter()
            .filter(move |h| h.players.iter().any(|p| p.player_id == Some(id)))
    }

    /// Returns an iterator over hands whose table size and button placement
    /// would seat at least one player at the given [`Position`].
    ///
    /// Useful for excluding short-handed hands when computing position-specific
    /// stats — e.g. `hands_by_position(Position::CO)` strips heads-up and
    /// 3-handed hands that have no cutoff. To filter to a specific player at a
    /// specific position, chain with [`Self::hands_by_player`] and then
    /// reapply.
    ///
    /// Returns no matches when the hand omits a button or its `players` list
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::position::Position;
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::new();
    /// assert_eq!(0, collection.hands_by_position(Position::CO).count());
    /// ```
    pub fn hands_by_position(&self, pos: Position) -> impl Iterator<Item = &HandHistory> {
        self.hands.iter().filter(move |h| hand_has_position(h, pos))
    }

    /// Returns an iterator over hands that reached showdown — i.e. at least
    /// two players' [`ResultEntry::outcome`] is anything other than
    /// [`Outcome::Fold`].
    ///
    /// Hands without a `results` block (legacy YAMLs from before `from_table_state`
    /// emitted them) are skipped.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::new();
    /// assert_eq!(0, collection.showdowns_only().count());
    /// ```
    pub fn showdowns_only(&self) -> impl Iterator<Item = &HandHistory> {
        self.hands.iter().filter(|h| {
            h.results
                .as_ref()
                .is_some_and(|results| results.iter().filter(|r| r.outcome != Outcome::Fold).count() >= 2)
        })
    }
}

/// Returns `true` when `hand` would seat at least one player at `pos` given
/// the recorded button and the count of occupied seats in `hand.players`.
///
/// Translates physical seat indices to logical (button-relative) positions
/// the same way [`crate::bot::table_snapshot::TableSnapshot::from_table`]
/// does, so sparse seat numbering after eliminations doesn't desync the
/// position math.
fn hand_has_position(hand: &HandHistory, pos: Position) -> bool {
    let Some(button_phys) = hand.table.button else {
        return false;
    };
    let mut occupied: Vec<u8> = hand.players.iter().map(|p| p.seat).collect();
    if occupied.is_empty() {
        return false;
    }
    occupied.sort_unstable();
    let Ok(seat_count) = u8::try_from(occupied.len()) else {
        return false;
    };
    let Some(button_logical_idx) = occupied.iter().position(|&s| s == button_phys) else {
        return false;
    };
    let Ok(button_logical) = u8::try_from(button_logical_idx) else {
        return false;
    };
    hand.players.iter().any(|p| {
        let Some(logical_idx) = occupied.iter().position(|&s| s == p.seat) else {
            return false;
        };
        let Ok(logical) = u8::try_from(logical_idx) else {
            return false;
        };
        Position::from_seat(logical, button_logical, seat_count) == Some(pos)
    })
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

    /// Serialize and save this collection to `generated/<run_name>_<unix_ts>.yaml`.
    ///
    /// Creates the `generated/` directory if it does not exist. Returns the path
    /// written on success.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the filesystem write fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::new();
    /// let path = collection.save("my_session").unwrap();
    /// assert!(path.starts_with("generated/my_session_"));
    /// # }
    /// ```
    pub fn save(&self, run_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let path = format!("generated/{run_name}_{ts}.yaml");
        let yaml = self.to_yaml()?;
        std::fs::create_dir_all("generated")?;
        std::fs::write(&path, &yaml)?;
        Ok(path)
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
///     stakes: Stakes { small_blind: 100.0, big_blind: 200.0, ante: None, straddle: None, bring_in: None },
///     betting_structure: Default::default(),
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

    /// Betting structure — no-limit / pot-limit / fixed-limit (EPIC-30
    /// Phase 9). Older YAML hand histories (recorded before this field
    /// existed) deserialize as
    /// [`BettingStructure::NoLimit`][crate::games::betting_structure::BettingStructure::NoLimit]
    /// via `#[serde(default)]`.
    #[serde(default)]
    pub betting_structure: crate::games::betting_structure::BettingStructure,
}

/// Blind and ante structure for a hand.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::Stakes;
///
/// let stakes = Stakes { small_blind: 5.0, big_blind: 10.0, ante: Some(2.0), straddle: None, bring_in: None };
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

    /// Stud-family bring-in amount (EPIC-32 Phase 9). `None` for Hold'em
    /// and Omaha; non-`None` for Stud Hi and Razz when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bring_in: Option<f64>,
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
///     player_id: None,
///     hole_cards: Some("A♠ K♠".to_string()),
///     posted: None,
///     hole_cards_visibility: None,
///     withdrawn: None,
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

    /// Stable per-player identity carried through `TableAction::PlayerSeated`.
    ///
    /// `None` when the entry comes from a legacy YAML file that was written
    /// before EPIC-26 added identity propagation. New sessions populate it
    /// from `PlayerNoCell::uuid`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_id: Option<Uuid>,

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

    /// Per-card visibility tags (EPIC-32 Phase 9). Length matches the
    /// number of cards in `hole_cards` when non-`None`; each entry is
    /// `"up"` or `"down"`. Used by Stud Hi and Razz to record which
    /// cards were dealt face-up on each street; Hold'em/Omaha records
    /// leave this `None` (cards are implicitly all Down).
    ///
    /// Replay (via [`crate::casino::table_no_cell::TableNoCell::inject_hole_cards`])
    /// reads this field when present and pushes each card to the seat's
    /// `SeatHand` with the recorded visibility. When `None`, all cards
    /// are pushed as `Visibility::Down`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hole_cards_visibility: Option<Vec<String>>,

    /// Cumulative chips this player has taken out of cash — the initial buy-in
    /// plus every subsequent reload — at the time the hand was recorded.
    /// `None` for legacy YAML files written before this field existed, and
    /// for any session that wasn't tracking reloads. Pairs with
    /// `Player::withdrawn` / `PlayerNoCell::withdrawn` at the player level
    /// and feeds the profit/loss calc `stack + chips_in_pot - withdrawn`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub withdrawn: Option<f64>,
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
    ///     player_id: None,
    ///     hole_cards: Some("A♠ K♠".to_string()),
    ///     posted: None,
    ///     hole_cards_visibility: None,
    ///     withdrawn: None,
    /// };
    /// assert!(player.to_two().is_ok());
    /// ```
    pub fn to_two(&self) -> Result<Two, PKError> {
        match &self.hole_cards {
            Some(s) => Two::from_str(s),
            None => Err(PKError::NotEnoughCards),
        }
    }

    /// Parses `hole_cards` as a 4-card Omaha hand (EPIC-31 Phase 4).
    ///
    /// # Errors
    ///
    /// Returns `PKError::NotEnoughCards` when there are no hole cards
    /// recorded, or an `Err` from `Four::from_str` if the string is
    /// malformed.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::PlayerEntry;
    ///
    /// let player = PlayerEntry {
    ///     seat: 0,
    ///     name: "A".to_string(),
    ///     stack: 1000.0,
    ///     player_id: None,
    ///     hole_cards: Some("A♠ K♠ Q♠ J♠".to_string()),
    ///     posted: None,
    ///     hole_cards_visibility: None,
    ///     withdrawn: None,
    /// };
    /// assert!(player.to_four().is_ok());
    /// ```
    pub fn to_four(&self) -> Result<crate::arrays::four::Four, PKError> {
        match &self.hole_cards {
            Some(s) => crate::arrays::four::Four::from_str(s),
            None => Err(PKError::NotEnoughCards),
        }
    }

    /// Parses `hole_cards` as a 7-card Stud hand (EPIC-32 Phase 9).
    ///
    /// # Errors
    ///
    /// Returns `PKError::NotEnoughCards` when there are no hole cards
    /// recorded, or an `Err` from `Seven::from_str` if the string is
    /// malformed.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::PlayerEntry;
    ///
    /// let player = PlayerEntry {
    ///     seat: 0,
    ///     name: "A".to_string(),
    ///     stack: 1000.0,
    ///     player_id: None,
    ///     hole_cards: Some("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠".to_string()),
    ///     posted: None,
    ///     hole_cards_visibility: None,
    ///     withdrawn: None,
    /// };
    /// assert!(player.to_seven().is_ok());
    /// ```
    pub fn to_seven(&self) -> Result<crate::arrays::seven::Seven, PKError> {
        match &self.hole_cards {
            Some(s) => crate::arrays::seven::Seven::from_str(s),
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
///         actions: vec![Action { seat: 1, player_id: None, action: ActionType::Fold, amount: None, all_in: None, agent: None }],
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
    pub fn from_event_log(log: &[TableAction]) -> Option<Self> {
        let seat_to_id: HashMap<u8, Uuid> = log
            .iter()
            .filter_map(|ev| match ev {
                TableAction::PlayerSeated(seat, id) => Some((*seat, *id)),
                _ => None,
            })
            .collect();
        Self::from_event_log_with_seat_ids(log, &seat_to_id)
    }

    /// Variant of [`Streets::from_event_log`] that takes the seat → [`Uuid`]
    /// mapping explicitly instead of scanning `log` for
    /// [`TableAction::PlayerSeated`] events.
    ///
    /// Use this when `log` is a *per-hand slice* of a longer-running session's
    /// event log — `PlayerSeated` events fire at table construction and are
    /// not present in subsequent per-hand slices, so the implicit scan would
    /// leave every `Action.player_id` as `None`.
    ///
    /// Returns `None` only if `log` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use pkcore::hand_history::Streets;
    /// use pkcore::casino::table::event::TableAction;
    /// use uuid::Uuid;
    ///
    /// let alice = Uuid::new_v4();
    /// let bob = Uuid::new_v4();
    /// let mut seat_to_id = HashMap::new();
    /// seat_to_id.insert(1, alice);
    /// seat_to_id.insert(2, bob);
    ///
    /// // Slice does NOT contain PlayerSeated events.
    /// let log = vec![
    ///     TableAction::ForcedBetSmallBlind(1, 50),
    ///     TableAction::ForcedBetBigBlind(2, 100),
    ///     TableAction::Fold(1),
    /// ];
    /// let streets = Streets::from_event_log_with_seat_ids(&log, &seat_to_id).unwrap();
    /// let action = &streets.preflop.as_ref().unwrap().actions[2];
    /// assert_eq!(action.player_id, Some(alice));
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn from_event_log_with_seat_ids(log: &[TableAction], seat_to_id: &HashMap<u8, Uuid>) -> Option<Self> {
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
                    if let Some(action) = table_action_to_hand_action(other, seat_to_id) {
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
/// `seat_to_id` supplies the per-seat `Uuid` previously announced via
/// `TableAction::PlayerSeated`; missing entries leave `Action.player_id` as
/// `None` (e.g. legacy event logs that pre-date EPIC-26).
#[allow(clippy::cast_precision_loss)]
fn table_action_to_hand_action(event: &TableAction, seat_to_id: &HashMap<u8, Uuid>) -> Option<Action> {
    let make = |seat: u8, action: ActionType, amount: Option<f64>, all_in: Option<bool>| Action {
        seat,
        player_id: seat_to_id.get(&seat).copied(),
        action,
        amount,
        all_in,
        agent: None,
    };
    match event {
        TableAction::ForcedBetSmallBlind(seat, amount)
        | TableAction::ForcedBetBigBlind(seat, amount)
        | TableAction::BetAnteForced(seat, amount)
        | TableAction::ForcedBet(seat, amount) => Some(make(*seat, ActionType::Post, Some(*amount as f64), None)),
        TableAction::Check(seat) => Some(make(*seat, ActionType::Check, None, None)),
        TableAction::Bet(seat, amount) => Some(make(*seat, ActionType::Bet, Some(*amount as f64), None)),
        TableAction::Call(seat, amount) => Some(make(*seat, ActionType::Call, Some(*amount as f64), None)),
        TableAction::Raise(seat, amount) => Some(make(*seat, ActionType::Raise, Some(*amount as f64), None)),
        TableAction::AllIn(seat, amount) => Some(make(*seat, ActionType::AllIn, Some(*amount as f64), Some(true))),
        TableAction::Fold(seat) => Some(make(*seat, ActionType::Fold, None, None)),
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
///     actions: vec![Action { seat: 1, player_id: None, action: ActionType::Check, amount: None, all_in: None, agent: None }],
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
///     actions: vec![Action { seat: 1, player_id: None, action: ActionType::Check, amount: None, all_in: None, agent: None }],
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
// Agent fidelity injection
// ─────────────────────────────────────────────────────────────────────────────

impl HandHistory {
    /// Attaches agent-fidelity metadata to this hand's **voluntary** actions in
    /// canonical order (preflop → flop → turn → river, in recorded order),
    /// skipping forced `Post` actions (blinds/antes).
    ///
    /// `entries` must be in that same canonical voluntary-action order; each is
    /// `(expected_seat, fidelity)`. This is a **strict positional pairing**:
    /// `entries[i]` is matched against the `i`-th voluntary action. If the
    /// seats agree the fidelity is assigned; if they disagree that slot is
    /// skipped (action left `None`) so a corrupted entry is never misattributed
    /// at its own position.
    ///
    /// The seat check is a per-slot guard, **not** a resynchronizer. It does not
    /// recover alignment after a length change: if `entries` has an extra or
    /// missing element relative to the voluntary actions, every later pair is
    /// offset, and any later same-seat collision will misattribute silently.
    /// Callers whose recorder can drop or duplicate entries should detect this
    /// via the return value (see below) or match by hand through
    /// [`HandHistory::voluntary_actions_mut`] using a key unique per decision.
    ///
    /// Returns the number of actions successfully annotated. In the intended
    /// 1:1 case this equals both `entries.len()` and the voluntary-action count;
    /// **any inequality signals drift** and should be treated as an error by the
    /// caller. Never panics, and never reorders or drops actions. This data is
    /// analysis-only: [`HandHistory::replay`] ignores it entirely.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{HandHistory, HandVariant, AgentFidelity, Action,
    ///     ActionType, Streets, PreflopStreet, PlayerEntry, HandMeta, TableInfo, Stakes};
    ///
    /// let mut hh = HandHistory {
    ///     pkcore_version: None, format_version: 1,
    ///     hand: HandMeta { id: "ex".to_string(), game: HandVariant::Holdem,
    ///         timestamp: None, source: None, description: None },
    ///     table: TableInfo { name: None, seats: Some(2), button: Some(0),
    ///         stakes: Stakes { small_blind: 50.0, big_blind: 100.0, ante: None, straddle: None, bring_in: None },
    ///         betting_structure: Default::default() },
    ///     players: vec![],
    ///     board: None,
    ///     streets: Some(Streets {
    ///         preflop: Some(PreflopStreet { actions: vec![
    ///             Action { seat: 0, player_id: None, action: ActionType::Post, amount: Some(50.0), all_in: None, agent: None },
    ///             Action { seat: 1, player_id: None, action: ActionType::Post, amount: Some(100.0), all_in: None, agent: None },
    ///             Action { seat: 0, player_id: None, action: ActionType::Raise, amount: Some(300.0), all_in: None, agent: None },
    ///             Action { seat: 1, player_id: None, action: ActionType::Fold, amount: None, all_in: None, agent: None },
    ///         ], pot: None }),
    ///         flop: None, turn: None, river: None,
    ///     }),
    ///     results: None, analysis: None, shuffled_deck: None,
    /// };
    ///
    /// let entries = [
    ///     (0, AgentFidelity { was_coerced: Some(true), ..Default::default() }),
    ///     (1, AgentFidelity::default()),
    /// ];
    /// assert_eq!(hh.attach_agent_fidelity(&entries), 2);
    /// assert!(hh.voluntary_actions_mut()[0].agent.is_some());
    /// ```
    pub fn attach_agent_fidelity(&mut self, entries: &[(u8, AgentFidelity)]) -> usize {
        let mut annotated = 0usize;
        // Strict positional pairing: entry[i] ↔ i-th voluntary action (`zip`
        // stops at the shorter). The seat check guards each slot against
        // misattribution but does NOT resynchronize — a length difference
        // offsets every later pair. Callers detect that via the returned count
        // (see the doc comment); robust matching belongs to the caller.
        for (action, (expected_seat, fidelity)) in self.voluntary_actions_mut().into_iter().zip(entries) {
            if *expected_seat == action.seat {
                action.agent = Some(fidelity.clone());
                annotated += 1;
            }
        }
        annotated
    }

    /// Mutable references to every voluntary (`!= Post`) action across all
    /// streets, in canonical order (preflop → flop → turn → river, in recorded
    /// order).
    ///
    /// Forced blind/ante `Post` actions are excluded. This is the low-level
    /// escape hatch behind [`HandHistory::attach_agent_fidelity`]; use it to
    /// implement bespoke matching when positional zipping is not enough.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{HandHistory, HandVariant, Action, ActionType,
    ///     Streets, PreflopStreet, HandMeta, TableInfo, Stakes};
    ///
    /// let mut hh = HandHistory {
    ///     pkcore_version: None, format_version: 1,
    ///     hand: HandMeta { id: "ex".to_string(), game: HandVariant::Holdem,
    ///         timestamp: None, source: None, description: None },
    ///     table: TableInfo { name: None, seats: Some(2), button: Some(0),
    ///         stakes: Stakes { small_blind: 50.0, big_blind: 100.0, ante: None, straddle: None, bring_in: None },
    ///         betting_structure: Default::default() },
    ///     players: vec![],
    ///     board: None,
    ///     streets: Some(Streets {
    ///         preflop: Some(PreflopStreet { actions: vec![
    ///             Action { seat: 0, player_id: None, action: ActionType::Post, amount: Some(50.0), all_in: None, agent: None },
    ///             Action { seat: 0, player_id: None, action: ActionType::Raise, amount: Some(300.0), all_in: None, agent: None },
    ///         ], pot: None }),
    ///         flop: None, turn: None, river: None,
    ///     }),
    ///     results: None, analysis: None, shuffled_deck: None,
    /// };
    ///
    /// let voluntary = hh.voluntary_actions_mut();
    /// assert_eq!(voluntary.len(), 1); // the Post is excluded
    /// assert_eq!(voluntary[0].action, ActionType::Raise);
    /// ```
    pub fn voluntary_actions_mut(&mut self) -> Vec<&mut Action> {
        let mut out: Vec<&mut Action> = Vec::new();
        let Some(streets) = self.streets.as_mut() else {
            return out;
        };
        if let Some(s) = streets.preflop.as_mut() {
            out.extend(s.actions.iter_mut().filter(|a| a.action != ActionType::Post));
        }
        if let Some(s) = streets.flop.as_mut() {
            out.extend(s.actions.iter_mut().filter(|a| a.action != ActionType::Post));
        }
        if let Some(s) = streets.turn.as_mut() {
            out.extend(s.actions.iter_mut().filter(|a| a.action != ActionType::Post));
        }
        if let Some(s) = streets.river.as_mut() {
            out.extend(s.actions.iter_mut().filter(|a| a.action != ActionType::Post));
        }
        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Actions
// ─────────────────────────────────────────────────────────────────────────────

/// Per-action provenance describing what an agent *produced* versus what the
/// table *applied*.
///
/// Optional and analysis-only: [`HandHistory::replay`] ignores it entirely, the
/// same way it ignores [`HandHistory::shuffled_deck`]. It is populated by arena
/// recorders (pkdealer EPIC-40) and is absent for hand histories imported from
/// other sources.
///
/// The *applied* action stays in the surrounding [`Action`] fields; this struct
/// records only the agent-side story: the raw response, whether the table had
/// to coerce the action, what the agent originally intended, and (for LLM
/// agents) token usage and model id.
///
/// Every field is `Option` and skips serialization when `None`, so a default
/// `AgentFidelity` emits as an empty map and never adds noise to a hand history.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{AgentFidelity, ActionType};
///
/// // An LLM agent whose bet was clamped to a legal size.
/// let fidelity = AgentFidelity {
///     raw_response: Some("I'll raise to 250".to_string()),
///     was_coerced: Some(true),
///     intended_action: Some(ActionType::Raise),
///     intended_amount: Some(250.0),
///     input_tokens: Some(1200),
///     output_tokens: Some(8),
///     model: Some("claude-sonnet".to_string()),
/// };
/// assert_eq!(fidelity.was_coerced, Some(true));
/// assert_eq!(fidelity.intended_action, Some(ActionType::Raise));
///
/// // A structured agent (rules/random) leaves the LLM-only fields empty.
/// let plain = AgentFidelity { model: Some("rules-v1".to_string()), ..Default::default() };
/// assert_eq!(plain.raw_response, None);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct AgentFidelity {
    /// Raw, unparsed model/agent response text (LLM agents). `None` for agents
    /// that produce a structured decision directly (rules/random).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_response: Option<String>,

    /// True when the applied action differs from what the agent intended —
    /// e.g. unparseable model output, a bet/raise clamped to a legal size, or a
    /// server-rejected action replaced by a safe fallback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub was_coerced: Option<bool>,

    /// The action the agent originally intended, when it differs from the
    /// applied [`Action::action`]. Pairs with [`Self::intended_amount`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intended_action: Option<ActionType>,

    /// Intended wager amount for an intended bet/raise/call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intended_amount: Option<f64>,

    /// Prompt/input tokens reported by the backend (LLM agents).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,

    /// Completion/output tokens reported by the backend (LLM agents).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u32>,

    /// Model / agent identifier (e.g. `"claude-..."`, `"rules-v1"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// A single player action within a betting round.
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{Action, ActionType};
///
/// let action = Action {
///     seat: 3,
///     player_id: None,
///     action: ActionType::Raise,
///     amount: Some(100.0),
///     all_in: None,
///     agent: None,
/// };
/// assert_eq!(action.seat, 3);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Action {
    /// Seat number of the acting player (1-indexed).
    pub seat: u8,

    /// Stable per-player identity stamped from `TableAction::PlayerSeated`.
    ///
    /// `None` for actions parsed from legacy YAML files written before
    /// EPIC-26 added identity propagation. Live event logs always populate
    /// this from the seat's `PlayerSeated` Uuid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_id: Option<Uuid>,

    /// The action taken.
    pub action: ActionType,

    /// Amount wagered (for `bet`, `raise`, `call`). Omit for `check`/`fold`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,

    /// Whether the player is all-in after this action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_in: Option<bool>,

    /// Optional agent-fidelity provenance: what an agent produced versus what
    /// the table applied.
    ///
    /// Analysis-only and ignored by [`HandHistory::replay`]. Populated by arena
    /// recorders via [`HandHistory::attach_agent_fidelity`]; `None` for forced
    /// `Post` actions and for hand histories imported from other sources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentFidelity>,
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
// Replay
// ─────────────────────────────────────────────────────────────────────────────

/// Result of replaying a [`HandHistory`] through the game engine.
///
/// Returned by [`HandHistory::replay`].
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::ReplayResult;
///
/// let r = ReplayResult { final_stacks: vec![(0, 1000)], is_consistent: true };
/// assert!(r.is_consistent);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayResult {
    /// Final chip count per seat (0-based seat index → chips) after `end_hand`.
    pub final_stacks: Vec<(u8, usize)>,
    /// `true` when every seat's final stack matches the recorded `net` P&L
    /// within a ±1 chip rounding tolerance.
    pub is_consistent: bool,
}

impl HandCollection {
    /// Replays every hand in this collection and returns one [`ReplayResult`]
    /// per hand (in order).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::HandCollection;
    ///
    /// let collection = HandCollection::new();
    /// let results = collection.replay_all();
    /// assert!(results.is_empty());
    /// ```
    #[cfg(feature = "bot-profiles")]
    pub fn replay_all(&self) -> Vec<Result<ReplayResult, PKError>> {
        self.hands.iter().map(HandHistory::replay).collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Private helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the best-hand evaluation for `hole_cards` on a completed 5-card
/// `board`.  Returns `None` if the board is not yet complete or cards cannot
/// be parsed.
fn rank_seven(hole_cards: &str, board: &str) -> Option<Eval> {
    if board.split_whitespace().count() < 5 {
        return None;
    }
    let seven = Seven::from_str(&format!("{hole_cards} {board}")).ok()?;
    let (hand_rank, hand) = seven.hand_rank_and_hand();
    Some(Eval::new(hand_rank, hand))
}

/// Converts a hand-history [`Action`] to a [`PlayerAction`] understood by the
/// game engine.  Returns `None` for `Post` entries (handled by
/// `act_forced_bets`).
#[cfg(feature = "bot-profiles")]
fn action_to_player_action(action: &Action) -> Option<PlayerAction> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    match action.action {
        ActionType::Fold => Some(PlayerAction::Fold),
        ActionType::Check => Some(PlayerAction::Check),
        ActionType::Call => Some(PlayerAction::Call),
        ActionType::Bet => action.amount.map(|a| PlayerAction::Bet(a as usize)),
        ActionType::Raise => action.amount.map(|a| PlayerAction::Raise(a as usize)),
        ActionType::AllIn => Some(PlayerAction::AllIn),
        ActionType::Post => None,
    }
}

/// Calls `end_hand` on `table` and builds a [`ReplayResult`], comparing final
/// chip counts against the recorded `results` when present.
/// Chip amounts are stored as `f64` in the YAML schema (for forward-compat with
/// fractional blinds), but represent whole-chip counts in practice.  The casts
/// here are intentional: stacks fit in `usize` and net values are bounded by the
/// starting stack.
#[cfg(feature = "bot-profiles")]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn build_replay_result(
    mut table: TableNoCell,
    players: &[PlayerEntry],
    results: Option<&[ResultEntry]>,
) -> Result<ReplayResult, PKError> {
    let winnings = table.end_hand()?;
    let _ = winnings;

    let final_stacks: Vec<(u8, usize)> = players
        .iter()
        .filter_map(|p| table.seats.get_seat(p.seat).map(|s| (p.seat, s.player.chips)))
        .collect();

    let is_consistent = match results {
        None => true,
        Some(entries) => entries.iter().all(|r| {
            let Some(net) = r.net else { return true };
            let Some(player) = players.iter().find(|p| p.seat == r.seat) else {
                return false;
            };
            // Expected final stack: starting stack + net result, clamped to 0.
            let expected = (player.stack + net).max(0.0).round() as usize;
            let actual = final_stacks.iter().find(|(s, _)| *s == r.seat).map_or(0, |(_, c)| *c);
            expected.abs_diff(actual) <= 1
        }),
    };

    Ok(ReplayResult {
        final_stacks,
        is_consistent,
    })
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
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: vec![],
            board: None,
            streets: None,
            results: None,
            analysis: None,
            shuffled_deck: None,
        };
        assert_eq!(hh.to_board(), Err(PKError::NotEnoughCards));
    }

    #[test]
    fn test_player_entry_no_hole_cards() {
        let player = PlayerEntry {
            seat: 1,
            player_id: None,
            name: "Alice".to_string(),
            stack: 200.0,
            hole_cards: None,
            posted: None,
            hole_cards_visibility: None,
            withdrawn: None,
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
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: vec![],
            board: None,
            streets: None,
            results: None,
            analysis: None,
            shuffled_deck: None,
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

    // ── EPIC-26 Phase 5a: HandCollection query helpers ──────────────────────

    /// Builds a hand with a custom button + the given player entries.
    fn hand_with_players(id: &str, button: u8, players: Vec<PlayerEntry>) -> HandHistory {
        let mut hh = make_minimal_hand(id, HandVariant::Holdem);
        hh.table.button = Some(button);
        hh.players = players;
        hh
    }

    fn entry(seat: u8, name: &str, player_id: Option<Uuid>) -> PlayerEntry {
        PlayerEntry {
            seat,
            name: name.to_string(),
            stack: 1_000.0,
            player_id,
            hole_cards: None,
            posted: None,
            hole_cards_visibility: None,
            withdrawn: None,
        }
    }

    fn result(seat: u8, outcome: Outcome) -> ResultEntry {
        ResultEntry {
            seat,
            best_hand: None,
            hand_rank: None,
            outcome,
            net: None,
            pot_won: None,
            mucked: None,
        }
    }

    #[test]
    fn hands_by_player_returns_only_matching() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();
        let mut collection = HandCollection::new();
        collection.push(hand_with_players(
            "h1",
            0,
            vec![entry(0, "Alice", Some(alice)), entry(1, "Bob", Some(bob))],
        ));
        collection.push(hand_with_players("h2", 0, vec![entry(0, "Bob", Some(bob))]));
        collection.push(hand_with_players("h3", 0, vec![entry(0, "Alice", Some(alice))]));

        let alice_hands: Vec<&HandHistory> = collection.hands_by_player(alice).collect();
        assert_eq!(2, alice_hands.len());
        assert_eq!("h1", alice_hands[0].hand.id);
        assert_eq!("h3", alice_hands[1].hand.id);
    }

    #[test]
    fn hands_by_player_skips_legacy_entries_without_id() {
        // A pre-EPIC-26 YAML file has player_id: None on every entry. Querying
        // by any Uuid should return nothing — there's no identity to match on.
        let some_id = Uuid::new_v4();
        let mut collection = HandCollection::new();
        collection.push(hand_with_players("legacy", 0, vec![entry(0, "Anon", None)]));
        assert_eq!(0, collection.hands_by_player(some_id).count());
    }

    #[test]
    fn hands_by_player_empty_collection() {
        let collection = HandCollection::new();
        assert_eq!(0, collection.hands_by_player(Uuid::new_v4()).count());
    }

    #[test]
    fn hands_by_position_excludes_short_handed() {
        // Heads-up has only BTN/BB; querying CO must skip it. A 6-handed hand
        // does have a CO (offset 5 from button). Mixed collection → only the
        // 6-handed hand matches.
        let mut collection = HandCollection::new();
        collection.push(hand_with_players(
            "headsup",
            0,
            vec![entry(0, "A", None), entry(1, "B", None)],
        ));
        let six_handed: Vec<PlayerEntry> = (0..6).map(|i| entry(i, "P", None)).collect();
        collection.push(hand_with_players("six", 0, six_handed));

        let cos: Vec<&HandHistory> = collection.hands_by_position(Position::CO).collect();
        assert_eq!(1, cos.len(), "only the 6-handed hand has a CO");
        assert_eq!("six", cos[0].hand.id);
    }

    #[test]
    fn hands_by_position_handles_sparse_seat_indices() {
        // Seats 2/4/6 occupied, button at seat 4 (button is at the second
        // logical seat among occupied). Logical mapping: occupied=[2,4,6],
        // button_logical=1. With seat_count=3 and button at logical 1, the
        // BB lives at logical offset 2 → physical seat 6.
        let mut collection = HandCollection::new();
        collection.push(hand_with_players(
            "sparse",
            4,
            vec![entry(2, "A", None), entry(4, "B", None), entry(6, "C", None)],
        ));
        // BB is present in any 3-handed hand.
        assert_eq!(1, collection.hands_by_position(Position::BB).count());
    }

    #[test]
    fn hands_by_position_skips_when_button_missing() {
        let mut collection = HandCollection::new();
        let mut hh = make_minimal_hand("no-btn", HandVariant::Holdem);
        hh.players = vec![entry(0, "A", None), entry(1, "B", None)];
        // Leave hh.table.button = None.
        collection.push(hh);
        assert_eq!(0, collection.hands_by_position(Position::BTN).count());
    }

    #[test]
    fn showdowns_only_requires_two_non_folders() {
        let mut collection = HandCollection::new();

        // Hand A: one Win + one Fold = won by all-folds, NOT a showdown.
        let mut a = make_minimal_hand("walk", HandVariant::Holdem);
        a.results = Some(vec![result(0, Outcome::Win), result(1, Outcome::Fold)]);
        collection.push(a);

        // Hand B: one Win + one Lose = both saw it through to showdown.
        let mut b = make_minimal_hand("showdown", HandVariant::Holdem);
        b.results = Some(vec![result(0, Outcome::Win), result(1, Outcome::Lose)]);
        collection.push(b);

        // Hand C: three-way with two Folds and one Win = walk, NOT showdown.
        let mut c = make_minimal_hand("3way-walk", HandVariant::Holdem);
        c.results = Some(vec![
            result(0, Outcome::Win),
            result(1, Outcome::Fold),
            result(2, Outcome::Fold),
        ]);
        collection.push(c);

        let sds: Vec<&HandHistory> = collection.showdowns_only().collect();
        assert_eq!(1, sds.len());
        assert_eq!("showdown", sds[0].hand.id);
    }

    #[test]
    fn showdowns_only_skips_hands_without_results() {
        let mut collection = HandCollection::new();
        let mut hh = make_minimal_hand("no-results", HandVariant::Holdem);
        hh.results = None;
        collection.push(hh);
        assert_eq!(0, collection.showdowns_only().count());
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

    // ─────────────────────────────────────────────────────────────────────────
    // from_table_state
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_from_table_state_hand_id_and_source() {
        use crate::casino::game::ForcedBets;
        use crate::casino::table::winnings::Winnings;

        let hh = HandHistory::from_table_state(
            3,
            0,
            0,
            &ForcedBets::new(50, 100),
            &[(0, "Alice".to_string(), 1000, None)],
            "",
            &Winnings::default(),
            &[],
            &[(0, 1000)],
            "test_source",
            None,
        );
        assert_eq!(hh.hand.id, "test_source-hand-003");
        assert_eq!(hh.hand.source.as_deref(), Some("test_source"));
        assert_eq!(hh.table.stakes.small_blind, 50.0);
        assert_eq!(hh.table.stakes.big_blind, 100.0);
    }

    #[test]
    fn test_from_table_state_net_calculation() {
        use crate::casino::game::ForcedBets;
        use crate::casino::table::winnings::Winnings;

        let hh = HandHistory::from_table_state(
            1,
            0,
            0,
            &ForcedBets::new(50, 100),
            &[(0, "A".to_string(), 1000, None), (1, "B".to_string(), 1000, None)],
            "",
            &Winnings::default(),
            &[],
            &[(0, 900), (1, 1100)],
            "test",
            None,
        );
        let results = hh.results.unwrap();
        let a = results.iter().find(|r| r.seat == 0).unwrap();
        let b = results.iter().find(|r| r.seat == 1).unwrap();
        assert_eq!(a.net, Some(-100.0));
        assert_eq!(b.net, Some(100.0));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HandHistory::replay
    // ─────────────────────────────────────────────────────────────────────────

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn test_hand_history_replay_preflop_fold() {
        // A 2-player hand where seat 0 folds preflop after posting SB.
        let hh = HandHistory {
            pkcore_version: None,
            format_version: FORMAT_VERSION,
            hand: HandMeta {
                id: "replay-test-001".to_string(),
                game: HandVariant::Holdem,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: Some(2),
                button: Some(0),
                stakes: Stakes {
                    small_blind: 50.0,
                    big_blind: 100.0,
                    ante: None,
                    straddle: None,
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: vec![
                PlayerEntry {
                    seat: 0,
                    player_id: None,
                    name: "A".to_string(),
                    stack: 1000.0,
                    hole_cards: Some("A♠ K♠".to_string()),
                    posted: None,
                    hole_cards_visibility: None,
                    withdrawn: None,
                },
                PlayerEntry {
                    seat: 1,
                    player_id: None,
                    name: "B".to_string(),
                    stack: 1000.0,
                    hole_cards: Some("7♦ 2♣".to_string()),
                    posted: None,
                    hole_cards_visibility: None,
                    withdrawn: None,
                },
            ],
            board: None,
            streets: Some(Streets {
                preflop: Some(PreflopStreet {
                    actions: vec![
                        Action {
                            seat: 0,
                            player_id: None,
                            action: ActionType::Post,
                            amount: Some(50.0),
                            all_in: None,
                            agent: None,
                        },
                        Action {
                            seat: 1,
                            player_id: None,
                            action: ActionType::Post,
                            amount: Some(100.0),
                            all_in: None,
                            agent: None,
                        },
                        Action {
                            seat: 0,
                            player_id: None,
                            action: ActionType::Fold,
                            amount: None,
                            all_in: None,
                            agent: None,
                        },
                    ],
                    pot: Some(150.0),
                }),
                flop: None,
                turn: None,
                river: None,
            }),
            results: Some(vec![
                ResultEntry {
                    seat: 0,
                    best_hand: None,
                    hand_rank: None,
                    outcome: Outcome::Lose,
                    net: Some(-50.0),
                    pot_won: None,
                    mucked: None,
                },
                ResultEntry {
                    seat: 1,
                    best_hand: None,
                    hand_rank: None,
                    outcome: Outcome::Win,
                    net: Some(50.0),
                    pot_won: Some(150.0),
                    mucked: None,
                },
            ]),
            analysis: None,
            shuffled_deck: None,
        };

        let result = hh.replay().expect("replay should succeed");
        assert!(
            result.is_consistent,
            "chip counts should match: {:?}",
            result.final_stacks
        );
    }

    #[test]
    fn test_replay_result_struct() {
        let r = ReplayResult {
            final_stacks: vec![(0, 500), (1, 1500)],
            is_consistent: true,
        };
        assert_eq!(r.final_stacks.len(), 2);
        assert!(r.is_consistent);
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn test_hand_collection_replay_all_empty() {
        let collection = HandCollection::new();
        let results = collection.replay_all();
        assert!(results.is_empty());
    }

    /// Regression test: 3-player flop where BB checks, BTN bets, BB folds.
    ///
    /// Seats 2/4/6 with button at 1 (empty). Seat 2 folds preflop; seats 4 and 6
    /// see the flop. On the flop seat 4 checks, seat 6 opens for 250, seat 4
    /// folds.  The second action by seat 4 was previously rejected as
    /// "out of order" because `SeatsNoCell::next_to_act` skipped the checker when
    /// `everyone_has_bet` was true but the current-bet comparison used the wrong
    /// branch ordering.
    #[cfg(feature = "bot-profiles")]
    #[test]
    fn replay_flop_check_then_bet_then_fold() {
        let hh = HandHistory {
            pkcore_version: None,
            format_version: 1,
            hand: HandMeta {
                id: "regression-check-bet-fold".to_string(),
                game: HandVariant::Holdem,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: Some(3),
                button: Some(1),
                stakes: Stakes {
                    small_blind: 50.0,
                    big_blind: 100.0,
                    ante: None,
                    straddle: None,
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: vec![
                PlayerEntry {
                    seat: 2,
                    player_id: None,
                    name: "gto".to_string(),
                    stack: 3675.0,
                    hole_cards: None,
                    posted: None,
                    hole_cards_visibility: None,
                    withdrawn: None,
                },
                PlayerEntry {
                    seat: 4,
                    player_id: None,
                    name: "loose_passive".to_string(),
                    stack: 9200.0,
                    hole_cards: None,
                    posted: None,
                    hole_cards_visibility: None,
                    withdrawn: None,
                },
                PlayerEntry {
                    seat: 6,
                    player_id: None,
                    name: "maniac".to_string(),
                    stack: 61075.0,
                    hole_cards: Some("5♦ 9♥".to_string()),
                    posted: None,
                    hole_cards_visibility: None,
                    withdrawn: None,
                },
            ],
            board: Some("7♠ Q♦ 8♣".to_string()),
            streets: Some(Streets {
                preflop: Some(PreflopStreet {
                    actions: vec![
                        Action {
                            seat: 2,
                            player_id: None,
                            action: ActionType::Post,
                            amount: Some(50.0),
                            all_in: None,
                            agent: None,
                        },
                        Action {
                            seat: 4,
                            player_id: None,
                            action: ActionType::Post,
                            amount: Some(100.0),
                            all_in: None,
                            agent: None,
                        },
                        Action {
                            seat: 6,
                            player_id: None,
                            action: ActionType::Call,
                            amount: Some(100.0),
                            all_in: None,
                            agent: None,
                        },
                        Action {
                            seat: 2,
                            player_id: None,
                            action: ActionType::Fold,
                            amount: None,
                            all_in: None,
                            agent: None,
                        },
                        Action {
                            seat: 4,
                            player_id: None,
                            action: ActionType::Check,
                            amount: None,
                            all_in: None,
                            agent: None,
                        },
                    ],
                    pot: Some(250.0),
                }),
                flop: Some(FlopStreet {
                    cards: "7♠ Q♦ 8♣".to_string(),
                    actions: vec![
                        Action {
                            seat: 4,
                            player_id: None,
                            action: ActionType::Check,
                            amount: None,
                            all_in: None,
                            agent: None,
                        },
                        Action {
                            seat: 6,
                            player_id: None,
                            action: ActionType::Bet,
                            amount: Some(250.0),
                            all_in: None,
                            agent: None,
                        },
                        Action {
                            seat: 4,
                            player_id: None,
                            action: ActionType::Fold,
                            amount: None,
                            all_in: None,
                            agent: None,
                        },
                    ],
                    pot: Some(250.0),
                }),
                turn: None,
                river: None,
            }),
            results: Some(vec![
                ResultEntry {
                    seat: 2,
                    best_hand: None,
                    hand_rank: None,
                    outcome: Outcome::Lose,
                    net: Some(-50.0),
                    pot_won: None,
                    mucked: None,
                },
                ResultEntry {
                    seat: 4,
                    best_hand: None,
                    hand_rank: None,
                    outcome: Outcome::Lose,
                    net: Some(-100.0),
                    pot_won: None,
                    mucked: None,
                },
                ResultEntry {
                    seat: 6,
                    best_hand: None,
                    hand_rank: None,
                    outcome: Outcome::Win,
                    net: Some(150.0),
                    pot_won: Some(500.0),
                    mucked: None,
                },
            ]),
            analysis: None,
            shuffled_deck: None,
        };
        let result = hh.replay().expect("check-bet-fold replay should succeed");
        assert!(
            result.is_consistent,
            "chip counts should match: {:?}",
            result.final_stacks
        );
    }

    /// Same scenario as `replay_flop_check_then_bet_then_fold` but loaded via
    /// YAML round-trip to catch any serde deserialization difference.
    #[test]
    #[cfg(all(feature = "hand-histories", feature = "bot-profiles"))]
    fn replay_flop_check_then_bet_then_fold_from_yaml() {
        let yaml = r#"
pkcore_version: "0.0.43"
format_version: 1
hands:
- format_version: 1
  hand:
    id: demo-hand-011
    game: holdem
    timestamp: '1776377436'
    source: demo
  table:
    name: demo
    seats: 3
    button: 1
    stakes:
      small_blind: 50.0
      big_blind: 100.0
  players:
  - seat: 2
    name: gto
    stack: 3675.0
  - seat: 4
    name: loose_passive
    stack: 9200.0
  - seat: 6
    name: maniac
    stack: 61075.0
    hole_cards: 5♦ 9♥
  board: 7♠ Q♦ 8♣
  streets:
    preflop:
      actions:
      - seat: 2
        action: post
        amount: 50.0
      - seat: 4
        action: post
        amount: 100.0
      - seat: 6
        action: call
        amount: 100.0
      - seat: 2
        action: fold
      - seat: 4
        action: check
      pot: 250.0
    flop:
      cards: 7♠ Q♦ 8♣
      actions:
      - seat: 4
        action: check
      - seat: 6
        action: bet
        amount: 250.0
      - seat: 4
        action: fold
      pot: 250.0
  results:
  - seat: 2
    outcome: lose
    net: -50.0
  - seat: 4
    outcome: lose
    net: -100.0
  - seat: 6
    outcome: win
    net: 150.0
    pot_won: 500.0
"#;
        let collection = HandCollection::from_yaml(yaml).expect("YAML should parse");
        assert_eq!(collection.len(), 1);
        let hh = &collection.hands()[0];
        let result = hh.replay().expect("check-bet-fold replay from YAML should succeed");
        assert!(
            result.is_consistent,
            "chip counts should match: {:?}",
            result.final_stacks
        );
    }

    // ── shuffled_deck field ───────────────────────────────────────────────────

    /// shuffled_deck round-trips through YAML and is omitted when None.
    #[cfg(feature = "hand-histories")]
    #[test]
    fn test_hand_history_shuffled_deck_round_trips() {
        use crate::casino::game::ForcedBets;
        use crate::casino::table::winnings::Winnings;

        let deck_str = "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣";

        let hh = HandHistory::from_table_state(
            1,
            0,
            0,
            &ForcedBets::new(50, 100),
            &[(1, "Alice".to_string(), 1000, None)],
            "",
            &Winnings::default(),
            &[],
            &[(1, 1000)],
            "test",
            Some(deck_str.to_string()),
        );

        let yaml = hh.to_yaml().expect("should serialize");
        assert!(
            yaml.contains("shuffled_deck"),
            "YAML should include shuffled_deck field"
        );

        let restored: HandHistory = HandHistory::from_yaml(&yaml).expect("should deserialize");
        assert_eq!(restored.shuffled_deck, Some(deck_str.to_string()));

        // A HandHistory without shuffled_deck should not emit the field.
        let hh_no_deck = HandHistory::from_table_state(
            2,
            0,
            0,
            &ForcedBets::new(50, 100),
            &[(1, "Alice".to_string(), 1000, None)],
            "",
            &Winnings::default(),
            &[],
            &[(1, 1000)],
            "test",
            None,
        );
        let yaml_no_deck = hh_no_deck.to_yaml().expect("should serialize");
        assert!(
            !yaml_no_deck.contains("shuffled_deck"),
            "YAML should omit shuffled_deck when None"
        );
    }

    /// Passing a deck string to from_table_state wires it into the HandHistory.
    #[test]
    fn test_from_table_state_stores_shuffled_deck() {
        use crate::casino::game::ForcedBets;
        use crate::casino::table::winnings::Winnings;

        let deck_str = "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣";

        let hh = HandHistory::from_table_state(
            1,
            0,
            0,
            &ForcedBets::new(50, 100),
            &[(1, "Alice".to_string(), 1000, None)],
            "",
            &Winnings::default(),
            &[],
            &[(1, 1000)],
            "test",
            Some(deck_str.to_string()),
        );
        assert_eq!(hh.shuffled_deck, Some(deck_str.to_string()));

        let hh_none = HandHistory::from_table_state(
            2,
            0,
            0,
            &ForcedBets::new(50, 100),
            &[(1, "Alice".to_string(), 1000, None)],
            "",
            &Winnings::default(),
            &[],
            &[(1, 1000)],
            "test",
            None,
        );
        assert_eq!(hh_none.shuffled_deck, None);
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

    #[test]
    fn streets_from_event_log_stamps_player_id() {
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();
        let log = vec![
            TableAction::PlayerSeated(1, alice),
            TableAction::PlayerSeated(2, bob),
            TableAction::ForcedBetSmallBlind(1, 50),
            TableAction::ForcedBetBigBlind(2, 100),
            TableAction::Fold(1),
            TableAction::Check(2),
            TableAction::PotSize(150),
        ];
        let streets = Streets::from_event_log(&log).expect("streets parse");
        let preflop = streets.preflop.as_ref().expect("preflop street");
        for action in &preflop.actions {
            let expected = match action.seat {
                1 => Some(alice),
                2 => Some(bob),
                other => panic!("unexpected seat {other}"),
            };
            assert_eq!(
                action.player_id, expected,
                "action {action:?} should carry stamped player_id"
            );
        }
    }

    #[test]
    fn streets_from_event_log_no_seated_yields_none() {
        // No PlayerSeated events — legacy event-log shape from before EPIC-26.
        let log = vec![
            TableAction::ForcedBetSmallBlind(1, 50),
            TableAction::ForcedBetBigBlind(2, 100),
            TableAction::Fold(1),
        ];
        let streets = Streets::from_event_log(&log).expect("streets parse");
        let preflop = streets.preflop.as_ref().expect("preflop street");
        assert!(preflop.actions.iter().all(|a| a.player_id.is_none()));
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn action_serde_round_trip_omits_none_player_id() {
        let action = Action {
            seat: 4,
            player_id: None,
            action: ActionType::Fold,
            amount: None,
            all_in: None,
            agent: None,
        };
        let yaml = serde_yaml_bw::to_string(&action).expect("serialize");
        assert!(
            !yaml.contains("player_id"),
            "absent field should be skipped, got: {yaml}"
        );
        let parsed: Action = serde_yaml_bw::from_str(&yaml).expect("deserialize");
        assert_eq!(parsed, action);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn action_serde_round_trip_emits_some_player_id() {
        let id = Uuid::new_v4();
        let action = Action {
            seat: 4,
            player_id: Some(id),
            action: ActionType::Raise,
            amount: Some(300.0),
            all_in: None,
            agent: None,
        };
        let yaml = serde_yaml_bw::to_string(&action).expect("serialize");
        assert!(yaml.contains("player_id"));
        assert!(yaml.contains(&id.to_string()));
        let parsed: Action = serde_yaml_bw::from_str(&yaml).expect("deserialize");
        assert_eq!(parsed, action);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn player_entry_serde_round_trip_omits_none_player_id() {
        let entry = PlayerEntry {
            seat: 1,
            name: "Alice".to_string(),
            stack: 500.0,
            player_id: None,
            hole_cards: Some("A♠ K♠".to_string()),
            posted: None,
            hole_cards_visibility: None,
            withdrawn: None,
        };
        let yaml = serde_yaml_bw::to_string(&entry).expect("serialize");
        assert!(
            !yaml.contains("player_id"),
            "absent field should be skipped, got: {yaml}"
        );
        let parsed: PlayerEntry = serde_yaml_bw::from_str(&yaml).expect("deserialize");
        assert_eq!(parsed, entry);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn player_entry_serde_round_trip_emits_some_player_id() {
        let id = Uuid::new_v4();
        let entry = PlayerEntry {
            seat: 1,
            name: "Alice".to_string(),
            stack: 500.0,
            player_id: Some(id),
            hole_cards: Some("A♠ K♠".to_string()),
            posted: None,
            hole_cards_visibility: None,
            withdrawn: None,
        };
        let yaml = serde_yaml_bw::to_string(&entry).expect("serialize");
        assert!(yaml.contains("player_id"));
        assert!(yaml.contains(&id.to_string()));
        let parsed: PlayerEntry = serde_yaml_bw::from_str(&yaml).expect("deserialize");
        assert_eq!(parsed, entry);
    }

    #[test]
    fn player_entry_round_trip_with_withdrawn() {
        let entry = PlayerEntry {
            seat: 1,
            name: "Reload Rita".to_string(),
            stack: 500.0,
            player_id: None,
            hole_cards: None,
            posted: None,
            hole_cards_visibility: None,
            withdrawn: Some(2_500.0),
        };
        let yaml = serde_yaml_bw::to_string(&entry).expect("serialize");
        assert!(yaml.contains("withdrawn"));
        assert!(yaml.contains("2500"));
        let parsed: PlayerEntry = serde_yaml_bw::from_str(&yaml).expect("deserialize");
        assert_eq!(parsed, entry);
        assert_eq!(Some(2_500.0), parsed.withdrawn);
    }

    #[test]
    fn player_entry_legacy_yaml_without_withdrawn_parses() {
        // Pre-`withdrawn` YAML file omits the field entirely.
        let yaml = "seat: 1\nname: Alice\nstack: 1000.0\n";
        let parsed: PlayerEntry = serde_yaml_bw::from_str(yaml).expect("deserialize legacy");
        assert_eq!(1, parsed.seat);
        assert_eq!("Alice", parsed.name);
        assert_eq!(1_000.0, parsed.stack);
        assert_eq!(None, parsed.withdrawn);
    }

    #[test]
    fn player_entry_yaml_omits_withdrawn_when_none() {
        let entry = PlayerEntry {
            seat: 1,
            name: "Anon".to_string(),
            stack: 100.0,
            player_id: None,
            hole_cards: None,
            posted: None,
            hole_cards_visibility: None,
            withdrawn: None,
        };
        let yaml = serde_yaml_bw::to_string(&entry).expect("serialize");
        assert!(
            !yaml.contains("withdrawn"),
            "yaml unexpectedly contained `withdrawn`: {yaml}"
        );
    }

    // ── Agent fidelity (EPIC-40 Phase 4) ──────────────────────────────────

    /// A single action with no agent metadata, for terse hand construction.
    fn af_act(seat: u8, action: ActionType, amount: Option<f64>, all_in: Option<bool>) -> Action {
        Action {
            seat,
            player_id: None,
            action,
            amount,
            all_in,
            agent: None,
        }
    }

    /// An `AgentFidelity` carrying just a raw response, to tag entries by id.
    fn af(raw: &str) -> AgentFidelity {
        AgentFidelity {
            raw_response: Some(raw.to_string()),
            ..Default::default()
        }
    }

    /// Minimal hand with a preflop and optional flop street. Not necessarily
    /// replay-valid — used to exercise voluntary-action iteration/attachment.
    fn af_hand(preflop: Vec<Action>, flop: Option<Vec<Action>>) -> HandHistory {
        let mut hh = make_minimal_hand("fidelity-test", HandVariant::Holdem);
        hh.streets = Some(Streets {
            preflop: Some(PreflopStreet {
                actions: preflop,
                pot: None,
            }),
            flop: flop.map(|actions| FlopStreet {
                cards: "7♠ Q♦ 8♣".to_string(),
                actions,
                pot: None,
            }),
            turn: None,
            river: None,
        });
        hh
    }

    /// Canonical two-street hand. Voluntary seats in order: `[3, 1, 2, 2, 3, 2]`.
    fn af_two_street() -> HandHistory {
        af_hand(
            vec![
                af_act(1, ActionType::Post, Some(50.0), None),
                af_act(2, ActionType::Post, Some(100.0), None),
                af_act(3, ActionType::Call, Some(100.0), None),
                af_act(1, ActionType::Fold, None, None),
                af_act(2, ActionType::Check, None, None),
            ],
            Some(vec![
                af_act(2, ActionType::Check, None, None),
                af_act(3, ActionType::Bet, Some(250.0), None),
                af_act(2, ActionType::Fold, None, None),
            ]),
        )
    }

    #[test]
    fn voluntary_actions_mut_skips_posts_in_canonical_order() {
        let mut hh = af_two_street();
        let seats: Vec<u8> = hh.voluntary_actions_mut().iter().map(|a| a.seat).collect();
        assert_eq!(seats, vec![3, 1, 2, 2, 3, 2]);
        assert!(hh.voluntary_actions_mut().iter().all(|a| a.action != ActionType::Post));
    }

    #[test]
    fn attach_agent_fidelity_aligned_annotates_every_voluntary_action() {
        let mut hh = af_two_street();
        let entries = vec![
            (3, af("a")),
            (1, af("b")),
            (2, af("c")),
            (2, af("d")),
            (3, af("e")),
            (2, af("f")),
        ];
        assert_eq!(hh.attach_agent_fidelity(&entries), 6);
        {
            let v = hh.voluntary_actions_mut();
            assert_eq!(v[0].agent.as_ref().and_then(|a| a.raw_response.as_deref()), Some("a"));
            assert_eq!(v[5].agent.as_ref().and_then(|a| a.raw_response.as_deref()), Some("f"));
        }
        // Forced Post actions are never annotated.
        let pre = hh.streets.as_ref().unwrap().preflop.as_ref().unwrap();
        assert!(pre.actions[0].agent.is_none());
    }

    /// Same-length *substitution*: one entry has a corrupted seat but the list
    /// length is unchanged, so the per-slot guard drops only that slot and every
    /// later pair stays aligned. This is NOT drift — see the insertion/deletion
    /// tests below for the cascade behavior under a length change.
    #[test]
    fn attach_agent_fidelity_same_length_substitution_skips_only_that_slot() {
        let mut hh = af_two_street(); // voluntary seats [3, 1, 2, 2, 3, 2]
        let entries = vec![
            (3, af("a")),
            (9, af("b")), // wrong seat for the second voluntary action (seat 1)
            (2, af("c")),
            (2, af("d")),
            (3, af("e")),
            (2, af("f")),
        ];
        assert_eq!(hh.attach_agent_fidelity(&entries), 5);
        let v = hh.voluntary_actions_mut();
        assert!(v[1].agent.is_none(), "seat-1 action left None after mismatch");
        assert_eq!(v[2].agent.as_ref().and_then(|a| a.raw_response.as_deref()), Some("c"));
    }

    /// Characterization test for a **missing entry** (the recorder dropped one).
    /// Documents the real, degraded contract: positional pairing does not
    /// resynchronize, so the deletion offsets every later pair — silently
    /// misattributing one same-seat slot and dropping the rest. Pins the
    /// limitation called out in the docs so any future change is deliberate.
    #[test]
    fn attach_agent_fidelity_missing_entry_cascades_after_the_gap() {
        let mut hh = af_two_street(); // voluntary seats [3, 1, 2, 2, 3, 2]
        // Aligned would be [a@3, b@1, c@2, d@2, e@3, f@2]; the seat-1 entry `b`
        // was dropped, leaving five entries.
        let entries = vec![(3, af("a")), (2, af("c")), (2, af("d")), (3, af("e")), (2, af("f"))];
        let annotated = hh.attach_agent_fidelity(&entries);
        assert_eq!(annotated, 2);
        // Return value is below entries.len() (5): the caller can detect drift.
        assert_ne!(annotated, entries.len());

        let v = hh.voluntary_actions_mut();
        let raw = |a: &Action| a.agent.as_ref().and_then(|f| f.raw_response.clone());
        assert_eq!(raw(v[0]).as_deref(), Some("a")); // still correct
        assert!(v[1].agent.is_none()); // its entry was the dropped one
        // MISATTRIBUTION: slot 2 should be "c" but the offset hands it "d".
        assert_eq!(raw(v[2]).as_deref(), Some("d"));
        // Everything after the gap is dropped, even though entries remained.
        assert!(v[3].agent.is_none());
        assert!(v[4].agent.is_none());
        assert!(v[5].agent.is_none());
    }

    /// Characterization test for an **extra applied action** with no entry — the
    /// drift the spec explicitly cites (a server-rejection retry that produced a
    /// duplicate applied action). Pins the cascade: after the orphan action,
    /// pairing is offset and most later annotations are lost/misattributed.
    #[test]
    fn attach_agent_fidelity_extra_action_cascades_after_the_orphan() {
        // Voluntary seats [3, 1, 1, 2, 2, 3, 2] — the second seat-1 action is the
        // duplicate retry. (Legality is irrelevant; this hand is never replayed.)
        let mut hh = af_hand(
            vec![
                af_act(1, ActionType::Post, Some(50.0), None),
                af_act(2, ActionType::Post, Some(100.0), None),
                af_act(3, ActionType::Call, Some(100.0), None),
                af_act(1, ActionType::Raise, Some(300.0), None),
                af_act(1, ActionType::Raise, Some(300.0), None), // duplicate retry
                af_act(2, ActionType::Call, Some(300.0), None),
                af_act(2, ActionType::Bet, Some(150.0), None),
                af_act(3, ActionType::Raise, Some(600.0), None),
                af_act(2, ActionType::Call, Some(600.0), None),
            ],
            None,
        );
        // Clean six entries, one per intended decision (no entry for the retry).
        let entries = vec![
            (3, af("a")),
            (1, af("b")),
            (2, af("c")),
            (2, af("d")),
            (3, af("e")),
            (2, af("f")),
        ];
        let annotated = hh.attach_agent_fidelity(&entries);
        assert_eq!(annotated, 3);
        assert_ne!(annotated, entries.len()); // detectable drift

        let v = hh.voluntary_actions_mut();
        let raw = |a: &Action| a.agent.as_ref().and_then(|f| f.raw_response.clone());
        assert_eq!(raw(v[0]).as_deref(), Some("a")); // correct
        assert_eq!(raw(v[1]).as_deref(), Some("b")); // correct
        assert!(v[2].agent.is_none()); // the orphan retry action
        // MISATTRIBUTION: slot 3 should be "c" but the offset hands it "d".
        assert_eq!(raw(v[3]).as_deref(), Some("d"));
        // The orphan cascades: the tail is dropped despite remaining entries.
        assert!(v[4].agent.is_none());
        assert!(v[5].agent.is_none());
        assert!(v[6].agent.is_none());
    }

    #[test]
    fn attach_agent_fidelity_all_fold_hand() {
        let mut hh = af_hand(
            vec![
                af_act(1, ActionType::Post, Some(50.0), None),
                af_act(2, ActionType::Post, Some(100.0), None),
                af_act(3, ActionType::Fold, None, None),
                af_act(1, ActionType::Fold, None, None),
            ],
            None,
        );
        assert_eq!(hh.attach_agent_fidelity(&[(3, af("x")), (1, af("y"))]), 2);
    }

    #[test]
    fn attach_agent_fidelity_all_in_action_is_voluntary() {
        let mut hh = af_hand(
            vec![
                af_act(1, ActionType::Post, Some(50.0), None),
                af_act(2, ActionType::Post, Some(100.0), None),
                af_act(3, ActionType::AllIn, Some(1000.0), Some(true)),
                af_act(1, ActionType::Fold, None, None),
            ],
            None,
        );
        assert_eq!(hh.attach_agent_fidelity(&[(3, af("shove")), (1, af("fold"))]), 2);
        let v = hh.voluntary_actions_mut();
        assert_eq!(v[0].action, ActionType::AllIn);
        assert!(v[0].agent.is_some());
    }

    #[test]
    fn attach_agent_fidelity_multi_raise_street() {
        let mut hh = af_hand(
            vec![
                af_act(1, ActionType::Post, Some(50.0), None),
                af_act(2, ActionType::Post, Some(100.0), None),
                af_act(3, ActionType::Raise, Some(300.0), None),
                af_act(1, ActionType::Raise, Some(900.0), None),
                af_act(2, ActionType::Raise, Some(2700.0), None),
                af_act(3, ActionType::Call, Some(2700.0), None),
                af_act(1, ActionType::Fold, None, None),
            ],
            None,
        );
        let entries = vec![
            (3, af("r1")),
            (1, af("r2")),
            (2, af("r3")),
            (3, af("call")),
            (1, af("fold")),
        ];
        assert_eq!(hh.attach_agent_fidelity(&entries), 5);
    }

    #[test]
    fn attach_agent_fidelity_dead_button_hand() {
        // Button on empty seat 1; seats 2/4/6 in play (mirrors the replay regression).
        let mut hh = af_hand(
            vec![
                af_act(2, ActionType::Post, Some(50.0), None),
                af_act(4, ActionType::Post, Some(100.0), None),
                af_act(6, ActionType::Call, Some(100.0), None),
                af_act(2, ActionType::Fold, None, None),
                af_act(4, ActionType::Check, None, None),
            ],
            Some(vec![
                af_act(4, ActionType::Check, None, None),
                af_act(6, ActionType::Bet, Some(250.0), None),
                af_act(4, ActionType::Fold, None, None),
            ]),
        );
        let entries = vec![
            (6, af("c")),
            (2, af("f")),
            (4, af("k")),
            (4, af("k2")),
            (6, af("b")),
            (4, af("f2")),
        ];
        assert_eq!(hh.attach_agent_fidelity(&entries), 6);
    }

    /// A fully-populated `AgentFidelity` for round-trip fixtures.
    fn af_full() -> AgentFidelity {
        AgentFidelity {
            raw_response: Some("raise to 250".to_string()),
            was_coerced: Some(true),
            intended_action: Some(ActionType::Raise),
            intended_amount: Some(250.0),
            input_tokens: Some(1200),
            output_tokens: Some(8),
            model: Some("claude-test".to_string()),
        }
    }

    /// JSON-side back-compat, feature-independent (`serde_json` is always built):
    /// metadata survives a round trip, absent metadata emits no key, and a
    /// legacy action lacking the key deserializes to `agent: None`.
    #[test]
    fn agent_fidelity_json_round_trips_and_omits_key_when_absent() {
        // Absent ⇒ no `agent` key.
        let plain = af_two_street();
        let plain_json = serde_json::to_string(&plain).expect("to_json");
        assert!(
            !plain_json.contains("\"agent\""),
            "json emitted agent key: {plain_json}"
        );

        // Legacy action (pre-EPIC-40, no `agent` key) ⇒ `None`.
        let legacy: Action = serde_json::from_str(r#"{"seat":4,"action":"raise","amount":250.0}"#).expect("legacy");
        assert_eq!(legacy.seat, 4);
        assert_eq!(legacy.agent, None);

        // Present ⇒ survives the round trip.
        let mut hh = af_two_street();
        let full = af_full();
        assert_eq!(hh.attach_agent_fidelity(&[(3, full.clone())]), 1);
        let json = serde_json::to_string(&hh).expect("to_json");
        assert_eq!(serde_json::from_str::<HandHistory>(&json).expect("from_json"), hh);
    }

    /// YAML-side equivalents, gated on the `hand-histories` feature that supplies
    /// `to_yaml`/`from_yaml` (and the optional `serde_yaml_bw` dependency).
    #[cfg(feature = "hand-histories")]
    #[test]
    fn agent_fidelity_yaml_round_trips_and_omits_key_when_absent() {
        // Absent ⇒ no `agent:` key.
        let plain_yaml = af_two_street().to_yaml().expect("to_yaml");
        assert!(!plain_yaml.contains("agent:"), "yaml emitted agent key: {plain_yaml}");

        // Present ⇒ survives the round trip, metadata intact.
        let mut hh = af_two_street();
        let full = af_full();
        assert_eq!(hh.attach_agent_fidelity(&[(3, full.clone())]), 1);
        let yaml = hh.to_yaml().expect("to_yaml");
        assert_eq!(HandHistory::from_yaml(&yaml).expect("from_yaml"), hh);

        let call = hh
            .streets
            .as_ref()
            .and_then(|s| s.preflop.as_ref())
            .and_then(|p| p.actions.iter().find(|a| a.seat == 3))
            .expect("seat-3 preflop action");
        assert_eq!(call.agent.as_ref(), Some(&full));
    }

    /// 2-player hand where seat 0 folds preflop after posting SB — a valid,
    /// replay-consistent hand used to prove replay ignores agent metadata.
    ///
    /// Gated with its sole consumer ([`replay_ignores_agent_fidelity`]) so it is
    /// not dead code when `bot-profiles` (and thus `replay`) is disabled.
    #[cfg(feature = "bot-profiles")]
    fn af_replayable_preflop_fold() -> HandHistory {
        HandHistory {
            pkcore_version: None,
            format_version: FORMAT_VERSION,
            hand: HandMeta {
                id: "fidelity-replay-001".to_string(),
                game: HandVariant::Holdem,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: Some(2),
                button: Some(0),
                stakes: Stakes {
                    small_blind: 50.0,
                    big_blind: 100.0,
                    ante: None,
                    straddle: None,
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: vec![
                PlayerEntry {
                    seat: 0,
                    player_id: None,
                    name: "A".to_string(),
                    stack: 1000.0,
                    hole_cards: Some("A♠ K♠".to_string()),
                    posted: None,
                    hole_cards_visibility: None,
                    withdrawn: None,
                },
                PlayerEntry {
                    seat: 1,
                    player_id: None,
                    name: "B".to_string(),
                    stack: 1000.0,
                    hole_cards: Some("7♦ 2♣".to_string()),
                    posted: None,
                    hole_cards_visibility: None,
                    withdrawn: None,
                },
            ],
            board: None,
            streets: Some(Streets {
                preflop: Some(PreflopStreet {
                    actions: vec![
                        af_act(0, ActionType::Post, Some(50.0), None),
                        af_act(1, ActionType::Post, Some(100.0), None),
                        af_act(0, ActionType::Fold, None, None),
                    ],
                    pot: Some(150.0),
                }),
                flop: None,
                turn: None,
                river: None,
            }),
            results: None,
            analysis: None,
            shuffled_deck: None,
        }
    }

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn replay_ignores_agent_fidelity() {
        let mut hh = af_replayable_preflop_fold();
        let before = hh.replay().expect("replay before attach");
        // The only voluntary action is seat 0's fold.
        assert_eq!(hh.attach_agent_fidelity(&[(0, af("i fold"))]), 1);
        let after = hh.replay().expect("replay after attach");
        assert_eq!(before.is_consistent, after.is_consistent);
        assert_eq!(before.final_stacks, after.final_stacks);
    }
}
