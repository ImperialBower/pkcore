use crate::arrays::two::Two;
use crate::cards::Cards;
use crate::{Card, PKError, Pile};
use itertools::Itertools;
use std::fmt;
use std::slice::Iter;
use std::str::FromStr;

/// To start with I am only focusing on supporting a single round of play.
///
/// `let mut v = Vec::with_capacity(10);`
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Hands(Vec<Two>);

impl Hands {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Hands {
        Hands(Vec::with_capacity(capacity))
    }

    /// For our get we're going to return a blank `Hand` if the index passed in is too high.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Two> {
        self.0.get(index)
    }

    pub fn iter(&self) -> Iter<'_, Two> {
        self.0.iter()
    }

    pub fn push(&mut self, two: Two) {
        self.0.push(two);
    }
}

impl Pile for Hands {
    fn to_vec(&self) -> Vec<Card> {
        let mut v: Vec<Card> = Vec::default();
        for two in &self.0 {
            v.push(two.first());
            v.push(two.second());
        }
        v
    }
}

impl fmt::Display for Hands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined = Itertools::join(&mut self.0.iter(), ", ");
        write!(f, "[{}]", joined)
    }
}

impl FromStr for Hands {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Hands::try_from(Cards::from_str(s)?)
    }
}

impl TryFrom<Cards> for Hands {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        let mut cards = cards;

        if cards.len() % 2 == 0 {
            let num_of_players = cards.len() / 2;
            let mut hands = Hands::with_capacity(num_of_players);

            for _ in 0..num_of_players {
                hands.push(Two::new(
                    cards.draw_one().unwrap(),
                    cards.draw_one().unwrap(),
                )?);
            }
            Ok(hands)
        } else {
            Err(PKError::InvalidCardCount)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play_hands_tests {
    use super::*;
    use crate::card::Card;
    use std::str::FromStr;

    #[test]
    fn get() {
        let the_hand = Hands::from_str("6♥ 6♠ 5♦ 5♣").unwrap();

        assert_eq!(
            *the_hand.get(0).unwrap(),
            Two::from([Card::SIX_SPADES, Card::SIX_HEARTS])
        );
        assert_eq!(
            *the_hand.get(1).unwrap(),
            Two::from([Card::FIVE_DIAMONDS, Card::FIVE_CLUBS])
        );
        // Check it again to make sure that the underlying vec is undamaged.
        assert_eq!(
            *the_hand.get(1).unwrap(),
            Two::from([Card::FIVE_DIAMONDS, Card::FIVE_CLUBS])
        );
        assert_eq!(the_hand.0.len(), 2);
        assert!(the_hand.get(2).is_none());
    }

    #[test]
    fn cards() {
        assert_eq!(
            "6♠ 6♥ 5♦ 5♣",
            Hands::from_str("6♥ 6♠ 5♦ 5♣").unwrap().cards().to_string()
        );
    }

    #[test]
    fn remaining_after() {
        let hands = Hands::from_str("6♠ 6♥ 5♦ 5♣").unwrap();
        let flop = Cards::from_str("9♣ 6♦ 5♥").unwrap();

        let remaining = hands.remaining_after(&flop);

        assert_eq!(remaining.to_string(), "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 8♣ 7♣ 6♣ 4♣ 3♣ 2♣");
    }

    #[test]
    fn display() {
        assert_eq!(
            "[6♠ 6♥, 5♦ 5♣]",
            Hands::from_str("6♥ 6♠ 5♦ 5♣").unwrap().to_string()
        );
    }

    #[test]
    fn from_str() {
        let expected = Hands(vec![
            Two::new(Card::SIX_SPADES, Card::SIX_HEARTS).unwrap(),
            Two::new(Card::FIVE_DIAMONDS, Card::FIVE_CLUBS).unwrap(),
        ]);

        assert_eq!(Hands::from_str("6♥ 6♠ 5♦ 5♣").unwrap(), expected);
    }

    #[test]
    fn try_from__cards() {
        let cards = Cards::from_str("6♥ 6♠ 5♦ 5♣").unwrap();
        let expected = Hands(vec![
            Two::new(Card::SIX_SPADES, Card::SIX_HEARTS).unwrap(),
            Two::new(Card::FIVE_DIAMONDS, Card::FIVE_CLUBS).unwrap(),
        ]);

        let hands = Hands::try_from(cards).unwrap();

        assert_eq!(hands, expected);
    }
}
