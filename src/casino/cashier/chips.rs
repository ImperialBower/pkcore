use crate::PKError;
use std::cell::Cell;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign};
use thousands::Separable;

/// - [Interior Mutability Explained: When and Why to Use Cell and RefCell](https://dev.to/sgchris/interior-mutability-explained-when-and-why-to-use-cell-and-refcell-4bek)
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

    /// This function, along with `bet` and `wins` were originally part of the `Betting` trait,
    /// however, because `Stack` uses interior mutability, there is no need for them to be
    /// mutable, and so I moved them here.
    ///
    /// # Errors
    ///
    /// - `PKError::Busted` - if the stack is empty
    pub fn all_in(&self) -> Result<Stack, PKError> {
        if self.count() == 0 {
            Err(PKError::Busted)
        } else {
            let all = Stack::new(self.count());
            self.0.set(0);
            Ok(all)
        }
    }

    /// # Errors
    ///
    /// - `PKError::InsufficientChips` - if the stack is less than the amount bet.
    pub fn bet(&self, amount: usize) -> Result<Stack, PKError> {
        if self.count() < amount {
            Err(PKError::InsufficientChips)
        } else {
            let bet = Stack::new(amount);
            let mut current = self.count();
            current -= amount;
            self.0.set(current);
            Ok(bet)
        }
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

    #[must_use]
    pub fn wins(&self, winnings: Stack) -> usize {
        self.add_to(winnings);
        self.count()
    }

    #[must_use]
    pub fn takes(&self) -> Self {
        Stack::new(self.0.take())
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
        let starting = Stack::new(1_000);
        let expected = starting.clone();

        let bet = starting.all_in();

        assert!(bet.is_ok());
        assert_eq!(expected, bet.unwrap());
        assert_eq!(0, starting.count());
    }

    #[test]
    fn all_in__busted() {
        let starting = Stack::default();

        let busted = starting.all_in();

        assert!(busted.is_err());
        assert_eq!(PKError::Busted, busted.unwrap_err());
        assert_eq!(starting, Stack::default());
    }

    #[test]
    fn bet() {
        let starting = Stack::new(1_000);
        let expected = Stack::new(50);

        let bet = starting.bet(50);

        assert!(bet.is_ok());
        assert_eq!(expected, bet.unwrap());
        assert_eq!(950, starting.count());
    }

    #[test]
    fn bet__insufficient() {
        let starting = Stack::new(1_000);

        let bet = starting.bet(1_001);

        assert!(bet.is_err());
        assert_eq!(PKError::InsufficientChips, bet.unwrap_err());
    }

    #[test]
    fn win() {
        let starting = Stack::new(1_000);

        let _ = starting.wins(Stack::new(1_000_000));

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
        assert_eq!(Stack::default().count(), 0);
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
