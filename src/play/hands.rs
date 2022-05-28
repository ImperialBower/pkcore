use crate::arrays::two::Two;
use crate::cards::Cards;
use crate::PKError;

/// To start with I am only focusing on supporting a single round of play.
///
/// let mut v = Vec::with_capacity(10);
#[derive(Clone, Debug, PartialEq)]
pub struct Hands(Vec<Two>);

impl Hands {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Hands {
        Hands(Vec::with_capacity(capacity))
    }
}

// impl TryFrom<Cards> for Hands {
//     type Error = PKError;
//
//     fn try_from(cards: Cards) -> Result<Self, Self::Error> {
//         let mut cards = cards;
//
//         if cards.len() % 2 = 0 {
//             let num_of_players = cards.len() / 2;
//             let mute hands = Hands::with_capacity(num_of_players);
//
//
//         } else {
//             Err(PKError::InvalidCardCount)
//         }
//     }
// }
