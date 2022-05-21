use crate::card::Card;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Six([Card; 6]);

impl Six {
    /// permutations to evaluate all 6 card combinations.
    pub const FIVE_CARD_PERMUTATIONS: [[u8; 5]; 6] = [
        [0, 1, 2, 3, 4],
        [0, 1, 2, 3, 5],
        [0, 1, 2, 4, 5],
        [0, 1, 3, 4, 5],
        [0, 2, 3, 4, 5],
        [1, 2, 3, 4, 5],
    ];

    //region accessors
    #[must_use]
    pub fn first(&self) -> Card {
        self.0[0]
    }

    #[must_use]
    pub fn second(&self) -> Card {
        self.0[1]
    }

    #[must_use]
    pub fn third(&self) -> Card {
        self.0[2]
    }

    #[must_use]
    pub fn forth(&self) -> Card {
        self.0[3]
    }

    #[must_use]
    pub fn fifth(&self) -> Card {
        self.0[4]
    }

    #[must_use]
    pub fn sixth(&self) -> Card {
        self.0[5]
    }

    #[must_use]
    pub fn to_arr(&self) -> [Card; 6] {
        self.0
    }

    //endregion
}

impl From<[Card; 6]> for Six {
    fn from(array: [Card; 6]) -> Self {
        Six(array)
    }
}
