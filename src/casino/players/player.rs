use crate::casino::chips::Chips;
use crate::util::name::Name;
use std::fmt::{Display, Formatter};
use thousands::Separable;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Player {
    pub id: Uuid,
    pub handle: String,
    pub chips: Chips,
}

impl Player {
    #[must_use]
    pub fn new(handle: String) -> Player {
        Player {
            id: Uuid::new_v4(),
            handle,
            chips: Chips::default(),
        }
    }

    #[must_use]
    pub fn new_with_chips(handle: String, stack: usize) -> Player {
        Player {
            id: Uuid::new_v4(),
            handle,
            chips: Chips::starting(stack),
        }
    }

    #[must_use]
    pub fn random(stack: usize) -> Player {
        Player {
            id: Uuid::new_v4(),
            handle: Name::generate(),
            chips: Chips::starting(stack),
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} chips",
            self.handle,
            self.chips.stack().separate_with_commas()
        )
    }
}
#[cfg(test)]
#[allow(non_snake_case)]
mod casino__players__player_tests {
    use super::*;

    #[test]
    fn new() {
        let player = Player::new("Elmer".to_string());

        assert_eq!("Elmer", player.handle);
        assert_eq!(0, player.chips.stack());
        assert_eq!("Elmer: 0 chips", player.to_string());
    }

    #[test]
    fn new_with_chips() {
        let player = Player::new_with_chips("Bugsy".to_string(), 1_000_002);

        assert_eq!("Bugsy", player.handle);
        assert_eq!(1_000_002, player.chips.stack());
        assert_eq!("Bugsy: 1,000,002 chips", player.to_string());
    }
}
