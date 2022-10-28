use crate::arrays::two::Two;
use crate::Pile;
use crate::{Card, PKError, TheNuts};

/// # PHASE FIVE: Concurrency
///
/// ## Take Two: Concurrency with Copy
///
/// I will confess that I am addicted to types in rust that implement the `Copy` trait. There
/// is so much joy in not having to worry about ownership. Part of me feels that this is a
/// total cop out. Another part of me thinks that this is smart, since, fundamentally, the
/// data I am working with is all collections of unsigned integers.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TwoBy2 {
    pub first: Two,
    pub second: Two,
}

impl TwoBy2 {
    /// # Errors
    ///
    /// Throws a `PKError::NotDealt` error if the hand isn't complete.
    pub fn new(first: Two, second: Two) -> Result<TwoBy2, PKError> {
        if first.is_dealt() && second.is_dealt() {
            Ok(TwoBy2 { first, second })
        } else {
            Err(PKError::NotDealt)
        }
    }
}

impl Pile for TwoBy2 {
    fn clean(&self) -> Self {
        TwoBy2::default()
    }

    fn the_nuts(&self) -> TheNuts {
        TheNuts::default()
    }

    fn to_vec(&self) -> Vec<Card> {
        vec![
            self.first.first(),
            self.first.second(),
            self.second.first(),
            self.second.second(),
        ]
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__two_by_2_tests {
    use super::*;

    #[test]
    fn new() {
        let expected = TwoBy2 {
            first: Two::HAND_JS_TH,
            second: Two::HAND_9H_9D,
        };

        let actual = TwoBy2::new(Two::HAND_JS_TH, Two::HAND_9H_9D);

        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn new__hands_not_dealt() {
        let actual = TwoBy2::new(Two::HAND_9H_9D, Two::default());

        assert!(actual.is_err());
        assert_eq!(PKError::NotDealt, actual.unwrap_err());
        assert!(TwoBy2::new(Two::default(), Two::HAND_9H_9D).is_err());
        assert!(TwoBy2::new(Two::default(), Two::default()).is_err());
    }

    #[test]
    fn pile__to_vec() {
        let actual = TwoBy2::new(Two::HAND_JC_4H, Two::HAND_8C_7C)
            .unwrap()
            .to_vec();

        let expected = vec![
            Card::JACK_CLUBS,
            Card::FOUR_HEARTS,
            Card::EIGHT_CLUBS,
            Card::SEVEN_CLUBS,
        ];

        assert_eq!(expected, actual);
    }
}
