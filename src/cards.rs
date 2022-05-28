use crate::arrays::five::Five;
use crate::card::Card;
use crate::card_number::CardNumber;
use crate::{PKError, SOK};
use indexmap::set::Iter;
use indexmap::IndexSet;
use itertools::{Combinations, Itertools};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use strum::IntoEnumIterator;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Cards(IndexSet<Card>);

impl Cards {
    #[must_use]
    pub fn deck() -> Cards {
        let mut cards = Cards::default();
        for card_number in CardNumber::iter() {
            cards.insert(Card::from(card_number as u32));
        }
        cards
    }

    pub fn combinations(&self, k: usize) -> Combinations<indexmap::set::IntoIter<Card>> {
        self.0.clone().into_iter().combinations(k)
    }

    /// # Errors
    ///
    /// Returns `PKError::NotEnoughCards` if not enough cards are available.
    pub fn draw(&mut self, number: usize) -> Result<Self, PKError> {
        if number > self.len() {
            Err(PKError::NotEnoughCards)
        } else {
            Ok(Cards(self.0.drain(0..number).collect()))
        }
    }

    pub fn draw_one(&mut self) -> Option<Card> {
        let cards = self.draw(1);
        match cards {
            Ok(mut c) => Some(c.0.pop()?),
            Err(_) => None,
        }
    }

    /// # Errors
    ///
    /// Returns `PKError::NotEnoughCards` if not enough cards are available.
    pub fn draw_from_the_bottom(&mut self, number: usize) -> Result<Self, PKError> {
        let l = self.len();
        if number > l {
            Err(PKError::NotEnoughCards)
        } else {
            Ok(Cards(self.0.drain(l - number..l).collect()))
        }
    }

    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<&Card> {
        self.0.get_index(index)
    }

    /// Allows you to insert a `PlayingCard` provided it isn't blank.
    pub fn insert(&mut self, card: Card) -> bool {
        if card.is_blank() {
            false
        } else {
            self.0.insert(card)
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn iter(&self) -> Iter<'_, Card> {
        self.0.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn sort(&self) -> Cards {
        let mut c = self.clone();
        c.sort_in_place();
        c
    }

    pub fn sort_in_place(&mut self) {
        self.0.sort();
        self.0.reverse();
    }
}

impl fmt::Display for Cards {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = self
            .iter()
            .map(Card::to_string)
            .collect::<Vec<String>>()
            .join(" ");

        write!(f, "{}", s)
    }
}

impl From<Vec<Card>> for Cards {
    fn from(v: Vec<Card>) -> Self {
        let filtered = v.iter().filter_map(|c| {
            let pc = *c;
            if pc.is_blank() {
                None
            } else {
                Some(pc)
            }
        });
        Cards(filtered.collect())
    }
}

impl FromStr for Cards {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cards = Cards::default();
        for s in s.split_whitespace() {
            let c = Card::from_str(s)?;
            if c.is_blank() {
                return Err(PKError::InvalidIndex);
            }
            cards.insert(c);
        }
        if cards.is_empty() {
            Err(PKError::InvalidIndex)
        } else {
            Ok(cards)
        }
    }
}

impl TryFrom<Card> for Cards {
    type Error = PKError;

    fn try_from(card: Card) -> Result<Self, Self::Error> {
        if card.salright() {
            let mut cards = Cards::default();
            cards.insert(card);
            Ok(cards)
        } else {
            Err(PKError::BlankCard)
        }
    }
}

impl TryFrom<Five> for Cards {
    type Error = PKError;

    /// The contract for arrays is that they have to not be blank.
    fn try_from(five: Five) -> Result<Self, Self::Error> {
        let mut cards = Cards::default();

        // TODO RF - Has to be a better way
        for card in five.to_arr() {
            if !cards.insert(card) {
                return Err(PKError::BlankCard);
            }
        }
        Ok(cards)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod card_tests {
    use super::*;

    #[test]
    fn deck() {
        let deck = Cards::deck();

        assert_eq!(deck.len(), 52);
        assert_eq!(deck.to_string(), "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣");
    }

    #[test]
    fn combinations() {
        assert_eq!(1_326, Cards::deck().combinations(2).count());
        assert_eq!(2_598_960, Cards::deck().combinations(5).count());
    }

    #[test]
    fn draw() {
        let mut deck = Cards::deck();

        let drawn = deck.draw(5).unwrap();

        assert_eq!(drawn.len(), 5);
        assert_eq!(deck.len(), 47);
        assert_eq!("A♠ K♠ Q♠ J♠ T♠", drawn.to_string());
    }

    #[test]
    fn draw__too_many() {
        let mut deck = Cards::deck();

        let drawn = deck.draw(53);

        assert!(drawn.is_err());
        assert_eq!(PKError::NotEnoughCards, drawn.unwrap_err());
        assert_eq!(deck.len(), 52);
    }

    #[test]
    fn draw_from_the_bottom() {
        let mut deck = Cards::deck();

        let drawn = deck.draw_from_the_bottom(2).unwrap();

        assert_eq!(drawn.len(), 2);
        assert_eq!(deck.len(), 50);
        assert_eq!("3♣ 2♣", drawn.to_string());
    }

    #[test]
    fn draw_from_the_bottom__too_many() {
        let mut deck = Cards::deck();

        let drawn = deck.draw_from_the_bottom(53);

        assert!(drawn.is_err());
        assert_eq!(PKError::NotEnoughCards, drawn.unwrap_err());
        assert_eq!(deck.len(), 52);
    }

    #[test]
    fn draw_one() {
        let mut cards = Cards::default();
        cards.insert(Card::ACE_HEARTS);

        let card = cards.draw_one();

        assert!(cards.is_empty());
        assert!(card.is_some());
        assert_eq!(card.unwrap(), Card::ACE_HEARTS);
    }

    #[test]
    fn get_index() {
        let cards = wheel();

        assert_eq!(cards.get_index(0).unwrap(), &Card::from_str("5c").unwrap());
        assert_eq!(cards.get_index(1).unwrap(), &Card::from_str("4c").unwrap());
        assert_eq!(cards.get_index(2).unwrap(), &Card::from_str("3c").unwrap());
        assert_eq!(cards.get_index(3).unwrap(), &Card::from_str("2c").unwrap());
        assert_eq!(cards.get_index(4).unwrap(), &Card::from_str("ac").unwrap());
        assert!(cards.get_index(5).is_none());
    }

    #[test]
    fn insert() {
        let mut cards = Cards::default();

        cards.insert(Card::ACE_HEARTS);
        cards.insert(Card::KING_HEARTS);

        let mut i = cards.iter();

        assert_eq!(&Card::ACE_HEARTS, i.next().unwrap());
        assert_eq!(&Card::KING_HEARTS, i.next().unwrap());
        assert!(i.next().is_none());
    }

    #[test]
    fn is_empty() {
        assert!(Cards::default().is_empty());
        assert!(!wheel().is_empty());
    }

    #[test]
    fn len() {
        assert_eq!(0, Cards::default().len());
        assert_eq!(5, wheel().len());
    }

    #[test]
    fn sort() {
        assert_eq!("A♣ 5♣ 4♣ 3♣ 2♣", wheel().sort().to_string());
    }

    #[test]
    fn sort_in_place() {
        let mut wheel = wheel();

        wheel.sort_in_place();

        assert_eq!("A♣ 5♣ 4♣ 3♣ 2♣", wheel.to_string());
    }

    // Traits

    #[test]
    fn display() {
        assert_eq!("5♣ 4♣ 3♣ 2♣ A♣", wheel().to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(wheel(), Cards::from_str("5♣ 4♣ 3♣ 2♣ A♣").unwrap());
    }

    #[test]
    fn from_str__invalid() {
        assert!(Cards::from_str("5♣ 4♣ 3A 2♣ A♣").is_err());
    }

    #[test]
    fn try_from__card() {
        assert!(Cards::try_from(Card::FOUR_DIAMONDS).is_ok());
        assert!(Cards::try_from(Card::BLANK).is_err());
    }

    #[test]
    fn try_from__five() {
        assert!(Cards::try_from(Five::from_str("AD KD QD JD TD").unwrap()).is_ok());
        assert!(Cards::try_from(Five::default()).is_err());
    }

    fn wheel() -> Cards {
        let mut cards = Cards::default();

        cards.insert(Card::FIVE_CLUBS);
        cards.insert(Card::FOUR_CLUBS);
        cards.insert(Card::TREY_CLUBS);
        cards.insert(Card::DEUCE_CLUBS);
        cards.insert(Card::ACE_CLUBS);

        cards
    }
}
