use crate::arrays::two::Two;
use crate::PKError;
use crate::Pile;

/// # PHASE FIVE: Concurrency
///
/// ## Take Two: Concurrency with Copy
///
///
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

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__two_by_2_tests {
    use super::*;

    #[test]
    fn new() {
        let expected = TwoBy2 {
            first: Two::HAND_9H_9D,
            second: Two::HAND_JS_TH,
        };

        let actual = TwoBy2::new(Two::HAND_9H_9D, Two::HAND_JS_TH);

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
}
