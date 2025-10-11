use crate::{Betting, PKError};
use std::cell::Cell;
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

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Stack(Cell<usize>);

impl Stack {
    #[must_use]
    pub fn new(stack: usize) -> Stack {
        Stack(Cell::new(stack))
    }

    /// This function forces the caller to pass by value, because the basic contract of a Stack
    /// is that they must come out of one to go into another. This is to avoid accidentally creating
    /// excess chips.
    #[allow(clippy::needless_pass_by_value)]
    pub fn add_to(&self, chips: Stack) {
        let mut current = self.count();
        current += chips.count();
        self.0.set(current);
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.0.get()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    #[must_use]
    pub fn remove(&mut self, chips: Stack) -> Option<Stack> {
        if self.count() < chips.count() {
            None
        } else {
            let mut current = self.count();
            current -= chips.count();
            self.0.set(current);
            Some(chips)
        }
    }

    pub fn set(&mut self, chips: Stack) {
        self.0 = chips.0;
    }
}

impl Add for Stack {
    type Output = Stack;

    fn add(self, rhs: Self) -> Self::Output {
        Stack::new(self.count() + rhs.count())
    }
}

impl AddAssign for Stack {
    fn add_assign(&mut self, rhs: Self) {
        let mut current = self.count();
        current += rhs.count();
        self.0.set(current);
    }
}

impl Sub for Stack {
    type Output = Stack;

    fn sub(self, rhs: Self) -> Self::Output {
        Stack::new(self.count() - rhs.count())
    }
}

impl SubAssign for Stack {
    fn sub_assign(&mut self, rhs: Self) {
        let mut current = self.count();
        current -= rhs.count();
        self.0.set(current);
    }
}

impl Betting for Stack {
    fn all_in(&mut self) -> Result<Stack, PKError> {
        if self.size() == 0 {
            Err(PKError::Busted)
        } else {
            let all = self.clone();
            self.0 = 0.into();
            Ok(all)
        }
    }

    fn bet(&mut self, amount: usize) -> Result<Stack, PKError> {
        if self.size() < amount {
            Err(PKError::InsufficientChips)
        } else {
            *self -= Stack::new(amount);
            Ok(Stack::new(amount))
        }
    }

    fn size(&self) -> usize {
        self.count()
    }

    fn wins(&mut self, winnings: Stack) -> usize {
        *self += winnings;
        self.size()
    }
}

impl Display for Stack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.count().separate_with_commas())
    }
}

impl From<usize> for Stack {
    fn from(value: usize) -> Self {
        Stack::new(value)
    }
}

impl From<u8> for Stack {
    fn from(value: u8) -> Self {
        Stack::new(value as usize)
    }
}

impl From<u16> for Stack {
    fn from(value: u16) -> Self {
        Stack::new(value as usize)
    }
}

impl From<u32> for Stack {
    fn from(value: u32) -> Self {
        Stack::new(value as usize)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__chips__stack_tests {
    use super::*;

    #[test]
    fn starting() {
        let chips = Stack::new(1_000);

        assert_eq!(chips.count(), 1_000);
    }

    #[test]
    fn all_in() {
        let mut starting = Stack::new(1_000);
        let expected = starting.clone();

        let bet = starting.all_in();

        assert!(bet.is_ok());
        assert_eq!(expected, bet.unwrap());
        assert_eq!(0, starting.size());
    }

    #[test]
    fn all_in__busted() {
        let mut starting = Stack::default();

        let busted = starting.all_in();

        assert!(busted.is_err());
        assert_eq!(PKError::Busted, busted.unwrap_err());
        assert_eq!(starting, Stack::default());
    }

    #[test]
    fn bet() {
        let mut starting = Stack::new(1_000);
        let expected = Stack::new(50);

        let bet = starting.bet(50);

        assert!(bet.is_ok());
        assert_eq!(expected, bet.unwrap());
        assert_eq!(950, starting.size());
    }

    #[test]
    fn bet__insufficient() {
        let mut starting = Stack::new(1_000);

        let bet = starting.bet(1_001);

        assert!(bet.is_err());
        assert_eq!(PKError::InsufficientChips, bet.unwrap_err());
    }

    #[test]
    fn win() {
        let mut starting = Stack::new(1_000);

        starting.wins(Stack::new(1_000_000));

        assert_eq!(Stack::new(1_001_000), starting);
    }

    #[test]
    fn add() {
        let mut stack = Stack::new(1_000);
        stack += Stack::new(2);

        assert_eq!(Stack::new(1_001), Stack::new(1_000) + Stack::new(1));
        assert_eq!(Stack::new(1_002), stack);
    }

    #[test]
    fn default() {
        assert_eq!(Stack::default().size(), 0);
    }

    #[test]
    fn sub() {
        assert_eq!(Stack::new(999), Stack::new(1_000) - Stack::new(1));
    }

    #[test]
    #[should_panic]
    fn sub_overflow() {
        assert_eq!(Stack::new(999), Stack::new(1_000) - Stack::new(1_001));
    }

    #[test]
    fn add_to() {
        let stack = Stack::new(1_000_000);

        stack.add_to(Stack::new(9));

        assert_eq!(1_000_009, stack.count());
    }
}
