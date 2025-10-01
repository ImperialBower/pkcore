use log::warn;
use std::cell::Cell;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use strum_macros::{EnumCount, EnumIter};

#[derive(Clone, Copy, Debug, Default, EnumCount, EnumIter, Eq, Hash, PartialEq, strum_macros::Display)]
pub enum PhaseHoldem {
    #[default]
    Init = 0,
    Preflop = 1,
    Flop = 2,
    Turn = 3,
    River = 4,
    Over = 5,
}

impl PhaseHoldem {
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn next(&self) -> Self {
        match *self {
            PhaseHoldem::Init => PhaseHoldem::Preflop,
            PhaseHoldem::Preflop => PhaseHoldem::Flop,
            PhaseHoldem::Flop => PhaseHoldem::Turn,
            PhaseHoldem::Turn => PhaseHoldem::River,
            PhaseHoldem::River => PhaseHoldem::Over,
            PhaseHoldem::Over => {
                warn!("PhaseHoldem::next() called when already over");
                PhaseHoldem::Over
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhaseHoldemTracker {
    phase: Cell<PhaseHoldem>,
}

impl PhaseHoldemTracker {
    pub fn current(&self) -> PhaseHoldem {
        self.phase.get()
    }

    /// NOTE: The original version of this code also incremented the phase.
    pub fn next(&self) -> PhaseHoldem {
        self.phase.get().next()
    }

    pub fn increment(&self) {
        if self.is_over() {
            warn!("PhaseHoldem::increment() called when already over");
        } else {
            self.phase.set(self.phase.get().next());
        }
    }

    pub fn is_init(&self) -> bool {
        self.phase.get() == PhaseHoldem::Init
    }

    pub fn is_preflop(&self) -> bool {
        self.phase.get() == PhaseHoldem::Preflop
    }

    pub fn is_flop(&self) -> bool {
        self.phase.get() == PhaseHoldem::Flop
    }

    pub fn is_turn(&self) -> bool {
        self.phase.get() == PhaseHoldem::Turn
    }

    pub fn is_river(&self) -> bool {
        self.phase.get() == PhaseHoldem::River
    }

    pub fn is_over(&self) -> bool {
        self.phase.get() == PhaseHoldem::Over
    }
}

impl Default for PhaseHoldemTracker {
    fn default() -> Self {
        Self {
            phase: Cell::new(PhaseHoldem::Init),
        }
    }
}

impl Display for PhaseHoldemTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.phase.get())
    }
}

/// NOTE AI generated code.
///
/// fucking brilliant
impl Hash for PhaseHoldemTracker {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.phase.get().hash(state);
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play_phases_tests {
    use super::*;

    /// NOTE AI generated code.
    #[test]
    fn next() {
        assert_eq!(PhaseHoldem::Preflop, PhaseHoldem::Init.next());
        assert_eq!(PhaseHoldem::Flop, PhaseHoldem::Preflop.next());
        assert_eq!(PhaseHoldem::Turn, PhaseHoldem::Flop.next());
        assert_eq!(PhaseHoldem::River, PhaseHoldem::Turn.next());
        assert_eq!(PhaseHoldem::Over, PhaseHoldem::River.next());
        assert_eq!(PhaseHoldem::Over, PhaseHoldem::Over.next());
    }

    #[test]
    fn tracker_default() {
        assert_eq!(PhaseHoldem::Init, PhaseHoldemTracker::default().current());
    }

    #[test]
    fn tracker_display() {
        assert_eq!("Init", PhaseHoldemTracker::default().to_string());
    }

    /// It's fun how wrong the AI is when I ask it to generate tests.
    #[test]
    fn tracker() {
        let tracker = PhaseHoldemTracker::default();
        assert_eq!(PhaseHoldem::Init, tracker.current());
        assert_eq!(PhaseHoldem::Preflop, tracker.next());

        tracker.increment();

        assert_eq!(PhaseHoldem::Preflop, tracker.current());
        assert_eq!(PhaseHoldem::Flop, tracker.next());

        tracker.increment();
        assert_eq!(PhaseHoldem::Flop, tracker.current());
        assert_eq!(PhaseHoldem::Turn, tracker.next());

        tracker.increment();
        assert_eq!(PhaseHoldem::Turn, tracker.current());
        assert_eq!(PhaseHoldem::River, tracker.next());

        tracker.increment();
        assert_eq!(PhaseHoldem::River, tracker.current());
        assert_eq!(PhaseHoldem::Over, tracker.next());

        tracker.increment();
        assert_eq!(PhaseHoldem::Over, tracker.current());
        assert_eq!(PhaseHoldem::Over, tracker.next());

        tracker.increment();
        assert_eq!(PhaseHoldem::Over, tracker.current());
        assert_eq!(PhaseHoldem::Over, tracker.next());
    }
}
