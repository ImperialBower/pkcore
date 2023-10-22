use crate::arrays::three::Three;
use crate::{Card, Pile, TheNuts};
use std::fmt;
use std::fmt::{Display, Formatter};

/// This is a convenience struct for Game. I'm not writing many tests *WHAT???* for it because I don't
/// feel it is necessary right now. Later on, who knows, but for now that's OK.
///
/// I mainly want this struct for the `From<Vec<Card>>` trait, which is there to make things
/// easier for me with the analysis code.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Four([Card; 4]);

impl Four {
    #[must_use]
    pub fn from_turn(flop: Three, turn: Card) -> Four {
        Four([flop.first(), flop.second(), flop.third(), turn])
    }

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
    pub fn to_arr(&self) -> [Card; 4] {
        self.0
    }
    //endregion
}

impl Display for Four {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.first(),
            self.second(),
            self.third(),
            self.forth()
        )
    }
}

impl From<[Card; 4]> for Four {
    fn from(array: [Card; 4]) -> Self {
        Four(array)
    }
}

impl From<Vec<Card>> for Four {
    /// I do want to test this baby, since it's the main reason we are here.
    fn from(v: Vec<Card>) -> Self {
        match v.len() {
            4 => {
                let one = match v.first() {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let two = match v.get(1) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let three = match v.get(2) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let four = match v.get(3) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let four = Four([one, two, three, four]);
                if four.is_dealt() {
                    four
                } else {
                    Four::default()
                }
            }
            _ => Four::default(),
        }
    }
}

impl Pile for Four {
    fn clean(&self) -> Self {
        Four([
            self.first().clean(),
            self.second().clean(),
            self.third().clean(),
            self.forth().clean(),
        ])
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__four_tests {
    use super::*;

    #[test]
    fn from__vec() {
        let cards = [
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_HEARTS,
            Card::FIVE_SPADES,
        ];
        let expected = Four(cards);

        let actual = Four::from(cards);

        assert_eq!(expected, actual);
    }
}
