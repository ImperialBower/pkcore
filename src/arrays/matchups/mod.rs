use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::PKError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, PartialOrd)]
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
