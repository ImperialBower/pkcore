use crate::PKError;
use crate::casino::cashier::chips::Stack;
use crate::util::name::Name;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Player {
    pub id: Uuid,
    pub handle: String,
    pub chips: Stack,
    pub bet: Stack,
}

impl Player {
    #[must_use]
    pub fn new(handle: String) -> Player {
        Player {
            id: Uuid::new_v4(),
            handle,
            chips: Stack::default(),
            bet: Stack::default(),
        }
    }

    #[must_use]
    pub fn new_with_chips(handle: String, stack: usize) -> Player {
        Player {
            id: Uuid::new_v4(),
            handle,
            chips: Stack::new(stack),
            bet: Stack::default(),
        }
    }

    /// # Errors
    ///
    /// * `PKError::InsufficientChips` - if the player does not have enough chips to make the bet
    pub fn bets(&mut self, amount: usize) -> Result<usize, PKError> {
        if amount > self.chips.count() {
            Err(PKError::InsufficientChips)
        } else {
            self.chips = Stack::new(self.chips.count() - amount);
            self.bet = Stack::new(self.bet.count() + amount);
            Ok(self.chips.count())
        }
    }

    #[must_use]
    pub fn random(stack: usize) -> Player {
        Player {
            id: Uuid::new_v4(),
            handle: Name::generate(),
            chips: Stack::new(stack),
            bet: Stack::default(),
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} chips", self.handle, self.chips)
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
        assert_eq!(0, player.chips.count());
        assert_eq!("Elmer: 0 chips", player.to_string());
    }

    #[test]
    fn new_with_chips() {
        let player = Player::new_with_chips("Bugsy".to_string(), 1_000_002);

        assert_eq!("Bugsy", player.handle);
        assert_eq!(1_000_002, player.chips.count());
        assert_eq!("Bugsy: 1,000,002 chips", player.to_string());
    }

    #[test]
    fn default() {
        let player = Player::default();

        println!("{player:?}");

        assert_eq!("", player.handle);
        assert_eq!(0, player.chips.count());
        assert_eq!(": 0 chips", player.to_string());
    }

    #[test]
    fn bets() {
        let mut player = Player::new_with_chips("The Russian".to_string(), 1_000);

        let did_bet = player.bets(100);

        assert!(did_bet.is_ok());
        assert_eq!(900, did_bet.unwrap());
    }
}
