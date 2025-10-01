use std::fmt::Formatter;
use strum_macros::{EnumCount, EnumIter};

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, EnumCount, EnumIter, Eq, Hash, PartialEq)]
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
}
