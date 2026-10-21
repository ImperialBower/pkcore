//! A step-by-step replay of a [`HandHistory`]: the [`Replay`] cursor.
//!
//! [`HandHistory::replay`] plays a whole hand and reports the result. A
//! [`Replay`] plays it as a list of [`ReplayEvent`]s and stands on any one of
//! them, so a viewer can step through a hand, scrub back and forth, and show the
//! [`Table`] as it stood after each event: the forced bets, every recorded
//! action, each street's deal, the showdown, and each award.
//!
//! The events come from the same walk [`HandHistory::replay_with`] makes, so a
//! cursor exists exactly when a replay succeeds, and fails the same way
//! (`PKError::ReplayAmountMismatch` included) when it does not.
//!
//! See [EPIC-93](https://github.com/ImperialBower/pkcore/blob/main/docs/epics/EPIC-93_Faithful_Replay.md).

use crate::PKError;
use crate::casino::action::{PlayerAction, TableAction};
use crate::casino::table::Table;
use crate::games::GamePhase;
use crate::hand_history::{HandHistory, ReplayStep, ReplayStreet};

/// One thing that happens to the table in a replayed hand.
///
/// A [`Replay`] is a list of these, always in this shape: one `Forced`, then
/// the recorded actions with a `Deal` before each street after the preflop,
/// then `Showdown` when two or more seats are still in the hand after the
/// river, then one `Award` for each share of each pot.
///
/// # Domain
///
/// - **Role:** domain event
/// - **Invariants:**
///   - `Deal(ReplayStreet::Preflop)` is never emitted: the hole cards are part
///     of `Forced`'s starting table
///   - `Award`s follow the engine's showdown: a split pot gives one award per
///     winner, a side-pot hand one per pot and winner, and the shares of one
///     hand sum to every chip put in
///
/// # Examples
///
/// ```
/// use pkcore::casino::action::PlayerAction;
/// use pkcore::hand_history::{ReplayEvent, ReplayStreet};
///
/// let event = ReplayEvent::Action {
///     seat: 0,
///     action: PlayerAction::Raise(300),
///     index: 0,
///     street: ReplayStreet::Preflop,
/// };
/// assert_eq!("preflop[0]: seat 0 Raise(300)", event.to_string());
/// assert_eq!("seat 1 wins 400", ReplayEvent::Award { seat: 1, amount: 400 }.to_string());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum ReplayEvent {
    /// The starting table: seats placed, blinds, antes and any bring-in
    /// posted, hole cards dealt. Always the first event, and only there.
    Forced,
    /// A recorded action the engine applied. `index` is its position in the
    /// street's `actions` list, posts included, as [`ReplayStep::index`].
    /// Recorded posts are not events: `Forced` made them.
    Action {
        /// The acting seat, 0-based.
        seat: u8,
        /// What the engine applied.
        action: PlayerAction,
        /// The position in the street's recorded `actions` list.
        index: usize,
        /// The street the action was recorded on.
        street: ReplayStreet,
    },
    /// The bets go into the pot and the street's cards are dealt.
    Deal(ReplayStreet),
    /// Two or more seats show down after the river; the last bets go into the
    /// pot.
    Showdown,
    /// `amount` chips of one pot share go from the pot to `seat`.
    Award {
        /// The seat paid, 0-based.
        seat: u8,
        /// The chips paid.
        amount: usize,
    },
}

impl std::fmt::Display for ReplayEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayEvent::Forced => f.write_str("forced bets"),
            ReplayEvent::Action {
                seat,
                action,
                index,
                street,
            } => write!(f, "{street}[{index}]: seat {seat} {action}"),
            ReplayEvent::Deal(street) => write!(f, "deal the {street}"),
            ReplayEvent::Showdown => f.write_str("showdown"),
            ReplayEvent::Award { seat, amount } => write!(f, "seat {seat} wins {amount}"),
        }
    }
}

/// A cursor over a replayed [`HandHistory`]: the hand's [`ReplayEvent`]s and
/// the [`Table`] as it stands after one of them.
///
/// Made by [`HandHistory::cursor`], which plays the whole hand once to list
/// the events. The cursor then stands on event 0, [`ReplayEvent::Forced`].
/// [`Replay::step`] moves it on by one event, [`Replay::seek`] to any event,
/// backwards included.
///
/// # Domain
///
/// - **Role:** read model
/// - **Invariants:**
///   - [`Replay::at`] is the index of the last event applied to
///     [`Replay::table`], always `< len()`
///   - the table after `seek(i)` is the same whichever event the cursor stood
///     on before, and the same as stepping from event 0 to `i`
///   - after the last event the pot is empty, no seat holds a bet, and every
///     stack is the engine's after its showdown
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "hand-histories")]
/// # {
/// use pkcore::hand_history::{HandHistory, ReplayEvent};
///
/// let hand = HandHistory::from_yaml(r"
/// hand: { id: ex, game: holdem }
/// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
/// players:
///   - { seat: 0, name: A, stack: 1000.0 }
///   - { seat: 1, name: B, stack: 1000.0 }
/// streets:
///   preflop:
///     actions:
///       - { seat: 0, action: raise, amount: 300.0 }
///       - { seat: 1, action: fold }
/// ").unwrap();
///
/// let mut replay = hand.cursor().unwrap();
/// while replay.step().unwrap() {}
/// assert_eq!(Some(&ReplayEvent::Award { seat: 0, amount: 400 }), replay.event(replay.at()));
/// assert_eq!(1_100, replay.table().seats.get_seat(0).unwrap().player.chips);
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Replay {
    hand: HandHistory,
    events: Vec<ReplayEvent>,
    at: usize,
    table: Table,
}

/// What [`HandHistory::walk`] reports as it plays a hand.
pub(crate) enum Walked<'a> {
    /// The street's bets went in and its cards were dealt.
    Dealt(ReplayStreet),
    /// A recorded entry was reached. `applied` is the action the engine
    /// applied, `None` for a post.
    Acted {
        step: ReplayStep<'a>,
        applied: Option<PlayerAction>,
    },
}

impl HandHistory {
    /// Plays the recorded hand on `table`, a table from `replay_table`, and
    /// calls `visit` after each street is dealt and after each recorded entry,
    /// posts included, with the table as it then stands.
    ///
    /// The one walk [`HandHistory::replay_with`] and [`HandHistory::cursor`]
    /// share.
    ///
    /// # Errors
    ///
    /// The same as [`HandHistory::replay_with`]; entries visited before the
    /// error were applied.
    pub(crate) fn walk<F>(&self, table: &mut Table, mut visit: F) -> Result<(), PKError>
    where
        F: FnMut(Walked<'_>, &Table),
    {
        self.play_streets(table, |table, street, actions| {
            if street != ReplayStreet::Preflop {
                visit(Walked::Dealt(street), table);
            }
            for (index, action) in actions.iter().enumerate() {
                let applied = Self::apply_recorded(table, action)?;
                visit(
                    Walked::Acted {
                        step: ReplayStep { street, index, action },
                        applied,
                    },
                    table,
                );
            }
            Ok(true)
        })
    }

    /// A [`Replay`] cursor over this hand, standing on its first event,
    /// [`ReplayEvent::Forced`].
    ///
    /// Plays the hand once to list its events, and runs the engine's showdown
    /// on a copy of the final table to list the awards.
    ///
    /// # Errors
    ///
    /// Whatever [`HandHistory::replay`] would return: the hand cannot be
    /// rebuilt, an action is refused, a recorded amount is not the engine's
    /// ([`PKError::ReplayAmountMismatch`]), the hand never finishes
    /// ([`PKError::ActionIsntFinished`]), or the chips do not balance.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::{HandHistory, ReplayEvent};
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 1, name: B, stack: 1000.0 }
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: raise, amount: 300.0 }
    ///       - { seat: 1, action: fold }
    /// ").unwrap();
    ///
    /// let replay = hand.cursor().unwrap();
    /// assert_eq!(4, replay.len());
    /// assert_eq!(0, replay.at());
    /// assert_eq!(Some(&ReplayEvent::Forced), replay.event(0));
    /// # }
    /// ```
    pub fn cursor(&self) -> Result<Replay, PKError> {
        let start = self.replay_table()?;
        let mut table = start.clone();
        let mut events = vec![ReplayEvent::Forced];
        self.walk(&mut table, |walked, _| match walked {
            Walked::Dealt(street) => events.push(ReplayEvent::Deal(street)),
            Walked::Acted {
                step,
                applied: Some(action),
            } => events.push(ReplayEvent::Action {
                seat: step.action.seat,
                action,
                index: step.index,
                street: step.street,
            }),
            Walked::Acted { applied: None, .. } => {}
        })?;
        if table.seats.active_in_hand().len() >= 2 {
            events.push(ReplayEvent::Showdown);
        }
        events.extend(
            Replay::awards(&table)?
                .into_iter()
                .map(|(seat, amount)| ReplayEvent::Award { seat, amount }),
        );
        Ok(Replay {
            hand: self.clone(),
            events,
            at: 0,
            table: start,
        })
    }
}

impl Replay {
    /// The number of events in the hand, [`ReplayEvent::Forced`] included.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::HandHistory;
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 1, name: B, stack: 1000.0 }
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: fold }
    /// ").unwrap();
    ///
    /// // Forced, the fold, the award.
    /// assert_eq!(3, hand.cursor().unwrap().len());
    /// # }
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Always `false`: every replay has at least [`ReplayEvent::Forced`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::HandHistory;
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 1, name: B, stack: 1000.0 }
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: fold }
    /// ").unwrap();
    ///
    /// assert!(!hand.cursor().unwrap().is_empty());
    /// # }
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// The index of the last event applied to [`Self::table`]: 0 (the forced
    /// bets) for a new cursor, `len() - 1` at the end of the hand.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::HandHistory;
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 1, name: B, stack: 1000.0 }
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: fold }
    /// ").unwrap();
    ///
    /// let mut replay = hand.cursor().unwrap();
    /// assert_eq!(0, replay.at());
    /// replay.step().unwrap();
    /// assert_eq!(1, replay.at());
    /// # }
    /// ```
    #[must_use]
    pub fn at(&self) -> usize {
        self.at
    }

    /// The event at `index`, or `None` past the end.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::hand_history::{HandHistory, ReplayEvent, ReplayStreet};
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 1, name: B, stack: 1000.0 }
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: fold }
    /// ").unwrap();
    ///
    /// let replay = hand.cursor().unwrap();
    /// assert_eq!(
    ///     Some(&ReplayEvent::Action {
    ///         seat: 0,
    ///         action: PlayerAction::Fold,
    ///         index: 0,
    ///         street: ReplayStreet::Preflop,
    ///     }),
    ///     replay.event(1)
    /// );
    /// assert_eq!(None, replay.event(replay.len()));
    /// # }
    /// ```
    #[must_use]
    pub fn event(&self, index: usize) -> Option<&ReplayEvent> {
        self.events.get(index)
    }

    /// The table as it stands after event [`Self::at`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::HandHistory;
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 1, name: B, stack: 1000.0 }
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: fold }
    /// ").unwrap();
    ///
    /// // The blinds are posted before the first action.
    /// let replay = hand.cursor().unwrap();
    /// assert_eq!(100, replay.table().bet);
    /// # }
    /// ```
    #[must_use]
    pub fn table(&self) -> &Table {
        &self.table
    }

    /// Moves the cursor to event `index`, forwards or back: rebuilds the
    /// starting table and applies events `0..=index`.
    ///
    /// # Errors
    ///
    /// [`PKError::InvalidPosition`] when `index >= len()`; the cursor does not
    /// move. Otherwise only what the engine returns rebuilding a table the
    /// cursor has already played, which leaves the cursor where it was.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::PKError;
    /// use pkcore::hand_history::HandHistory;
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 1, name: B, stack: 1000.0 }
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: fold }
    /// ").unwrap();
    ///
    /// let mut replay = hand.cursor().unwrap();
    /// let start = replay.table().clone();
    /// replay.seek(replay.len() - 1).unwrap();
    /// replay.seek(0).unwrap();
    /// assert_eq!(&start, replay.table());
    /// assert_eq!(Err(PKError::InvalidPosition), replay.seek(replay.len()));
    /// # }
    /// ```
    pub fn seek(&mut self, index: usize) -> Result<(), PKError> {
        if index >= self.events.len() {
            return Err(PKError::InvalidPosition);
        }
        let mut table = self.hand.replay_table()?;
        for event in self.events.iter().take(index + 1) {
            self.apply(&mut table, event)?;
        }
        self.table = table;
        self.at = index;
        Ok(())
    }

    /// Applies the next event. Returns `Ok(false)`, and changes nothing, when
    /// the cursor already stands on the last event.
    ///
    /// # Errors
    ///
    /// Only what the engine returns applying an event the cursor has already
    /// played once; the cursor then does not move.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::HandHistory;
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 1, name: B, stack: 1000.0 }
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: fold }
    /// ").unwrap();
    ///
    /// let mut replay = hand.cursor().unwrap();
    /// assert!(replay.step().unwrap()); // the fold
    /// assert!(replay.step().unwrap()); // the award
    /// assert!(!replay.step().unwrap()); // the end
    /// assert_eq!(1_050, replay.table().seats.get_seat(1).unwrap().player.chips);
    /// # }
    /// ```
    pub fn step(&mut self) -> Result<bool, PKError> {
        let next = self.at + 1;
        let Some(event) = self.events.get(next) else {
            return Ok(false);
        };
        let mut table = self.table.clone();
        self.apply(&mut table, event)?;
        self.table = table;
        self.at = next;
        Ok(true)
    }

    /// Applies one event to `table`, the table after the event before it.
    fn apply(&self, table: &mut Table, event: &ReplayEvent) -> Result<(), PKError> {
        match event {
            // The starting table already holds the forced bets.
            ReplayEvent::Forced => Ok(()),
            ReplayEvent::Action { seat, action, .. } => table.apply_action(*seat, *action),
            ReplayEvent::Deal(street) => {
                table.bring_it_in()?;
                self.hand.lay_street(table, *street)?.ok_or(PKError::NotDealt)?;
                Ok(())
            }
            ReplayEvent::Showdown => {
                Self::gather(table);
                table.phase = GamePhase::Showdown;
                Ok(())
            }
            ReplayEvent::Award { seat, amount } => {
                Self::gather(table);
                table.pot = table.pot.checked_sub(*amount).ok_or(PKError::InsufficientChips)?;
                let winner = table.seats.get_seat_mut(*seat).ok_or(PKError::InvalidSeatNumber)?;
                winner.player.chips += *amount;
                Ok(())
            }
        }
    }

    /// Moves every bet still in front of a seat into the pot, as the engine's
    /// showdown does before it pays.
    fn gather(table: &mut Table) {
        let mut collected = 0;
        for seat in &mut table.seats {
            collected += seat.player.bet;
            seat.player.bet = 0;
        }
        table.pot += collected;
        table.bet = 0;
    }

    /// The pot shares the engine's showdown pays from `end`, the table after
    /// the last recorded action, as `(seat, chips)` in the order it pays them.
    ///
    /// Read from the engine's log, one entry per share of each pot. A seat's
    /// shares that do not add up to the chips it gained are replaced by one
    /// share of what it gained, so the awards always move exactly the chips the
    /// engine moved (a fold-win logs no share). The copy is then reset and its
    /// chips audited, as [`Table::end_hand`] does.
    fn awards(end: &Table) -> Result<Vec<(u8, usize)>, PKError> {
        let mut settled = end.clone();
        let logged_from = settled.event_log.len();
        settled.showdown()?;
        let mut shares: Vec<(u8, usize)> = settled
            .event_log
            .iter()
            .skip(logged_from)
            .filter_map(|entry| match entry {
                TableAction::PlayerWins(seat, _, _, _, share)
                | TableAction::PlayerWinsMainPot(seat, share)
                | TableAction::PlayerWinsSidePot(seat, share) => Some((*seat, *share)),
                _ => None,
            })
            .collect();
        for (index, (before, after)) in end.seats.iter().zip(settled.seats.iter()).enumerate() {
            let seat = u8::try_from(index).map_err(|_| PKError::InvalidSeatNumber)?;
            let gained = after.player.chips.saturating_sub(before.player.chips);
            let logged: usize = shares
                .iter()
                .filter(|(paid, _)| *paid == seat)
                .map(|(_, chips)| chips)
                .sum();
            if logged != gained {
                shares.retain(|(paid, _)| *paid != seat);
                shares.push((seat, gained));
            }
        }
        settled.reset();
        settled.audit_chip_total()?;
        shares.retain(|(_, chips)| *chips > 0);
        Ok(shares)
    }
}

#[cfg(all(test, feature = "hand-histories"))]
#[allow(non_snake_case)]
mod hand_history__replay_tests {
    use super::*;
    use std::path::PathBuf;

    /// Heads-up: seat 0 raises, seat 1 folds.
    const FOLD_OUT: &str = r"
hand: { id: fold-out, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: raise, amount: 300.0 }
      - { seat: 1, action: fold }
";

    /// Heads-up, both all in preflop for the same 1,000, aces win.
    const ALL_IN: &str = r#"
hand: { id: all-in, game: holdem }
table: { seats: 2, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0, hole_cards: "7♦ 2♣" }
  - { seat: 1, name: B, stack: 1000.0, hole_cards: "A♠ A♥" }
streets:
  preflop:
    actions:
      - { seat: 0, action: all_in, amount: 1000.0, all_in: true }
      - { seat: 1, action: call, amount: 900.0 }
  flop:
    cards: "K♠ K♣ 9♦"
    actions: []
  turn:
    card: "8♠"
    actions: []
  river:
    card: "4♥"
    actions: []
"#;

    /// Heads-up, both all in preflop, the board plays: the pot is split.
    const SPLIT: &str = r#"
hand: { id: split, game: holdem }
table: { seats: 2, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0, hole_cards: "2♦ 3♣" }
  - { seat: 1, name: B, stack: 1000.0, hole_cards: "2♠ 3♥" }
streets:
  preflop:
    actions:
      - { seat: 0, action: all_in, amount: 1000.0, all_in: true }
      - { seat: 1, action: call, amount: 900.0 }
  flop:
    cards: "A♠ K♣ Q♦"
    actions: []
  turn:
    card: "J♠"
    actions: []
  river:
    card: "T♥"
    actions: []
"#;

    fn hand(yaml: &str) -> HandHistory {
        HandHistory::from_yaml(yaml).unwrap()
    }

    fn defect(name: &str) -> HandHistory {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/defects")
            .join(name);
        hand(&std::fs::read_to_string(path).unwrap())
    }

    /// Every `tests/defects/DEFECT_*.yaml` hand that `replay()` accepts.
    fn replayable_defects() -> Vec<(String, HandHistory)> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/defects");
        let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.extension().is_some_and(|ext| ext == "yaml")
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("DEFECT_"))
            })
            .collect();
        paths.sort();
        paths
            .into_iter()
            .filter_map(|path| {
                let hand = HandHistory::from_yaml(&std::fs::read_to_string(&path).unwrap()).ok()?;
                hand.replay().ok()?;
                Some((path.display().to_string(), hand))
            })
            .collect()
    }

    fn to_end(replay: &mut Replay) {
        replay.seek(replay.len() - 1).unwrap();
    }

    fn awards(replay: &Replay) -> Vec<(u8, usize)> {
        (0..replay.len())
            .filter_map(|index| match replay.event(index) {
                Some(ReplayEvent::Award { seat, amount }) => Some((*seat, *amount)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn cursor__seek_back_equals_fresh_walk() {
        let hands = replayable_defects();
        assert!(hands.len() > 10, "only {} defect hands replay", hands.len());
        for (path, hand) in hands {
            let mut scrubbed = hand.cursor().unwrap();
            for i in 0..scrubbed.len() {
                to_end(&mut scrubbed);
                scrubbed.seek(i).unwrap();
                let mut fresh = hand.cursor().unwrap();
                fresh.seek(i).unwrap();
                assert_eq!(fresh.table(), scrubbed.table(), "{path}: event {i}");
                assert_eq!(i, scrubbed.at(), "{path}: event {i}");
            }
        }
    }

    #[test]
    fn cursor__stepping_equals_seeking() {
        for (path, hand) in replayable_defects() {
            let mut stepped = hand.cursor().unwrap();
            let mut sought = hand.cursor().unwrap();
            while stepped.step().unwrap() {
                sought.seek(stepped.at()).unwrap();
                assert_eq!(sought.table(), stepped.table(), "{path}: event {}", stepped.at());
            }
        }
    }

    #[test]
    fn cursor__all_in_preflop_deals_three_events() {
        let replay = hand(ALL_IN).cursor().unwrap();
        let last_preflop = (0..replay.len())
            .filter(|&index| {
                matches!(
                    replay.event(index),
                    Some(ReplayEvent::Action {
                        street: ReplayStreet::Preflop,
                        ..
                    })
                )
            })
            .max()
            .unwrap();

        let after: Vec<&ReplayEvent> = (last_preflop + 1..replay.len())
            .filter_map(|index| replay.event(index))
            .collect();

        assert_eq!(
            vec![
                &ReplayEvent::Deal(ReplayStreet::Flop),
                &ReplayEvent::Deal(ReplayStreet::Turn),
                &ReplayEvent::Deal(ReplayStreet::River),
                &ReplayEvent::Showdown,
                &ReplayEvent::Award { seat: 1, amount: 2_000 },
            ],
            after
        );
    }

    #[test]
    fn cursor__ends_on_award_with_empty_pot() {
        for (path, hand) in replayable_defects() {
            let mut replay = hand.cursor().unwrap();
            to_end(&mut replay);
            assert!(
                matches!(replay.event(replay.at()), Some(ReplayEvent::Award { .. })),
                "{path}"
            );
            assert_eq!(0, replay.table().pot, "{path}");
            assert!(replay.table().seats.iter().all(|seat| seat.player.bet == 0), "{path}");
        }
    }

    #[test]
    fn cursor__ends_on_the_stacks_replay_reports() {
        for (path, hand) in replayable_defects() {
            let mut replay = hand.cursor().unwrap();
            to_end(&mut replay);
            for (seat, chips) in hand.replay().unwrap().final_stacks {
                let got = replay.table().seats.get_seat(seat).unwrap().player.chips;
                assert_eq!(chips, got, "{path}: seat {seat}");
            }
        }
    }

    #[test]
    fn cursor__fold_out_has_no_showdown() {
        let replay = hand(FOLD_OUT).cursor().unwrap();

        assert!((0..replay.len()).all(|index| replay.event(index) != Some(&ReplayEvent::Showdown)));
        // Seat 0's 300 comes back with seat 1's big blind.
        assert_eq!(vec![(0, 400)], awards(&replay));
        assert_eq!(
            Some(&ReplayEvent::Award { seat: 0, amount: 400 }),
            replay.event(replay.len() - 1)
        );
    }

    #[test]
    fn cursor__side_pot_awards_each_share() {
        let hand = defect("DEFECT_003_short_winner_excess_returned.yaml");
        let replay = hand.cursor().unwrap();
        let shares = awards(&replay);

        // The short stack's matched 400, then the deep stack's uncalled 800
        // back as its own share, as the engine's showdown pays it.
        assert_eq!(vec![(1, 400), (0, 800)], shares);
        // Both seats are all in: every chip at the table was put in.
        let put_in = hand.replay_table().unwrap().table_chip_count();
        assert_eq!(put_in, shares.iter().map(|(_, chips)| chips).sum::<usize>());
    }

    #[test]
    fn cursor__split_pot_awards_each_winner() {
        let replay = hand(SPLIT).cursor().unwrap();
        let mut shares = awards(&replay);
        shares.sort_unstable();

        assert_eq!(vec![(0, 1_000), (1, 1_000)], shares);
    }

    #[test]
    fn cursor__events_follow_the_walk() {
        let hand = hand(ALL_IN);
        let replay = hand.cursor().unwrap();
        let mut walked = Vec::new();
        hand.replay_with(|step, _| walked.push((step.street, step.index, step.action.seat)))
            .unwrap();

        let actions: Vec<(ReplayStreet, usize, u8)> = (0..replay.len())
            .filter_map(|index| match replay.event(index) {
                Some(ReplayEvent::Action {
                    seat, index, street, ..
                }) => Some((*street, *index, *seat)),
                _ => None,
            })
            .collect();
        assert_eq!(walked, actions);
        assert_eq!(Some(&ReplayEvent::Forced), replay.event(0));
        assert!(
            (1..replay.len()).all(|index| replay.event(index) != Some(&ReplayEvent::Forced)
                && replay.event(index) != Some(&ReplayEvent::Deal(ReplayStreet::Preflop)))
        );
    }

    #[test]
    fn cursor__skips_recorded_posts() {
        let hand = hand(
            r"
hand: { id: posts, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: post, amount: 50.0 }
      - { seat: 1, action: post, amount: 100.0 }
      - { seat: 0, action: fold }
",
        );
        let replay = hand.cursor().unwrap();

        assert_eq!(3, replay.len());
        assert_eq!(
            Some(&ReplayEvent::Action {
                seat: 0,
                action: PlayerAction::Fold,
                index: 2,
                street: ReplayStreet::Preflop,
            }),
            replay.event(1)
        );
    }

    #[test]
    fn cursor__rejects_a_mismatched_amount_as_replay_does() {
        let hand = hand(
            r"
hand: { id: mismatch, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: raise, amount: 300.0 }
      - { seat: 1, action: call, amount: 250.0 }
",
        );

        let replayed = hand.replay().map(|_| ());
        assert!(matches!(replayed, Err(PKError::ReplayAmountMismatch { seat: 1, .. })));
        assert_eq!(replayed, hand.cursor().map(|_| ()));
    }

    #[test]
    fn replay__len_and_is_empty() {
        let replay = hand(FOLD_OUT).cursor().unwrap();

        // Forced, raise, fold, award.
        assert_eq!(4, replay.len());
        assert!(!replay.is_empty());
    }

    #[test]
    fn replay__at_starts_on_forced() {
        let replay = hand(FOLD_OUT).cursor().unwrap();

        assert_eq!(0, replay.at());
        assert_eq!(Some(&ReplayEvent::Forced), replay.event(replay.at()));
        assert_eq!(&hand(FOLD_OUT).replay_table().unwrap(), replay.table());
    }

    #[test]
    fn replay__event_past_the_end_is_none() {
        let replay = hand(FOLD_OUT).cursor().unwrap();

        assert!(replay.event(replay.len() - 1).is_some());
        assert_eq!(None, replay.event(replay.len()));
    }

    #[test]
    fn replay__table_follows_the_cursor() {
        let mut replay = hand(FOLD_OUT).cursor().unwrap();
        replay.seek(1).unwrap();

        assert_eq!(300, replay.table().bet);
        assert_eq!(700, replay.table().seats.get_seat(0).unwrap().player.chips);
    }

    #[test]
    fn replay__seek_past_the_end_does_not_move() {
        let mut replay = hand(FOLD_OUT).cursor().unwrap();
        replay.seek(1).unwrap();
        let before = replay.table().clone();

        assert_eq!(Err(PKError::InvalidPosition), replay.seek(replay.len()));
        assert_eq!(1, replay.at());
        assert_eq!(&before, replay.table());
    }

    #[test]
    fn replay__step_stops_at_the_end() {
        let mut replay = hand(FOLD_OUT).cursor().unwrap();
        let mut steps = 0;
        while replay.step().unwrap() {
            steps += 1;
        }
        let end = replay.table().clone();

        assert_eq!(replay.len() - 1, steps);
        assert_eq!(replay.len() - 1, replay.at());
        assert!(!replay.step().unwrap());
        assert_eq!(&end, replay.table());
        assert_eq!(1_100, end.seats.get_seat(0).unwrap().player.chips);
        assert_eq!(900, end.seats.get_seat(1).unwrap().player.chips);
    }

    #[test]
    fn replay_event__display() {
        assert_eq!("forced bets", ReplayEvent::Forced.to_string());
        assert_eq!("deal the flop", ReplayEvent::Deal(ReplayStreet::Flop).to_string());
        assert_eq!("showdown", ReplayEvent::Showdown.to_string());
        assert_eq!("seat 2 wins 10", ReplayEvent::Award { seat: 2, amount: 10 }.to_string());
        assert_eq!(
            "river[3]: seat 4 Call",
            ReplayEvent::Action {
                seat: 4,
                action: PlayerAction::Call,
                index: 3,
                street: ReplayStreet::River,
            }
            .to_string()
        );
    }
}
