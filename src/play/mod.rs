use crate::PKError;
use std::fmt::{Display, Formatter};
use strum_macros::{EnumCount, EnumIter};

pub mod actions;
pub mod board;
pub mod game;
pub mod hole_cards;
pub mod phases;
pub mod positions;
pub mod stages;

#[derive(Clone, Copy, Debug, Default, EnumCount, EnumIter, Eq, Hash, PartialEq)]
pub enum Position6Max {
    #[default]
    SB = 1,
    BB = 2,
    UTG = 3,
    MP = 4,
    CO = 5,
    BTN = 6,
}

impl Position6Max {
    #[must_use]
    pub fn description(&self) -> &str {
        match *self {
            Position6Max::SB => "Small Blind",
            Position6Max::BB => "Big Blind",
            Position6Max::UTG => "Under the Gun",
            Position6Max::MP => "Middle Position",
            Position6Max::CO => "Cutoff",
            Position6Max::BTN => "The Button",
        }
    }

    #[must_use]
    pub fn next(&self) -> Self {
        match *self {
            Position6Max::SB => Position6Max::BB,
            Position6Max::BB => Position6Max::UTG,
            Position6Max::UTG => Position6Max::MP,
            Position6Max::MP => Position6Max::CO,
            Position6Max::CO => Position6Max::BTN,
            Position6Max::BTN => Position6Max::SB,
        }
    }
}

impl Display for Position6Max {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Position6Max::SB => write!(f, "SB"),
            Position6Max::BB => write!(f, "BB"),
            Position6Max::UTG => write!(f, "UTG"),
            Position6Max::MP => write!(f, "MP"),
            Position6Max::CO => write!(f, "CO"),
            Position6Max::BTN => write!(f, "BTN"),
        }
    }
}

impl TryFrom<u8> for Position6Max {
    type Error = PKError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Position6Max::SB),
            2 => Ok(Position6Max::BB),
            3 => Ok(Position6Max::UTG),
            4 => Ok(Position6Max::MP),
            5 => Ok(Position6Max::CO),
            6 => Ok(Position6Max::BTN),
            _ => Err(PKError::InvalidPosition),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__tests {
    use super::*;

    #[test]
    fn value() {
        assert_eq!(6, Position6Max::BTN as u8);
    }

    #[test]
    fn next() {
        assert_eq!(Position6Max::BB, Position6Max::SB.next());
        assert_eq!(Position6Max::UTG, Position6Max::BB.next());
        assert_eq!(Position6Max::MP, Position6Max::UTG.next());
        assert_eq!(Position6Max::CO, Position6Max::MP.next());
        assert_eq!(Position6Max::BTN, Position6Max::CO.next());
        assert_eq!(Position6Max::SB, Position6Max::BTN.next());
    }
}
