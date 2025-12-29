use crate::PKError;
use crate::card::Card;
use crate::cards_cell::CardsCell;
use crate::prelude::{Seats, TheNuts};
use crate::util::terminal::Terminal;
use crate::{Cards, Forgiving, Pile};
use std::fmt::Display;
use std::str::FromStr;

/// This is an attempt at a refactoring of could be seen as the abomination that is my
/// arrays structs. They do have the advantage of being geared for my direct use cases within
/// the hand analysis, but I am feeling that in the future that would be better suited
/// to traits instead of what currently is.
/// ```
/// use pkcore::prelude::*;
///
/// let index = "T♠ 2♠";
///
/// let boxed_cards =  boxed!(index);
///
/// assert_eq!(boxed_cards.to_string(), index);
/// ```
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoxedCards(Box<[Card]>);

impl BoxedCards {
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let blanks = BoxedCards::blanks(3);
    ///
    /// assert_eq!(3, blanks.len());
    /// assert_eq!("__ __ __", blanks.to_string());
    /// ```
    #[must_use]
    pub fn blanks(len: usize) -> Self {
        BoxedCards(vec![Card::BLANK; len].into_boxed_slice())
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let mut boxed_cards = BoxedCards::from_str("T♠ __ __").unwrap();
    /// assert_eq!("T♠ __ __", boxed_cards.to_string());
    /// boxed_cards.deal(Card::ACE_DIAMONDS);
    ///
    /// assert_eq!("T♠ A♦ __", boxed_cards.to_string());
    /// ```
    ///
    /// # Errors
    ///
    /// `PKError::NoBlankSlots` if there are no blank slots to deal into.
    pub fn deal(&mut self, card: Card) -> Result<(), PKError> {
        for slot in &mut self.0 {
            if *slot == Card::BLANK {
                *slot = card;
                return Ok(());
            }
        }

        Err(PKError::NoBlankSlots)
    }

    // /// Idea stolen from my `CardPack.rs` library.
    // ///
    // /// This is exactly the same code as in the `Cards` struct, the difference being that
    // /// `BoxedCards` struct's from_str accepts `Card::BLANK` as a valid card.
    // #[must_use]
    // pub fn forgiving_from_str(index: &str) -> Self {
    //     Self::from_str(index).unwrap_or_else(|_| {
    //         log::warn!("BoxedCards::forgiving_from_str(): {index} is invalid. Returning empty Pile.");
    //         Self::default()
    //     })
    // }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(boxed!("T♥ 2♠ 8♣ 7♣").has_cards());
    /// assert!(boxed!("__ __ 7♣").has_cards());
    /// assert!(!boxed!("__ __ __").has_cards());
    ///
    /// ```
    #[must_use]
    pub fn has_cards(&self) -> bool {
        self.0.iter().any(|c| *c != Card::BLANK)
    }

    #[must_use]
    pub fn is_dealt(&self) -> bool {
        !self.0.contains(&Card::BLANK)
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(BoxedCards::default().is_empty());
    /// assert!(BoxedCards::blanks(3).is_empty());
    /// assert!(!BoxedCards::from_str("T♠ 2♠").unwrap().is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cards().is_empty()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(BoxedCards::default().is_even());
    /// assert!(BoxedCards::from_str("T♠ 2♠").unwrap().is_even());
    /// assert!(BoxedCards::from_str("T♥ 2♠ 8♣ 7♣").unwrap().is_even());
    /// assert!(!BoxedCards::from_str("T♣ 2♠ 3♥").unwrap().is_even());
    /// ```
    #[must_use]
    pub fn is_even(&self) -> bool {
        self.len() % 2 == 0
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert_eq!(0, BoxedCards::default().len());
    /// assert_eq!(2, BoxedCards::from_str("T♠ 2♠").unwrap().len());
    /// assert_eq!(3, BoxedCards::from_str("T♣ 2♠ 3♥").unwrap().len());
    /// assert_eq!(4, BoxedCards::from_str("T♥ 2♠ 8♣ 7♣").unwrap().len());
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn number_of_dealt_cards(&self) -> usize {
        self.0.iter().filter(|c| **c != Card::BLANK).count()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let boxed_cards = BoxedCards::from_str("T♠ 2♠ 8♣").unwrap();
    /// let slice = boxed_cards.as_slice();
    ///
    /// assert_eq!(slice.len(), 3);
    /// assert_eq!(slice[0], Card::from_str("T♠").unwrap());
    /// assert_eq!(slice[1], Card::from_str("2♠").unwrap());
    /// assert_eq!(slice[2], Card::from_str("8♣").unwrap());
    ///
    /// // Returns an empty slice for empty BoxedCards
    /// assert!(BoxedCards::default().as_slice().is_empty());
    /// ```
    #[must_use]
    pub fn as_slice(&self) -> &[Card] {
        &self.0
    }

    pub fn take(&mut self) -> Box<[Card]> {
        let cards = std::mem::take(&mut self.0);
        self.0 = vec![Card::BLANK; cards.len()].into_boxed_slice();
        cards
    }
}

impl Display for BoxedCards {
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert_eq!("A♠ K♦", boxed!("AS KD").to_string());
    /// assert_eq!("A♠ K♦ __", boxed!("AS KD __").to_string());
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let box_strings: Vec<String> = self.0.iter().map(std::string::ToString::to_string).collect();
        write!(f, "{}", box_strings.join(" "))
    }
}

impl Forgiving for BoxedCards {}

impl From<Cards> for BoxedCards {
    fn from(cards: Cards) -> Self {
        BoxedCards::from(cards.to_vec())
    }
}

impl From<Vec<Card>> for BoxedCards {
    fn from(value: Vec<Card>) -> Self {
        BoxedCards(value.into_boxed_slice())
    }
}

impl From<CardsCell> for BoxedCards {
    fn from(cards_cell: CardsCell) -> Self {
        BoxedCards::from(cards_cell.to_vec())
    }
}

impl FromStr for BoxedCards {
    type Err = PKError;

    /// The analysis layer way of doing things isn't going to work at the Table level. Suddenly,
    /// having a `Card::BLANK` in one of the slots is perfectly normal, as cards are dealt out to
    /// the players.
    ///
    /// Here's what I originally wrote:
    ///
    /// ```txt
    /// let cards = cards!(s);
    /// Ok(BoxedCards::from(cards))
    /// ```
    ///
    /// Clean, crisp, clear, and totally wrong.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert_eq!("__ __ __", BoxedCards::from_str("__ __ __").unwrap().to_string());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut v: Vec<Card> = Vec::new();
        let binding = Terminal::index_cleaner(s);
        let s = binding.as_str();
        for s in s.split_whitespace() {
            let c = Card::from_str(s).unwrap_or(Card::BLANK);
            v.push(c);
        }
        if v.is_empty() {
            Err(PKError::InvalidCardIndex)
        } else {
            Ok(BoxedCards::from(v))
        }
    }
}

impl Pile for BoxedCards {
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert_eq!(
    ///     Card::BLANK,
    ///     BoxedCards::from(vec![Card::ACE_DIAMONDS, Card::BLANK]).card_at(1).unwrap()
    /// );
    /// assert_eq!(Card::KING_CLUBS, boxed!("T♥ 2♠ K♣ 7♣").card_at(2).unwrap())
    /// ```
    fn card_at(self, index: usize) -> Option<Card> {
        self.0.get(index).copied()
    }

    fn clean(&self) -> Self {
        BoxedCards::blanks(self.len())
    }

    fn swap(&mut self, index: usize, card: Card) -> Option<Card> {
        match self.0.get(index) {
            Some(old_card) => {
                let old_card = *old_card;
                self.0[index] = card;
                Some(old_card)
            }
            None => None,
        }
    }

    fn the_nuts(&self) -> TheNuts {
        todo!("Doesn't apply")
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Boxes(pub Box<[BoxedCards]>);

impl Boxes {
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let boxes = Boxes::blanks(2, 6);
    ///
    /// assert_eq!(6, boxes.len());
    /// assert_eq!("__ __, __ __, __ __, __ __, __ __, __ __", boxes.to_string());
    /// ```
    #[must_use]
    pub fn blanks(box_size: usize, numer_of_boxes: usize) -> Self {
        Boxes::from(vec![BoxedCards::blanks(box_size); numer_of_boxes])
    }

    /// Creates `Boxes` by dividing the provided Cards into equal sizes.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let cards = cards!("8♣ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♦ 4♣ 7♣ 2♣");
    ///
    /// let boxes = Boxes::box_up(&cards, 2).unwrap();
    ///
    /// assert_eq!(7, boxes.len());
    /// assert_eq!(14, boxes.card_count());
    /// assert!(boxes.is_aligned());
    /// assert_eq!("8♣ 3♥, A♦ Q♣, 5♦ 5♣, 6♠ 6♥, K♠ J♦, 4♦ 4♣, 7♣ 2♣", boxes.to_string());
    /// ```
    ///
    /// # Errors
    ///
    /// `PKError::InvalidLength` if the capacity is zero.
    pub fn box_up(cards: &Cards, capacity: usize) -> Result<Self, PKError> {
        if capacity == 0 {
            return Err(PKError::InvalidLength);
        }

        Ok(Boxes::from(cards.as_chunks(capacity)))
    }

    /// Works the same as `box_up`, but each Card is folded in as if it was dealt.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let deck = cards!("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠");
    ///
    /// let boxes = Boxes::box_up_horizontal(&deck, 2).unwrap();
    ///
    /// assert_eq!("A♠ 8♠, K♠ 7♠, Q♠ 6♠, J♠ 5♠, T♠ 4♠, 9♠ 3♠", boxes.to_string());
    /// ```
    ///
    /// # Errors
    ///
    /// `PKError::InvalidLength` if the capacity is zero.
    /// `PKError::NotEnoughCards` if not enough cards are available.
    pub fn box_up_horizontal(cards: &Cards, box_size: usize) -> Result<Self, PKError> {
        if box_size == 0 {
            return Err(PKError::InvalidLength);
        }

        let number_of_boxes = cards.len() / box_size;
        let mut boxes = Boxes::blanks(box_size, number_of_boxes);

        let ccell = CardsCell::from(cards);

        while !ccell.is_empty() {
            let _ = boxes.deal(0, ccell.draw_one()?);
        }

        Ok(boxes)
    }

    /// Works the same as `box_up`, but verifies that the resulting Boxes are aligned.
    /// Nothing different here.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let cards = cards!("8♣ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♦ 4♣ 7♣ 2♣");
    ///
    /// let boxes = Boxes::box_up_aligned(&cards, 2).unwrap();
    ///
    /// assert_eq!(7, boxes.len());
    /// assert_eq!(14, boxes.card_count());
    /// assert!(boxes.is_aligned());
    /// assert_eq!("8♣ 3♥, A♦ Q♣, 5♦ 5♣, 6♠ 6♥, K♠ J♦, 4♦ 4♣, 7♣ 2♣", boxes.to_string());
    /// ```
    ///
    /// But something very different here.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let cards = cards!("8♣ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♦ 4♣ 7♣");
    ///
    /// let boxes = Boxes::box_up_aligned(&cards, 2);
    ///
    /// assert!(boxes.is_err());
    /// assert_eq!(PKError::Misaligned, boxes.unwrap_err());
    /// ```
    /// # Errors
    ///
    /// `PKError::InvalidLength` if the capacity is zero.
    /// `PKError::Misaligned` if the resulting Boxes are not aligned.
    pub fn box_up_aligned(cards: &Cards, capacity: usize) -> Result<Self, PKError> {
        let boxes = Self::box_up(cards, capacity)?;

        if boxes.is_aligned() {
            Ok(boxes)
        } else {
            Err(PKError::Misaligned)
        }
    }

    /// Verifies if all cards across the Boxes are unique, and so could
    /// have come from the same deck.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(Boxes::from(vec![
    ///    boxed!("T♥ 2♠"),
    ///    boxed!("8♣ 7♣ 9♥"),
    /// ]).are_unique());
    /// assert!(!Boxes::from(vec![
    ///    boxed!("T♥ 2♠"),
    ///    boxed!("8♣ 7♣ 9♥ T♥ 2♠"),
    /// ]).are_unique());
    /// ```
    ///
    #[must_use]
    pub fn are_unique(&self) -> bool {
        Cards::from(self).len() == self.card_count()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let deck = cards!("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠");
    /// let boxes = Boxes::box_up_horizontal(&deck, 2).unwrap();
    ///
    /// assert_eq!("A♠ 8♠, K♠ 7♠, Q♠ 6♠, J♠ 5♠, T♠ 4♠, 9♠ 3♠", boxes.to_string());
    /// assert_eq!("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠", boxes.as_dealt().to_string());
    /// ```
    pub fn as_dealt(&self) -> Cards {
        if self.0.is_empty() {
            return Cards::default();
        }

        let mut dealt_cards = Vec::new();
        let max_len = self.0.iter().map(BoxedCards::len).max().unwrap_or(0);

        for cycle in 0..max_len {
            for boxed in &self.0 {
                if let Some(&card) = boxed.as_slice().get(cycle) {
                    if card != Card::BLANK {
                        dealt_cards.push(card);
                    }
                }
            }
        }

        Cards::from(dealt_cards)
    }

    /// Returns the total number of cards within the `Boxes`.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert_eq!(5, Boxes::from(vec![
    ///    boxed!("T♥ 2♠"),
    ///    boxed!("8♣ 7♣ 9♥"),
    /// ]).card_count());
    /// ```
    #[must_use]
    pub fn card_count(&self) -> usize {
        self.0.iter().map(BoxedCards::len).sum()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let mut boxes = Boxes::blanks(2, 4);
    ///
    /// assert!(boxes.deal(2, Card::ACE_HEARTS).is_ok());
    /// assert_eq!("__ __, __ __, A♥ __, __ __", boxes.to_string());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PKError::AlreadyDealt` is there is no empty slot available.
    pub fn deal_old(&mut self, utg: usize, card: Card) -> Result<(), PKError> {
        let slice = &mut self.0;
        let start = utg;

        for i in 0..slice.len() {
            let index = (start + i) % slice.len();
            let boxed = &mut slice[index];
            if boxed.deal(card).is_ok() {
                return Ok(());
            }
        }

        Err(PKError::AlreadyDealt)
    }

    /// Deals one card to each `BoxedCards` in sequence before dealing another card
    /// to the starting position.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let mut boxes = Boxes::blanks(2, 4);
    ///
    /// assert!(boxes.deal(2, Card::ACE_HEARTS).is_ok());
    /// assert_eq!("__ __, __ __, A♥ __, __ __", boxes.to_string());
    ///
    /// assert!(boxes.deal(2, Card::KING_SPADES).is_ok());
    /// assert_eq!("__ __, __ __, A♥ __, K♠ __", boxes.to_string());
    ///
    /// assert!(boxes.deal(2, Card::QUEEN_DIAMONDS).is_ok());
    /// assert_eq!("Q♦ __, __ __, A♥ __, K♠ __", boxes.to_string());
    ///
    /// assert!(boxes.deal(2, Card::QUEEN_CLUBS).is_ok());
    /// assert_eq!("Q♦ __, Q♣ __, A♥ __, K♠ __", boxes.to_string());
    ///
    /// assert!(boxes.deal(2, Card::TEN_CLUBS).is_ok());
    /// assert_eq!("Q♦ __, Q♣ __, A♥ T♣, K♠ __", boxes.to_string());
    /// ```
    ///
    /// ## Diary
    ///
    /// This is me admitting defeat against a problem that I am just too tired to try to tackle
    /// right now. I want to do this cute thing where it deals the way a dealer sends one card to
    /// each player in a circle, instead of all of the cards to each player at a time. I let Claude
    /// do it and it got it right, even if it feels like there is a real refactoring opportunity
    /// here.
    ///
    /// I worry that the more I use `CoPilot`, the worse a programmer I will become.
    ///
    /// Here's the prompt I used: `Can you refactor the Boxes.deal function so that it deals one
    /// card to each BoxedCards struct before dealing another card to the original BoxedCards?`
    ///
    /// The original version of the code was:
    ///
    /// ```txt
    /// use pkcore::prelude::*;
    ///
    /// pub fn deal_old(&mut self, utg: usize, card: Card) -> Result<(), PKError> {
    ///     let slice = &mut self.0;
    ///     let start = utg;
    ///
    ///     for i in 0..slice.len() {
    ///         let index = (start + i) % slice.len();
    ///         let boxed = &mut slice[index];
    ///         if boxed.deal(card).is_ok() {
    ///             return Ok(());
    ///         }
    ///     }
    ///
    ///     Err(PKError::AlreadyDealt)
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PKError::AlreadyDealt` if there is no empty slot available.
    pub fn deal(&mut self, utg: usize, card: Card) -> Result<(), PKError> {
        let slice = &mut self.0;
        let len = slice.len();

        // Find the minimum number of dealt cards to determine current round
        let min_dealt = slice.iter().map(BoxedCards::number_of_dealt_cards).min().unwrap_or(0);

        // Try to deal to boxes that have exactly min_dealt cards (current round)
        for i in 0..len {
            let index = (utg + i) % len;
            let boxed = &mut slice[index];

            if boxed.number_of_dealt_cards() == min_dealt && boxed.deal(card).is_ok() {
                return Ok(());
            }
        }

        Err(PKError::AlreadyDealt)
    }

    /// Returns cards in the order they would need to be dealt to recreate the current state.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let mut boxes = Boxes::blanks(2, 3);
    /// boxes.deal(1, Card::ACE_HEARTS).unwrap();
    /// boxes.deal(1, Card::KING_SPADES).unwrap();
    /// boxes.deal(1, Card::QUEEN_DIAMONDS).unwrap();
    /// boxes.deal(1, Card::JACK_CLUBS).unwrap();
    /// boxes.deal(1, Card::TEN_HEARTS).unwrap();
    /// boxes.deal(1, Card::NINE_SPADES).unwrap();
    ///
    /// assert_eq!("Q♦ 9♠, A♥ J♣, K♠ T♥", boxes.to_string());
    ///
    /// let undealt = boxes.undeal(1);
    /// assert_eq!("A♥ K♠ Q♦ J♣ T♥ 9♠", undealt.to_string());
    ///
    /// // Verify we can recreate the boxes
    /// let mut new_boxes = Boxes::blanks(2, 3);
    /// for card in undealt.to_vec() {
    ///     new_boxes.deal(1, card).unwrap();
    /// }
    /// assert_eq!(boxes.to_string(), new_boxes.to_string());
    /// ```
    pub fn undeal(&self, utg: usize) -> Cards {
        let len = self.0.len();
        if len == 0 {
            return Cards::default();
        }

        // Find max number of cards any BoxedCards has
        let max_cards = self.0.iter().map(BoxedCards::number_of_dealt_cards).max().unwrap_or(0);

        let mut result = Vec::new();

        // For each round (0 to max_cards)
        for round in 0..max_cards {
            // Go through each position starting from utg
            for i in 0..len {
                let index = (utg + i) % len;
                let boxed = &self.0[index];

                // Get the card at this round position if it exists and isn't blank
                if let Some(&card) = boxed.as_slice().get(round) {
                    if card != Card::BLANK {
                        result.push(card);
                    }
                }
            }
        }

        Cards::from(result)
    }

    /// # Errors
    ///
    /// `PKError::InvalidPosition` if the provided `box_index` is out of range.
    /// `PKError::NoBlankSlots` if there are no blank slots to deal into.
    pub fn deal_at(&mut self, box_index: usize, card: Card) -> Result<(), PKError> {
        match self.0.get_mut(box_index) {
            Some(boxed_cards) => boxed_cards.deal(card),
            None => Err(PKError::InvalidPosition),
        }
    }

    /// Verifies that the individual `BoxedCards` are all of the same length.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(Boxes::from(vec![
    ///    boxed!("T♥ 2♠"),
    ///    boxed!("8♣ 7♣"),
    /// ]).is_aligned());
    ///
    /// assert!(!Boxes::from(vec![
    ///    boxed!("T♥ 2♠"),
    ///    boxed!("8♣ 7♣ 9♥"),
    /// ]).is_aligned());
    /// ```
    #[must_use]
    pub fn is_aligned(&self) -> bool {
        if self.is_empty() {
            return true;
        }

        let first_len = self.0[0].len();
        self.0.iter().all(|b| b.len() == first_len)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn is_dealt(&self) -> bool {
        self.0.iter().all(BoxedCards::is_dealt)
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let boxes = Boxes::from(vec![
    ///     boxed!("T♥ 2♠"),
    ///     boxed!("8♣ 7♣ 9♥"),
    /// ]);
    ///
    /// assert_eq!(2, boxes.len());
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let cards = cards!("8♣ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♦ 4♣");
    /// let boxes = Boxes::box_up_aligned(&cards, 2).unwrap();
    ///
    /// assert_eq!(12, boxes.number_of_card_slots());
    /// assert_eq!(12, Boxes::blanks(4, 3).number_of_card_slots());
    /// ```
    #[must_use]
    pub fn number_of_card_slots(&self) -> usize {
        self.0.iter().map(BoxedCards::len).sum()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let cards = cards!("8♣ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♦ 4♣");
    /// let boxes = Boxes::box_up_aligned(&cards, 2).unwrap();
    ///
    /// assert_eq!(12, boxes.number_of_dealt_cards());
    /// assert_eq!(0, Boxes::blanks(4, 3).number_of_dealt_cards());
    /// ```
    #[must_use]
    pub fn number_of_dealt_cards(&self) -> usize {
        self.0.iter().map(BoxedCards::number_of_dealt_cards).sum()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let mut boxes = Boxes::blanks(4, 3);
    /// assert_eq!("__ __ __ __, __ __ __ __, __ __ __ __", boxes.to_string());
    ///
    /// let old_card = boxes.swap(1, 2, Card::KING_HEARTS);
    /// assert_eq!(old_card, Some(Card::BLANK));
    /// assert_eq!("__ __ __ __, __ __ K♥ __, __ __ __ __", boxes.to_string());
    /// ```
    pub fn swap(&mut self, box_index: usize, card_index: usize, card: Card) -> Option<Card> {
        match self.0.get(box_index) {
            Some(boxed_cards) => {
                let mut boxed_cards = boxed_cards.clone();
                let old_card = boxed_cards.swap(card_index, card);
                self.0[box_index] = boxed_cards;
                old_card
            }
            None => None,
        }
    }
}

impl Display for Boxes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let box_strings: Vec<String> = self.0.iter().map(std::string::ToString::to_string).collect();
        write!(f, "{}", box_strings.join(", "))
    }
}

impl From<&Seats> for Boxes {
    fn from(seats: &Seats) -> Self {
        Boxes::from(
            seats
                .borrow_all()
                .iter()
                .map(|seat_cell| seat_cell.borrow().cards.clone())
                .collect::<Vec<_>>(),
        )
    }
}

impl From<Vec<BoxedCards>> for Boxes {
    fn from(value: Vec<BoxedCards>) -> Self {
        Boxes(value.into_boxed_slice())
    }
}

impl From<Vec<Vec<Card>>> for Boxes {
    fn from(v: Vec<Vec<Card>>) -> Self {
        Boxes::from(v.into_iter().map(BoxedCards::from).collect::<Vec<_>>())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__sliced_tests {
    use super::*;
    use crate::util::data::TestData;

    #[test]
    fn number_of_dealt_cards() {
        let blank_boxed: BoxedCards = boxed!("__ __");

        assert_eq!(2, blank_boxed.len());
        assert_eq!(0, blank_boxed.number_of_dealt_cards());
        assert_eq!(2, boxed!("T♣ __ 3♥").number_of_dealt_cards());
        assert_eq!(3, boxed!("T♣ 2♠ 3♥").number_of_dealt_cards());
    }

    #[test]
    fn take() {
        let mut boxed_cards: BoxedCards = boxed!("T♥ 2♠ A♦ 7♣");
        let taken = boxed_cards.take();

        assert_eq!(boxed!("T♥ 2♠ A♦ 7♣").to_vec(), taken.to_vec());
        assert_eq!("__ __ __ __", boxed_cards.to_string());
    }

    #[test]
    fn boxed_cards__swap() {
        let mut boxed_cards: BoxedCards = boxed!("T♥ 2♠ __ 7♣");
        let swapped = boxed_cards.swap(2, Card::ACE_DIAMONDS);

        assert_eq!(Some(Card::BLANK), swapped);
        assert_eq!("T♥ 2♠ A♦ 7♣", boxed_cards.to_string());
        assert_eq!(None, boxed_cards.swap(6, Card::KING_CLUBS));
    }

    /// OK, I officially hate it when `CoPilot` farts out a test that is actually pretty good.
    #[test]
    fn boxed_cards_deal() {
        let mut boxed_cards: BoxedCards = boxed!("T♥ __ __ 7♣");

        assert!(boxed_cards.deal(Card::ACE_DIAMONDS).is_ok());
        assert_eq!("T♥ A♦ __ 7♣", boxed_cards.to_string());

        assert!(boxed_cards.deal(Card::KING_CLUBS).is_ok());
        assert_eq!("T♥ A♦ K♣ 7♣", boxed_cards.to_string());

        let result = boxed_cards.deal(Card::QUEEN_SPADES);
        assert!(result.is_err());
        assert_eq!(PKError::NoBlankSlots, result.unwrap_err());
    }

    #[test]
    fn boxed_cards_deal__basic() {
        let mut deck = TestData::the_hand_cards_dealable();
        let mut boxes = Boxes::blanks(2, 7);

        while !boxes.is_dealt() {
            let card = deck.draw_one().unwrap();
            boxes.deal(0, card).unwrap();
            println!("{boxes}");
        }

        assert!(boxes.deal(0, deck.draw_one().unwrap()).is_err());
    }
}
