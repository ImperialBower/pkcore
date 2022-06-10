use crate::card::Card;
use crate::card_number::CardNumber;
use crate::rank::Rank;
use crate::util::random_ordering::RandomOrdering;
use crate::{PKError, SOK};
use indexmap::set::Iter;
use indexmap::IndexSet;
use itertools::{Combinations, Itertools};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use strum::IntoEnumIterator;

/// What are the contracts for Cards?
///
/// 1. Cards should be saved in order.
/// 2. Cards should be unique.
/// 3. Cards should be legitimate cards. (No blanks)
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Cards(IndexSet<Card>);

impl Cards {
    const NUMBER_OF_SHUFFLES: u8 = 5;

    #[must_use]
    pub fn deck() -> Cards {
        let mut cards = Cards::default();
        for card_number in CardNumber::iter() {
            cards.insert(Card::from(card_number as u32));
        }
        cards
    }

    /// TODO RF: :-P
    #[must_use]
    pub fn deck_minus(cards: &Cards) -> Cards {
        let mut minus = Cards::default();
        let deck = Cards::deck();
        for card in deck.iter() {
            if cards.get(card).is_none() {
                minus.insert(*card);
            }
        }
        minus
    }

    pub fn add(&mut self, cards: &Cards) {
        for card in cards.iter() {
            self.insert(*card);
        }
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

    /// # Errors
    /// Returns `PKError::NotEnoughCards` if there are no more cards left.
    pub fn draw_one(&mut self) -> Result<Card, PKError> {
        match self.0.shift_remove_index(0) {
            Some(card) => Ok(card),
            None => Err(PKError::NotEnoughCards),
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

    /// One of the big problems with our Card data type is that it's just a binary number
    /// so it's hard to figure out what's going on with it. To help deal with this I try to
    /// add some methods just to help out with debugging.
    ///
    /// Later on, we might be able to use this for logging as a part of a larger system. Right now
    /// we're using println!, which is in itself a kind of technical debt. Usually, when I reach
    /// a point in a library where I think it's about ready to integrate into the larger crate
    /// community, I will search these out and replace them with actually log statements. For now
    /// though, I don't want to deal with it. Do what you can. Take your time. Perfection is a goal;
    /// never a reality.
    ///
    /// ASIDE: One of the best compliments I ever got from another developer was from the person
    /// I dislike more than any other in my career. _There was this one guy at a startup who tried
    /// to forge commands as if he was me from our servers to try to get me fired because I had
    /// the audacity to call him on his bullshit, but to be honest, he was doing me a favor by
    /// driving me out of that place._
    pub fn dump(&self) {
        for card in self.iter() {
            println!("{} {}\n", card.bit_string_guided(), card);
        }
    }

    /// Sets the card's paired bit to true for all cards in the collection.
    #[must_use]
    pub fn flag_paired(&self) -> Cards {
        Cards::from(self.iter().map(Card::frequency_paired).collect::<Vec<_>>())
    }

    /// Sets the card's tripped bit to true for all cards in the collection.
    #[must_use]
    pub fn flag_tripped(&self) -> Cards {
        Cards::from(self.iter().map(Card::frequency_tripped).collect::<Vec<_>>())
    }

    /// Sets the card's quaded bit to true for all cards in the collection.
    #[must_use]
    pub fn flag_quaded(&self) -> Cards {
        Cards::from(self.iter().map(Card::frequency_quaded).collect::<Vec<_>>())
    }

    /// This function is most likely going to be a shit show. I could just cast everything over
    /// to my [cardpack.rs](https://github.com/ContractBridge/cardpack.rs) library where this is
    /// [already solved](https://github.com/ContractBridge/cardpack.rs/blob/main/src/cards/pile.rs#L448),
    /// but I'm trying to keep this library as dependency clean as possible. Plus, how can I
    /// refactor something if I just pass the work onto a library where that won't work?
    ///
    /// DEFECT: In git history original version fucks up on non weighted cards.
    ///
    /// The only time this is really needed is to display `Five` so that it sorts based on the
    /// `HandRank`.
    #[must_use]
    pub fn frequency_weighted(&self) -> Cards {
        let mappy = self.map_by_rank();
        let mut cards = Cards::default();
        for rank in mappy.keys() {
            match mappy.get(rank) {
                None => {}
                Some(c) => match c.len() {
                    0 => {}
                    1 => cards.add(c),
                    2 => cards.add(&c.flag_paired()),
                    3 => cards.add(&c.flag_tripped()),
                    _ => cards.add(&c.flag_quaded()),
                },
            }
        }
        cards.sort()
    }

    #[must_use]
    pub fn get(&self, card: &Card) -> Option<&Card> {
        self.0.get(card)
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
    pub fn deal_from_the_bottom(&mut self) -> Option<Card> {
        self.0.pop()
    }

    #[must_use]
    pub fn shuffle(&self) -> Cards {
        let mut shuffled = self.clone();
        shuffled.shuffle_in_place();
        shuffled
    }

    pub fn shuffle_in_place(&mut self) {
        for _ in 0..Cards::NUMBER_OF_SHUFFLES {
            self.0
                .sort_by(|_, _| rand::random::<RandomOrdering>().into());
        }
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

    //region private functions

    fn map_by_rank(&self) -> HashMap<Rank, Cards> {
        // Why is this variable called mappy? Now that is a long and winding tale.
        // Many, many years ago, when I was in middle school in SF, me and my friends would
        // Play D&D, eat Georgio's pizza, and play video games at an ice cream show. The two
        // games they had were [Mr. Do!](https://en.wikipedia.org/wiki/Mr._Do!) and
        // [Mappy](https://en.wikipedia.org/wiki/Mappy). In honor of this nostalgia I try to
        // name any private variables of hashmaps after the mouse plagued police cat. _Aside:
        // Everytime [Wil Wheaton posts about his Mr. Do! machine](https://wilwheaton.net/2019/02/)
        // I let out a [Sheldonesque WHEATON!!!!](https://www.youtube.com/watch?v=bUWXjs2jPQI)
        // inside._
        //
        // BTW, if you are ever in the sunset district of SF, checkout Georgio's for dinner and
        // then stop by Toy Boat ice cream for dessert. No, they're not the shop with the
        // video games, which closed a while ago, but they are great.
        let mut mappy: HashMap<Rank, Cards> = HashMap::new();
        for rank in Rank::iter() {
            let pile: Vec<Card> = self
                .iter()
                .copied()
                .filter(|card| card.get_rank() == rank)
                .collect();
            mappy.insert(rank, Cards::from(pile));
        }
        mappy
    }

    //endregion
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

impl From<Vec<&Card>> for Cards {
    fn from(v: Vec<&Card>) -> Self {
        // TODO RF: Hack :-P
        let filtered = v.iter().filter_map(|c| {
            let pc = **c;
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

impl IntoIterator for Cards {
    type Item = Card;
    type IntoIter = indexmap::set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
    fn deck_minus() {
        let cards = Cards::from_str("Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣").unwrap().shuffle();

        let minus = Cards::deck_minus(&cards);

        assert_eq!("A♠ K♠", minus.to_string());
    }

    #[test]
    fn add() {
        let mut pile = Cards::from_str("5♣ 4♣").unwrap();

        pile.add(&Cards::from_str("3♣ 2♣ A♣").unwrap());

        assert_eq!(Cards::from_str("5♣ 4♣ 3♣ 2♣ A♣").unwrap(), pile);
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
        assert!(card.is_ok());
        assert_eq!(card.unwrap(), Card::ACE_HEARTS);
    }

    #[test]
    fn flag_paired() {
        let mut cards = Cards::from_str("T♠ T♥").unwrap().flag_paired();

        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_PAIRED_MASK));
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_PAIRED_MASK));
        assert!(!Cards::from_str("T♠")
            .unwrap()
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_PAIRED_MASK));
    }

    #[test]
    fn flag_tripped() {
        let mut cards = Cards::from_str("T♠ T♥ T♦").unwrap().flag_tripped();

        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert!(!Cards::from_str("T♠")
            .unwrap()
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_TRIPPED_MASK));
    }

    #[test]
    fn flag_quaded() {
        let mut cards = Cards::from_str("T♠ T♥ T♦ T♣").unwrap().flag_quaded();

        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(!Cards::from_str("T♠")
            .unwrap()
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_QUADED_MASK));
    }

    #[test]
    fn frequency_weighted() {
        let cards = Cards::from_str("T♠ T♥ T♦ 9♠ 9♥").unwrap();

        let mut cards = cards.frequency_weighted();

        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_PAIRED_MASK));
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_PAIRED_MASK));
    }

    #[test]
    fn frequency_weighted_quads() {
        let cards = Cards::from_str("T♠ T♥ T♦ T♣ 9♥").unwrap();

        let mut cards = cards.frequency_weighted();

        assert_eq!(5, cards.len());
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(cards
            .draw_one()
            .unwrap()
            .is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert!(!cards.draw_one().unwrap().is_flagged(Card::FREQUENCY_MASK));
    }

    #[test]
    fn get() {
        let cards = wheel();

        assert_eq!(cards.get(&Card::FIVE_CLUBS).unwrap(), &Card::FIVE_CLUBS);
        assert!(cards.get(&Card::FIVE_DIAMONDS).is_none());
    }

    #[test]
    fn get_index() {
        let cards = wheel();

        assert_eq!(cards.get_index(0).unwrap(), &Card::FIVE_CLUBS);
        assert_eq!(cards.get_index(1).unwrap(), &Card::FOUR_CLUBS);
        assert_eq!(cards.get_index(2).unwrap(), &Card::TREY_CLUBS);
        assert_eq!(cards.get_index(3).unwrap(), &Card::DEUCE_CLUBS);
        assert_eq!(cards.get_index(4).unwrap(), &Card::ACE_CLUBS);
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

    // #[test]
    // fn sort_by_frequency() {
    //     assert_eq!("A♣ 5♣ 4♣ 3♣ 2♣", wheel().sort().to_string());
    // }

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

    //region private function tests

    #[test]
    fn map_by_rank() {
        let cards = Cards::from_str("A♠ T♠ 9♠ 8♠ T♥").unwrap();

        let mappy = cards.map_by_rank();

        assert_eq!(2, mappy.get(&Rank::TEN).unwrap().len());
        assert_eq!(1, mappy.get(&Rank::ACE).unwrap().len());
        assert_eq!(1, mappy.get(&Rank::NINE).unwrap().len());
        assert_eq!(1, mappy.get(&Rank::EIGHT).unwrap().len());
    }

    //endregion

    //region trait tests

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

    fn wheel() -> Cards {
        let mut cards = Cards::default();

        cards.insert(Card::FIVE_CLUBS);
        cards.insert(Card::FOUR_CLUBS);
        cards.insert(Card::TREY_CLUBS);
        cards.insert(Card::DEUCE_CLUBS);
        cards.insert(Card::ACE_CLUBS);

        cards
    }
    //endregion
}
