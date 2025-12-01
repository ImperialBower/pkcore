use crate::PKError;
use crate::casino::cashier::chips::Stack;
use crate::prelude::PlayerStateCell;
use crate::util::name::Name;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Player {
    pub id: Uuid,
    pub handle: String,
    pub chips: Stack,
    pub bet: Stack,
    pub state: PlayerStateCell,
}

impl Player {
    #[must_use]
    pub fn new(handle: String) -> Player {
        Player {
            id: Uuid::new_v4(),
            handle,
            chips: Stack::default(),
            bet: Stack::default(),
            state: PlayerStateCell::default(),
        }
    }

    #[must_use]
    pub fn new_with_chips(handle: String, stack: usize) -> Player {
        Player {
            id: Uuid::new_v4(),
            handle,
            chips: Stack::new(stack),
            bet: Stack::default(),
            state: PlayerStateCell::default(),
        }
    }

    /// Working with cells this way is a completely different way of coding in `Rust`. It turns
    /// your natural instict to make everything mutable on its head. When I first coded this
    /// I made everything mutable even though I was working with a `Cell`.
    ///
    /// # Errors
    ///
    /// * `PKError::InsufficientChips` - if the player does not have enough chips to make the bet
    pub fn bets(&self, amount: usize) -> Result<usize, PKError> {
        if amount > self.chips.count() {
            Err(PKError::InsufficientChips)
        } else {
            let bet_chips = self.chips.bet(amount)?;
            self.bet.add_to(bet_chips);
            Ok(self.chips.count())
        }
    }

    pub fn folds(&self) -> Stack {
        self.bet.takes()
    }

    pub fn is_all_in(&self) -> bool {
        self.chips.count() == 0 && self.bet.count() > 0
    }

    pub fn is_tapped_out(&self) -> bool {
        self.chips.count() == 0 && self.bet.count() == 0
    }

    pub fn lose_bet(&self) {}

    #[must_use]
    pub fn random(stack: usize) -> Player {
        Player {
            id: Uuid::new_v4(),
            handle: Name::generate(),
            chips: Stack::new(stack),
            bet: Stack::default(),
            state: PlayerStateCell::default(),
        }
    }

    /// Returns the total count of the player that is in play.
    #[must_use]
    pub fn total_chip_count(&self) -> usize {
        self.chips.count() + self.bet.count()
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
        let player = Player::new_with_chips("The Russian".to_string(), 1_000);

        let did_bet = player.bets(100);

        assert!(did_bet.is_ok());
        assert_eq!(900, did_bet.unwrap());
    }

    #[test]
    fn is_all_in() {
        let player = Player::new_with_chips("All In Andy".to_string(), 500);
        assert!(!player.is_all_in());

        let _ = player.bets(500);
        assert!(player.is_all_in());
    }

    #[test]
    fn is_tapped_out() {
        let player = Player::new_with_chips("Tapped Out Tom".to_string(), 0);
        assert!(player.is_tapped_out());

        let player2 = Player::new_with_chips("Not Tapped Out Nancy".to_string(), 100);
        assert!(!player2.is_tapped_out());

        let _ = player2.bets(100);
        assert!(!player2.is_tapped_out());
    }
}
