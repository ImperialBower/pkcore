//! A read-only snapshot of the poker table from one player's perspective.
//!
//! [`TableSnapshot`] is the input to
//! [`BotDecider::decide`](crate::bot::decider::BotDecider::decide).  It
//! captures everything a player can legitimately observe at their decision
//! point: their own hole cards, the community board, pot size, stack sizes,
//! and the current betting context.  Opponents' hole cards are **never**
//! included.
//!
//! When the `player-stats` feature is enabled, snapshots may also carry an
//! optional borrow on a [`StatsRegistry`] via
//! [`TableSnapshot::from_table_with_stats`].  See EPIC-26 for design.

use crate::Pile;
use crate::cards::Cards;
use crate::casino::table::event::TableAction;
use crate::casino::table::position::Position;
use crate::casino::table_no_cell::TableNoCell;
use crate::games::GamePhase;
use crate::games::betting_structure::{BetTier, BettingStructure};
use uuid::Uuid;

#[cfg(feature = "player-stats")]
use crate::analysis::player_stats::StatsRegistry;

// ── SeatInfo ──────────────────────────────────────────────────────────────────

/// Per-seat chip and activity information visible to all players.
///
/// Every occupied seat appears in [`TableSnapshot::stacks`].
///
/// # Examples
///
/// ```
/// use pkcore::bot::table_snapshot::SeatInfo;
/// use uuid::Uuid;
///
/// let info = SeatInfo {
///     id: Uuid::default(),
///     seat: 2,
///     name: "Carol".to_string(),
///     chips: 800,
///     bet: 100,
///     is_active: true,
/// };
/// assert_eq!(info.seat, 2);
/// assert_eq!(info.chips, 800);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeatInfo {
    /// Unique player identifier, used to look up entries in [`StatsRegistry`].
    pub id: Uuid,
    /// Zero-based seat index.
    pub seat: u8,
    /// Player handle / display name.
    pub name: String,
    /// Chips remaining in the player's stack (not yet committed this round).
    pub chips: usize,
    /// Chips committed to the current betting round.
    pub bet: usize,
    /// `true` when this player is still active in the current hand.
    pub is_active: bool,
}

// ── TableSnapshot ─────────────────────────────────────────────────────────────

/// A point-in-time snapshot of the table from one player's perspective.
///
/// Constructed by [`TableSnapshot::from_table`] and passed to
/// [`BotDecider::decide`](crate::bot::decider::BotDecider::decide).  Most
/// fields are owned values; the snapshot's lifetime parameter `'a` exists
/// solely to carry the optional [`Self::opponent_stats`] borrow added in
/// EPIC-26 Phase 3.  Snapshots constructed without stats can use any
/// lifetime (e.g. `TableSnapshot<'static>`).
///
/// # Visibility rules
///
/// - **`hole_cards`** — only this player's own cards are shown.
/// - **`board`** — full community cards.
/// - **`stacks`** — chip counts and bets for every seated player (no hole
///   cards for opponents).
/// - **`opponent_stats`** *(feature `player-stats`)* — optional borrow on a
///   [`StatsRegistry`] for exploitative deciders.  `None` for snapshots
///   built via [`Self::from_table`].
///
/// # Examples
///
/// ```
/// use pkcore::bot::table_snapshot::TableSnapshot;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
///
/// let seats = SeatsNoCell::new(vec![
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 1_000)),
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 1_000)),
/// ]);
/// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let snap = TableSnapshot::from_table(&table, 0);
/// assert_eq!(0, snap.seat);
/// assert_eq!(1_000, snap.my_chips);
/// assert_eq!(2, snap.stacks.len());
/// assert_eq!(100, snap.big_blind);
/// ```
#[derive(Clone, Debug)]
pub struct TableSnapshot<'a> {
    /// The seat index this snapshot was built for.
    pub seat: u8,
    /// Current game phase (preflop, flop, turn, river, …).
    pub phase: GamePhase,
    /// Community cards dealt so far.  Empty before the flop.
    pub board: Cards,
    /// This player's hole cards.  Empty if cards have not been dealt yet.
    pub hole_cards: Cards,
    /// Total pot — main pot plus all chips committed this street (swept + live bets).
    pub pot: usize,
    /// Chips needed for this player to call the current bet.
    /// `0` means no bet is outstanding and the player may check.
    pub to_call: usize,
    /// Current highest bet on this street.
    pub current_bet: usize,
    /// Minimum legal raise increment (big blind, or the last raise size).
    pub min_raise: usize,
    /// This player's remaining chip stack (chips not yet committed this round).
    pub my_chips: usize,
    /// All occupied seats and their chip / bet state.  Ordered by seat index.
    pub stacks: Vec<SeatInfo>,
    /// Big blind amount — the baseline bet unit for sizing decisions.
    pub big_blind: usize,
    /// Betting structure for the table (EPIC-30 Phase 5). Deciders that
    /// need to size raises differently for Fixed-Limit / Pot-Limit
    /// variants consult this field instead of computing pot fractions.
    pub betting_structure: BettingStructure,
    /// Bet tier for the *current* street (EPIC-30 Phase 5). Fixed-Limit
    /// variants need this to choose between the small-bet and big-bet
    /// increment; No-Limit / Pot-Limit ignore it. Sourced from
    /// [`TableNoCell::current_bet_tier`].
    pub bet_tier: BetTier,
    /// `true` when this player has already checked during the current betting
    /// street.  Used by [`crate::bot::decider::RuleBasedDecider`] to detect
    /// check-raise opportunities (checked, faced a bet, now can raise).
    pub checked_this_street: bool,
    /// Logical position of the dealer button within the sorted list of
    /// occupied seats (`0` = earliest occupied seat is the button).
    /// `None` when the table has not started a hand yet.
    pub dealer_button: Option<u8>,
    /// Number of occupied (non-empty) seats at this table.
    pub seat_count: u8,
    /// Logical position of this player within the sorted list of occupied
    /// seats (`0` = earliest occupied seat). Used by [`Self::position`].
    /// `None` when the seat is not in the occupied list (should not occur
    /// during normal play).
    pub logical_seat: Option<u8>,
    /// Optional borrow on the per-player aggregator. `None` for snapshots
    /// built via [`Self::from_table`]; `Some(_)` when built via
    /// [`Self::from_table_with_stats`]. Reserved for future exploitative
    /// deciders — the shipped `RuleBasedDecider` and `JokerDecider` ignore
    /// this field and produce identical decisions whether it is set or not.
    #[cfg(feature = "player-stats")]
    pub opponent_stats: Option<&'a StatsRegistry>,
    /// Holds the lifetime parameter when the `player-stats` feature is off.
    /// Zero-sized; not exposed publicly.
    #[cfg(not(feature = "player-stats"))]
    _stats_lifetime: std::marker::PhantomData<&'a ()>,
}

impl<'a> TableSnapshot<'a> {
    /// Constructs a `TableSnapshot` from a live `TableNoCell` from `seat`'s
    /// perspective.
    ///
    /// The `board` and `hole_cards` fields are cloned from the table; the
    /// returned snapshot does not borrow from the table itself.  Built with
    /// `opponent_stats: None` — use [`Self::from_table_with_stats`] to
    /// attach an opponent stats registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::table_snapshot::TableSnapshot;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("X".to_string(), 500)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("Y".to_string(), 500)),
    /// ]);
    /// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(5, 10));
    /// let snap = TableSnapshot::from_table(&table, 1);
    /// assert_eq!(1, snap.seat);
    /// assert_eq!(10, snap.big_blind);
    /// assert_eq!(500, snap.my_chips);
    /// ```
    #[must_use]
    pub fn from_table(table: &TableNoCell, seat: u8) -> Self {
        let hole_cards = table
            .seats
            .get_seat(seat)
            .filter(|s| s.cards.has_cards())
            .map(|s| s.cards.cards())
            .unwrap_or_default();

        let committed: usize = table.seats.0.iter().map(|s| s.player.bet).sum();
        let pot = table.pot + committed;

        let my_chips = table.seats.get_seat(seat).map_or(0, |s| s.player.chips);

        let stacks = table
            .seats
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if s.is_empty() {
                    None
                } else {
                    u8::try_from(i).ok().map(|idx| SeatInfo {
                        id: s.player.id,
                        seat: idx,
                        name: s.player.handle.clone(),
                        chips: s.player.chips,
                        bet: s.player.bet,
                        is_active: s.is_active(),
                    })
                }
            })
            .collect();

        // Find the start of the current betting street in the event log.
        // ForcedBetBigBlind is the last event before the first preflop action;
        // DealtFlop/Turn/River mark postflop street starts.
        // Using rposition handles multi-hand simulations where the log is cumulative.
        let street_start = table
            .event_log
            .iter()
            .rposition(|a| {
                matches!(
                    a,
                    TableAction::ForcedBetBigBlind(_, _)
                        | TableAction::DealtFlop(_)
                        | TableAction::DealtTurn(_)
                        | TableAction::DealtRiver(_)
                )
            })
            .map_or(0, |i| i + 1);
        let checked_this_street = table.event_log[street_start..]
            .iter()
            .any(|a| matches!(a, TableAction::Check(s) if *s == seat));

        // Build a sorted list of occupied physical seat indices so we can
        // translate the physical button index and our own seat to logical
        // positions (0-based within occupied seats). This avoids unsigned
        // underflow when the physical button index exceeds the occupied count
        // after player eliminations create gaps in the seat numbering.
        let occupied: Vec<u8> = table
            .seats
            .0
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_empty())
            .filter_map(|(i, _)| u8::try_from(i).ok())
            .collect();
        let seat_count = u8::try_from(occupied.len()).unwrap_or(0);
        let dealer_button = occupied
            .iter()
            .position(|&s| s == table.button)
            .and_then(|p| u8::try_from(p).ok());
        let logical_seat = occupied
            .iter()
            .position(|&s| s == seat)
            .and_then(|p| u8::try_from(p).ok());

        TableSnapshot {
            seat,
            phase: table.phase,
            board: table.board.clone(),
            hole_cards,
            pot,
            to_call: table.to_call(seat),
            current_bet: table.bet,
            min_raise: table.min_raise(),
            my_chips,
            stacks,
            big_blind: table.forced.big_blind,
            betting_structure: table.betting,
            bet_tier: table.current_bet_tier(),
            checked_this_street,
            dealer_button,
            seat_count,
            logical_seat,
            #[cfg(feature = "player-stats")]
            opponent_stats: None,
            #[cfg(not(feature = "player-stats"))]
            _stats_lifetime: std::marker::PhantomData,
        }
    }

    /// Constructs a snapshot with an attached [`StatsRegistry`] borrow.
    ///
    /// Equivalent to [`Self::from_table`] followed by setting
    /// `opponent_stats = Some(registry)`.  The shipped deciders ignore this
    /// field — see EPIC-26 Phase 3 for the non-behavior-changing contract.
    ///
    /// Only available when the `player-stats` feature is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "player-stats")] {
    /// use pkcore::analysis::player_stats::StatsRegistry;
    /// use pkcore::bot::table_snapshot::TableSnapshot;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("X".to_string(), 500)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("Y".to_string(), 500)),
    /// ]);
    /// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(5, 10));
    /// let registry = StatsRegistry::new();
    /// let snap = TableSnapshot::from_table_with_stats(&table, 0, &registry);
    /// assert!(snap.opponent_stats.is_some());
    /// # }
    /// ```
    #[cfg(feature = "player-stats")]
    #[must_use]
    pub fn from_table_with_stats(table: &TableNoCell, seat: u8, registry: &'a StatsRegistry) -> Self {
        let mut snap = Self::from_table(table, seat);
        snap.opponent_stats = Some(registry);
        snap
    }

    /// Returns this player's table position relative to the dealer button.
    ///
    /// Returns `None` when the dealer button is unset or the table size is not
    /// one of the supported formats (2, 3, 4, 5, 6, 9 seats).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::table_snapshot::TableSnapshot;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::position::Position;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// // Button starts at seat 0 → seat 0 is BTN, seat 1 is BB.
    /// assert_eq!(Some(Position::BTN), TableSnapshot::from_table(&table, 0).position());
    /// assert_eq!(Some(Position::BB),  TableSnapshot::from_table(&table, 1).position());
    /// ```
    #[must_use]
    pub fn position(&self) -> Option<Position> {
        let btn = self.dealer_button?;
        let logical = self.logical_seat?;
        Position::from_seat(logical, btn, self.seat_count)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__table_snapshot_tests {
    use super::*;
    use crate::casino::game::ForcedBets;
    use crate::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    use uuid::Uuid;

    fn two_player_table() -> TableNoCell {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 1_000)),
        ]);
        TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100))
    }

    #[test]
    fn from_table_seat_zero() {
        let table = two_player_table();
        let snap = TableSnapshot::from_table(&table, 0);
        assert_eq!(0, snap.seat);
        assert_eq!(1_000, snap.my_chips);
        assert_eq!(100, snap.big_blind);
        assert_eq!(2, snap.stacks.len());
    }

    #[test]
    fn from_table_seat_one() {
        let table = two_player_table();
        let snap = TableSnapshot::from_table(&table, 1);
        assert_eq!(1, snap.seat);
        assert_eq!(1_000, snap.my_chips);
    }

    #[test]
    fn hole_cards_empty_before_deal() {
        let table = two_player_table();
        let snap = TableSnapshot::from_table(&table, 0);
        // No cards dealt yet
        assert!(snap.hole_cards.is_empty());
        assert!(snap.board.is_empty());
    }

    #[test]
    fn pot_includes_committed() {
        let table = two_player_table();
        let snap = TableSnapshot::from_table(&table, 0);
        // Before any bets, pot is 0 and no committed chips
        assert_eq!(0, snap.pot);
    }

    #[test]
    fn stacks_all_seats() {
        let table = two_player_table();
        let snap = TableSnapshot::from_table(&table, 0);
        let seats: Vec<u8> = snap.stacks.iter().map(|s| s.seat).collect();
        assert!(seats.contains(&0));
        assert!(seats.contains(&1));
    }

    #[test]
    fn seat_info_fields() {
        let info = SeatInfo {
            id: Uuid::default(),
            seat: 3,
            name: "Dave".to_string(),
            chips: 2_500,
            bet: 0,
            is_active: true,
        };
        assert_eq!(3, info.seat);
        assert_eq!("Dave", info.name);
        assert_eq!(2_500, info.chips);
        assert!(info.is_active);
    }

    #[test]
    fn min_raise_is_big_blind() {
        let table = two_player_table();
        let snap = TableSnapshot::from_table(&table, 0);
        // Before any raises, min_raise == big_blind
        assert_eq!(snap.big_blind, snap.min_raise);
    }

    #[test]
    fn checked_this_street_false_on_fresh_table() {
        // No actions taken yet — no one has checked.
        let table = two_player_table();
        let snap = TableSnapshot::from_table(&table, 0);
        assert!(!snap.checked_this_street);
    }

    #[test]
    fn checked_this_street_true_after_flop_check() {
        use crate::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell};
        use crate::prelude::PlayerState;

        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        // Complete preflop: SB calls, BB is set to Check state manually
        // (acts as "BB option closes action" without going through act_check which
        // requires it to be BB's turn in the exact sequence).
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb).unwrap();
        table.seats.get_seat_mut(bb).unwrap().player.state = PlayerState::Check;
        table.bring_it_in().unwrap();

        // Deal flop — this logs DealtFlop, which becomes the new street boundary.
        table.deal_flop().unwrap();
        table.seats.reset_state_in_hand();

        // First player on the flop checks.
        let first = table.next_to_act();
        table.act_check(first).unwrap();

        // Snapshot for the player who just checked → checked_this_street should be true.
        let snap = TableSnapshot::from_table(&table, first);
        assert!(snap.checked_this_street, "seat {first} checked on flop, expected true");
    }

    #[test]
    fn checked_this_street_false_for_other_seat_after_flop_check() {
        use crate::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell};
        use crate::prelude::PlayerState;

        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb).unwrap();
        table.seats.get_seat_mut(bb).unwrap().player.state = PlayerState::Check;
        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.seats.reset_state_in_hand();

        // Only the first player checks.
        let first = table.next_to_act();
        table.act_check(first).unwrap();

        // Snapshot for the OTHER seat — it has not checked this street.
        let other = if first == 0 { 1 } else { 0 };
        let snap = TableSnapshot::from_table(&table, other);
        assert!(
            !snap.checked_this_street,
            "seat {other} has not checked, expected false"
        );
    }

    #[test]
    fn snapshot_position_btn_seat_zero() {
        // 2-player table, button starts at seat 0 → seat 0 is BTN.
        let table = two_player_table();
        let snap = TableSnapshot::from_table(&table, 0);
        assert_eq!(Some(Position::BTN), snap.position());
    }

    #[test]
    fn snapshot_position_bb_seat_one() {
        // 2-player table, button at seat 0 → seat 1 is BB.
        let table = two_player_table();
        let snap = TableSnapshot::from_table(&table, 1);
        assert_eq!(Some(Position::BB), snap.position());
    }

    #[test]
    fn snapshot_position_none_when_dealer_button_unset() {
        let table = two_player_table();
        let mut snap = TableSnapshot::from_table(&table, 0);
        snap.dealer_button = None;
        assert_eq!(None, snap.position());
    }

    #[cfg(feature = "player-stats")]
    #[test]
    fn from_table_sets_opponent_stats_none() {
        let table = two_player_table();
        let snap = TableSnapshot::from_table(&table, 0);
        assert!(snap.opponent_stats.is_none());
    }

    #[cfg(feature = "player-stats")]
    #[test]
    fn from_table_with_stats_attaches_registry() {
        use crate::analysis::player_stats::StatsRegistry;
        let table = two_player_table();
        let registry = StatsRegistry::new();
        let snap = TableSnapshot::from_table_with_stats(&table, 0, &registry);
        assert!(snap.opponent_stats.is_some());
        // Same scalar fields as from_table — only opponent_stats changes.
        let plain = TableSnapshot::from_table(&table, 0);
        assert_eq!(plain.seat, snap.seat);
        assert_eq!(plain.my_chips, snap.my_chips);
        assert_eq!(plain.big_blind, snap.big_blind);
    }

    #[cfg(feature = "player-stats")]
    #[test]
    fn from_table_with_stats_borrows_existing_registry() {
        // Tripwire: the registry borrow shows the *same* state the caller
        // populated, not a clone — so future ingestions are visible to any
        // decider holding the snapshot in the same scope.
        use crate::analysis::player_stats::StatsRegistry;
        let table = two_player_table();
        let registry = StatsRegistry::new();
        let snap = TableSnapshot::from_table_with_stats(&table, 0, &registry);
        let borrowed = snap.opponent_stats.expect("just attached");
        assert_eq!(0, borrowed.len());
        assert!(borrowed.is_empty());
    }

    #[test]
    fn checked_this_street_resets_across_streets() {
        use crate::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell};
        use crate::prelude::PlayerState;

        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        // Complete preflop (BB checks option via state mutation).
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb).unwrap();
        table.seats.get_seat_mut(bb).unwrap().player.state = PlayerState::Check;
        table.bring_it_in().unwrap();

        // Flop: both players check.
        table.deal_flop().unwrap();
        table.seats.reset_state_in_hand();
        let first = table.next_to_act();
        table.act_check(first).unwrap();
        let second = table.next_to_act();
        table.act_check(second).unwrap();
        table.bring_it_in().unwrap();

        // Turn dealt — new street boundary; neither player has checked the turn yet.
        table.deal_turn().unwrap();
        table.seats.reset_state_in_hand();

        let snap_first = TableSnapshot::from_table(&table, first);
        assert!(
            !snap_first.checked_this_street,
            "new street: checked_this_street should reset to false"
        );
    }
}
