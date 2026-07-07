use crate::card::Card;
use crate::casino::table_celled::position::Position;
use crate::pokerbench::action::PokerBenchAction;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The big-blind chip unit assumed for PokerBench scenarios.
///
/// PokerBench expresses pot and bet sizes already in big blinds (a pot of
/// `24.0` means 24 bb), so the unit baseline is `1`.
pub const PB_BIG_BLIND: u32 = 1;

/// The effective starting stack (in big blinds) seeded for each active position.
///
/// PokerBench's structured forms carry no per-position stacks; the standard
/// assumption for the dataset is 100 bb deep, which we surface so a downstream
/// seat-indexed state (e.g. pkdealer's `HandState`) is well-formed. Documented as
/// a convention, not data parsed from the item.
pub const PB_EFFECTIVE_STACK: u32 = 100;

/// Which PokerBench split a scenario came from.
///
/// PokerBench reports metrics separately for the two splits, so the split is
/// carried through to aggregation.
///
/// # Examples
/// ```
/// use pkcore::pokerbench::PokerBenchSplit;
///
/// assert_eq!(PokerBenchSplit::Preflop.to_string(), "preflop");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PokerBenchSplit {
    /// The pre-flop split (positions only, empty board).
    Preflop,
    /// The post-flop split (flop/turn/river board present).
    Postflop,
}

impl fmt::Display for PokerBenchSplit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PokerBenchSplit::Preflop => write!(f, "preflop"),
            PokerBenchSplit::Postflop => write!(f, "postflop"),
        }
    }
}

/// One PokerBench item: a parsed 6-max No-Limit Hold'em decision point plus the
/// solver-optimal action.
///
/// Analysis-only; constructed by the loaders
/// ([`load_csv`](PokerBenchScenario::load_csv) /
/// [`load_json`](PokerBenchScenario::load_json)). Sizes (`pot`, `to_call`, and
/// the amounts inside `optimal`/`history`/`legal`) are in the dataset's native
/// big-blind unit (see [`PB_BIG_BLIND`]).
///
/// # Examples
/// ```
/// use pkcore::pokerbench::{PokerBenchAction, PokerBenchScenario, PokerBenchSplit};
/// use pkcore::casino::table_celled::position::Position;
///
/// let scenario = PokerBenchScenario {
///     instruction: "...".to_string(),
///     hero: Position::BTN,
///     board: vec![],
///     hole: vec![],
///     pot: 3,
///     to_call: 1,
///     big_blind: 1,
///     stacks: vec![(Position::BTN, 100), (Position::BB, 100)],
///     history: vec![],
///     legal: vec![PokerBenchAction::Fold, PokerBenchAction::Call],
///     optimal: PokerBenchAction::Call,
///     split: PokerBenchSplit::Preflop,
/// };
/// assert_eq!(scenario.optimal, PokerBenchAction::Call);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PokerBenchScenario {
    /// Original natural-language instruction text (the LLM prompt).
    pub instruction: String,
    /// Hero position (one of UTG/HJ/CO/BTN/SB/BB for 6-max PokerBench).
    pub hero: Position,
    /// Community cards (empty pre-flop).
    pub board: Vec<Card>,
    /// Hero hole cards.
    pub hole: Vec<Card>,
    /// Pot before the hero acts (chips/bb).
    pub pot: u32,
    /// Chips the hero must call (`0` = check available).
    pub to_call: u32,
    /// Big blind (chip unit baseline).
    pub big_blind: u32,
    /// Per-position stacks at the decision point.
    pub stacks: Vec<(Position, u32)>,
    /// Action line leading to the decision, in order.
    pub history: Vec<PokerBenchAction>,
    /// Legal moves offered at the decision point.
    pub legal: Vec<PokerBenchAction>,
    /// The solver-optimal action (the label being predicted).
    pub optimal: PokerBenchAction,
    /// Which split this came from.
    pub split: PokerBenchSplit,
}

/// One seat in a scenario's canonical 6-max seating.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalSeat {
    /// 0-based seat index (button at seat 0; see [`PokerBenchScenario::canonical_seating`]).
    pub seat: u8,
    /// The PokerBench position occupying this seat.
    pub position: Position,
    /// Synthesized player name (the short position code, e.g. `"BTN"`).
    pub name: String,
    /// Stack at the decision point.
    pub chips: u32,
}

/// Canonical, seat-indexed view of a scenario for seat-based downstream state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalSeating {
    /// The seat the hero occupies.
    pub hero_seat: u8,
    /// Occupied seats, ascending by seat index.
    pub seats: Vec<CanonicalSeat>,
}

/// Maps a PokerBench 6-max position to a 0-based seat using pkcore's
/// button-relative convention (button at seat 0): BTN→0, SB→1, BB→2, then UTG→3,
/// HJ→4, CO→5. Returns `None` for positions outside the 6-max PokerBench set.
fn pb_seat(position: Position) -> Option<u8> {
    Some(match position {
        Position::BTN => 0,
        Position::SB => 1,
        Position::BB => 2,
        Position::UTG => 3,
        Position::HJ => 4,
        Position::CO => 5,
        _ => return None,
    })
}

/// The short position code used to synthesize a seat's player name.
fn pb_position_name(position: Position) -> &'static str {
    match position {
        Position::BTN => "BTN",
        Position::SB => "SB",
        Position::BB => "BB",
        Position::UTG => "UTG",
        Position::HJ => "HJ",
        Position::CO => "CO",
        _ => "?",
    }
}

impl PokerBenchScenario {
    /// Canonical seating for this scenario.
    ///
    /// Resolves each position in [`stacks`](PokerBenchScenario::stacks) to a
    /// 0-based seat (button at seat 0, mirroring pkcore's
    /// [`Position::from_seat`](crate::casino::table_celled::position::Position::from_seat)
    /// convention), synthesizes a player name from the position, and identifies
    /// the hero's seat. Positions outside the 6-max PokerBench set are skipped.
    /// This decides the position→seat mapping once, in the library, so a
    /// downstream seat-indexed state (e.g. pkdealer's `HandState`) is a trivial
    /// field map.
    ///
    /// # Examples
    /// ```
    /// use pkcore::pokerbench::{PokerBenchAction, PokerBenchScenario, PokerBenchSplit};
    /// use pkcore::casino::table_celled::position::Position;
    ///
    /// let scenario = PokerBenchScenario {
    ///     instruction: String::new(),
    ///     hero: Position::CO,
    ///     board: vec![],
    ///     hole: vec![],
    ///     pot: 3,
    ///     to_call: 1,
    ///     big_blind: 1,
    ///     stacks: vec![(Position::CO, 100), (Position::BTN, 100), (Position::BB, 100)],
    ///     history: vec![],
    ///     legal: vec![],
    ///     optimal: PokerBenchAction::Fold,
    ///     split: PokerBenchSplit::Preflop,
    /// };
    /// let seating = scenario.canonical_seating();
    /// assert_eq!(seating.hero_seat, 5); // CO sits at seat 5
    /// assert_eq!(seating.seats[0].seat, 0); // BTN first, ascending
    /// assert_eq!(seating.seats[0].name, "BTN");
    /// ```
    #[must_use]
    pub fn canonical_seating(&self) -> CanonicalSeating {
        let mut seats = Vec::with_capacity(self.stacks.len());
        let mut hero_seat = 0u8;
        for (position, chips) in &self.stacks {
            if let Some(seat) = pb_seat(*position) {
                if *position == self.hero {
                    hero_seat = seat;
                }
                seats.push(CanonicalSeat {
                    seat,
                    position: *position,
                    name: pb_position_name(*position).to_string(),
                    chips: *chips,
                });
            }
        }
        seats.sort_by_key(|s| s.seat);
        CanonicalSeating { hero_seat, seats }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod pokerbench__tests {
    use super::*;

    fn sample(hero: Position, stacks: Vec<(Position, u32)>) -> PokerBenchScenario {
        PokerBenchScenario {
            instruction: "test".to_string(),
            hero,
            board: vec![],
            hole: vec![],
            pot: 6,
            to_call: 2,
            big_blind: PB_BIG_BLIND,
            stacks,
            history: vec![],
            legal: vec![PokerBenchAction::Fold, PokerBenchAction::Call],
            optimal: PokerBenchAction::Call,
            split: PokerBenchSplit::Preflop,
        }
    }

    #[test]
    fn split_display() {
        assert_eq!(PokerBenchSplit::Preflop.to_string(), "preflop");
        assert_eq!(PokerBenchSplit::Postflop.to_string(), "postflop");
    }

    #[test]
    fn consts_have_expected_values() {
        assert_eq!(PB_BIG_BLIND, 1);
        assert_eq!(PB_EFFECTIVE_STACK, 100);
    }

    #[test]
    fn canonical_seating_resolves_all_six_positions() {
        let stacks = vec![
            (Position::UTG, 100),
            (Position::HJ, 100),
            (Position::CO, 100),
            (Position::BTN, 100),
            (Position::SB, 100),
            (Position::BB, 100),
        ];
        let seating = sample(Position::UTG, stacks).canonical_seating();
        let by_pos: Vec<(Position, u8)> = seating.seats.iter().map(|s| (s.position, s.seat)).collect();
        assert!(by_pos.contains(&(Position::BTN, 0)));
        assert!(by_pos.contains(&(Position::SB, 1)));
        assert!(by_pos.contains(&(Position::BB, 2)));
        assert!(by_pos.contains(&(Position::UTG, 3)));
        assert!(by_pos.contains(&(Position::HJ, 4)));
        assert!(by_pos.contains(&(Position::CO, 5)));
    }

    #[test]
    fn canonical_seating_identifies_hero() {
        let stacks = vec![(Position::BTN, 100), (Position::BB, 100)];
        let seating = sample(Position::BB, stacks).canonical_seating();
        assert_eq!(seating.hero_seat, 2);
    }

    #[test]
    fn canonical_seating_is_sorted_ascending() {
        let stacks = vec![(Position::CO, 100), (Position::SB, 100), (Position::BTN, 100)];
        let seating = sample(Position::CO, stacks).canonical_seating();
        let seats: Vec<u8> = seating.seats.iter().map(|s| s.seat).collect();
        assert_eq!(seats, vec![0, 1, 5]);
    }

    #[test]
    fn canonical_seating_preserves_stacks() {
        let stacks = vec![(Position::BTN, 80), (Position::BB, 120)];
        let seating = sample(Position::BTN, stacks).canonical_seating();
        let btn = seating.seats.iter().find(|s| s.position == Position::BTN).unwrap();
        assert_eq!(btn.chips, 80);
    }

    #[test]
    fn canonical_seat_name_is_short_code() {
        let stacks = vec![(Position::HJ, 100)];
        let seating = sample(Position::HJ, stacks).canonical_seating();
        assert_eq!(seating.seats[0].name, "HJ");
    }

    #[test]
    fn scenario_survives_serde_round_trip() {
        let stacks = vec![(Position::BTN, 100), (Position::BB, 100)];
        let scenario = sample(Position::BTN, stacks);
        let json = serde_json::to_string(&scenario).unwrap();
        let back: PokerBenchScenario = serde_json::from_str(&json).unwrap();
        assert_eq!(scenario, back);
    }
}
