use crate::{Betting, PKError};
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};
use thousands::Separable;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Chips(usize);

impl Chips {
    #[must_use]
    pub fn new(stack: usize) -> Chips {
        Chips(stack)
    }

    #[must_use]
    pub fn stack(&self) -> usize {
        self.0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub fn remove(&mut self, chips: Chips) -> Option<Chips> {
        if self.is_empty() || (chips.stack() > self.stack()) {
            None
        } else {
            self.0 -= chips.0;
            Some(chips)
        }
    }
}

impl Add for Chips {
    type Output = Chips;

    fn add(self, rhs: Self) -> Self::Output {
        Chips::new(self.0 + rhs.0)
    }
}

impl AddAssign for Chips {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Chips {
    type Output = Chips;

    fn sub(self, rhs: Self) -> Self::Output {
        Chips::new(self.0 - rhs.0)
    }
}

impl SubAssign for Chips {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Betting for Chips {
    fn all_in(&mut self) -> Result<Chips, PKError> {
        if self.size() == 0 {
            Err(PKError::Busted)
        } else {
            let all = *self;
            self.0 = 0;
            Ok(all)
        }
    }

    fn bet(&mut self, amount: usize) -> Result<Chips, PKError> {
        if self.size() < amount {
            Err(PKError::InsufficientChips)
        } else {
            self.0 -= amount;
            Ok(Chips::new(amount))
        }
    }

    fn size(&self) -> usize {
        self.0
    }

    fn wins(&mut self, winnings: Chips) -> usize {
        *self += winnings;
        self.size()
    }
}

impl Display for Chips {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.size().separate_with_commas())
    }
}

impl From<usize> for Chips {
    fn from(value: usize) -> Self {
        Chips::new(value)
    }
}

impl From<u8> for Chips {
    fn from(value: u8) -> Self {
        Chips::new(value as usize)
    }
}

impl From<u16> for Chips {
    fn from(value: u16) -> Self {
        Chips::new(value as usize)
    }
}

impl From<u32> for Chips {
    fn from(value: u32) -> Self {
        Chips::new(value as usize)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__chips_tests {
    use super::*;

    #[test]
    fn starting() {
        let chips = Chips::new(1_000);

        assert_eq!(chips.size(), 1_000);
    }

    #[test]
    fn all_in() {
        let mut starting = Chips::new(1_000);
        let expected = starting.clone();

        let bet = starting.all_in();

        assert!(bet.is_ok());
        assert_eq!(expected, bet.unwrap());
        assert_eq!(0, starting.size());
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
        let mut starting = Chips::new(1_000);
        let expected = Chips::new(50);

        let bet = starting.bet(50);

        assert!(bet.is_ok());
        assert_eq!(expected, bet.unwrap());
        assert_eq!(950, starting.size());
    }

    #[test]
    fn bet__insufficient() {
        let mut starting = Chips::new(1_000);

        let bet = starting.bet(1_001);

        assert!(bet.is_err());
        assert_eq!(PKError::InsufficientChips, bet.unwrap_err());
    }

    #[test]
    fn win() {
        let mut starting = Chips::new(1_000);

        starting.wins(Chips::new(1_000_000));

        assert_eq!(Chips::new(1_001_000), starting);
    }

    #[test]
    fn add() {
        assert_eq!(Chips::new(1_001), Chips::new(1_000) + Chips::new(1));
    }

    #[test]
    fn default() {
        assert_eq!(Chips::default().size(), 0);
    }

    #[test]
    fn sub() {
        assert_eq!(Chips::new(999), Chips::new(1_000) - Chips::new(1));
    }

    #[test]
    #[should_panic]
    fn sub_overflow() {
        assert_eq!(Chips::new(999), Chips::new(1_000) - Chips::new(1_001));
    }
}
