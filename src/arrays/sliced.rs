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
/// let boxed_cards = BoxedCards::from_str(index).unwrap();
///
/// assert_eq!(boxed_cards.to_string(), index);
/// ```
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoxedCards(Box<[Card]>);

impl BoxedCards {
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
        Cards::from(self.as_slice()).fmt(f)
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

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__sliced_tests {
    use super::*;
    use crate::cards;
}
