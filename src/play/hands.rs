use crate::arrays::two::Two;
use crate::cards::Cards;
use crate::PKError;

/// To start with I am only focusing on supporting a single round of play.
///
/// `let mut v = Vec::with_capacity(10);`
#[derive(Clone, Debug, PartialEq)]
pub struct Hands(Vec<Two>);

impl Hands {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Hands {
        Hands(Vec::with_capacity(capacity))
    }

    pub fn push(&mut self, two: Two) {
        self.0.push(two);
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
mod play_cards_tests {
    use super::*;
    use crate::card::Card;
    use std::str::FromStr;

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
