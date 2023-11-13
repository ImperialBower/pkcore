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
    #[must_use]
    pub fn new(count: usize) -> Self {
        let mut actors = Vec::with_capacity(count);
        for _ in 0..count {
            actors.push(Actor::default());
        }
        Action { actors }
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

        assert_eq!(PlayState::YetToAct, actions.state(0));
        assert_eq!(PlayState::YetToAct, actions.state(1));
        assert_eq!(PlayState::YetToAct, actions.state(2));
    }

    #[test]
    fn player_count() {
        let actions = Action::new(6);
        assert_eq!(6, actions.player_count());

        actions.set_state(0, PlayState::Fold);
        assert_eq!(5, actions.player_count());
    }
}
