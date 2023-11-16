use strum_macros::{EnumCount, EnumIter};

#[derive(Clone, Copy, Debug, Default, EnumCount, EnumIter, Eq, Hash, PartialEq)]
pub enum Position {
    #[default]
    SB = 1,
    BB = 2,
    UTG = 3,
    UTGP1 = 4,
    UTGP2 = 5,
    MP = 6,
    LMP = 7,
    Hijack = 8,
    CO = 9,
    BTN = 10,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Positions(Vec<Position>);

impl Positions {
    #[must_use]
    pub fn heads_up() -> Self {
        Positions(vec![Position::BB, Position::BTN])
    }

    #[must_use]
    pub fn three_way() -> Self {
        Positions(vec![Position::SB, Position::BB, Position::BTN])
    }

    #[must_use]
    pub fn four_way() -> Self {
        todo!()
    }
}