use crate::PKError;
use std::fmt::{Display, Formatter};
use thousands::Separable;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Chips(usize);

impl Chips {
    #[must_use]
    pub fn starting(stack: usize) -> Chips {
        Chips(stack)
    }

    /// # Errors
    ///
    /// Returns `PKError::InsufficientChips` if there are insufficient chips.
    pub fn bet(&mut self, amount: usize) -> Result<Chips, PKError> {
        if self.stack() < amount {
            Err(PKError::InsufficientChips)
        } else {
            self.0 -= amount;
            Ok(Chips::starting(amount))
        }
    }

    #[must_use]
    pub fn stack(&self) -> usize {
        self.0
    }
}

impl Display for Chips {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.stack().separate_with_commas())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__chips_tests {
    use super::*;

    #[test]
    fn starting() {
        let chips = Chips::starting(1_000);

        assert_eq!(chips.stack(), 1_000);
    }

    #[test]
    fn bet() {
        let mut starting = Chips::starting(1_000);
        let expected = Chips::starting(50);

        let bet = starting.bet(50);

        assert!(bet.is_ok());
        assert_eq!(expected, bet.unwrap());
        assert_eq!(950, starting.stack());
    }

    #[test]
    fn default() {
        assert_eq!(Chips::default().stack(), 0);
    }
}
