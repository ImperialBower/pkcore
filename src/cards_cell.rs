use crate::analysis::the_nuts::TheNuts;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use crate::prelude::BoxedCards;
use crate::{PKError, Pile};
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::FromStr;

#[derive(Clone, Debug, Default)]
pub struct CardsCell(pub(crate) RefCell<Cards>);

impl CardsCell {
    /// ```
    /// use pkcore::cards_cell::CardsCell;
    /// use pkcore::deck_cell;
    ///
    /// let deck = CardsCell::deck();
    ///
    /// assert_eq!(deck_cell!(), deck);
    /// assert_eq!(deck.len(), 52);
    /// assert_eq!(
    ///     deck.to_string(),
    ///     "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣"
    /// );
    /// ```
    #[must_use]
    pub fn deck() -> Self {
        Self::from(Cards::deck())
    }

    /// Creates a new `CardsCell` containing the given `Cards`.
    ///
    /// # Deprecatred
    ///
    /// I really need to get out of the habit of using new methods for simple froms.
    /// Another example of the Java dev poison still flowing through my veins.
    #[deprecated(since = "0.8.0", note = "Use `CardsCell::from` instead")]
    #[must_use]
    pub fn new(cards: Cards) -> Self {
        Self(RefCell::new(cards))
    }

    pub fn cards(&self) -> Cards {
        self.0.borrow().clone()
    }

    /// REFACTOR: Changing this to taking a `CardCell` reference. I'm feeling that we need to keep
    /// things in the [family](https://www.youtube.com/watch?v=IQuc7wfO16Q).
    #[must_use]
    pub fn deck_minus(cards: &CardsCell) -> CardsCell {
        let deck = Self::deck();
        deck.remove_all(cards);
        deck
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let deck = deck_cell!();
    /// let without_aces = deck.minus(&cc!("A♠ A♥ A♦ A♣"));
    ///
    /// assert_eq!(without_aces.len(), 48);
    /// assert_eq!(without_aces.to_string(), "K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣");
    /// ```
    #[must_use]
    pub fn minus(&self, cards: &CardsCell) -> CardsCell {
        Self::from(self.0.borrow_mut().minus(&Cards::from(cards)))
    }

    /// Gets a clone of the internal `Cards`.
    ///
    /// ```
    /// use pkcore::cards_cell::CardsCell;
    /// use pkcore::deck_cell;
    ///
    /// let deck = deck_cell!();
    ///
    /// assert_eq!(deck.draw(2).unwrap().to_string(), "A♠ K♠");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PKError::NotEnoughCards` if not enough cards are available.
    pub fn draw(&self, n: usize) -> Result<Self, PKError> {
        let mut internal = self.0.borrow_mut();
        let drawn_cards = internal.draw(n)?;
        // drawn_cards.map(Self::new)
        Ok(Self::from(drawn_cards))
    }

    #[must_use]
    pub fn draw_all(&self) -> Self {
        let mut internal = self.0.borrow_mut();
        let drawn_cards = internal.draw_all();
        Self::from(drawn_cards)
    }

    /// ```
    /// use pkcore::cards_cell::CardsCell;
    /// use pkcore::deck_cell;
    ///
    /// let deck = deck_cell!();
    ///
    /// assert_eq!(deck.draw_one().unwrap().to_string(), "A♠");
    /// ```
    /// # Errors
    ///
    /// Returns `PKError::NotEnoughCards` if there are no more cards left.
    pub fn draw_one(&self) -> Result<Card, PKError> {
        let mut internal = self.0.borrow_mut();
        let drawn_card = internal.draw_one()?;
        Ok(drawn_card)
    }

    /// ```
    /// use pkcore::cards_cell::CardsCell;
    /// use pkcore::deck_cell;
    ///
    /// let deck = deck_cell!();
    ///
    /// assert_eq!(deck.draw_from_the_bottom(2).unwrap().to_string(), "3♣ 2♣");
    /// ```
    /// # Errors
    ///
    /// Returns `PKError::NotEnoughCards` if not enough cards are available.
    pub fn draw_from_the_bottom(&self, number: usize) -> Result<Self, PKError> {
        let mut internal = self.0.borrow_mut();
        let drawn_cards = internal.draw_from_the_bottom(number)?;
        Ok(Self::from(drawn_cards))
    }

    pub fn dump(&self) {
        let internal = self.0.borrow_mut();
        internal.dump();
    }

    /// # Errors
    ///
    /// Returns `PKError::CardNotFound` if the specified card is not found in the collection.
    pub fn force_draw(&self, card: Card) -> Result<Card, PKError> {
        let mut internal = self.0.borrow_mut();
        if internal.remove(&card) {
            Ok(card)
        } else {
            Err(PKError::CardNotFound)
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// ```
    /// use pkcore::cards_cell::CardsCell;
    ///
    /// assert_eq!(CardsCell::deck().len(), 52);
    /// assert_eq!(CardsCell::default().len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        let internal = self.0.borrow_mut();
        internal.len()
    }

    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::cards_cell::CardsCell;
    ///
    /// let cards = CardsCell::default();
    ///
    /// cards.insert(Card::NINE_SPADES);
    ///
    /// assert_eq!(cards.to_string(), "9♠");
    /// ```
    pub fn insert(&self, card: Card) {
        let mut internal = self.0.borrow_mut();
        internal.insert(card);
    }

    /// ```
    /// use pkcore::cards::Cards;
    /// use pkcore::cards_cell::CardsCell;
    /// use std::str::FromStr;
    ///
    /// let cards = CardsCell::default();
    /// let to_insert = Cards::from_str("9♠ 8♠ T♠").unwrap();
    ///
    /// cards.insert_all(to_insert);
    ///
    /// assert_eq!(cards.to_string(), "9♠ 8♠ T♠");
    /// ```
    pub fn insert_all(&self, cards: Cards) {
        let mut internal = self.0.borrow_mut();
        for card in cards {
            internal.insert(card);
        }
    }

    pub fn insert_at(&self, index: usize, card: Card) {
        let mut internal = self.0.borrow_mut();
        internal.insert_at(index, card);
    }

    /// Removes all cards from this `CardsCell` that are present in the given `CardsCell`.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let deck = deck_cell!();
    /// let aces = cc!("A♠ A♥ A♦ A♣");
    ///
    /// deck.remove_all(&aces);
    ///
    /// assert_eq!(deck.len(), 48);
    /// assert!(!deck.contains(&Card::ACE_SPADES));
    /// assert!(!deck.contains(&Card::ACE_DIAMONDS));
    /// assert!(!deck.contains(&Card::ACE_HEARTS));
    /// assert!(!deck.contains(&Card::ACE_CLUBS));
    /// ```
    pub fn remove_all(&self, cards: &CardsCell) {
        let mut internal = self.0.borrow_mut();
        let cards_to_remove = cards.0.borrow();

        for card in cards_to_remove.iter() {
            internal.remove(card);
        }
    }

    #[must_use]
    pub fn shuffle(&self) -> Self {
        let internal = self.clone();
        internal.shuffle_in_place();
        internal
    }

    /// ```
    /// use pkcore::cards_cell::CardsCell;
    ///
    /// let deck = CardsCell::deck();
    /// deck.shuffle_in_place();
    ///
    /// println!("{deck}");
    /// ```
    pub fn shuffle_in_place(&self) {
        let mut internal = self.0.borrow_mut();
        internal.shuffle_in_place();
    }

    /// ```
    /// use pkcore::cards_cell::CardsCell;
    ///
    /// let deck = CardsCell::deck();
    /// let shuffled_deck = deck.shuffle();
    ///
    /// assert_eq!(shuffled_deck.sort(), deck);
    /// ```
    #[must_use]
    pub fn sort(&self) -> Self {
        let internal = self.clone();
        let cards = internal.0.borrow_mut();
        Self::from(cards.sort())
    }

    pub fn swap(&self, _index: usize, _card: Card) -> Option<Card> {
        // let mut internal = self.0.borrow_mut();
        // internal.
        todo!()
    }

    /// ```
    /// use pkcore::cards_cell::CardsCell;
    ///
    /// let deck = CardsCell::deck();
    /// let shuffled_deck = deck.shuffle();
    /// shuffled_deck.shuffle_in_place();
    ///
    /// assert_eq!(shuffled_deck, deck);
    /// ```
    pub fn sort_in_place(&self) {
        let mut internal = self.0.borrow_mut();
        internal.sort_in_place();
    }

    /// Takes the value of the cell, leaving `Default::default()` in its place.
    ///
    /// ```
    /// use pkcore::cards::Cards;
    /// use pkcore::cards_cell::CardsCell;
    ///
    /// let cards_cell = CardsCell::deck();
    ///
    /// assert_eq!(cards_cell.take(), Cards::deck());
    /// assert_eq!(cards_cell, CardsCell::default());
    /// ```
    pub fn take(&self) -> Cards {
        self.0.take()
    }
}

impl Display for CardsCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let internal = self.0.borrow_mut();
        write!(f, "{internal}")
    }
}

impl Eq for CardsCell {}

impl PartialEq for CardsCell {
    fn eq(&self, other: &Self) -> bool {
        let self_internal = self.0.borrow_mut().clone();
        let other_internal = other.0.borrow_mut().clone();
        self_internal == other_internal
    }
}

impl Hash for CardsCell {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let internal = self.0.borrow_mut();
        internal.hash(state);
    }
}

impl From<Bard> for CardsCell {
    fn from(bard: Bard) -> Self {
        CardsCell::from(Cards::from(bard))
    }
}

impl From<Box<[Card]>> for CardsCell {
    fn from(boxed_cards: Box<[Card]>) -> Self {
        Self(RefCell::new(Cards::from(boxed_cards.as_ref())))
    }
}

impl From<BoxedCards> for CardsCell {
    fn from(boxed_cards: BoxedCards) -> Self {
        Self(RefCell::new(Cards::from(boxed_cards)))
    }
}

impl From<Cards> for CardsCell {
    fn from(cards: Cards) -> Self {
        Self(RefCell::new(cards))
    }
}

impl From<&Cards> for CardsCell {
    fn from(cards: &Cards) -> Self {
        Self(RefCell::new(cards.clone()))
    }
}

impl From<Vec<Card>> for CardsCell {
    fn from(cards: Vec<Card>) -> Self {
        CardsCell::from(Cards::from(cards))
    }
}

impl FromStr for CardsCell {
    type Err = PKError;

    /// ```
    /// use pkcore::cards_cell::CardsCell;
    /// use std::str::FromStr;
    ///
    /// let cards_cell = CardsCell::from_str("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣").unwrap();
    /// assert_eq!(cards_cell.len(), 52);
    /// assert_eq!(
    ///    cards_cell.to_string(),
    ///   "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣"
    /// );
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards = Cards::from_str(s)?;
        Ok(CardsCell::from(cards))
    }
}

impl Pile for CardsCell {
    fn card_at(self, _index: usize) -> Option<Card> {
        todo!()
    }

    fn clean(&self) -> Self {
        todo!()
    }

    fn contains(&self, card: &Card) -> bool {
        let internal = self.0.borrow();
        internal.contains(card)
    }

    fn swap(&mut self, _tttttttttindex: usize, _card: Card) -> Option<Card> {
        todo!()
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        let internal = self.0.borrow();
        internal.to_vec()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod cards_cell_tests {
    use super::*;
    use crate::Forgiving;
    use rstest::rstest;

    #[test]
    fn draw_all() {
        let deck = CardsCell::deck();

        let drawn = deck.draw_all();

        assert_eq!(deck.len(), 0);
        assert_eq!(drawn.len(), 52);
    }

    #[rstest]
    #[case(Card::ACE_SPADES, true)]
    #[case(Card::ACE_DIAMONDS, false)]
    fn pile__contains(#[case] card: Card, #[case] assert: bool) {
        let cards = CardsCell::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

        assert_eq!(cards.contains(&card), assert);
    }

    #[test]
    fn pile__to_vec() {
        let cards = deck_cell!();

        let back_again = CardsCell::from(cards.to_vec());

        assert_eq!(cards, back_again);
    }

    #[test]
    fn macro__cc() {
        let cards = cc!("AS KH QC JD TC 9H 8D");

        assert_eq!("A♠ K♥ Q♣ J♦ T♣ 9♥ 8♦", cards.to_string());
    }
}
