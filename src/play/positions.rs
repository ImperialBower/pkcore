use crate::play::Position6Max;
use std::cell::Cell;

#[cfg(not(test))]
#[allow(unused_imports)]
use log::{debug, info, warn}; // Use log crate when building application

#[cfg(test)]
#[allow(unused_imports)]
use std::{println as info, println as warn};

/// Idea: `Position6MaxPointer`
/// I want a struct that manages who is active in a hand. It should be able to
/// tell me who is next to act who is active in the hand. If a seat fold, it
///
/// Note: AI is close but I want it to leverage
///
/// UPDATE: I am not feeling these count specific seat types. I want something that supports
/// anything from 2 to 11 players.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Position6MaxPointer {
    pub position: Cell<Position6Max>,
    pub active: [Cell<bool>; 6],
}

impl Position6MaxPointer {
    /// NOTE AI generated code.
    pub fn current(&self) -> Position6Max {
        let current = self.position.get();
        log::debug!(
            "Position6MaxPointer.current() {} is current player to act in hand",
            current.description().to_ascii_uppercase()
        );
        current
    }

    pub fn fold(&self, position: Position6Max) {
        log::debug!(
            "Position6MaxPointer.fold() {} folds",
            position.description().to_ascii_uppercase()
        );
        self.active[position as usize - 1].set(false);
    }

    /// These methods aren't needed for the functionality of the code. Things will be reset with
    /// a new instance.
    ///
    /// ```txt
    /// /// NOTE AI generated code.
    /// pub fn set_active(&self, position: Position6Max, is_active: bool) {
    ///     self.active[position as usize - 1].set(is_active);
    /// }
    ///
    /// /// NOTE AI generated code.
    /// pub fn reset(&self) {
    ///     self.position.set(Position6Max::UTG);
    ///     for i in 0..6 {
    ///         self.active[i].set(true);
    ///     }
    /// }
    /// ```
    ///
    /// I have discovered the [`FilterMap`](https://doc.rust-lang.org/std/iter/struct.FilterMap.html)
    /// for the first time.
    ///
    /// The code that copilot generated is very helpful. It starts at `c.get()`. Once I started
    /// the idea, it ran with it. This pattern is really beautiful.
    ///
    /// ```txt
    /// let v =
    ///     self.active.iter().filter_map(|c| match c.get() {
    ///         true => Some(c),
    ///         false => None,
    ///     }).collect::<Vec<&Cell<bool>>>();
    /// v.len()
    /// ```
    ///
    /// Now to refactor down and write more tests.
    pub(crate) fn in_hand_count(&self) -> usize {
        self.active
            .iter()
            .filter(|c| c.get())
            .collect::<Vec<&Cell<bool>>>()
            .len()
    }

    /// NOTE AI generated code.
    pub fn increment(&self) {
        self.position.set(self.current());
        match self.next() {
            None => {
                info!("Position6MaxPointer::increment() called when already over");
            }
            Some(position) => {
                log::debug!(
                    "Position6MaxPointer.increment() action passes to {}",
                    position.description().to_ascii_uppercase()
                );
                self.position.set(position);
            }
        }
    }

    /// NOTE AI generated code.
    pub fn is_over(&self) -> bool {
        self.in_hand_count() < 2
    }

    /// NOTE AI generated code.
    pub fn is_active(&self, position: Position6Max) -> bool {
        self.active[position as usize - 1].get()
    }

    /// NOTE AI generated code.
    pub fn next(&self) -> Option<Position6Max> {
        let mut next = self.position.get().next();
        loop {
            // debug!("Position6MaxPointer.next() checking if {} is next to act in hand", next.description().to_ascii_uppercase());
            if self.is_active(next) {
                if self.is_over() {
                    log::debug!("Position6MaxPointer.next() hand is over");
                    log::debug!(
                        "Position6MaxPointer.next() {} wins the hand",
                        next.description().to_ascii_uppercase()
                    );
                } else {
                    log::debug!(
                        "Position6MaxPointer.next() {} is next to act in hand",
                        next.description().to_ascii_uppercase()
                    );
                }
                return Some(next);
            }
            next = next.next();
        }
    }

    pub fn set(&self, position: Position6Max) {
        self.position.set(position);
    }
}

impl Default for Position6MaxPointer {
    fn default() -> Self {
        Position6MaxPointer {
            position: Cell::new(Position6Max::SB),
            active: [
                Cell::new(true),
                Cell::new(true),
                Cell::new(true),
                Cell::new(true),
                Cell::new(true),
                Cell::new(true),
            ],
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__positions_tests {
    use super::*;

    #[test]
    fn in_hand_count() {
        let pointer = Position6MaxPointer::default();
        assert_eq!(6, pointer.in_hand_count());
    }

    #[test]
    fn in_hand_count__setting_some_to_false() {
        let pointer = Position6MaxPointer::default();
        pointer.fold(Position6Max::SB);
        pointer.fold(Position6Max::UTG);

        assert_eq!(4, pointer.in_hand_count());
    }

    #[test]
    fn is_over() {
        let pointer = Position6MaxPointer::default();

        pointer.fold(Position6Max::SB);
        pointer.fold(Position6Max::BB);
        pointer.fold(Position6Max::UTG);
        pointer.fold(Position6Max::MP);
        pointer.fold(Position6Max::CO);

        assert!(pointer.is_over());
    }

    #[test]
    fn is_over__false() {
        let pointer = Position6MaxPointer::default();
        assert!(!pointer.is_over());

        pointer.fold(Position6Max::SB);
        pointer.fold(Position6Max::UTG);

        assert!(!pointer.is_over());
    }

    #[test]
    fn next() {
        let pointer = Position6MaxPointer::default();

        assert_eq!(Position6Max::SB, pointer.current());
        assert_eq!(Position6Max::BB, pointer.next().unwrap());
        pointer.increment();
        println!("===================");

        assert_eq!(Position6Max::BB, pointer.current());
        assert_eq!(Position6Max::UTG, pointer.next().unwrap());
        pointer.increment();
        println!("===================");

        assert_eq!(Position6Max::UTG, pointer.current());
        assert_eq!(Position6Max::MP, pointer.next().unwrap());
        pointer.fold(Position6Max::UTG);
        println!("===================");

        pointer.increment();
        assert_eq!(Position6Max::MP, pointer.current());
        assert_eq!(Position6Max::CO, pointer.next().unwrap());
        pointer.fold(Position6Max::MP);
        println!("===================");

        pointer.increment();
        assert_eq!(Position6Max::CO, pointer.current());
        assert_eq!(Position6Max::BTN, pointer.next().unwrap());
        pointer.fold(Position6Max::CO);
        println!("===================");

        pointer.increment();
        assert_eq!(Position6Max::BTN, pointer.current());
        assert_eq!(Position6Max::SB, pointer.next().unwrap());
        println!("===================");

        pointer.increment();
        assert_eq!(Position6Max::SB, pointer.current());
        assert_eq!(Position6Max::BB, pointer.next().unwrap());
        pointer.fold(Position6Max::SB);
        println!("===================");

        pointer.increment();
        assert_eq!(Position6Max::BB, pointer.current());
        assert_eq!(Position6Max::BTN, pointer.next().unwrap());
        println!("===================");

        pointer.increment();
        assert_eq!(Position6Max::BTN, pointer.current());
        assert_eq!(Position6Max::BB, pointer.next().unwrap());
        println!("===================");

        pointer.increment();
        assert_eq!(Position6Max::BB, pointer.current());
        assert_eq!(Position6Max::BTN, pointer.next().unwrap());
        pointer.fold(Position6Max::BB);
        println!("===================");

        pointer.increment();
        assert_eq!(Position6Max::BTN, pointer.current());
        assert_eq!(Position6Max::BTN, pointer.next().unwrap());
    }

    #[test]
    fn next__only_one() {
        let pointer = Position6MaxPointer::default();

        pointer.fold(Position6Max::SB);
        pointer.fold(Position6Max::BB);
        pointer.fold(Position6Max::UTG);
        pointer.fold(Position6Max::MP);
        pointer.fold(Position6Max::CO);

        assert_eq!(Position6Max::BTN, pointer.next().unwrap());
    }

    #[test]
    fn set() {
        let pointer = Position6MaxPointer::default();
        assert_eq!(Position6Max::SB, pointer.current());
        assert_eq!(Position6Max::BB, pointer.next().unwrap());
        pointer.increment();

        pointer.set(Position6Max::SB);

        assert_eq!(Position6Max::SB, pointer.current());
    }
}
