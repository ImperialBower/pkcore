pub mod heads_up;
pub mod results;
pub mod win;
pub mod wins;

/// This modules started out as its own crate born out of my need to calculate win percentages
/// that included ties. One thing I noticed on the `Poker` TV shows that showed winning percentages
/// was that they didn't include ties in the results they displayed on the screen. While that is
/// fine, I want to be able to show both types of results if I want to. This module is designed to
/// allow me to do that if I want.
///
/// For now I am going to include it in this library so that I can easily update it and not worry
/// about publishing updates and keeping things in sync. At some point, I may decided to return it
/// back to the fold. I see it as having its own utility outside of this work, and generally like to
/// keep my libraries independent for flexibility. That's just how I roll.
///
/// TODO RF: Refactor this as a `struct PlayerFlag(u16)`.
///
/// In retrospect, I should never do these fracking type aliases. I always
/// regret it. Just wrap it,
pub type PlayerFlag = u16;

pub trait Result {
    #[must_use]
    fn is_tie(&self) -> bool;

    #[must_use]
    fn win_for(&self, count: PlayerFlag) -> bool;
}

impl Result for PlayerFlag {
    fn is_tie(&self) -> bool {
        self.count_ones() > 1
    }

    fn win_for(&self, count: PlayerFlag) -> bool {
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
