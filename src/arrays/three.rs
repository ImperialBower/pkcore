use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::cards::Cards;
use crate::util::Util;
use crate::{PKError, Pile, Plurable, TheNuts};
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

impl From<Vec<Card>> for Three {
    /// While this is not the most elegant solution to me, I do love it's straight-forward
    /// simplicity. _Ship it!_
    fn from(v: Vec<Card>) -> Self {
        match v.len() {
            3 => {
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
                let three = Three([one, two, three]);
                if three.is_dealt() { three } else { Three::default() }
            }
            _ => Three::default(),
        }
    }
}

impl FromStr for Three {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Three::try_from(Cards::from_str(s)?)
    }
}

impl Plurable for Three {
    fn from_pluribus(s: &str) -> Result<Self, PKError> {
        let s = s.trim();
        match s.len() {
            0..=5 => Err(PKError::NotEnoughCards),
            6 => Self::from_str(Util::str_len_splitter(s, 2).as_str()),
            _ => Err(PKError::TooManyCards),
        }
    }
}

impl Pile for Three {
    fn card_at(self, _index: usize) -> Option<Card> {
        todo!()
    }

    fn clean(&self) -> Self {
        Three([self.first().clean(), self.second().clean(), self.third().clean()])
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        todo!()
    }

    fn the_nuts(&self) -> TheNuts {
        if !self.is_dealt() {
            return TheNuts::default();
        }

        let mut the_nuts = TheNuts::default();

        for v in self.remaining().combinations(2) {
            let hand = Five::from_2and3(Two::from(v), *self);
            the_nuts.push(hand.eval());
        }
        the_nuts.sort_in_place();

        the_nuts
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl TryFrom<Cards> for Three {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=2 => Err(PKError::NotEnoughCards),
            3 => Ok(Three::from([
                *cards.get_index(0).unwrap_or(&Card::BLANK),
                *cards.get_index(1).unwrap_or(&Card::BLANK),
                *cards.get_index(2).unwrap_or(&Card::BLANK),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__three_tests {
    use super::*;
    use crate::Evals;
    use crate::util::data::TestData;

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
    fn from__vec() {
        assert_eq!(
            Three(THE_FLOP),
            Three::from(vec![Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS])
        );
        // It should return default if any Let's do all the permutations, just to be sure.
        //
        // This is good that we did, because our initial version of our missed the cases where
        // if any card is blank, than they all need to be blank. I'm worried that this is getting
        // to be a very heavy operation for an internal call.
        assert_eq!(
            Three::default(),
            Three::from(vec![Card::BLANK, Card::BLANK, Card::BLANK])
        );
        assert_eq!(
            Three::default(),
            Three::from(vec![Card::BLANK, Card::ACE_HEARTS, Card::BLANK])
        );
        assert_eq!(
            Three::default(),
            Three::from(vec![Card::BLANK, Card::BLANK, Card::ACE_HEARTS])
        );
        assert_eq!(
            Three::default(),
            Three::from(vec![Card::BLANK, Card::ACE_HEARTS, Card::SEVEN_HEARTS])
        );
        assert_eq!(
            Three::default(),
            Three::from(vec![Card::ACE_HEARTS, Card::BLANK, Card::SEVEN_HEARTS])
        );
        assert_eq!(
            Three::default(),
            Three::from(vec![Card::ACE_HEARTS, Card::SEVEN_HEARTS, Card::BLANK])
        );
        assert_eq!(Three::default(), Three::from(vec![Card::ACE_HEARTS]));
        assert_eq!(
            Three::default(),
            Three::from(vec![Card::ACE_HEARTS, Card::SEVEN_HEARTS])
        );
        assert_eq!(
            Two::default(),
            Two::from(vec![
                Card::ACE_HEARTS,
                Card::SEVEN_HEARTS,
                Card::SEVEN_DIAMONDS,
                Card::SIX_DIAMONDS,
            ])
        );
        assert!(!Three::from(vec![Card::BLANK, Card::BLANK, Card::BLANK]).is_dealt());
    }

    #[test]
    fn from_str() {
        assert_eq!(Three::from(THE_FLOP), Three::from_str("9♣ 6♦ 5♥").unwrap());
        assert_eq!(PKError::InvalidCardIndex, Three::from_str("").unwrap_err());
        assert_eq!(PKError::InvalidCardIndex, Three::from_str(" ").unwrap_err());
        assert_eq!(PKError::InvalidCardIndex, Three::from_str(" __ ").unwrap_err());
        assert_eq!(PKError::NotEnoughCards, Three::from_str("AC 2D").unwrap_err());
        assert!(Three::from_str("AD KD QD JD TD 9D").is_err());
        assert_eq!(PKError::TooManyCards, Three::from_str("AD KD QD JD").unwrap_err());
    }

    #[test]
    fn from_pluribus() {
        assert_eq!(Three(THE_FLOP), Three::from_pluribus(" 9c6d5h").unwrap());
        assert_eq!(Three(THE_FLOP), Three::from_pluribus("9c6d5h ").unwrap());
        assert_eq!(Three(THE_FLOP), Three::from_pluribus("9c6d5h").unwrap());
        assert_eq!(PKError::NotEnoughCards, Three::from_pluribus("9c6d").unwrap_err());
        assert_eq!(PKError::TooManyCards, Three::from_pluribus("9c6d5h4h").unwrap_err());
        assert_eq!(PKError::InvalidCardIndex, Three::from_pluribus("AHASAa").unwrap_err());
        // assert_eq!(Two::HAND_8S_7H, Three::from_pluribus("8s7h"));
        // assert_eq!(Two::HAND_8S_7H, Three::from_pluribus(" 7h8s"));
        // assert_eq!(Two::HAND_AS_AH, Three::from_pluribus("AhAs   "));
        // assert_eq!(Two::default(), Three::from_pluribus("AH"));
    }

    #[test]
    fn pile__are_unique() {
        assert!(Three::from([Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS]).are_unique());
        assert!(!Three::from([Card::NINE_CLUBS, Card::NINE_CLUBS, Card::FIVE_HEARTS]).are_unique());
    }

    #[test]
    fn pile__cards() {
        assert_eq!(0, Three::default().cards().len());
        assert_eq!("9♣ 6♦ 5♥", Three(THE_FLOP).cards().to_string());
    }

    #[test]
    fn pile__evals() {
        let three = Three::from([Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS]);

        let evals = three.evals();

        assert_eq!(26, evals.len());
        assert_eq!(1605, evals.get(0).unwrap().hand_rank.value);
        assert_eq!(1996, evals.get(1).unwrap().hand_rank.value);
        assert_eq!(2251, evals.get(3).unwrap().hand_rank.value);
        assert_eq!(3058, evals.get(5).unwrap().hand_rank.value);
        assert_eq!(7420, evals.get(25).unwrap().hand_rank.value);
        assert!(evals.get(26).is_none());
    }

    #[test]
    fn pile__remaining() {
        assert_eq!(
            TestData::the_hand().board.flop.remaining().sort().to_string(),
            "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣"
        );
    }

    #[test]
    fn pile__the_nuts__blank() {
        let three = Three::from([Card::BLANK, Card::SIX_DIAMONDS, Card::FIVE_HEARTS]);

        let the_nuts = three.the_nuts();

        assert_eq!(TheNuts::default(), the_nuts);
    }

    #[test]
    fn pile__the_nuts__the_hand() {
        let three = Three::from([Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS]);

        let the_nuts = three.the_nuts();
        let evals = the_nuts.to_evals();

        assert_eq!(26, evals.len());
        // assert_eq!(TheNuts::default(), the_nuts);
    }

    #[test]
    fn pile__evals__blank() {
        let three = Three::from([Card::BLANK, Card::SIX_DIAMONDS, Card::FIVE_HEARTS]);

        let evals = three.the_nuts().to_evals();

        assert_eq!(Evals::default(), evals);
    }

    /// NOTE: These tests will quickly become out of hand if applied to the larger arrays.
    #[test]
    fn pile__is_dealt() {
        assert!(Three::from(THE_FLOP).is_dealt());
        assert!(!Three::from([Card::BLANK, Card::DEUCE_SPADES, Card::SIX_DIAMONDS]).is_dealt());
        assert!(!Three::from([Card::DEUCE_SPADES, Card::BLANK, Card::SIX_DIAMONDS]).is_dealt());
        assert!(!Three::from([Card::BLANK, Card::BLANK, Card::BLANK]).is_dealt());
        assert!(!Three::from([Card::DEUCE_SPADES, Card::DEUCE_SPADES, Card::SIX_DIAMONDS]).is_dealt());
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
