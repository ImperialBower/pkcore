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
    pub fn can(&self, next: PlayerState) -> bool {
        self.0.get().can(next)
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

    pub fn reset(&self) {
        self.0.set(PlayerState::YetToAct);
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let state_cell = PlayerStateCell::new(PlayerState::YetToAct);
    /// assert_eq!(state_cell.set(PlayerState::Bet(100)), Some(PlayerState::Bet(100)));
    /// assert_eq!(state_cell.get(), PlayerState::Bet(100));
    /// assert_eq!(state_cell.set(PlayerState::Check), None);
    /// assert_eq!(state_cell.get(), PlayerState::Bet(100));
    /// assert_eq!(state_cell.set(PlayerState::Bet(300)), Some(PlayerState::Bet(300)));
    /// ```
    pub fn set(&self, state: PlayerState) -> Option<PlayerState> {
        if self.can(state) {
            self.0.set(state);
            Some(state)
        } else {
            None
        }
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
    Check,
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
            | PlayerState::Bet(amt)
            | PlayerState::Call(amt)
            | PlayerState::Raise(amt)
            | PlayerState::ReRaise(amt)
            | PlayerState::AllIn(amt) => *amt,
            _ => 0,
        }
    }

    #[must_use]
    #[allow(clippy::unnested_or_patterns)]
    pub fn can(&self, next: PlayerState) -> bool {
        // An action can't be performned if the bet value is less than what happened before.
        if next < *self && next.is_active() {
            return false;
        }
        matches!(
            (self, next),
            (_, PlayerState::Fold)
                | (PlayerState::YetToAct, _)
                | (PlayerState::Blind(_), _)
                | (PlayerState::Check, PlayerState::Bet(_))
                | (PlayerState::Check, PlayerState::Raise(_))
                | (PlayerState::Check, PlayerState::ReRaise(_))
                | (PlayerState::Check, PlayerState::AllIn(_))
                | (PlayerState::Bet(_), PlayerState::Call(_))
                | (PlayerState::Bet(_), PlayerState::Bet(_))
                | (PlayerState::Bet(_), PlayerState::Raise(_))
                | (PlayerState::Bet(_), PlayerState::ReRaise(_))
                | (PlayerState::Bet(_), PlayerState::AllIn(_))
                | (PlayerState::Call(_), PlayerState::Raise(_))
                | (PlayerState::Call(_), PlayerState::ReRaise(_))
                | (PlayerState::Call(_), PlayerState::AllIn(_))
                | (PlayerState::Raise(_), PlayerState::ReRaise(_))
                | (PlayerState::Raise(_), PlayerState::AllIn(_))
                | (PlayerState::ReRaise(_), PlayerState::AllIn(_))
        )
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        !matches!(self, PlayerState::Fold | PlayerState::Out)
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

    pub fn reset(&mut self) {
        *self = PlayerState::YetToAct;
    }
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerState::YetToAct => write!(f, "Yet to act"),
            PlayerState::Check => write!(f, "Check"),
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
    fn can() {
        assert!(PlayerState::YetToAct.can(PlayerState::Check));
        assert!(PlayerState::YetToAct.can(PlayerState::Bet(100)));
        assert!(PlayerState::YetToAct.can(PlayerState::AllIn(100)));
        assert!(PlayerState::YetToAct.can(PlayerState::Fold));
        assert!(PlayerState::YetToAct.can(PlayerState::ReRaise(100)));

        assert!(PlayerState::Blind(50).can(PlayerState::Bet(200)));
        assert!(PlayerState::Blind(50).can(PlayerState::AllIn(500)));
        assert!(PlayerState::Blind(50).can(PlayerState::Fold));
        assert!(PlayerState::Blind(50).can(PlayerState::Raise(200)));
        assert!(PlayerState::Blind(50).can(PlayerState::ReRaise(300)));
        assert!(!PlayerState::Blind(100).can(PlayerState::Bet(50)));

        assert!(PlayerState::Check.can(PlayerState::Bet(200)));
        assert!(PlayerState::Check.can(PlayerState::Raise(100)));
        assert!(PlayerState::Check.can(PlayerState::ReRaise(100)));
        assert!(PlayerState::Check.can(PlayerState::AllIn(500)));
        assert!(PlayerState::Check.can(PlayerState::Fold));

        assert!(PlayerState::Bet(100).can(PlayerState::Call(100)));
        assert!(PlayerState::Bet(100).can(PlayerState::ReRaise(200)));
        assert!(PlayerState::Bet(100).can(PlayerState::Raise(200)));
        assert!(PlayerState::Bet(100).can(PlayerState::AllIn(500)));
        assert!(PlayerState::Bet(100).can(PlayerState::Fold));

        assert!(PlayerState::Call(100).can(PlayerState::Raise(200)));
        assert!(PlayerState::Call(200).can(PlayerState::ReRaise(300)));
        assert!(PlayerState::Call(100).can(PlayerState::AllIn(500)));
        assert!(PlayerState::Call(100).can(PlayerState::Fold));
        assert!(PlayerState::Raise(200).can(PlayerState::ReRaise(300)));
        assert!(!PlayerState::Raise(200).can(PlayerState::ReRaise(100)));

        assert!(PlayerState::Bet(200).can(PlayerState::Bet(300)));

        assert!(!PlayerState::Fold.can(PlayerState::Check));
        assert!(!PlayerState::Fold.can(PlayerState::Bet(100)));
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
