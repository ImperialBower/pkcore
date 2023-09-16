use bitvec::macros::internal::funty::Fundamental;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Chips(Box<usize>);

impl Chips {
    pub fn starting(stack: usize) -> Chips {
        Chips(Box::new(stack))
    }

    pub fn stack(&self) -> usize {
        self.0.as_usize()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__chips_tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(Chips::default().stack(), 0);
    }
}
