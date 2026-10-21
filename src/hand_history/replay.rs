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

use crate::casino::action::{PlayerAction, TableAction};
use crate::casino::table::Table;
use crate::games::GamePhase;
use crate::hand_history::{HandHistory, HandVariant, ReplayStep, ReplayStreet};
use crate::{Card, Cards, PKError};
use std::str::FromStr;

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

    /// A 52-card deck that, dealt by the engine, reproduces the cards this
    /// hand recorded: the same hole cards to the same seats and the same flop,
    /// turn and river.
    ///
    /// The deck is laid out in the order [`Table`] draws it. The hole cards go
    /// first, one card to each seat still in the hand per pass, as many passes
    /// as the game has hole cards (two for hold'em, four for Omaha), starting
    /// with the seat after the button; then a burn, the flop, a burn, the turn,
    /// a burn and the river. Chairs that are empty or out of the hand are
    /// skipped, as the engine skips them. The rest of the deck follows.
    ///
    /// A record can be partial: a folded player's cards unseen, a river that
    /// was never dealt. Every position the record leaves open, the burns
    /// included, is filled from the cards the record does not use, in
    /// [`Cards::deck`] order. The streets' `cards` are the board; the `board`
    /// field fills any board position the streets leave open. Recorded cards
    /// belonging to a chair that is not dealt in end the deck.
    ///
    /// Pair it with [`PokerSession::start_hand_with_deck`](crate::casino::session::PokerSession::start_hand_with_deck) to take over a
    /// recorded hand: the session deals the record's hole cards, and the board
    /// follows as the engine deals the streets.
    ///
    /// # Errors
    ///
    /// - [`PKError::NotImplemented`] for every game but hold'em and Omaha:
    ///   stud, razz and draw deal in a different order, and Omaha hi-lo and
    ///   Kuhn are not rebuilt as their own table.
    /// - The error [`HandHistory::replay`] gives when the hand cannot be
    ///   rebuilt as a table, such as `PKError::TooManyPlayers`.
    /// - [`PKError::InvalidCardCount`] for a player's hole cards that are not
    ///   the game's count, or a flop that is not three cards.
    /// - [`PKError::DuplicateCard`] when one card is recorded twice.
    /// - A parse error for a card that is not one.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::HandHistory;
    ///
    /// let hand = HandHistory::from_yaml(r#"
    /// hand: { id: ex, game: holdem }
    /// table: { seats: 2, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0, hole_cards: "7♦ 2♣" }
    ///   - { seat: 1, name: B, stack: 1000.0, hole_cards: "A♠ A♥" }
    /// streets:
    ///   flop: { cards: "K♠ K♣ 9♦", actions: [] }
    ///   turn: { card: "8♠", actions: [] }
    ///   river: { card: "4♥", actions: [] }
    /// "#).unwrap();
    ///
    /// let deck = hand.stacked_deck().unwrap();
    /// assert_eq!(52, deck.len());
    /// // Seat 1, left of the button, is dealt first; one card a pass.
    /// let top: Vec<String> = deck.iter().take(4).map(ToString::to_string).collect();
    /// assert_eq!(vec!["A♠", "7♦", "A♥", "2♣"], top);
    /// # }
    /// ```
    pub fn stacked_deck(&self) -> Result<Cards, PKError> {
        if !matches!(self.hand.game, HandVariant::Holdem | HandVariant::Omaha) {
            return Err(PKError::NotImplemented);
        }
        let table = self.replay_table()?;
        let cards_per = table.game.cards_per_player() as usize;
        let recorded_holes = self.recorded_hole_cards(cards_per)?;
        let board = self.recorded_board()?;

        // The engine's deal, `Table::deal_cards_to_seats`: from the seat after
        // the button, one card per seat in the hand per pass.
        let seat_count = table.seats.size() as usize;
        if seat_count == 0 {
            return Err(PKError::InvalidSeatNumber);
        }
        let mut slots: Vec<Option<Card>> = Vec::with_capacity(Cards::DECK_SIZE);
        for pass in 0..cards_per {
            for step in 0..seat_count {
                let seat = u8::try_from((table.button as usize + 1 + step) % seat_count).unwrap_or(0);
                if table.seats.is_seat_in_hand(seat) {
                    let recorded = recorded_holes.iter().find(|(owner, _)| *owner == seat);
                    slots.push(recorded.and_then(|(_, cards)| cards.get(pass)).copied());
                }
            }
        }
        // Then burn, flop, burn, turn, burn, river: `deal_flop` and friends.
        slots.push(None);
        slots.extend_from_slice(&board[0..3]);
        slots.push(None);
        slots.push(board[3]);
        slots.push(None);
        slots.push(board[4]);
        if slots.len() > Cards::DECK_SIZE {
            return Err(PKError::TooManyCards);
        }

        let mut recorded: Vec<Card> = Vec::new();
        for card in recorded_holes
            .iter()
            .flat_map(|(_, cards)| cards.iter())
            .chain(board.iter().flatten())
        {
            if recorded.contains(card) {
                return Err(PKError::DuplicateCard);
            }
            recorded.push(*card);
        }
        let used = Cards::from(recorded.clone());
        let mut spare = Cards::deck_minus(&used).into_iter();

        let mut stacked: Vec<Card> = Vec::with_capacity(Cards::DECK_SIZE);
        for slot in slots {
            stacked.push(match slot {
                Some(card) => card,
                None => spare.next().ok_or(PKError::NotEnoughCards)?,
            });
        }
        stacked.extend(spare);
        // Recorded cards no seat in the hand is dealt close the deck.
        let unplaced: Vec<Card> = recorded.into_iter().filter(|card| !stacked.contains(card)).collect();
        stacked.extend(unplaced);

        let deck = Cards::from(stacked);
        if deck.len() != Cards::DECK_SIZE {
            return Err(PKError::InvalidCardCount);
        }
        Ok(deck)
    }

    /// Each player's recorded hole cards, as `(seat, cards)`.
    fn recorded_hole_cards(&self, cards_per: usize) -> Result<Vec<(u8, Vec<Card>)>, PKError> {
        let mut recorded = Vec::new();
        for player in &self.players {
            let Some(text) = &player.hole_cards else {
                continue;
            };
            let cards: Vec<Card> = Cards::from_str(text)?.iter().copied().collect();
            if cards.len() != cards_per {
                return Err(PKError::InvalidCardCount);
            }
            recorded.push((player.seat, cards));
        }
        Ok(recorded)
    }

    /// The five board positions, flop first, with the cards the record has.
    /// The streets are the authority; the `board` field fills the gaps.
    fn recorded_board(&self) -> Result<[Option<Card>; 5], PKError> {
        let mut board: [Option<Card>; 5] = [None; 5];
        if let Some(text) = &self.board {
            let cards: Vec<Card> = Cards::from_str(text)?.iter().copied().collect();
            if cards.len() > board.len() {
                return Err(PKError::TooManyCards);
            }
            for (slot, card) in board.iter_mut().zip(cards) {
                *slot = Some(card);
            }
        }
        if let Some(streets) = &self.streets {
            if let Some(flop) = &streets.flop {
                let cards: Vec<Card> = Cards::from_str(&flop.cards)?.iter().copied().collect();
                if cards.len() != 3 {
                    return Err(PKError::InvalidCardCount);
                }
                for (slot, card) in board.iter_mut().zip(cards) {
                    *slot = Some(card);
                }
            }
            if let Some(turn) = &streets.turn {
                board[3] = Some(Card::from_str(&turn.card)?);
            }
            if let Some(river) = &streets.river {
                board[4] = Some(Card::from_str(&river.card)?);
            }
        }
        Ok(board)
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
    use crate::casino::game::ForcedBets;
    use crate::casino::session::PokerSession;
    use crate::casino::table::{Player, Seat, Seats};
    use crate::{Forgiving, Pile};
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

    /// Eight seats, button 0, every hand recorded and all five board cards.
    const EIGHT_SEATS: &str = r#"
hand: { id: eight-seats, game: holdem }
table: { seats: 8, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: P0, stack: 1000.0, hole_cards: "A♠ K♠" }
  - { seat: 1, name: P1, stack: 1000.0, hole_cards: "Q♠ J♠" }
  - { seat: 2, name: P2, stack: 1000.0, hole_cards: "T♠ 9♠" }
  - { seat: 3, name: P3, stack: 1000.0, hole_cards: "8♠ 7♠" }
  - { seat: 4, name: P4, stack: 1000.0, hole_cards: "6♠ 5♠" }
  - { seat: 5, name: P5, stack: 1000.0, hole_cards: "4♠ 3♠" }
  - { seat: 6, name: P6, stack: 1000.0, hole_cards: "2♠ A♥" }
  - { seat: 7, name: P7, stack: 1000.0, hole_cards: "K♥ Q♥" }
streets:
  flop:
    cards: "J♥ T♥ 9♥"
    actions: []
  turn:
    card: "8♥"
    actions: []
  river:
    card: "7♥"
    actions: []
"#;

    /// Eight seats with the button on 5, so the deal wraps the ring.
    const EIGHT_SEATS_BUTTON_FIVE: &str = r#"
hand: { id: eight-seats-b5, game: holdem }
table: { seats: 8, button: 5, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: P0, stack: 1000.0, hole_cards: "A♠ K♠" }
  - { seat: 1, name: P1, stack: 1000.0, hole_cards: "Q♠ J♠" }
  - { seat: 2, name: P2, stack: 1000.0, hole_cards: "T♠ 9♠" }
  - { seat: 3, name: P3, stack: 1000.0, hole_cards: "8♠ 7♠" }
  - { seat: 4, name: P4, stack: 1000.0, hole_cards: "6♠ 5♠" }
  - { seat: 5, name: P5, stack: 1000.0, hole_cards: "4♠ 3♠" }
  - { seat: 6, name: P6, stack: 1000.0, hole_cards: "2♠ A♥" }
  - { seat: 7, name: P7, stack: 1000.0, hole_cards: "K♥ Q♥" }
streets:
  flop:
    cards: "J♥ T♥ 9♥"
    actions: []
"#;

    /// Four seats, only two shown; the flop is recorded but not the turn or river.
    const PARTIAL: &str = r#"
hand: { id: partial, game: holdem }
table: { seats: 4, button: 2, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: P0, stack: 1000.0, hole_cards: "A♠ K♠" }
  - { seat: 1, name: P1, stack: 1000.0 }
  - { seat: 2, name: P2, stack: 1000.0 }
  - { seat: 3, name: P3, stack: 1000.0, hole_cards: "7♦ 2♣" }
streets:
  flop:
    cards: "J♥ T♥ 9♥"
    actions: []
"#;

    /// Five chairs, empty between the players, the button on an empty one.
    const EMPTY_CHAIRS: &str = r#"
hand: { id: empty-chairs, game: holdem }
table: { seats: 5, button: 3, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: P0, stack: 1000.0, hole_cards: "A♠ K♠" }
  - { seat: 2, name: P2, stack: 1000.0, hole_cards: "Q♠ J♠" }
  - { seat: 4, name: P4, stack: 1000.0, hole_cards: "T♠ 9♠" }
streets:
  flop:
    cards: "J♥ T♥ 9♥"
    actions: []
  turn:
    card: "8♥"
    actions: []
  river:
    card: "7♥"
    actions: []
"#;

    /// Three seats of Omaha, four cards each.
    const OMAHA: &str = r#"
hand: { id: omaha, game: omaha }
table: { seats: 3, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: P0, stack: 1000.0, hole_cards: "A♠ K♠ Q♠ J♠" }
  - { seat: 1, name: P1, stack: 1000.0, hole_cards: "A♥ K♥ Q♥ J♥" }
  - { seat: 2, name: P2, stack: 1000.0, hole_cards: "A♦ K♦ Q♦ J♦" }
streets:
  flop:
    cards: "2♣ 3♣ 4♣"
    actions: []
  turn:
    card: "5♣"
    actions: []
  river:
    card: "6♣"
    actions: []
"#;

    /// The cards each recorded hole-card string and board string name, the
    /// recorded text parsed.
    fn recorded_hole_cards(hand: &HandHistory) -> Vec<(u8, Cards)> {
        hand.players
            .iter()
            .filter_map(|player| {
                player
                    .hole_cards
                    .as_ref()
                    .map(|text| (player.seat, Cards::from_str(text).unwrap()))
            })
            .collect()
    }

    /// The hand's players in a ring, as before the first deal: full stacks,
    /// nothing posted, nothing dealt.
    fn fresh_seats(hand: &HandHistory) -> Seats {
        let size = usize::from(hand.table.seats.unwrap_or(2));
        let mut seats: Vec<Seat> = (0..size).map(|_| Seat::new(Player::default())).collect();
        for player in &hand.players {
            seats[usize::from(player.seat)] = Seat::new(Player::with_id(
                uuid::Uuid::from_u128(u128::from(player.seat) + 1),
                player.name.clone(),
                player.stack as usize,
            ));
        }
        Seats::new(seats)
    }

    /// Deals `hand.stacked_deck()` on a fresh engine table and checks every
    /// recorded card turns up where the record puts it.
    fn assert_round_trip(hand: &HandHistory) {
        let deck = hand.stacked_deck().unwrap();
        assert_eq!(52, deck.len(), "{}: a full deck", hand.hand.id);

        let forced = ForcedBets::new(50, 100);
        let mut table = if hand.hand.game == HandVariant::Omaha {
            let mut table = Table::plo_from_seats_with_id(fresh_seats(hand), (50, 100), uuid::Uuid::nil());
            table.deck = deck.clone();
            table
        } else {
            Table::nlh_primed_with_id(fresh_seats(hand), &deck, forced, uuid::Uuid::nil())
        };
        table.button = hand.table.button.unwrap_or(0);
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        for (seat, cards) in recorded_hole_cards(hand) {
            let dealt = table.seats.get_seat(seat).unwrap().cards.cards();
            assert_eq!(cards, dealt, "{}: seat {seat}", hand.hand.id);
        }

        let streets = hand.streets.as_ref().unwrap();
        if let Some(flop) = &streets.flop {
            table.deal_flop().unwrap();
            assert_eq!(
                Cards::from_str(&flop.cards).unwrap(),
                table.board,
                "{}: flop",
                hand.hand.id
            );
        }
        if let Some(turn) = &streets.turn {
            table.deal_turn().unwrap();
            assert!(table.board.contains(&Card::from_str(&turn.card).unwrap()));
            assert_eq!(4, table.board.len());
        }
        if let Some(river) = &streets.river {
            table.deal_river().unwrap();
            assert!(table.board.contains(&Card::from_str(&river.card).unwrap()));
            assert_eq!(5, table.board.len());
        }
    }

    #[test]
    fn stacked_deck__reproduces_recorded_deal() {
        assert_round_trip(&hand(ALL_IN));
        assert_round_trip(&hand(SPLIT));
        assert_round_trip(&hand(EIGHT_SEATS));
        assert_round_trip(&hand(EIGHT_SEATS_BUTTON_FIVE));
        assert_round_trip(&hand(EMPTY_CHAIRS));
    }

    #[test]
    fn stacked_deck__reproduces_an_omaha_deal() {
        assert_round_trip(&hand(OMAHA));
    }

    #[test]
    fn stacked_deck__heads_up_order_is_big_blind_first() {
        // Button 0: seat 1 is dealt first, one card a pass, then a burn, the
        // flop, a burn, the turn, a burn, the river.
        let deck = hand(ALL_IN).stacked_deck().unwrap();
        let text = deck.to_string();
        let top: Vec<&str> = text.split(' ').take(4).collect();
        assert_eq!(vec!["A♠", "7♦", "A♥", "2♣"], top);
    }

    #[test]
    fn stacked_deck__partial_record_fills_the_gaps_in_deck_order() {
        let partial = hand(PARTIAL);
        let deck = partial.stacked_deck().unwrap();
        assert_eq!(52, deck.len());

        // Button 2 deals 3, 0, 1, 2: pass one A♠'s partner is last.
        let cards: Vec<Card> = deck.iter().copied().collect();
        assert_eq!(Card::from_str("7♦").unwrap(), cards[0]);
        assert_eq!(Card::from_str("A♠").unwrap(), cards[1]);
        assert_eq!(Card::from_str("2♣").unwrap(), cards[4]);
        assert_eq!(Card::from_str("K♠").unwrap(), cards[5]);

        // The unrecorded positions (seats 1 and 2, and the burns, turn and
        // river) come off the rest of the deck in order.
        let recorded = cards!("A♠ K♠ 7♦ 2♣ J♥ T♥ 9♥");
        let mut spare = Cards::deck_minus(&recorded).into_iter();
        for position in [2, 3, 6, 7, 8, 12, 13, 14, 15] {
            let expected = spare.next().unwrap();
            assert_eq!(expected, cards[position], "position {position}");
        }
        assert_eq!(spare.collect::<Vec<_>>(), cards[16..].to_vec());
        assert_round_trip(&partial);
    }

    #[test]
    fn stacked_deck__no_recorded_cards_is_the_deck_in_order() {
        let hand = hand(FOLD_OUT);
        assert_eq!(Cards::deck().to_string(), hand.stacked_deck().unwrap().to_string());
    }

    #[test]
    fn stacked_deck__a_card_recorded_twice_is_refused() {
        let mut twice = hand(ALL_IN);
        twice.players[1].hole_cards = Some("A♠ 7♦".to_string());
        assert_eq!(Err(PKError::DuplicateCard), twice.stacked_deck());

        let mut on_board = hand(ALL_IN);
        on_board.streets.as_mut().unwrap().turn.as_mut().unwrap().card = "7♦".to_string();
        assert_eq!(Err(PKError::DuplicateCard), on_board.stacked_deck());
    }

    #[test]
    fn stacked_deck__an_unreadable_or_miscounted_card_is_refused() {
        let mut bad = hand(ALL_IN);
        bad.players[0].hole_cards = Some("7♦ 9?".to_string());
        assert!(bad.stacked_deck().is_err());

        let mut three = hand(ALL_IN);
        three.players[0].hole_cards = Some("7♦ 2♣ 3♣".to_string());
        assert_eq!(Err(PKError::InvalidCardCount), three.stacked_deck());

        let mut short_flop = hand(ALL_IN);
        short_flop.streets.as_mut().unwrap().flop.as_mut().unwrap().cards = "K♠ K♣".to_string();
        assert_eq!(Err(PKError::InvalidCardCount), short_flop.stacked_deck());
    }

    #[test]
    fn stacked_deck__a_game_it_cannot_stack_is_not_implemented() {
        for game in ["omaha_hi_lo", "kuhn", "razz", "stud_hi_lo", "draw_high"] {
            let other = hand(&format!(
                r#"
hand: {{ id: other, game: {game} }}
table: {{ seats: 2, button: 0, stakes: {{ small_blind: 50.0, big_blind: 100.0 }} }}
players:
  - {{ seat: 0, name: A, stack: 1000.0, hole_cards: "A♠ K♠ Q♠ J♠" }}
  - {{ seat: 1, name: B, stack: 1000.0 }}
"#
            ));
            assert_eq!(Err(PKError::NotImplemented), other.stacked_deck(), "{game}");
        }
    }

    #[test]
    fn stacked_deck__stud_is_not_implemented() {
        let stud = hand(
            r"
hand: { id: stud, game: stud }
table: { seats: 2, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0, ante: 10.0, bring_in: 20.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
",
        );
        assert_eq!(Err(PKError::NotImplemented), stud.stacked_deck());
    }

    #[test]
    fn stacked_deck__falls_back_to_the_board_field() {
        let mut flopless = hand(EIGHT_SEATS);
        flopless.streets = None;
        flopless.board = Some("J♥ T♥ 9♥ 8♥ 7♥".to_string());
        let with_streets = hand(EIGHT_SEATS).stacked_deck().unwrap();
        assert_eq!(with_streets.to_string(), flopless.stacked_deck().unwrap().to_string());
    }

    #[test]
    fn stacked_deck__a_session_deals_the_recorded_hole_cards() {
        let hand = hand(EIGHT_SEATS_BUTTON_FIVE);
        let mut session = PokerSession::new(Table::nlh_from_seats_with_id(
            fresh_seats(&hand),
            ForcedBets::new(50, 100),
            uuid::Uuid::nil(),
        ));
        session.table.button = hand.table.button.unwrap_or(0);

        session.start_hand_with_deck(hand.stacked_deck().unwrap()).unwrap();

        for (seat, cards) in recorded_hole_cards(&hand) {
            let dealt = session.table.seats.get_seat(seat).unwrap().cards.cards();
            assert_eq!(cards, dealt, "seat {seat}");
        }
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
