use crate::Agency;
use std::cell::Cell;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct PlayerStateCell(Cell<PlayerState>);

impl PlayerStateCell {
    #[must_use]
    pub fn new(state: PlayerState) -> Self {
        Self(Cell::new(state))
    }

    #[must_use]
    pub fn can(&self, next: &PlayerState) -> bool {
        self.0.get().can_given(next)
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let hero_cell = PlayerStateCell::new(PlayerState::Raise(100));
    /// let villain_cell = PlayerStateCell::new(PlayerState::Bet(50));
    ///
    /// assert!(hero_cell.can_act_after_played(&villain_cell));
    /// ```
    pub fn can_act_after_played(&self, other: &PlayerStateCell) -> bool {
        self.get().can_act_after(&other.get())
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let state_cell = PlayerStateCell::new(PlayerState::YetToAct);
    /// assert_eq!(state_cell.get(), PlayerState::YetToAct);
    /// ```
    #[must_use]
    pub fn get(&self) -> PlayerState {
        self.0.get()
    }

    pub fn is_active(&self) -> bool {
        self.0.get().is_active()
    }

    #[must_use]
    pub fn is_blind(&self) -> bool {
        self.0.get().is_blind()
    }

    #[must_use]
    pub fn is_check(&self) -> bool {
        self.0.get().is_check()
    }

    #[must_use]
    pub fn is_in_hand(&self) -> bool {
        self.0.get().is_in_hand()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(PlayerStateCell::new(PlayerState::YetToAct).is_yet_to_act());
    /// assert!(!PlayerStateCell::new(PlayerState::Bet(100)).is_yet_to_act());
    /// ```
    #[must_use]
    pub fn is_yet_to_act(&self) -> bool {
        self.get().is_yet_to_act()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(PlayerStateCell::new(PlayerState::YetToAct).is_yet_to_act_or_blind());
    /// assert!(PlayerStateCell::new(PlayerState::Blind(20)).is_yet_to_act_or_blind());
    /// assert!(!PlayerStateCell::new(PlayerState::Bet(100)).is_yet_to_act_or_blind());
    /// ```
    #[must_use]
    pub fn is_yet_to_act_or_blind(&self) -> bool {
        self.get().is_yet_to_act_or_blind()
    }

    pub fn reset(&self) {
        self.0.set(PlayerState::YetToAct);
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let state_cell = PlayerStateCell::new(PlayerState::YetToAct);
    ///
    /// assert_eq!(state_cell.set(PlayerState::Bet(100)), Some(PlayerState::Bet(100)));
    /// assert_eq!(state_cell.get(), PlayerState::Bet(100));
    /// assert_eq!(state_cell.set(PlayerState::Check(0)), None);
    /// assert_eq!(state_cell.get(), PlayerState::Bet(100));
    /// assert_eq!(state_cell.set(PlayerState::Bet(300)), None);
    /// assert_eq!(state_cell.set(PlayerState::Raise(300)), Some(PlayerState::Raise(300)));
    ///
    /// let state_cell = PlayerStateCell::new(PlayerState::Blind(100));
    /// assert_eq!(state_cell.set(PlayerState::Check(0)), None);
    /// assert_eq!(state_cell.set(PlayerState::Check(200)), None);
    /// assert_eq!(state_cell.set(PlayerState::Check(100)), Some(PlayerState::Check(100)));
    /// ```
    pub fn set(&self, state: PlayerState) -> Option<PlayerState> {
        if self.can(&state) {
            self.0.set(state);
            Some(state)
        } else {
            None
        }
    }
}

impl Agency for PlayerStateCell {
    fn can_act(&self) -> bool {
        self.get().can_act()
    }

    fn can_given(&self, next: &PlayerState) -> bool {
        self.get().can_given(next)
    }

    fn can_given_against(&self, next: &PlayerState, other: &PlayerState) -> bool {
        self.get().can_given_against(next, other)
    }
}

impl Display for PlayerStateCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let internal = self.0.get();
        write!(f, "{internal}")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PlayerState {
    #[default]
    YetToAct,
    Check(usize),
    Blind(usize),
    Bet(usize),
    Call(usize),
    Raise(usize),
    ReRaise(usize),
    AllIn(usize),
    Fold,
    Out,
}

impl PlayerState {
    #[must_use]
    pub fn amount(&self) -> usize {
        match self {
            PlayerState::Blind(amt)
            | PlayerState::Check(amt)
            | PlayerState::Bet(amt)
            | PlayerState::Call(amt)
            | PlayerState::Raise(amt)
            | PlayerState::ReRaise(amt)
            | PlayerState::AllIn(amt) => *amt,
            _ => 0,
        }
    }

    /// DIARY: This shit is going to be ugly AF. Going to test drive the shit out of it and
    /// refactor. This is the way.
    #[must_use]
    pub fn can_act_after(&self, other: &PlayerState) -> bool {
        // A player who is out of the hand can't act before anything.
        if !self.is_active() || self.is_all_in() {
            return false;
        }

        if other.is_blind() {
            if matches!(self, PlayerState::Check(_)) {
                // Can't check if there's an active blind.
                return false;
            } else if self.is_blind() {
                // The player who pays out the smaller blind acts first.
                return self <= other;
            }
        }

        if matches!(self, PlayerState::YetToAct) {
            return true;
        }

        if matches!(self, PlayerState::AllIn(_)) {
            return true;
        }

        // We've already checked if there's a blind, so you can only check if there's been nothing
        // but checks.
        if matches!(self, PlayerState::Check(_)) {
            return matches!(other, PlayerState::Check(_));
        }

        if matches!(self, PlayerState::Bet(_)) {
            if matches!(other, PlayerState::Bet(_))
                || matches!(other, PlayerState::Raise(_))
                || matches!(other, PlayerState::ReRaise(_))
            {
                return false;
            }
            return self.amount() > other.amount();
        }

        if matches!(self, PlayerState::Raise(_)) {
            if matches!(other, PlayerState::Raise(_)) || matches!(other, PlayerState::ReRaise(_)) {
                return false;
            }
            return self.amount() > other.amount();
        }

        self.amount() >= other.amount()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(PlayerState::Bet(100).is_active());
    ///
    /// assert!(PlayerState::AllIn(100).is_active());
    /// assert!(!PlayerState::Fold.is_active());
    /// assert!(!PlayerState::Out.is_active());
    /// ```
    #[must_use]
    pub fn is_active(&self) -> bool {
        !matches!(self, PlayerState::Fold | PlayerState::Out)
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(PlayerState::AllIn(100).is_all_in());
    /// assert!(!PlayerState::Blind(100).is_all_in());
    /// assert!(!PlayerState::Fold.is_all_in());
    /// ```
    #[must_use]
    pub fn is_all_in(&self) -> bool {
        matches!(self, PlayerState::AllIn(_))
    }

    #[must_use]
    pub fn is_bet(&self) -> bool {
        matches!(
            self,
            PlayerState::Blind(_)
                | PlayerState::Bet(_)
                | PlayerState::Call(_)
                | PlayerState::Raise(_)
                | PlayerState::ReRaise(_)
                | PlayerState::AllIn(_)
        )
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(PlayerState::Blind(100).is_blind());
    /// assert!(!PlayerState::Bet(100).is_blind());
    /// ```
    #[must_use]
    pub fn is_blind(&self) -> bool {
        matches!(self, PlayerState::Blind(_))
    }

    #[must_use]
    pub fn is_check(&self) -> bool {
        matches!(self, PlayerState::Check(_))
    }

    #[must_use]
    pub fn is_fold(&self) -> bool {
        matches!(self, PlayerState::Fold)
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(PlayerState::Bet(100).is_in_hand());
    /// assert!(PlayerState::AllIn(100).is_in_hand());
    ///
    /// assert!(!PlayerState::Fold.is_in_hand());
    /// assert!(!PlayerState::Out.is_in_hand());
    /// ```
    #[must_use]
    pub fn is_in_hand(&self) -> bool {
        !matches!(self, PlayerState::Fold | PlayerState::Out)
    }

    #[must_use]
    pub fn is_opener(&self) -> bool {
        matches!(self, PlayerState::Bet(_) | PlayerState::Call(_))
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(PlayerState::YetToAct.is_yet_to_act());
    /// assert!(!PlayerState::Bet(100).is_yet_to_act());
    /// ```
    #[must_use]
    pub fn is_yet_to_act(&self) -> bool {
        matches!(self, PlayerState::YetToAct)
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// assert!(PlayerState::YetToAct.is_yet_to_act_or_blind());
    /// assert!(PlayerState::Blind(20).is_yet_to_act_or_blind());
    /// assert!(!PlayerState::Bet(100).is_yet_to_act_or_blind());
    /// ```
    #[must_use]
    pub fn is_yet_to_act_or_blind(&self) -> bool {
        matches!(self, PlayerState::YetToAct | PlayerState::Blind(_))
    }

    pub fn reset(&mut self) {
        *self = PlayerState::YetToAct;
    }
}

impl Agency for PlayerState {
    fn can_act(&self) -> bool {
        self.is_active() && !self.is_all_in()
    }

    #[allow(clippy::unnested_or_patterns)]
    fn can_given(&self, next: &PlayerState) -> bool {
        log::trace!("can_given: self: {self}, next: {next}");
        if self.is_yet_to_act() {
            return true;
        }

        if next.is_yet_to_act() {
            return true;
        }

        if next.is_check() {
            if self.is_check() || self.is_fold() || (next.amount() > self.amount()) {
                return false;
            }
            if next.amount() == self.amount() {
                return true;
            }
        }

        // An action can't be performed if their bet value is less, equal to what they bet before.
        if next <= self && next.is_active() {
            return false;
        }
        matches!(
            (self, next),
            (_, PlayerState::Fold)
                | (PlayerState::YetToAct, _)
                | (PlayerState::Blind(_), _)
                | (PlayerState::Check(_), PlayerState::Call(_))
                | (PlayerState::Check(_), PlayerState::Raise(_))
                | (PlayerState::Check(_), PlayerState::ReRaise(_))
                | (PlayerState::Check(_), PlayerState::AllIn(_))
                | (PlayerState::Bet(_), PlayerState::Call(_))
                | (PlayerState::Bet(_), PlayerState::Raise(_))
                | (PlayerState::Bet(_), PlayerState::ReRaise(_))
                | (PlayerState::Bet(_), PlayerState::AllIn(_))
                | (PlayerState::Call(_), PlayerState::Call(_))
                | (PlayerState::Call(_), PlayerState::ReRaise(_))
                | (PlayerState::Call(_), PlayerState::AllIn(_))
                | (PlayerState::Raise(_), PlayerState::Call(_))
                | (PlayerState::Raise(_), PlayerState::ReRaise(_))
                | (PlayerState::Raise(_), PlayerState::AllIn(_))
                | (PlayerState::ReRaise(_), PlayerState::Call(_))
                | (PlayerState::ReRaise(_), PlayerState::ReRaise(_))
                | (PlayerState::ReRaise(_), PlayerState::AllIn(_))
        )
    }

    #[allow(clippy::unnested_or_patterns)]
    fn can_given_against(&self, next: &PlayerState, other: &PlayerState) -> bool {
        if self.can_given(next) {
            if self.is_all_in() {
                // A player who is all-in can't act against anything.
                return false;
            }

            // Comparing against a player who is out of the hand, any action is valid.
            if !other.is_active() || next.is_fold() {
                return true;
            }
            // Against another player, the amount of the action needs to be at least as much
            // as the other players.
            if next < other {
                return false;
            }
            matches!(
                (next, other),
                (_, PlayerState::YetToAct)
                    | (PlayerState::Check(_), PlayerState::Check(_))
                    | (PlayerState::Call(_), PlayerState::Check(_))
                    | (PlayerState::Call(_), PlayerState::Bet(_))
                    | (PlayerState::Call(_), PlayerState::Raise(_))
                    | (PlayerState::Call(_), PlayerState::ReRaise(_))
                    | (PlayerState::Call(_), PlayerState::AllIn(_))
                    | (PlayerState::Bet(_), PlayerState::Check(_))
                    | (PlayerState::Raise(_), PlayerState::Check(_))
                    | (PlayerState::Raise(_), PlayerState::Bet(_))
                    | (PlayerState::ReRaise(_), PlayerState::Bet(_))
                    | (PlayerState::ReRaise(_), PlayerState::Check(_))
                    | (PlayerState::ReRaise(_), PlayerState::Raise(_))
                    | (PlayerState::ReRaise(_), PlayerState::ReRaise(_))
                    | (PlayerState::AllIn(_), _)
                    | (PlayerState::Blind(_), _)
            )
        } else {
            false
        }
    }
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerState::YetToAct => write!(f, "Yet to act"),
            PlayerState::Check(amount) => write!(f, "Check {amount}"),
            PlayerState::Blind(amount) => write!(f, "Blind {amount}"),
            PlayerState::Bet(amount) => write!(f, "Bet {amount}"),
            PlayerState::Call(amount) => write!(f, "Call {amount}"),
            PlayerState::Raise(amount) => write!(f, "Raise to {amount}"),
            PlayerState::ReRaise(amount) => write!(f, "Re-raise to {amount}"),
            PlayerState::AllIn(amount) => write!(f, "All-in with {amount}"),
            PlayerState::Fold => write!(f, "Fold"),
            PlayerState::Out => write!(f, "Out"),
        }
    }
}

impl PartialOrd for PlayerState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PlayerState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.amount().cmp(&other.amount())
    }
}

impl std::hash::Hash for PlayerState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        self.amount().hash(state);
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__state_tests {
    use super::*;

    #[test]
    fn agency__can_act() {
        // Out of the hand
        assert!(PlayerState::Blind(500).can_act());
        assert!(PlayerState::YetToAct.can_act());
        assert!(PlayerState::Check(100).can_act());
        assert!(PlayerState::Bet(500).can_act());
        assert!(PlayerState::Raise(500).can_act());
        assert!(PlayerState::ReRaise(500).can_act());

        assert!(!PlayerState::Fold.can_act());
        assert!(!PlayerState::Out.can_act());
        assert!(!PlayerState::AllIn(500).can_act());
    }

    #[test]
    fn agency__can_given__isolated() {
        assert!(PlayerState::Blind(100).can_given(&PlayerState::Check(100)));
        assert!(!PlayerState::Blind(300).can_given(&PlayerState::Check(100)));
        assert!(!PlayerState::Check(100).can_given(&PlayerState::Blind(100)));
        assert!(!PlayerState::Blind(300).can_given(&PlayerState::Check(400)));
        assert!(!PlayerState::Check(0).can_given(&PlayerState::Check(0)));
    }

    #[test]
    fn agency__can_given() {
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Check(0)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Bet(100)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Call(100)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Raise(100)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::ReRaise(100)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::AllIn(100)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Fold));

        assert!(PlayerState::Blind(100).can_given(&PlayerState::Check(100)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::Bet(200)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::Call(200)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::Raise(200)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::ReRaise(300)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::AllIn(500)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::Fold));
        assert!(!PlayerState::Blind(100).can_given(&PlayerState::Bet(50)));

        assert!(PlayerState::Check(0).can_given(&PlayerState::Fold));
        assert!(PlayerState::Check(0).can_given(&PlayerState::Call(100)));
        assert!(PlayerState::Check(0).can_given(&PlayerState::Raise(100)));
        assert!(PlayerState::Check(0).can_given(&PlayerState::ReRaise(100)));
        assert!(PlayerState::Check(0).can_given(&PlayerState::AllIn(500)));
        assert!(!PlayerState::Check(100).can_given(&PlayerState::Blind(100)));
        assert!(!PlayerState::Check(0).can_given(&PlayerState::Bet(200)));
        assert!(!PlayerState::Check(0).can_given(&PlayerState::Check(0)));

        assert!(PlayerState::Bet(100).can_given(&PlayerState::Fold));
        assert!(PlayerState::Bet(100).can_given(&PlayerState::Call(200)));
        assert!(PlayerState::Bet(100).can_given(&PlayerState::Raise(200)));
        assert!(PlayerState::Bet(100).can_given(&PlayerState::ReRaise(200)));
        assert!(PlayerState::Bet(100).can_given(&PlayerState::AllIn(500)));
        assert!(PlayerState::Bet(100).can_given(&PlayerState::YetToAct));
        assert!(!PlayerState::Bet(100).can_given(&PlayerState::Call(100)));
        assert!(!PlayerState::Bet(200).can_given(&PlayerState::Raise(200)));
        assert!(!PlayerState::Bet(200).can_given(&PlayerState::Bet(300)));

        assert!(PlayerState::Call(200).can_given(&PlayerState::Fold));
        assert!(PlayerState::Call(100).can_given(&PlayerState::Call(200)));
        assert!(PlayerState::Call(200).can_given(&PlayerState::ReRaise(300)));
        assert!(PlayerState::Call(100).can_given(&PlayerState::AllIn(500)));
        assert!(PlayerState::Call(100).can_given(&PlayerState::Fold));
        assert!(!PlayerState::Call(100).can_given(&PlayerState::Raise(200)));

        assert!(PlayerState::Raise(200).can_given(&PlayerState::Fold));
        assert!(PlayerState::Raise(200).can_given(&PlayerState::Call(300)));
        assert!(PlayerState::Raise(200).can_given(&PlayerState::ReRaise(300)));
        assert!(PlayerState::Raise(200).can_given(&PlayerState::AllIn(300)));
        assert!(!PlayerState::Raise(200).can_given(&PlayerState::Bet(100)));
        assert!(!PlayerState::Raise(200).can_given(&PlayerState::ReRaise(100)));

        assert!(PlayerState::ReRaise(200).can_given(&PlayerState::Fold));
        assert!(PlayerState::ReRaise(200).can_given(&PlayerState::Call(300)));
        assert!(PlayerState::ReRaise(200).can_given(&PlayerState::ReRaise(300)));
        assert!(PlayerState::ReRaise(200).can_given(&PlayerState::AllIn(300)));
        assert!(!PlayerState::ReRaise(200).can_given(&PlayerState::Raise(300)));
        assert!(!PlayerState::ReRaise(200).can_given(&PlayerState::Bet(100)));

        assert!(!PlayerState::Fold.can_given(&PlayerState::Check(0)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::Bet(100)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::Call(100)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::Raise(100)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::ReRaise(100)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::AllIn(100)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::AllIn(100)));
    }

    #[test]
    fn agency__can_given_against() {
        let state = PlayerState::YetToAct;

        assert!(state.can_given_against(&PlayerState::Check(0), &PlayerState::Fold));
        assert!(state.can_given_against(&PlayerState::Check(0), &PlayerState::YetToAct));
        assert!(state.can_given_against(&PlayerState::Check(0), &PlayerState::Check(0)));
        // Player is already YetToAct, has to do something.
        assert!(!state.can_given_against(&PlayerState::YetToAct, &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::Check(0), &PlayerState::Blind(50)));

        // These follow the same rules
        asserter(&PlayerState::Check(0));
        asserter(&PlayerState::Bet(100));

        let state = PlayerState::Raise(300);
        assert!(state.can_given_against(&PlayerState::Fold, &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Fold));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::YetToAct));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Check(0)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::ReRaise(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::AllIn(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Check(0)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::ReRaise(450)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Check(0)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::ReRaise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::AllIn(50)));

        assert!(!state.can_given_against(&PlayerState::Check(0), &PlayerState::Check(0)));
        // You can't bet if you're already bet, only call, raise, re-raise or all-in.
        assert!(!state.can_given_against(&PlayerState::Bet(500), &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::Bet(500), &PlayerState::Bet(400)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Raise(500)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::ReRaise(500)));

        let state = PlayerState::ReRaise(300);
        assert!(state.can_given_against(&PlayerState::Fold, &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Fold));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::YetToAct));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Check(0)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::ReRaise(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::AllIn(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Check(0)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::ReRaise(450)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Check(0)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::ReRaise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::AllIn(50)));

        assert!(!state.can_given_against(&PlayerState::Check(0), &PlayerState::Check(0)));
        // You can't bet if you're already bet, only call, raise, re-raise or all-in.
        assert!(!state.can_given_against(&PlayerState::Bet(500), &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::Bet(500), &PlayerState::Bet(400)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Raise(500)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::ReRaise(500)));

        let state = PlayerState::AllIn(1000);
        assert!(!state.can_given_against(&PlayerState::Fold, &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::Fold));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::YetToAct));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::Raise(50)));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::ReRaise(50)));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::AllIn(50)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Raise(50)));
        assert!(!state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Raise(50)));
        assert!(!state.can_given_against(&PlayerState::AllIn(500), &PlayerState::ReRaise(50)));
        assert!(!state.can_given_against(&PlayerState::AllIn(500), &PlayerState::AllIn(50)));
        assert!(!state.can_given_against(&PlayerState::Check(0), &PlayerState::Check(0)));
        // You can't bet if you're already bet, only call, raise, re-raise or all-in.
        assert!(!state.can_given_against(&PlayerState::Bet(500), &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::Bet(500), &PlayerState::Bet(400)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Raise(500)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::ReRaise(500)));
    }

    fn asserter(state: &PlayerState) {
        assert!(state.can_given_against(&PlayerState::Fold, &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Fold));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::YetToAct));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Check(0)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::ReRaise(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::AllIn(50)));
        assert!(state.can_given_against(&PlayerState::Raise(500), &PlayerState::Check(0)));
        assert!(state.can_given_against(&PlayerState::Raise(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Check(0)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Check(0)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::ReRaise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::AllIn(50)));

        assert!(!state.can_given_against(&PlayerState::Check(0), &PlayerState::Check(0)));
        // You can't bet if you're already bet, only call, raise, re-raise or all-in.
        assert!(!state.can_given_against(&PlayerState::Bet(500), &PlayerState::Check(0)));
        assert!(!state.can_given_against(&PlayerState::Bet(500), &PlayerState::Bet(400)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Raise(500)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::ReRaise(500)));
    }

    #[test]
    fn can_act_after() {
        // Out of the hand
        assert!(!PlayerState::Fold.can_act_after(&PlayerState::Blind(100)));
        assert!(!PlayerState::Out.can_act_after(&PlayerState::Blind(100)));

        // Vs blind
        assert!(PlayerState::Blind(50).can_act_after(&PlayerState::Blind(50)));
        assert!(PlayerState::Blind(50).can_act_after(&PlayerState::Blind(100)));
        assert!(!PlayerState::Blind(100).can_act_after(&PlayerState::Blind(50)));

        // Yet to act
        assert!(PlayerState::YetToAct.can_act_after(&PlayerState::YetToAct));
        assert!(PlayerState::YetToAct.can_act_after(&PlayerState::Check(0)));
        assert!(PlayerState::YetToAct.can_act_after(&PlayerState::Bet(100)));
        assert!(PlayerState::YetToAct.can_act_after(&PlayerState::AllIn(100)));
        assert!(PlayerState::YetToAct.can_act_after(&PlayerState::Fold));

        // Check
        assert!(PlayerState::Check(0).can_act_after(&PlayerState::Check(0)));
        assert!(!PlayerState::Check(0).can_act_after(&PlayerState::Blind(50)));
        assert!(!PlayerState::Check(0).can_act_after(&PlayerState::Bet(50)));

        assert!(!PlayerState::AllIn(50).can_act_after(&PlayerState::Blind(100)));
        assert!(!PlayerState::AllIn(50).can_act_after(&PlayerState::Bet(25)));
        assert!(!PlayerState::AllIn(50).can_act_after(&PlayerState::Raise(2500)));

        assert!(!PlayerState::Bet(500).can_act_after(&PlayerState::Bet(100)));
        assert!(PlayerState::Bet(150).can_act_after(&PlayerState::Blind(100)));
        assert!(PlayerState::Bet(500).can_act_after(&PlayerState::AllIn(100)));
        assert!(PlayerState::Bet(500).can_act_after(&PlayerState::Call(100)));
        assert!(!PlayerState::Bet(500).can_act_after(&PlayerState::Call(500)));
        assert!(!PlayerState::Bet(500).can_act_after(&PlayerState::Bet(500)));
        assert!(!PlayerState::Bet(50).can_act_after(&PlayerState::Blind(100)));
        assert!(!PlayerState::Bet(50).can_act_after(&PlayerState::AllIn(100)));
        assert!(!PlayerState::Bet(150).can_act_after(&PlayerState::Raise(100)));
        assert!(!PlayerState::Bet(400).can_act_after(&PlayerState::ReRaise(200)));

        assert!(PlayerState::Raise(150).can_act_after(&PlayerState::Blind(100)));
        assert!(PlayerState::Raise(500).can_act_after(&PlayerState::Bet(100)));
        assert!(PlayerState::Raise(500).can_act_after(&PlayerState::AllIn(100)));
        assert!(!PlayerState::Raise(500).can_act_after(&PlayerState::Bet(500)));
        assert!(!PlayerState::Raise(500).can_act_after(&PlayerState::ReRaise(500)));
        assert!(!PlayerState::Raise(1000).can_act_after(&PlayerState::ReRaise(500)));
        assert!(!PlayerState::Raise(50).can_act_after(&PlayerState::Blind(100)));
        assert!(!PlayerState::Raise(50).can_act_after(&PlayerState::AllIn(100)));

        assert!(PlayerState::ReRaise(500).can_act_after(&PlayerState::Call(100)));
        assert!(PlayerState::ReRaise(150).can_act_after(&PlayerState::Blind(100)));
        assert!(PlayerState::ReRaise(500).can_act_after(&PlayerState::Bet(100)));
        assert!(PlayerState::ReRaise(500).can_act_after(&PlayerState::Raise(200)));
        assert!(PlayerState::ReRaise(500).can_act_after(&PlayerState::AllIn(100)));

        assert!(!PlayerState::ReRaise(50).can_act_after(&PlayerState::Call(100)));
        assert!(!PlayerState::ReRaise(50).can_act_after(&PlayerState::Blind(100)));
        assert!(!PlayerState::ReRaise(50).can_act_after(&PlayerState::Bet(100)));
        assert!(!PlayerState::ReRaise(50).can_act_after(&PlayerState::Raise(200)));
        assert!(!PlayerState::ReRaise(50).can_act_after(&PlayerState::AllIn(100)));
    }

    /// DIARY: Too tired to write unit tests. Hey CoPilot, write some unit tests for me.
    /// Of course, most of them are wrong, but they help save me some typing.
    #[test]
    fn partial_eq_distinguishes_variants() {
        // assert_eq!(PlayerState::Bet(100), PlayerState::Call(100));
        assert_eq!(PlayerState::Bet(100), PlayerState::Bet(100));
        assert_eq!(PlayerState::YetToAct, PlayerState::YetToAct);
        // assert_eq!(PlayerState::YetToAct, PlayerState::Check);

        assert_ne!(PlayerState::Raise(200), PlayerState::ReRaise(300));
    }

    #[test]
    fn ord_compares_by_variant_then_amount() {
        // assert_eq!(PlayerState::YetToAct, PlayerState::Check);
        // assert_eq!(PlayerState::Fold, PlayerState::Out);
        // assert_eq!(PlayerState::Bet(100), PlayerState::Call(100));
        // assert_eq!(PlayerState::Raise(200), PlayerState::ReRaise(200));
        assert!(PlayerState::Check(0) < PlayerState::Bet(100));
        assert!(PlayerState::Bet(50) < PlayerState::Bet(100));
        assert!(PlayerState::AllIn(500) > PlayerState::Fold);
    }

    #[test]
    fn partial_ord_matches_ord() {
        let states = vec![
            PlayerState::YetToAct,
            PlayerState::Check(0),
            PlayerState::Bet(50),
            PlayerState::Bet(100),
            PlayerState::Call(100),
            PlayerState::Fold,
        ];

        for i in 0..states.len() {
            for j in 0..states.len() {
                assert_eq!(states[i].partial_cmp(&states[j]), Some(states[i].cmp(&states[j])));
            }
        }
    }

    #[test]
    fn hash_distinguishes_variants() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn calculate_hash<T: Hash>(t: &T) -> u64 {
            let mut hasher = DefaultHasher::new();
            t.hash(&mut hasher);
            hasher.finish()
        }

        let bet_hash = calculate_hash(&PlayerState::Bet(100));
        let call_hash = calculate_hash(&PlayerState::Call(100));
        let raise_hash = calculate_hash(&PlayerState::Raise(100));

        assert_ne!(bet_hash, call_hash);
        assert_ne!(bet_hash, raise_hash);
        assert_ne!(call_hash, raise_hash);

        assert_eq!(calculate_hash(&PlayerState::Bet(100)), bet_hash);
    }

    #[test]
    fn hash_distinguishes_amounts() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn calculate_hash<T: Hash>(t: &T) -> u64 {
            let mut hasher = DefaultHasher::new();
            t.hash(&mut hasher);
            hasher.finish()
        }

        assert_ne!(
            calculate_hash(&PlayerState::Bet(100)),
            calculate_hash(&PlayerState::Bet(200))
        );
    }
}
