use crate::arrays::five::Five;
use crate::arrays::two::Two;
use crate::arrays::HandRanker;
use crate::card::Card;
use crate::cards::Cards;
use crate::{PKError, Pile, TheNuts, SOK};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Three([Card; 3]);

impl Three {
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
    pub fn to_arr(&self) -> [Card; 3] {
        self.0
    }
    //endregion
}

impl Display for Three {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.first(), self.second(), self.third())
    }
}

impl From<[Card; 3]> for Three {
    fn from(array: [Card; 3]) -> Self {
        Three(array)
    }
}

impl FromStr for Three {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Three::try_from(Cards::from_str(s)?)
    }
}

impl Pile for Three {
    fn clean(&self) -> Self {
        Three([
            self.first().clean(),
            self.second().clean(),
            self.third().clean(),
        ])
    }

    fn the_nuts(&self) -> TheNuts {
        if !self.salright() {
            return TheNuts::default();
        }

        let mut hands: Vec<Five> = Vec::default();

        for v in self.remaining().combinations(2) {
            let two = Two::from(v);
            let hand = Five::from_2and3(two, *self);
            // println!("> {}", hand);
            hands.push(hand);
        }

        let mut the_nuts = TheNuts::from(hands);
        the_nuts.sort_in_place();
        the_nuts
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl SOK for Three {
    fn salright(&self) -> bool {
        (self.first().salright() && self.second().salright() && self.third().salright())
            && (self.first() != self.second())
            && (self.first() != self.third())
            && (self.second() != self.third())
    }
}

impl TryFrom<Cards> for Three {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=2 => Err(PKError::NotEnoughCards),
            3 => Ok(Three::from([
                *cards.get_index(0).unwrap(),
                *cards.get_index(1).unwrap(),
                *cards.get_index(2).unwrap(),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays_three_tests {
    use super::*;
    use crate::cards::Cards;
    use crate::hand_rank::class::Class;
    use std::str::FromStr;

    /// <https://www.youtube.com/watch?v=vjM60lqRhPg />
    const THE_FLOP: [Card; 3] = [Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS];

    #[test]
    fn display() {
        assert_eq!("9♣ 6♦ 5♥", Three(THE_FLOP).to_string());
    }

    #[test]
    fn from__array() {
        assert_eq!(Three(THE_FLOP), Three::from(THE_FLOP));
    }

    #[test]
    fn from_str() {
        assert_eq!(Three::from(THE_FLOP), Three::from_str("9♣ 6♦ 5♥").unwrap());
        assert_eq!(PKError::InvalidIndex, Three::from_str("").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Three::from_str(" ").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Three::from_str(" __ ").unwrap_err());
        assert_eq!(
            PKError::NotEnoughCards,
            Three::from_str("AC 2D").unwrap_err()
        );
        assert!(Three::from_str("AD KD QD JD TD 9D").is_err());
        assert_eq!(
            PKError::TooManyCards,
            Three::from_str("AD KD QD JD").unwrap_err()
        );
    }

    #[test]
    fn cards() {
        assert_eq!(0, Three::default().cards().len());
        assert_eq!("9♣ 6♦ 5♥", Three(THE_FLOP).cards().to_string());
    }

    #[test]
    fn the_nuts() {
        let three = Three::from([Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS]);

        let the_nuts = three.the_nuts();

        // for e in the_nuts.to_vec().iter() {
        //     println!("{}", e);
        // }

        assert_eq!(
            Class::NineHighStraight,
            the_nuts.get(0).unwrap().hand_rank.class()
        );
        assert_eq!(3058, the_nuts.get(3).unwrap().hand_rank.value());
        assert_eq!(3058, the_nuts.get(5).unwrap().hand_rank.value());
    }

    #[test]
    fn the_nuts__blank() {
        let three = Three::from([Card::BLANK, Card::SIX_DIAMONDS, Card::FIVE_HEARTS]);

        let the_nuts = three.the_nuts();

        assert_eq!(TheNuts::default(), the_nuts);
    }

    /// NOTE: These tests will quickly become out of hand if applied to the larger arrays.
    #[test]
    fn sok() {
        assert!(Three::from(THE_FLOP).salright());
        assert!(!Three::from([Card::BLANK, Card::DEUCE_SPADES, Card::SIX_DIAMONDS]).salright());
        assert!(!Three::from([Card::DEUCE_SPADES, Card::BLANK, Card::SIX_DIAMONDS]).salright());
        assert!(!Three::from([Card::BLANK, Card::BLANK, Card::BLANK]).salright());
        assert!(
            !Three::from([Card::DEUCE_SPADES, Card::DEUCE_SPADES, Card::SIX_DIAMONDS]).salright()
        );
    }

    #[test]
    fn try_from__cards() {
        assert_eq!(
            Three::try_from(Cards::from_str("9♣ 6♦ 5♥").unwrap()).unwrap(),
            Three::from(THE_FLOP)
        );
        assert_eq!(
            Three::try_from(Cards::from_str("9♣ 6♦").unwrap()).unwrap_err(),
            PKError::NotEnoughCards
        );
        assert_eq!(
            Three::try_from(Cards::from_str("9♣ 6♦ 5♥ 4♥").unwrap()).unwrap_err(),
            PKError::TooManyCards
        );
    }
}
