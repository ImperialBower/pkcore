pub mod heads_up;
pub mod results;
pub mod win;
pub mod wins;

// TODO RF: Refactor this as a `struct Count(u16)`.
pub type Count = u16;

pub trait Result {
    #[must_use]
    fn is_tie(&self) -> bool;

    #[must_use]
    fn win_for(&self, count: Count) -> bool;
}

impl Result for Count {
    fn is_tie(&self) -> bool {
        self.count_ones() > 1
    }

    fn win_for(&self, count: Count) -> bool {
        self & count == count
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__wincounter__result__tests {
    use super::*;
    use crate::util::wincounter::win::Win;

    #[test]
    fn is_tie() {
        let r = Win::FIRST | Win::SECOND;

        assert_eq!(2, r.count_ones());
        assert!(r.is_tie());
    }

    #[test]
    fn win_for() {
        let tie = Win::FIRST | Win::THIRD;

        assert!(Win::FIRST.win_for(Win::FIRST));
        assert!(tie.win_for(Win::FIRST));
        assert!(tie.win_for(Win::THIRD));
        assert!(!tie.win_for(Win::SECOND));
        assert!(!Win::FIRST.win_for(Win::SECOND));
        assert!(!Win::FIRST.win_for(Win::THIRD));
    }
}
