use crate::analysis::the_nuts::TheNuts;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use crate::{PKError, Pile};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct SortedHeadsUp {
    higher: Two,
    lower: Two,
}

impl SortedHeadsUp {
    #[must_use]
    pub fn new(first: Two, second: Two) -> SortedHeadsUp {
        if first > second {
            SortedHeadsUp {
                higher: first,
                lower: second,
            }
        } else {
            SortedHeadsUp {
                higher: second,
                lower: first,
            }
        }
    }

    #[must_use]
    pub fn contains(&self, two: &Two) -> bool {
        &self.lower == two || &self.higher == two
    }

    #[must_use]
    pub fn higher(&self) -> Two {
        self.higher
    }

    #[must_use]
    pub fn higher_as_bard(&self) -> Bard {
        todo!()
    }

    #[must_use]
    pub fn lower(&self) -> Two {
        self.lower
    }

    /// This is going to be a heavy calculation.
    ///
    /// Oops!
    /// ```txt
    /// error[E0599]: the method `insert` exists for struct `HashSet<SortedHeadsUp>`, but its trait bounds were not satisfied
    ///   --> src/arrays/matchups/mod.rs:63:20
    ///    |
    /// 13 | pub struct SortedHeadsUp {
    ///    | ------------------------ doesn't satisfy `SortedHeadsUp: Hash`
    /// ...
    /// 63 |                 hs.insert(SortedHeadsUp::new(hero, villain));
    ///    |                    ^^^^^^
    ///    |
    ///    = note: the following trait bounds were not satisfied:
    ///            `SortedHeadsUp: Hash`
    /// help: consider annotating `SortedHeadsUp` with `#[derive(Hash)]`
    ///    |
    /// 13 + #[derive(Hash)]
    /// 14 | pub struct SortedHeadsUp {
    ///    |
    /// ```
    ///
    /// Man, I love the [Rust compiler](https://doc.rust-lang.org/std/collections/struct.HashSet.html#examples).
    /// Much love to everyone who worked on this thing.
    ///
    /// # Errors
    ///
    /// Shrugs.
    pub fn all_possible() -> Result<HashSet<SortedHeadsUp>, PKError> {
        let mut hs: HashSet<SortedHeadsUp> = HashSet::new();
        for v in Cards::deck().combinations(2) {
            let hero = Two::try_from(v.as_slice())?;
            for r in hero.remaining().combinations(2) {
                let villain = Two::try_from(r.as_slice())?;
                hs.insert(SortedHeadsUp::new(hero, villain));
            }
        }
        Ok(hs)
    }
}

impl Display for SortedHeadsUp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.higher, self.lower)
    }
}

impl Pile for SortedHeadsUp {
    /// Shoot. Forgot about my frequency mask idea. Still has potential, but later.
    fn clean(&self) -> Self {
        todo!()
    }

    /// Implementing this would be interesting. What's the best possible hand from either of these
    /// two hands?
    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    /// This is the only one we need to implement for what we want. Maybe this interface is doing
    /// too much.
    fn to_vec(&self) -> Vec<Card> {
        let mut v = self.higher.to_vec();
        v.extend(self.lower.to_vec());
        v
    }
}

impl TryFrom<Vec<Two>> for SortedHeadsUp {
    type Error = PKError;

    fn try_from(v: Vec<Two>) -> Result<Self, Self::Error> {
        match v.len() {
            0..=1 => Err(PKError::NotEnoughHands),
            2 => Ok(SortedHeadsUp::new(
                *v.get(0).ok_or(PKError::InvalidHand)?,
                *v.get(1).ok_or(PKError::InvalidHand)?,
            )),
            _ => Err(PKError::TooManyHands),
        }
    }
}

/// TODO do I need this? I'm overthinking and underknowing.
impl TryFrom<Vec<&Two>> for SortedHeadsUp {
    type Error = PKError;

    fn try_from(v: Vec<&Two>) -> Result<Self, Self::Error> {
        match v.len() {
            0..=1 => Err(PKError::NotEnoughHands),
            2 => Ok(SortedHeadsUp::new(
                **v.get(0).ok_or(PKError::InvalidHand)?,
                **v.get(1).ok_or(PKError::InvalidHand)?,
            )),
            _ => Err(PKError::TooManyHands),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__sorted_heads_up {
    use super::*;
    use crate::util::data::TestData;

    const EXPECTED: SortedHeadsUp = SortedHeadsUp {
        higher: Two::HAND_7D_7C,
        lower: Two::HAND_6S_6H,
    };

    #[test]
    fn new() {
        assert_eq!(
            EXPECTED,
            SortedHeadsUp::new(Two::HAND_6S_6H, Two::HAND_7D_7C)
        );
        assert_eq!(
            EXPECTED,
            SortedHeadsUp::new(Two::HAND_7D_7C, Two::HAND_6S_6H)
        );
    }

    #[test]
    fn contains() {
        assert!(EXPECTED.contains(&Two::HAND_6S_6H));
        assert!(EXPECTED.contains(&Two::HAND_7D_7C));
        assert!(!EXPECTED.contains(&Two::HAND_7S_7C));
    }

    #[test]
    fn display() {
        assert_eq!(
            "6♠ 6♥ - 5♦ 5♣",
            TestData::the_hand_sorted_headsup().to_string()
        );
    }

    #[test]
    fn pile__to_vec() {
        assert_eq!(
            EXPECTED.to_vec(),
            vec![
                Card::SEVEN_DIAMONDS,
                Card::SEVEN_CLUBS,
                Card::SIX_SPADES,
                Card::SIX_HEARTS
            ]
        );
    }

    /// I don't believe that I need this test. The foundations are already tested. Still, I like
    /// doing double checks. Part of me is just like how cool is it that I can even do this?!
    #[test]
    fn pile__remaining() {
        assert_eq!(EXPECTED.remaining().sort().to_string(), "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 6♣ 5♣ 4♣ 3♣ 2♣");
    }

    #[test]
    fn try_from() {
        let v = vec![Two::HAND_6S_6H, Two::HAND_7D_7C];

        assert_eq!(EXPECTED, SortedHeadsUp::try_from(v).unwrap());
    }

    #[test]
    fn try_from__error() {
        assert_eq!(
            PKError::NotEnoughHands,
            SortedHeadsUp::try_from(vec![Two::HAND_6S_6H]).unwrap_err()
        );
        assert_eq!(
            PKError::TooManyHands,
            SortedHeadsUp::try_from(vec![Two::HAND_6S_6H, Two::HAND_7S_7C, Two::HAND_2D_2C])
                .unwrap_err()
        );
    }
}
