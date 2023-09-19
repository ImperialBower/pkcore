use crate::PKError;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign};
use thousands::Separable;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Chips(usize);

impl Chips {
    #[must_use]
    pub fn starting(stack: usize) -> Chips {
        Chips(stack)
    }

    /// # Errors
    ///
    /// Returns `PKError::Busted` if there are no chips.
    pub fn all_in(&mut self) -> Result<Chips, PKError> {
        if self.stack() == 0 {
            Err(PKError::Busted)
        } else {
            let all = *self;
            self.0 = 0;
            Ok(all)
        }
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
    pub fn is_busted(&self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub fn stack(&self) -> usize {
        self.0
    }

    pub fn win(&mut self, winnings: Chips) {
        *self += winnings;
    }
}

impl Add for Chips {
    type Output = Chips;

    fn add(self, rhs: Self) -> Self::Output {
        Chips::starting(self.0 + rhs.0)
    }
}

impl AddAssign for Chips {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
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
    fn all_in() {
        let mut starting = Chips::starting(1_000);
        let expected = starting.clone();

        let bet = starting.all_in();

        assert!(bet.is_ok());
        assert_eq!(expected, bet.unwrap());
        assert_eq!(0, starting.stack());
    }

    #[test]
    fn all_in__busted() {
        let mut starting = Chips::default();

        let busted = starting.all_in();

        assert!(busted.is_err());
        assert_eq!(PKError::Busted, busted.unwrap_err());
        assert_eq!(starting, Chips::default());
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
    fn bet__insufficient() {
        let mut starting = Chips::starting(1_000);

        let bet = starting.bet(1_001);

        assert!(bet.is_err());
        assert_eq!(PKError::InsufficientChips, bet.unwrap_err());
    }

    #[test]
    fn win() {
        let mut starting = Chips::starting(1_000);

        starting.win(Chips::starting(1_000_000));

        assert_eq!(Chips::starting(1_001_000), starting);
    }

    #[test]
    fn add() {
        let sum = Chips::starting(1_000) + Chips::starting(1);

        assert_eq!(Chips::starting(1_001), sum);
    }

    #[test]
    fn default() {
        assert_eq!(Chips::default().stack(), 0);
    }
}
