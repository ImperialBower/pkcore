use crate::PKError;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::FromStr;

#[macro_export]
macro_rules! deck_cell {
    () => {
        CardsCell::deck()
    };
}

#[derive(Clone, Debug, Default)]
pub struct CardsCell(RefCell<Cards>);

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
        Self::new(Cards::deck())
    }

    /// Creates a new `CardsCell` containing the given `Cards`.
    #[must_use]
    pub fn new(cards: Cards) -> Self {
        Self(RefCell::new(cards))
    }

    #[must_use]
    pub fn deck_minus(cards: &Cards) -> CardsCell {
        Self::new(Cards::deck_minus(cards))
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
        Ok(Self::new(drawn_cards))
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
        Ok(Self::new(drawn_cards))
    }

    pub fn dump(&self) {
        let internal = self.0.borrow_mut();
        internal.dump();
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
        Self::new(cards.sort())
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
    pub fn sort_in_place(&mut self) {
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
        CardsCell::new(Cards::from(bard))
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
        Ok(CardsCell::new(cards))
    }
}
