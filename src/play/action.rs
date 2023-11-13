use std::cell::Cell;
use strum_macros::{EnumCount, EnumIter};

#[derive(Clone, Copy, Debug, Default, EnumCount, EnumIter, Eq, Hash, PartialEq)]
pub enum PlayState {
    SmallBlind,
    BigBlind,
    #[default]
    YetToAct,
    Fold,
    Check,
    Bet,
    Call,
    Raise,
    Fubar,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Actor {
    pub state: Cell<PlayState>,
}

/// `play::Action` is a struct designed to track the state of play within a specific round of Holdem
/// poker.
///
/// I am surprised at how challenging I am finding to code something that is so second nature to me.
/// I have been playing poker for over 30 years, and the flow of action within a hand comes easily
/// to me, but it is entirely ingrained, and, as it turns out, is rather complex. This will require
/// some serious test-driving.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Action {
    pub actors: Vec<Actor>,
}

impl Action {
    /// # Panics
    ///
    /// Throws a panic if `count` is less than 2.
    #[must_use]
    pub fn new(count: usize) -> Self {
        let mut actors = Vec::with_capacity(count);
        for _ in 0..count {
            actors.push(Actor::default());
        }
        actors.first().unwrap().state.set(PlayState::SmallBlind);
        actors.get(1).unwrap().state.set(PlayState::BigBlind);
        Action { actors }
    }

    #[must_use]
    pub fn is_open(&self) -> bool {
        if self.actors.len() < 2 {
            return false;
        }

        self.actors
            .iter()
            .any(|a| a.state.get() == PlayState::YetToAct)
    }

    #[must_use]
    pub fn player_count(&self) -> usize {
        self.actors
            .iter()
            .filter(|a| a.state.get() != PlayState::Fold)
            .count()
    }

    pub fn set_state(&self, position: usize, state: PlayState) {
        self.actors[position].state.set(state);
    }

    #[must_use]
    pub fn state(&self, position: usize) -> PlayState {
        if position > self.actors.len() - 1 {
            return PlayState::Fubar;
        }
        self.actors[position].state.get()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__action__tests {
    use super::*;

    #[test]
    fn new() {
        let actions = Action::new(6);

        assert_eq!(PlayState::SmallBlind, actions.state(0));
        assert_eq!(PlayState::BigBlind, actions.state(1));
        assert_eq!(PlayState::YetToAct, actions.state(2));
        assert_eq!(PlayState::YetToAct, actions.state(3));
        assert_eq!(PlayState::YetToAct, actions.state(4));
        assert_eq!(PlayState::YetToAct, actions.state(5));
    }

    #[test]
    fn is_open__any_yet_to_act() {
        let actions = Action::new(6);
        assert!(actions.is_open());

        actions.set_state(0, PlayState::Call);
        assert!(actions.is_open());

        actions.set_state(1, PlayState::Fold);
        assert!(actions.is_open());

        actions.set_state(2, PlayState::Fold);
        assert!(actions.is_open());

        actions.set_state(3, PlayState::Fold);
        assert!(actions.is_open());

        actions.set_state(4, PlayState::Fold);
        assert!(actions.is_open());

        actions.set_state(5, PlayState::Fold);
        assert!(!actions.is_open());
    }

    // #[test]
    // fn is_open__only_one_player_let_standing() {
    //     let actions = Action::new(6);
    //
    // }

    #[test]
    fn player_count() {
        let actions = Action::new(6);
        assert_eq!(6, actions.player_count());

        actions.set_state(0, PlayState::Fold);
        assert_eq!(5, actions.player_count());
    }

    #[test]
    fn state_invalid_index() {
        let actions = Action::new(6);

        assert_eq!(PlayState::Fubar, actions.state(6));
    }
}
