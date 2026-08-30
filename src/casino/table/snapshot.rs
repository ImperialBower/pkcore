//! The written-down form of a [`Table`] — mid-hand, resumable, and portable.
//!
//! [`TableState`] is a DTO, deliberately **not** `#[derive(Serialize)]` on
//! `Table` itself. Deriving on the engine would freeze its 21 public fields
//! into the wire format, and a snapshot outlives the process that wrote it, so
//! that shape becomes a compatibility obligation. This type is the contract;
//! `Table` stays free to change behind it. The same call was made for
//! `SessionView` (`docs/epics/EPIC-37_Mobile_Engine.md`).
//!
//! # The bytes are the future
//!
//! A snapshot carries the **undealt deck in order**. Anyone holding it can read
//! the runout before it happens. Store snapshots in the host's private storage
//! and never transmit one to a player or a spectator.
//!
//! # Design notes
//!
//! Three shapes here are load-bearing, and each answers a defect found while
//! building this (EPIC-88):
//!
//! - **`Vec<String>` for card piles, not `String`.** `Cards::from_str("")`
//!   returns `Err` rather than an empty pile (`src/cards.rs:920-922`), so a
//!   pre-flop board — the most common state in the game — cannot survive a
//!   single-string field.
//! - **`Vec<Option<String>>` for a seat's cards.** [`BoxedCards`] is a
//!   fixed-width box that may hold blanks; `None` records a blank slot so
//!   `blanks(2)` and two real cards stay distinguishable.
//! - **`Vec<(u8, …)>` instead of `Table`'s `HashMap<u8, BoxedCards>`.**
//!   `HashMap` iteration order is unspecified, so a `HashMap` on the wire would
//!   make two snapshots of one table differ byte-for-byte and break `Eq`.

use crate::PKError;
use crate::arrays::sliced::BoxedCards;
use crate::card::Card;
use crate::cards::Cards;
use crate::casino::game::ForcedBets;
use crate::casino::state::PlayerState;
use crate::casino::table::{Player, Seat, Seats, Table};
use crate::games::betting_structure::BettingStructure;
use crate::games::{GamePhase, GameType};
use crate::play::seat_hand::SeatHand;
use crate::play::visibility::Visibility;
use crate::prelude::TableAction;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// Wire-format version. Bump on any breaking change to [`TableState`]'s shape;
/// [`Table::restore`] refuses a version it does not recognise before reading
/// any other field.
pub const SNAPSHOT_VERSION: u16 = 1;

/// Postcard-safe mirror of [`BettingStructure`].
///
/// The real enum is `#[serde(tag = "kind")]` — *internally tagged* — which
/// serde can only deserialize via `deserialize_any`, and a non-self-describing
/// format like `postcard` cannot provide that (*"a feature `PostCard` will never
/// implement"*). Changing the real enum's representation is not an option: that
/// tagged form is already on disk in every shipped bot profile
/// (`data/bots/**/*.yaml`), so it is a compatibility obligation. The DTO
/// carries its own externally-tagged mirror instead — which is precisely what
/// a DTO is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BettingState {
    NoLimit,
    PotLimit,
    FixedLimit {
        small_bet: usize,
        big_bet: usize,
        raise_cap: u8,
    },
}

impl From<BettingStructure> for BettingState {
    fn from(betting: BettingStructure) -> Self {
        match betting {
            BettingStructure::NoLimit => BettingState::NoLimit,
            BettingStructure::PotLimit => BettingState::PotLimit,
            BettingStructure::FixedLimit {
                small_bet,
                big_bet,
                raise_cap,
            } => BettingState::FixedLimit {
                small_bet,
                big_bet,
                raise_cap,
            },
        }
    }
}

impl From<BettingState> for BettingStructure {
    fn from(state: BettingState) -> Self {
        match state {
            BettingState::NoLimit => BettingStructure::NoLimit,
            BettingState::PotLimit => BettingStructure::PotLimit,
            BettingState::FixedLimit {
                small_bet,
                big_bet,
                raise_cap,
            } => BettingStructure::FixedLimit {
                small_bet,
                big_bet,
                raise_cap,
            },
        }
    }
}

/// One seat, flattened for the wire.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SeatState {
    pub id: Uuid,
    pub handle: String,
    pub chips: usize,
    pub bet: usize,
    pub chips_in_play: usize,
    pub withdrawn: usize,
    pub state: PlayerState,
    /// One entry per slot in the seat's [`BoxedCards`]; `None` is a blank slot.
    pub cards: Vec<Option<String>>,
    /// `(index, is_up)` per card in the seat's [`SeatHand`], in order.
    /// Visibility is carried as a bool rather than the enum so the wire format
    /// does not pin [`Visibility`]'s variant order.
    pub hand: Vec<(String, bool)>,
    /// [`SeatHand::seat`]'s own value, carried verbatim.
    ///
    /// It is **not** the seat's index, and deriving it from the index would
    /// make `snapshot` → `restore` something other than an identity. Today
    /// `Seat::new` always builds `SeatHand::new(0)` (`table/seat.rs:51,73,101`),
    /// so this is `0` on every seat regardless of position, and nothing in the
    /// engine ever reads it back. A snapshot reproduces what is there rather
    /// than quietly correcting it; if that field is ever given real meaning,
    /// old snapshots stay faithful to what they captured.
    pub hand_seat: u8,
    pub bet_level_when_last_acted: usize,
}

/// The written-down form of a [`Table`]. See the [module docs](self).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TableState {
    pub version: u16,
    pub id: Uuid,
    pub name: String,
    pub game: GameType,
    pub betting: BettingState,
    pub phase: GamePhase,
    pub forced: ForcedBets,
    pub button: u8,
    /// Ordered. **The undealt remainder — i.e. the future of the hand.**
    pub deck: Vec<String>,
    pub board: Vec<String>,
    pub muck: Vec<String>,
    pub pot: usize,
    pub bet: usize,
    pub raise_increment: usize,
    pub hand_chip_total: usize,
    pub raises_this_street: u8,
    pub actions_this_street: u8,
    pub chip_actions_this_street: u8,
    pub blind_shortfall: usize,
    pub seats: Vec<SeatState>,
    /// `(seat, cards)`, sorted by seat so the bytes are deterministic.
    pub dealt_hole_cards: Vec<(u8, Vec<Option<String>>)>,
    pub event_log: Vec<TableAction>,
}

fn pile_to_strings(cards: &Cards) -> Vec<String> {
    cards.iter().map(Card::to_string).collect()
}

fn strings_to_pile(raw: &[String]) -> Result<Cards, PKError> {
    let mut cards = Cards::default();
    for index in raw {
        cards.insert(card_from_index(index)?);
    }
    Ok(cards)
}

/// Parses one card index, accepting the blank sentinel that
/// [`Card::from_str`] rejects by design.
fn card_from_index(index: &str) -> Result<Card, PKError> {
    if index.trim() == Card::BLANK_INDEX {
        return Ok(Card::BLANK);
    }
    Card::from_str(index)
}

fn boxed_to_slots(boxed: &BoxedCards) -> Vec<Option<String>> {
    boxed
        .as_slice()
        .iter()
        .map(|card| {
            if *card == Card::BLANK {
                None
            } else {
                Some(card.to_string())
            }
        })
        .collect()
}

fn slots_to_boxed(slots: &[Option<String>]) -> Result<BoxedCards, PKError> {
    let mut cards: Vec<Card> = Vec::with_capacity(slots.len());
    for slot in slots {
        match slot {
            None => cards.push(Card::BLANK),
            Some(index) => cards.push(card_from_index(index)?),
        }
    }
    Ok(BoxedCards::from(cards))
}

impl From<&Table> for TableState {
    /// Infallible: reading a table always works. The inverse is
    /// [`TryFrom<&TableState>`], because parsing can fail.
    fn from(table: &Table) -> Self {
        let seats = table
            .seats
            .iter()
            .map(|seat| SeatState {
                id: seat.player.id,
                handle: seat.player.handle.clone(),
                chips: seat.player.chips,
                bet: seat.player.bet,
                chips_in_play: seat.player.chips_in_play,
                withdrawn: seat.player.withdrawn,
                state: seat.player.state,
                cards: boxed_to_slots(&seat.cards),
                hand: seat
                    .hand
                    .iter()
                    .map(|hole| (hole.card().to_string(), hole.is_up()))
                    .collect(),
                hand_seat: seat.hand.seat(),
                bet_level_when_last_acted: seat.bet_level_when_last_acted,
            })
            .collect();

        // Sorted, because `HashMap` iteration order is unspecified and two
        // snapshots of the same table must compare equal.
        let mut dealt: Vec<(u8, Vec<Option<String>>)> = table
            .dealt_hole_cards
            .iter()
            .map(|(seat, boxed)| (*seat, boxed_to_slots(boxed)))
            .collect();
        dealt.sort_by_key(|(seat, _)| *seat);

        TableState {
            version: SNAPSHOT_VERSION,
            id: table.id,
            name: table.name.clone(),
            game: table.game,
            betting: BettingState::from(table.betting),
            phase: table.phase,
            forced: table.forced,
            button: table.button,
            deck: pile_to_strings(&table.deck),
            board: pile_to_strings(&table.board),
            muck: pile_to_strings(&table.muck),
            pot: table.pot,
            bet: table.bet,
            raise_increment: table.raise_increment,
            hand_chip_total: table.hand_chip_total,
            raises_this_street: table.raises_this_street,
            actions_this_street: table.actions_this_street,
            chip_actions_this_street: table.chip_actions_this_street,
            blind_shortfall: table.blind_shortfall,
            seats,
            dealt_hole_cards: dealt,
            event_log: table.event_log.clone(),
        }
    }
}

impl From<Table> for TableState {
    fn from(table: Table) -> Self {
        TableState::from(&table)
    }
}

impl TryFrom<&TableState> for Table {
    type Error = PKError;

    /// Rebuilds a live table. Fallible because every card index is re-parsed.
    ///
    /// # Errors
    ///
    /// - [`PKError::SnapshotVersion`] if `version` is not [`SNAPSHOT_VERSION`].
    ///   Checked first, so a mismatched payload is never half-applied.
    /// - [`PKError::InvalidCardIndex`] if any card index does not parse.
    fn try_from(state: &TableState) -> Result<Self, Self::Error> {
        if state.version != SNAPSHOT_VERSION {
            return Err(PKError::SnapshotVersion {
                found: state.version,
                expected: SNAPSHOT_VERSION,
            });
        }

        let mut seats: Vec<Seat> = Vec::with_capacity(state.seats.len());
        for seat_state in &state.seats {
            let mut player = Player::new(seat_state.handle.clone());
            player.id = seat_state.id;
            player.chips = seat_state.chips;
            player.bet = seat_state.bet;
            player.chips_in_play = seat_state.chips_in_play;
            player.withdrawn = seat_state.withdrawn;
            player.state = seat_state.state;

            // `SeatHand`'s fields are private, so it is rebuilt through `push`,
            // which is also what preserves per-card visibility for the stud
            // family.
            let mut hand = SeatHand::new(seat_state.hand_seat);
            for (card_index, is_up) in &seat_state.hand {
                let visibility = if *is_up { Visibility::Up } else { Visibility::Down };
                hand.push(card_from_index(card_index)?, visibility);
            }

            let mut seat = Seat::new_with_cards(player, slots_to_boxed(&seat_state.cards)?);
            seat.hand = hand;
            seat.bet_level_when_last_acted = seat_state.bet_level_when_last_acted;
            seats.push(seat);
        }

        let mut dealt_hole_cards = std::collections::HashMap::new();
        for (seat, slots) in &state.dealt_hole_cards {
            dealt_hole_cards.insert(*seat, slots_to_boxed(slots)?);
        }

        Ok(Table {
            id: state.id,
            name: state.name.clone(),
            game: state.game,
            forced: state.forced,
            phase: state.phase,
            seats: Seats::new(seats),
            button: state.button,
            deck: strings_to_pile(&state.deck)?,
            board: strings_to_pile(&state.board)?,
            muck: strings_to_pile(&state.muck)?,
            pot: state.pot,
            bet: state.bet,
            raise_increment: state.raise_increment,
            event_log: state.event_log.clone(),
            hand_chip_total: state.hand_chip_total,
            dealt_hole_cards,
            betting: BettingStructure::from(state.betting),
            raises_this_street: state.raises_this_street,
            actions_this_street: state.actions_this_street,
            chip_actions_this_street: state.chip_actions_this_street,
            blind_shortfall: state.blind_shortfall,
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__snapshot_tests {
    use super::*;
    use crate::casino::action::PlayerAction;

    fn two_player_table() -> Table {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("Alice".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Bob".to_string(), 10_000)),
        ]);
        Table::nlh_from_seats(seats, ForcedBets::new(50, 100))
    }

    /// Mid-hand: blinds posted and hole cards dealt, so the table carries a
    /// partial deck, dealt hole cards, live bets and a populated event log.
    fn mid_hand_table() -> Table {
        let mut table = two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        table
    }

    #[test]
    fn snapshot_round_trips_a_fresh_table() {
        let table = two_player_table();
        let restored = Table::restore(&table.snapshot().unwrap()).unwrap();
        assert_eq!(table, restored);
    }

    #[test]
    fn snapshot_round_trips_a_mid_hand_table() {
        let table = mid_hand_table();
        let restored = Table::restore(&table.snapshot().unwrap()).unwrap();
        assert_eq!(table, restored);
    }

    /// The undealt remainder is the hand's *future*. An unordered round-trip
    /// would still compare equal on every scalar field while silently changing
    /// every runout from here on, so the order gets its own assertion.
    #[test]
    fn snapshot_preserves_deck_order() {
        let table = mid_hand_table();
        let restored = Table::restore(&table.snapshot().unwrap()).unwrap();

        let before: Vec<Card> = table.deck.iter().copied().collect();
        let after: Vec<Card> = restored.deck.iter().copied().collect();
        assert_eq!(before, after);
        assert!(before.len() > 40, "a mid-hand deck still holds most of the pack");
    }

    /// A seat holds `BoxedCards::blanks(n)` before the deal. Blank and dealt
    /// must stay distinguishable, which is why the DTO uses
    /// `Vec<Option<String>>` rather than a card list.
    #[test]
    fn snapshot_preserves_blank_seat_cards() {
        let table = two_player_table();
        assert!(
            table.seats.iter().all(|seat| !seat.cards.is_dealt()),
            "pre-deal seats hold blanks"
        );

        let restored = Table::restore(&table.snapshot().unwrap()).unwrap();
        for (before, after) in table.seats.iter().zip(restored.seats.iter()) {
            assert_eq!(before.cards, after.cards);
            assert_eq!(before.cards.len(), after.cards.len());
        }
    }

    /// `Table` carries `dealt_hole_cards` as a `HashMap`, whose iteration order
    /// is unspecified. If the DTO stored a map, two snapshots of one table
    /// could differ byte-for-byte; the sorted `Vec` is what prevents that.
    #[test]
    fn snapshot_is_deterministic() {
        let table = mid_hand_table();
        assert_eq!(table.snapshot().unwrap(), table.snapshot().unwrap());
        assert_eq!(TableState::from(&table), TableState::from(&table));
    }

    #[test]
    fn restore_rejects_garbage_bytes() {
        assert_eq!(Err(PKError::SnapshotCorrupt), Table::restore(&[0xff, 0xfe, 0xfd]));
        assert_eq!(Err(PKError::SnapshotCorrupt), Table::restore(&[]));
    }

    /// The version tag is checked before any other field, so a payload from a
    /// future build is refused whole rather than half-applied.
    #[test]
    fn restore_rejects_an_unknown_version() {
        let mut state = TableState::from(&mid_hand_table());
        state.version = SNAPSHOT_VERSION + 1;

        let err = Table::try_from(&state).unwrap_err();
        assert_eq!(
            PKError::SnapshotVersion {
                found: SNAPSHOT_VERSION + 1,
                expected: SNAPSHOT_VERSION,
            },
            err
        );
    }

    /// The pay-off from hardening `deserialize_card_index` (EPIC-88 Phase 0a):
    /// an unparseable index is an error rather than a board full of blanks.
    #[test]
    fn restore_rejects_an_unparseable_card() {
        let mut state = TableState::from(&mid_hand_table());
        state.deck[0] = "NOT_A_CARD".to_string();

        assert_eq!(Err(PKError::InvalidCardIndex), Table::try_from(&state));
    }

    /// `Card::BLANK` writes itself as `"__"` but `Card::from_str` refuses to
    /// read that back, so blanks round-tripped only by accident before 0.11.0.
    /// A mid-hand table is full of them.
    #[test]
    fn snapshot_carries_blank_cards() {
        let mut state = TableState::from(&two_player_table());
        assert!(
            state.seats.iter().all(|seat| seat.cards.iter().all(Option::is_none)),
            "undealt slots serialize as None"
        );
        state.version = SNAPSHOT_VERSION;
        assert!(Table::try_from(&state).is_ok());
    }

    /// The behaviour that matters: a hand interrupted mid-street and resumed
    /// from bytes must play out **identically** to the same hand played
    /// straight through. Everything else in this module is shape; this is the
    /// requirement.
    #[test]
    fn snapshot_mid_street_resumes_to_identical_winnings() {
        use crate::casino::session::{PokerSession, SessionStep};

        /// Drives the canonical step loop to the end, checking where legal and
        /// calling otherwise, and returns the payout.
        fn play_out(session: &mut PokerSession) -> crate::casino::winnings::Winnings {
            loop {
                match session.next_step() {
                    SessionStep::PlayerToAct(seat) => {
                        let legal = session.table.legal_actions(seat);
                        let action = if legal.contains(&PlayerAction::Check) {
                            PlayerAction::Check
                        } else {
                            PlayerAction::Call
                        };
                        session.apply_action(seat, action).unwrap();
                    }
                    SessionStep::StreetAdvanced => {}
                    SessionStep::HandComplete => return session.end_hand().unwrap(),
                    SessionStep::Failed(e) => panic!("hand could not complete: {e:?}"),
                }
            }
        }

        let mut control = PokerSession::new(two_player_table());
        control.start_hand().unwrap();

        // Snapshot mid-street: one seat has acted, the other has not.
        if let SessionStep::PlayerToAct(seat) = control.next_step() {
            control.apply_action(seat, PlayerAction::Call).unwrap();
        }

        let restored = Table::restore(&control.table.snapshot().unwrap()).unwrap();
        assert_eq!(control.table, restored, "the resume starts from identical state");

        let mut resumed = PokerSession::new(restored);
        resumed.hand_number = control.hand_number;

        let control_winnings = play_out(&mut control);
        let resumed_winnings = play_out(&mut resumed);

        assert_eq!(control_winnings, resumed_winnings);
        assert_eq!(control.table.table_chip_count(), resumed.table.table_chip_count());
        assert_eq!(20_000, resumed.table.table_chip_count());
    }

    fn four_seats() -> Seats {
        Seats::new(vec![
            Seat::new(Player::new_with_chips("Alice".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Bob".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Carol".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Dave".to_string(), 10_000)),
        ])
    }

    #[test]
    fn snapshot_round_trips_plo() {
        let mut table = Table::plo_from_seats(four_seats(), (50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        let restored = Table::restore(&table.snapshot().unwrap()).unwrap();
        assert_eq!(table, restored);
        assert_eq!(GameType::PLO, restored.game);
    }

    /// Fixed-Limit carries a `BettingStructure::FixedLimit { .. }` payload,
    /// which is the variant the postcard-safe [`BettingState`] mirror exists
    /// for — the real enum is internally tagged and cannot decode from a
    /// non-self-describing format.
    #[test]
    fn snapshot_round_trips_fixed_limit_payload() {
        let mut table = Table::nlh_from_seats(four_seats(), ForcedBets::new(50, 100));
        table.betting = BettingStructure::FixedLimit {
            small_bet: 100,
            big_bet: 200,
            raise_cap: 3,
        };

        let restored = Table::restore(&table.snapshot().unwrap()).unwrap();
        assert_eq!(table.betting, restored.betting);
        assert_eq!(
            BettingStructure::FixedLimit {
                small_bet: 100,
                big_bet: 200,
                raise_cap: 3
            },
            restored.betting
        );
    }

    /// The stud family is the case a card-list-only DTO loses silently: each
    /// seat's [`SeatHand`] carries per-card [`Visibility`], and an up-card that
    /// comes back down changes what every opponent can legally reason about.
    #[test]
    fn snapshot_preserves_stud_up_card_visibility() {
        let mut table = Table::stud_hi_from_seats(four_seats(), 10, 20, 50, 100).unwrap();
        table.act_forced_bets().unwrap();
        table.deal_stud_3rd_street().unwrap();

        let up_before: usize = table.seats.iter().map(|seat| seat.hand.up_cards().count()).sum();
        assert!(up_before > 0, "third street deals one up-card per seat");

        let restored = Table::restore(&table.snapshot().unwrap()).unwrap();
        assert_eq!(table, restored);

        let up_after: usize = restored.seats.iter().map(|seat| seat.hand.up_cards().count()).sum();
        assert_eq!(up_before, up_after);
        for (before, after) in table.seats.iter().zip(restored.seats.iter()) {
            assert_eq!(
                before.hand.up_cards().collect::<Vec<_>>(),
                after.hand.up_cards().collect::<Vec<_>>()
            );
            assert_eq!(
                before.hand.down_cards().collect::<Vec<_>>(),
                after.hand.down_cards().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn snapshot_round_trips_razz() {
        let mut table = Table::razz_from_seats(four_seats(), 10, 20, 50, 100).unwrap();
        table.act_forced_bets().unwrap();
        table.deal_stud_3rd_street().unwrap();

        let restored = Table::restore(&table.snapshot().unwrap()).unwrap();
        assert_eq!(table, restored);
        assert_eq!(GameType::Razz, restored.game);
    }

    /// Chips are conserved across the process boundary: `hand_chip_total` is
    /// carried, so the books still balance after a resume.
    #[test]
    fn snapshot_survives_audit_chip_total() {
        let table = mid_hand_table();
        let mut resumed = Table::restore(&table.snapshot().unwrap()).unwrap();

        assert_eq!(table.hand_chip_total, resumed.hand_chip_total);
        assert_eq!(20_000, resumed.hand_chip_total);
        assert!(resumed.audit_chip_total().is_ok());
    }
}
