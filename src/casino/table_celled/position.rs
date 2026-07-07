use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use strum_macros::{EnumCount, EnumIter};

#[derive(
    Clone, Copy, Debug, Default, Ord, PartialOrd, EnumCount, EnumIter, Eq, Hash, PartialEq, Serialize, Deserialize,
)]
pub enum Position {
    #[default]
    SB = 1,
    BB = 2,
    UTG = 3,
    UTGP1 = 4,
    UTGP2 = 5,
    EP = 6,
    MP = 7,
    LJ = 8,
    HJ = 9,
    CO = 10,
    BTN = 11,
}

impl Position {
    /// Derives the `Position` for `seat` given the dealer `button` seat index
    /// and the total number of occupied `seat_count` seats.
    ///
    /// Uses clockwise offset arithmetic: `offset = (seat − button + seat_count) % seat_count`.
    /// Returns `None` for unsupported table sizes (anything other than 2, 3, 4, 5, 6, 9).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_celled::position::Position;
    ///
    /// // 6-max, button at seat 0
    /// assert_eq!(Some(Position::BTN), Position::from_seat(0, 0, 6));
    /// assert_eq!(Some(Position::SB),  Position::from_seat(1, 0, 6));
    /// assert_eq!(Some(Position::BB),  Position::from_seat(2, 0, 6));
    /// assert_eq!(Some(Position::CO),  Position::from_seat(5, 0, 6));
    ///
    /// // Unsupported size returns None
    /// assert_eq!(None, Position::from_seat(0, 0, 7));
    /// ```
    #[must_use]
    pub fn from_seat(seat: u8, button: u8, seat_count: u8) -> Option<Position> {
        if seat_count == 0 {
            return None;
        }
        // Defense-in-depth: caller must pass logical (button-relative) indices
        // — `seat` and `button` should both be in `0..seat_count`. If `button`
        // exceeds `seat + seat_count` (the caller forgot to translate physical
        // → logical for sparse seating), return None instead of panicking.
        let offset = (seat as usize + seat_count as usize).checked_sub(button as usize)? % seat_count as usize;
        match (seat_count, offset) {
            (2 | 3 | 4 | 5 | 6 | 9, 0) => Some(Position::BTN),
            (3 | 4 | 5 | 6 | 9, 1) => Some(Position::SB),
            // BB is offset 1 in heads-up, offset 2 for 3+ players.
            (2, 1) | (3 | 4 | 5 | 6 | 9, 2) => Some(Position::BB),
            // UTG is offset 3 for 4-max, 5-max, and 9-max (6-max uses LJ at offset 3).
            (4 | 5 | 9, 3) => Some(Position::UTG),
            (9, 4) => Some(Position::UTGP1),
            (9, 5) => Some(Position::EP),
            // LJ and HJ appear at different offsets in 6-max and 9-max.
            (6, 3) | (9, 6) => Some(Position::LJ),
            (6, 4) | (9, 7) => Some(Position::HJ),
            (5, 4) | (6, 5) | (9, 8) => Some(Position::CO),
            _ => None,
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Position::SB => write!(f, "Small Blind"),
            Position::BB => write!(f, "Big Blind"),
            Position::UTG => write!(f, "Under the Gun"),
            Position::UTGP1 => write!(f, "Under the Gun +1"),
            Position::UTGP2 => write!(f, "Under the Gun +2"),
            Position::EP => write!(f, "Early Position"),
            Position::MP => write!(f, "Middle Position"),
            Position::LJ => write!(f, "Lojack"),
            Position::HJ => write!(f, "Hijack"),
            Position::CO => write!(f, "Cutoff"),
            Position::BTN => write!(f, "Button"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Positions(Vec<Position>);

impl Positions {
    #[must_use]
    pub fn heads_up() -> Self {
        Positions(vec![Position::BB, Position::BTN])
    }

    #[must_use]
    pub fn three_handed() -> Self {
        Positions(vec![Position::BTN, Position::SB, Position::BB])
    }

    #[must_use]
    pub fn four_handed() -> Self {
        Positions(vec![Position::UTG, Position::BTN, Position::SB, Position::BB])
    }

    #[must_use]
    pub fn five_handed() -> Self {
        Positions(vec![
            Position::UTG,
            Position::CO,
            Position::BTN,
            Position::SB,
            Position::BB,
        ])
    }

    #[must_use]
    pub fn six_handed() -> Self {
        Positions(vec![
            Position::LJ,
            Position::HJ,
            Position::CO,
            Position::BTN,
            Position::SB,
            Position::BB,
        ])
    }

    #[must_use]
    pub fn nine_handed() -> Self {
        Positions(vec![
            Position::UTG,
            Position::UTGP1,
            Position::EP,
            Position::LJ,
            Position::HJ,
            Position::CO,
            Position::BTN,
            Position::SB,
            Position::BB,
        ])
    }

    /// Consumes this `Positions` and returns the inner `Vec<Position>`.
    #[must_use]
    pub fn into_inner(self) -> Vec<Position> {
        self.0
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__position_tests {
    use super::*;

    #[test]
    fn from_seat_heads_up() {
        // Button at seat 0.
        assert_eq!(Some(Position::BTN), Position::from_seat(0, 0, 2));
        assert_eq!(Some(Position::BB), Position::from_seat(1, 0, 2));
        // Button at seat 1.
        assert_eq!(Some(Position::BTN), Position::from_seat(1, 1, 2));
        assert_eq!(Some(Position::BB), Position::from_seat(0, 1, 2));
    }

    #[test]
    fn from_seat_six_max() {
        // Button at seat 3 in a 6-seat game.
        let btn = 3u8;
        assert_eq!(Some(Position::BTN), Position::from_seat(3, btn, 6));
        assert_eq!(Some(Position::SB), Position::from_seat(4, btn, 6));
        assert_eq!(Some(Position::BB), Position::from_seat(5, btn, 6));
        assert_eq!(Some(Position::LJ), Position::from_seat(0, btn, 6));
        assert_eq!(Some(Position::HJ), Position::from_seat(1, btn, 6));
        assert_eq!(Some(Position::CO), Position::from_seat(2, btn, 6));
    }

    #[test]
    fn from_seat_nine_max_round_trip() {
        // All 9 positions with button at seat 0.
        let expected = [
            Position::BTN,
            Position::SB,
            Position::BB,
            Position::UTG,
            Position::UTGP1,
            Position::EP,
            Position::LJ,
            Position::HJ,
            Position::CO,
        ];
        for (seat, &pos) in expected.iter().enumerate() {
            assert_eq!(
                Some(pos),
                Position::from_seat(seat as u8, 0, 9),
                "seat {seat} with button 0 in 9-max"
            );
        }
    }

    #[test]
    fn from_seat_unsupported_size_returns_none() {
        assert_eq!(None, Position::from_seat(0, 0, 0));
        assert_eq!(None, Position::from_seat(0, 0, 7));
        assert_eq!(None, Position::from_seat(0, 0, 8));
    }

    #[test]
    fn from_seat_button_overflow_returns_none() {
        // Tripwire: a caller passing physical seat indices into an API that
        // expects logical (button-relative) indices used to panic with
        // `attempt to subtract with overflow`. The checked_sub guard turns
        // that class of bug into a None return so consumers can detect &
        // recover rather than crash. Concrete trigger: seat=0, seat_count=3,
        // button=5 (e.g. a 6-max table where the BB and CO were eliminated
        // and the caller forgot to translate physical→logical).
        assert_eq!(None, Position::from_seat(0, 5, 3));
        assert_eq!(None, Position::from_seat(1, 9, 2));
        // Boundary: button == seat + seat_count is fine (subtracts to zero).
        assert_eq!(Some(Position::BTN), Position::from_seat(1, 3, 2));
    }
}
