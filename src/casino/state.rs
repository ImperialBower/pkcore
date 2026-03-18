use crate::Agency;
use serde::{Deserialize, Serialize};
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

    pub fn is_all_in(&self) -> bool {
        self.0.get().is_all_in()
    }

    #[must_use]
    pub fn is_blind(&self) -> bool {
        self.0.get().is_blind()
    }

    #[must_use]
    pub fn is_call(&self) -> bool {
        self.0.get().is_call()
    }

    #[must_use]
    pub fn is_check(&self) -> bool {
        self.0.get().is_check()
    }

    #[must_use]
    pub fn is_in_hand(&self) -> bool {
        self.0.get().is_in_hand()
    }

    #[must_use]
    pub fn is_out(&self) -> bool {
        self.0.get().is_out()
    }

    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.0.get().is_ready()
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
    /// assert_eq!(PlayerStateCell::new(PlayerState::YetToAct).set(PlayerState::Bet(100)), Some(PlayerState::Bet(100)));
    /// assert_eq!(PlayerStateCell::new(PlayerState::YetToAct).set(PlayerState::Check), Some(PlayerState::Check));
    /// assert_eq!(PlayerStateCell::new(PlayerState::YetToAct).set(PlayerState::Bet(300)), Some(PlayerState::Bet(300)));
    /// assert_eq!(PlayerStateCell::new(PlayerState::YetToAct).set(PlayerState::Raise(300)), Some(PlayerState::Raise(300)));
    ///
    /// assert_eq!(PlayerStateCell::new(PlayerState::Blind(100)).set(PlayerState::Check), Some(PlayerState::Check));
    /// ```
    ///
    /// TODO RF: This sucks.
    ///
    /// DIARY: i am an idiot. Reusing the `state_cell` struct for multiple tests makes it alter the
    /// state between each assertion.
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

/// ## DIARY RF:
///
/// I originally had `Check` carry a value to represent the amount of chips the player had, figuring
/// this would be good information. The problem is, that it breaks the fundamental rule of
/// imperatives. Any part of a system has a fundamental imperative. Anything that system does that
/// is not a part of that imperative should not be there.
///
/// `PlayerState` is about what a player just did. Adding context outside of that is begging for
/// bugs and confusion. A check has no value in poker, so it shouldn't have any value in the enum.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PlayerState {
    /// Player is seated and ready to be dealt into the next hand.
    Ready,
    /// Player is active in the hand and has not acted in the current betting round.
    YetToAct,
    /// Player passes action without adding chips because no additional bet is required.
    Check,
    /// Player posts a blind amount.
    Blind(usize),
    /// Player makes the first voluntary wager in the current betting round.
    Bet(usize),
    /// Player matches the current required amount to continue.
    Call(usize),
    /// Player increases the current wager.
    Raise(usize),
    /// Player raises again after a prior raise.
    ReRaise(usize),
    /// Player commits their entire remaining stack.
    AllIn(usize),
    /// Player reaches showdown with the amount committed.
    Showdown(usize),
    /// Player relinquishes the hand.
    Fold,
    /// Player is not participating in the current hand.
    #[default]
    Out,
}

impl PlayerState {
    #[must_use]
    pub fn amount(&self) -> usize {
        match self {
            PlayerState::Blind(amt)
            | PlayerState::Bet(amt)
            | PlayerState::Call(amt)
            | PlayerState::Raise(amt)
            | PlayerState::ReRaise(amt)
            | PlayerState::AllIn(amt)
            | PlayerState::Showdown(amt) => *amt,
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
            if matches!(self, PlayerState::Check) {
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
        if matches!(self, PlayerState::Check) {
            return matches!(other, PlayerState::Check);
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
        !matches!(self, PlayerState::Fold | PlayerState::Out | PlayerState::Ready)
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
    pub fn is_call(&self) -> bool {
        matches!(self, PlayerState::Call(_))
    }

    #[must_use]
    pub fn is_check(&self) -> bool {
        matches!(self, PlayerState::Check)
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

    #[must_use]
    pub fn is_out(&self) -> bool {
        matches!(self, PlayerState::Out)
    }

    #[must_use]
    pub fn is_ready(&self) -> bool {
        matches!(self, PlayerState::Ready)
    }

    #[must_use]
    pub fn is_showdown(&self) -> bool {
        matches!(self, PlayerState::Showdown(_))
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
        if self.is_out() && matches!(next, PlayerState::Ready | PlayerState::YetToAct) {
            return true;
        }

        if self.is_ready() && matches!(next, PlayerState::YetToAct) {
            return true;
        }

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
            if (next.amount() == self.amount()) || self.is_blind() {
                return true;
            }
        }

        if self.is_bet() && next.is_bet() && (next.amount() > self.amount()) {
            return true;
        }

        if next.is_showdown() {
            return true;
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
                | (PlayerState::Check, PlayerState::Bet(_))
                | (PlayerState::Check, PlayerState::Call(_))
                | (PlayerState::Check, PlayerState::Raise(_))
                | (PlayerState::Check, PlayerState::ReRaise(_))
                | (PlayerState::Check, PlayerState::AllIn(_))
                | (PlayerState::Check, PlayerState::Showdown(_))
                | (PlayerState::Bet(_), PlayerState::Call(_))
                | (PlayerState::Bet(_), PlayerState::Raise(_))
                | (PlayerState::Bet(_), PlayerState::ReRaise(_))
                | (PlayerState::Bet(_), PlayerState::AllIn(_))
                | (PlayerState::Bet(_), PlayerState::Showdown(_))
                | (PlayerState::Call(_), PlayerState::Call(_))
                | (PlayerState::Call(_), PlayerState::ReRaise(_))
                | (PlayerState::Call(_), PlayerState::AllIn(_))
                | (PlayerState::Call(_), PlayerState::Showdown(_))
                | (PlayerState::Raise(_), PlayerState::Call(_))
                | (PlayerState::Raise(_), PlayerState::ReRaise(_))
                | (PlayerState::Raise(_), PlayerState::AllIn(_))
                | (PlayerState::Raise(_), PlayerState::Showdown(_))
                | (PlayerState::ReRaise(_), PlayerState::Call(_))
                | (PlayerState::ReRaise(_), PlayerState::ReRaise(_))
                | (PlayerState::ReRaise(_), PlayerState::AllIn(_))
                | (PlayerState::ReRaise(_), PlayerState::Showdown(_))
                | (PlayerState::AllIn(_), PlayerState::Showdown(_))
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
                    | (PlayerState::Check, PlayerState::Check)
                    | (PlayerState::Call(_), PlayerState::Check)
                    | (PlayerState::Call(_), PlayerState::Bet(_))
                    | (PlayerState::Call(_), PlayerState::Raise(_))
                    | (PlayerState::Call(_), PlayerState::ReRaise(_))
                    | (PlayerState::Call(_), PlayerState::AllIn(_))
                    | (PlayerState::Bet(_), PlayerState::Check)
                    | (PlayerState::Bet(_), PlayerState::Bet(_))
                    | (PlayerState::Bet(_), PlayerState::Raise(_))
                    | (PlayerState::Bet(_), PlayerState::ReRaise(_))
                    | (PlayerState::Bet(_), PlayerState::AllIn(_))
                    | (PlayerState::Raise(_), PlayerState::Check)
                    | (PlayerState::Raise(_), PlayerState::Bet(_))
                    | (PlayerState::ReRaise(_), PlayerState::Bet(_))
                    | (PlayerState::ReRaise(_), PlayerState::Check)
                    | (PlayerState::ReRaise(_), PlayerState::Raise(_))
                    | (PlayerState::ReRaise(_), PlayerState::ReRaise(_))
                    | (PlayerState::AllIn(_), _)
                    | (PlayerState::Showdown(_), _)
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
            PlayerState::Ready => write!(f, "Ready for next hand"),
            PlayerState::YetToAct => write!(f, "Yet to act"),
            PlayerState::Check => write!(f, "Check"),
            PlayerState::Blind(amount) => write!(f, "Blind {amount}"),
            PlayerState::Bet(amount) => write!(f, "Bet {amount}"),
            PlayerState::Call(amount) => write!(f, "Call {amount}"),
            PlayerState::Raise(amount) => write!(f, "Raise to {amount}"),
            PlayerState::ReRaise(amount) => write!(f, "Re-raise to {amount}"),
            PlayerState::AllIn(amount) => write!(f, "All-in with {amount}"),
            PlayerState::Showdown(amount) => write!(f, "Showdown with {amount}"),
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
        assert!(PlayerState::Check.can_act());
        assert!(PlayerState::Bet(500).can_act());
        assert!(PlayerState::Raise(500).can_act());
        assert!(PlayerState::ReRaise(500).can_act());

        assert!(!PlayerState::Fold.can_act());
        assert!(!PlayerState::Out.can_act());
        assert!(!PlayerState::AllIn(500).can_act());
    }

    #[test]
    fn agency__can_given__isolated() {
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Raise(300)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Showdown(1000)));

        assert!(!PlayerState::AllIn(1000).can_given(&PlayerState::Raise(300)));
        assert!(PlayerState::Blind(100).can_given(&PlayerState::Check));
        assert!(!PlayerState::Check.can_given(&PlayerState::Blind(100)));
        assert!(!PlayerState::Check.can_given(&PlayerState::Check));
    }

    #[test]
    fn agency__can_given_against__isolated() {
        assert!(PlayerState::Check.can_given_against(&PlayerState::Bet(500), &PlayerState::Bet(400)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Bet(500), &PlayerState::Bet(400)));
    }

    #[test]
    fn agency__can_given() {
        assert!(PlayerState::Out.can_given(&PlayerState::Ready));
        assert!(PlayerState::Out.can_given(&PlayerState::YetToAct));
        assert!(PlayerState::Ready.can_given(&PlayerState::YetToAct));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Check));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Bet(100)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Call(100)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Raise(100)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::ReRaise(100)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::AllIn(100)));
        assert!(PlayerState::YetToAct.can_given(&PlayerState::Fold));

        assert!(PlayerState::Blind(50).can_given(&PlayerState::Bet(200)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::Call(200)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::Raise(200)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::ReRaise(300)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::AllIn(500)));
        assert!(PlayerState::Blind(50).can_given(&PlayerState::Fold));
        assert!(PlayerState::Blind(100).can_given(&PlayerState::Check));
        assert!(!PlayerState::Blind(100).can_given(&PlayerState::Bet(50)));

        assert!(PlayerState::Check.can_given(&PlayerState::Fold));
        assert!(PlayerState::Check.can_given(&PlayerState::Call(100)));
        assert!(PlayerState::Check.can_given(&PlayerState::Bet(200)));
        assert!(PlayerState::Check.can_given(&PlayerState::Raise(100)));
        assert!(PlayerState::Check.can_given(&PlayerState::ReRaise(100)));
        assert!(PlayerState::Check.can_given(&PlayerState::AllIn(500)));
        assert!(!PlayerState::Check.can_given(&PlayerState::Blind(100)));
        assert!(!PlayerState::Check.can_given(&PlayerState::Check));

        assert!(PlayerState::Bet(100).can_given(&PlayerState::Fold));
        assert!(PlayerState::Bet(100).can_given(&PlayerState::Call(200)));
        assert!(PlayerState::Bet(100).can_given(&PlayerState::Raise(200)));
        assert!(PlayerState::Bet(100).can_given(&PlayerState::ReRaise(200)));
        assert!(PlayerState::Bet(100).can_given(&PlayerState::AllIn(500)));
        assert!(PlayerState::Bet(100).can_given(&PlayerState::YetToAct));
        assert!(PlayerState::Bet(300).can_given(&PlayerState::Bet(600))); // Updated to deal with Pluribus
        assert!(PlayerState::Bet(200).can_given(&PlayerState::Bet(300)));
        assert!(!PlayerState::Bet(100).can_given(&PlayerState::Call(100)));
        assert!(!PlayerState::Bet(200).can_given(&PlayerState::Raise(200)));

        assert!(PlayerState::Call(200).can_given(&PlayerState::Fold));
        assert!(PlayerState::Call(100).can_given(&PlayerState::Call(200)));
        assert!(PlayerState::Call(200).can_given(&PlayerState::ReRaise(300)));
        assert!(PlayerState::Call(100).can_given(&PlayerState::AllIn(500)));
        assert!(PlayerState::Call(100).can_given(&PlayerState::Fold));
        assert!(PlayerState::Call(100).can_given(&PlayerState::Raise(200)));

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
        assert!(PlayerState::ReRaise(200).can_given(&PlayerState::Raise(300)));
        assert!(!PlayerState::ReRaise(200).can_given(&PlayerState::Bet(100)));

        assert!(PlayerState::AllIn(1000).can_given(&PlayerState::Showdown(1000)));

        assert!(!PlayerState::Fold.can_given(&PlayerState::Check));
        assert!(!PlayerState::Fold.can_given(&PlayerState::Bet(100)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::Call(100)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::Raise(100)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::ReRaise(100)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::AllIn(100)));
        assert!(!PlayerState::Fold.can_given(&PlayerState::AllIn(100)));
    }

    #[test]
    fn agency__can_given_against__asserter_check() {
        assert!(PlayerState::Check.can_given_against(&PlayerState::Fold, &PlayerState::Bet(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::Call(500), &PlayerState::Fold));
        assert!(PlayerState::Check.can_given_against(&PlayerState::Call(500), &PlayerState::YetToAct));
        assert!(PlayerState::Check.can_given_against(&PlayerState::Call(500), &PlayerState::Check));
        assert!(PlayerState::Check.can_given_against(&PlayerState::Call(500), &PlayerState::Bet(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::Call(500), &PlayerState::Raise(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::Call(500), &PlayerState::ReRaise(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::Call(500), &PlayerState::AllIn(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::Raise(500), &PlayerState::Check));
        assert!(PlayerState::Check.can_given_against(&PlayerState::Raise(500), &PlayerState::Bet(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Check));
        assert!(PlayerState::Check.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Bet(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Raise(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::AllIn(500), &PlayerState::Check));
        assert!(PlayerState::Check.can_given_against(&PlayerState::AllIn(500), &PlayerState::Bet(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::AllIn(500), &PlayerState::Raise(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::AllIn(500), &PlayerState::ReRaise(50)));
        assert!(PlayerState::Check.can_given_against(&PlayerState::AllIn(500), &PlayerState::AllIn(50)));

        assert!(!PlayerState::Check.can_given_against(&PlayerState::Check, &PlayerState::Check));
        // You can't bet if you're already bet, only call, raise, re-raise or all-in.
        // UPDATE: This is no longer true because of `Pluribus`.
        assert!(PlayerState::Check.can_given_against(&PlayerState::Bet(500), &PlayerState::Check));
        assert!(PlayerState::Check.can_given_against(&PlayerState::Bet(500), &PlayerState::Bet(400)));
        assert!(!PlayerState::Check.can_given_against(&PlayerState::Raise(500), &PlayerState::Raise(500)));
        assert!(!PlayerState::Check.can_given_against(&PlayerState::Raise(500), &PlayerState::ReRaise(500)));
    }

    #[test]
    fn agency__can_given_against__asserter_bet() {
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Fold, &PlayerState::Bet(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Call(500), &PlayerState::Fold));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Call(500), &PlayerState::YetToAct));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Call(500), &PlayerState::Check));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Call(500), &PlayerState::Bet(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Call(500), &PlayerState::Raise(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Call(500), &PlayerState::ReRaise(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Call(500), &PlayerState::AllIn(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Raise(500), &PlayerState::Check));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Raise(500), &PlayerState::Bet(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::ReRaise(500), &PlayerState::Check));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::ReRaise(500), &PlayerState::Bet(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::ReRaise(500), &PlayerState::Raise(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::AllIn(500), &PlayerState::Check));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::AllIn(500), &PlayerState::Bet(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::AllIn(500), &PlayerState::Raise(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::AllIn(500), &PlayerState::ReRaise(50)));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::AllIn(500), &PlayerState::AllIn(50)));

        assert!(!PlayerState::Bet(100).can_given_against(&PlayerState::Check, &PlayerState::Check));
        // You can't bet if you're already bet, only call, raise, re-raise or all-in.
        // UPDATE: This is no longer true because of `Pluribus`.
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Bet(500), &PlayerState::Check));
        assert!(PlayerState::Bet(100).can_given_against(&PlayerState::Bet(500), &PlayerState::Bet(400)));
        assert!(!PlayerState::Bet(100).can_given_against(&PlayerState::Raise(500), &PlayerState::Raise(500)));
        assert!(!PlayerState::Bet(100).can_given_against(&PlayerState::Raise(500), &PlayerState::ReRaise(500)));
    }

    #[test]
    fn agency__can_given_against() {
        let state = PlayerState::YetToAct;

        assert!(state.can_given_against(&PlayerState::Check, &PlayerState::Fold));
        assert!(state.can_given_against(&PlayerState::Check, &PlayerState::YetToAct));
        assert!(state.can_given_against(&PlayerState::Check, &PlayerState::Check));
        // Player is already YetToAct, has to do something.
        assert!(!state.can_given_against(&PlayerState::YetToAct, &PlayerState::Check));
        assert!(!state.can_given_against(&PlayerState::Check, &PlayerState::Blind(50)));

        let state = PlayerState::Raise(300);
        assert!(state.can_given_against(&PlayerState::Fold, &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Fold));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::YetToAct));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Check));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::ReRaise(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::AllIn(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Check));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::ReRaise(450)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Check));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::ReRaise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::AllIn(50)));

        assert!(!state.can_given_against(&PlayerState::Check, &PlayerState::Check));
        // You can't bet if you're already bet, only call, raise, re-raise or all-in.
        // UPDATE: This is no longer true because of `Pluribus`.
        assert!(state.can_given_against(&PlayerState::Bet(500), &PlayerState::Check));
        assert!(state.can_given_against(&PlayerState::Bet(500), &PlayerState::Bet(400)));
        assert!(state.can_given_against(&PlayerState::Raise(500), &PlayerState::Check));
        assert!(state.can_given_against(&PlayerState::Raise(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Raise(500)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::ReRaise(500)));

        let state = PlayerState::ReRaise(300);
        assert!(state.can_given_against(&PlayerState::Fold, &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Fold));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::YetToAct));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Check));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::ReRaise(50)));
        assert!(state.can_given_against(&PlayerState::Call(500), &PlayerState::AllIn(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Check));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::ReRaise(450)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Check));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Bet(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Raise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::ReRaise(50)));
        assert!(state.can_given_against(&PlayerState::AllIn(500), &PlayerState::AllIn(50)));

        assert!(!state.can_given_against(&PlayerState::Check, &PlayerState::Check));
        // You can't bet if you're already bet, only call, raise, re-raise or all-in.
        // No longer true. Relaxed things to deal with `Pluribus`.
        assert!(state.can_given_against(&PlayerState::Bet(500), &PlayerState::Check));
        assert!(state.can_given_against(&PlayerState::Bet(500), &PlayerState::Bet(400)));
        assert!(state.can_given_against(&PlayerState::Raise(500), &PlayerState::Check));
        assert!(state.can_given_against(&PlayerState::Raise(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Raise(500)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::ReRaise(500)));

        let state = PlayerState::AllIn(1000);
        assert!(!state.can_given_against(&PlayerState::Fold, &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::Fold));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::YetToAct));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::Check));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::Raise(50)));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::ReRaise(50)));
        assert!(!state.can_given_against(&PlayerState::Call(500), &PlayerState::AllIn(50)));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Check));
        assert!(!state.can_given_against(&PlayerState::Raise(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Check));
        assert!(!state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::ReRaise(500), &PlayerState::Raise(50)));
        assert!(!state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Check));
        assert!(!state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Bet(50)));
        assert!(!state.can_given_against(&PlayerState::AllIn(500), &PlayerState::Raise(50)));
        assert!(!state.can_given_against(&PlayerState::AllIn(500), &PlayerState::ReRaise(50)));
        assert!(!state.can_given_against(&PlayerState::AllIn(500), &PlayerState::AllIn(50)));
        assert!(!state.can_given_against(&PlayerState::Check, &PlayerState::Check));
        // You can't bet if you're already bet, only call, raise, re-raise or all-in.
        assert!(!state.can_given_against(&PlayerState::Bet(500), &PlayerState::Check));
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
        assert!(PlayerState::YetToAct.can_act_after(&PlayerState::Check));
        assert!(PlayerState::YetToAct.can_act_after(&PlayerState::Bet(100)));
        assert!(PlayerState::YetToAct.can_act_after(&PlayerState::AllIn(100)));
        assert!(PlayerState::YetToAct.can_act_after(&PlayerState::Fold));

        // Check
        assert!(PlayerState::Check.can_act_after(&PlayerState::Check));
        assert!(!PlayerState::Check.can_act_after(&PlayerState::Blind(50)));
        assert!(!PlayerState::Check.can_act_after(&PlayerState::Bet(50)));

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

    #[test]
    fn set() {
        assert_eq!(
            PlayerStateCell::new(PlayerState::YetToAct).set(PlayerState::Bet(100)),
            Some(PlayerState::Bet(100))
        );
        assert_eq!(
            PlayerStateCell::new(PlayerState::YetToAct).set(PlayerState::Check),
            Some(PlayerState::Check)
        );
        assert_eq!(
            PlayerStateCell::new(PlayerState::YetToAct).set(PlayerState::Bet(300)),
            Some(PlayerState::Bet(300))
        );
        assert_eq!(
            PlayerStateCell::new(PlayerState::YetToAct).set(PlayerState::Raise(300)),
            Some(PlayerState::Raise(300))
        );

        assert_eq!(
            PlayerStateCell::new(PlayerState::Blind(100)).set(PlayerState::Check),
            Some(PlayerState::Check)
        );
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
        assert!(PlayerState::Check < PlayerState::Bet(100));
        assert!(PlayerState::Bet(50) < PlayerState::Bet(100));
        assert!(PlayerState::AllIn(500) > PlayerState::Fold);
    }

    #[test]
    fn partial_ord_matches_ord() {
        let states = vec![
            PlayerState::YetToAct,
            PlayerState::Check,
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
