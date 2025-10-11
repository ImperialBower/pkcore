use crate::casino::cashier::chips::Stack;
use std::cell::Cell;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Player {
    pub name: String,
    pub chips: Stack,
}

impl Player {
    #[must_use]
    pub fn new(name: String, chips: Stack) -> Player {
        Player {
            name,
            chips,
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
            chips: Stack::new(500),
        };

        let actual = Player::new("Flub".to_string(), Stack::new(500));

        assert_eq!(expected, actual);
    }
}
