use std::fmt::{Display, Formatter};
use bitvec::macros::internal::funty::Fundamental;
use thousands::Separable;
use crate::PKError;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Chips(Box<usize>);

impl Chips {
    #[must_use]
    pub fn starting(stack: usize) -> Chips {
        Chips(Box::new(stack))
    }

    pub fn bet(amount: usize) -> Result<usize, PKError> {
        todo!()
    }

    #[must_use]
    pub fn stack(&self) -> usize {
        self.0.as_usize()
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
    fn default() {
        assert_eq!(Chips::default().stack(), 0);
    }
}
