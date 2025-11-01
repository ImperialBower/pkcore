use crate::card::Card;
use crate::{Cards, Pile};
use crate::{PKError, cards};
use std::fmt::Display;
use std::str::FromStr;

#[macro_export]
#[allow(clippy::pedantic)]
macro_rules! boxed {
    ($card_str:expr) => {
        BoxedCards::from(Cards::forgiving_from_str($card_str))
    };
}

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
    /// assert!(BoxedCards::default().is_empty());
    /// assert!(!BoxedCards::from_str("T♠ 2♠").unwrap().is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
}

impl Display for BoxedCards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let box_strings: Vec<String> = self.0.iter().map(std::string::ToString::to_string).collect();
        write!(f, "{}", box_strings.join(" "))
    }
}

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

impl FromStr for BoxedCards {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards = cards!(s);
        Ok(BoxedCards::from(cards))
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
    pub fn blanks(size: usize, count: usize) -> Self {
        Boxes::from(vec![BoxedCards::blanks(size); count])
    }

    /// Creates `Boxes` by dividing the provided Cards into equal sizes.
    ///
    /// ```
    /// use pkcore::cards;
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
}

impl Display for Boxes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let box_strings: Vec<String> = self.0.iter().map(std::string::ToString::to_string).collect();
        write!(f, "{}", box_strings.join(", "))
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

// #[cfg(test)]
// #[allow(non_snake_case)]
// mod arrays__sliced_tests {
//     use super::*;
//     use crate::cards;
// }
