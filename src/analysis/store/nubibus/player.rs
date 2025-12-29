use crate::analysis::store::nubibus::chips::Chips;
use std::cell::Cell;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Player {
    pub name: String,
    pub chips: Cell<Chips>,
}

impl Player {
    #[must_use]
    pub fn new(name: String, starting_chips: Chips) -> Player {
        Player {
            name,
            chips: Cell::new(starting_chips),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod store_pluribus_seat_tests {
    use super::*;

    #[test]
    fn new() {
        let expected = Player {
            name: "Flub".to_string(),
            chips: Cell::new(Chips::new(500)),
        };

        let actual = Player::new("Flub".to_string(), Chips::new(500));

        assert_eq!(expected, actual);
    }
}
