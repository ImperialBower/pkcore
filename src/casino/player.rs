use crate::PKError;
use crate::casino::cashier::chips::Stack;
use crate::prelude::{PlayerState, PlayerStateCell};
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
    /// **UPDATE:** The original version of this code simply removed chips from the player's stack,
    /// and placed them in the best stack, erroring out of there weren't enough chips for the bet.
    ///
    /// I've updated it to reflect how play works in reality. When a player bets, and then raises
    /// in a round, they announce the total amount they are betting. So if they bet 50, and then
    /// raise to 100, they are only putting in an additional 50 chips, not 100 more.
    ///
    /// Now, the function, subtracts the amount already bet from the bet amount, and processes
    /// the bet based on the total chips committed to the pot for that round.
    ///
    /// NOTE: At the
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let player = Player::new_with_chips("The Russian".to_string(), 1_000);
    ///
    /// let first_bet = player.bet(50);
    /// let second_bet = player.bet(100);
    /// let third_bet = player.bet(100);
    ///
    /// assert!(first_bet.is_ok());
    /// assert!(second_bet.is_ok());
    /// assert!(third_bet.is_err());
    /// assert_eq!(PKError::InsufficientChips, third_bet.unwrap_err());
    /// assert_eq!(950, first_bet.unwrap());
    /// assert_eq!(900, second_bet.unwrap());
    /// assert_eq!(100, player.bet.count());
    /// assert_eq!(900, player.chips.count());
    /// ```
    ///
    /// # Errors
    ///
    /// * `PKError::InsufficientChips` - if the player does not have enough chips to make the bet
    pub fn bet_internal(&self, bet_type: PlayerState) -> Result<usize, PKError> {
        if bet_type.amount() > self.chips.count() {
            Err(PKError::InsufficientChips)
        } else {
            // How many chips are there above what's already committed to the round?
            let additional_bet = bet_type.amount() .saturating_sub(self.bet.count());

            // Throw an error if the result is 0, meaning they aren't betting anything.
            if additional_bet == 0 {
                log::warn!("InsufficientChips: Bet amount already placed.");
                return Err(PKError::InsufficientChips);
            }

            let bet_chips = self.chips.bet(additional_bet)?;
            self.state.set(bet_type);
            self.bet.add_to(bet_chips);
            Ok(self.chips.count())
        }
    }

    pub fn bet(&self, amount: usize) -> Result<usize, PKError> {
        self.bet_internal(PlayerState::Bet(amount))
    }

    /// # Errors
    ///
    /// * `PKError::InsufficientChips` - if the player does not have enough chips to make the bet
    pub fn bet_blind(&self, amount: usize) -> Result<usize, PKError> {
        self.bet_internal(PlayerState::Blind(amount))
    }

    pub fn folds(&self) -> Stack {
        self.state.set(PlayerState::Fold);
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
        write!(f, "{}: {} chips [{}]", self.handle, self.chips, self.state)
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
        assert_eq!("Elmer: 0 chips [Yet to act]", player.to_string());
    }

    #[test]
    fn new_with_chips() {
        let player = Player::new_with_chips("Bugsy".to_string(), 1_000_002);

        assert_eq!("Bugsy", player.handle);
        assert_eq!(1_000_002, player.chips.count());
        assert_eq!("Bugsy: 1,000,002 chips [Yet to act]", player.to_string());
    }

    #[test]
    fn default() {
        let player = Player::default();

        println!("{player:?}");

        assert_eq!("", player.handle);
        assert_eq!(0, player.chips.count());
        assert_eq!(": 0 chips [Yet to act]", player.to_string());
    }

    #[test]
    fn bets() {
        let player = Player::new_with_chips("The Russian".to_string(), 1_000);

        let did_bet = player.bet_internal(PlayerState::Bet(100));

        assert!(did_bet.is_ok());
        assert_eq!(900, did_bet.unwrap());
    }

    #[test]
    fn is_all_in() {
        let player = Player::new_with_chips("All In Andy".to_string(), 500);
        assert!(!player.is_all_in());

        let _ = player.bet_internal(PlayerState::Bet(500));
        assert!(player.is_all_in());
    }

    #[test]
    fn is_tapped_out() {
        let player = Player::new_with_chips("Tapped Out Tom".to_string(), 0);
        assert!(player.is_tapped_out());

        let player2 = Player::new_with_chips("Not Tapped Out Nancy".to_string(), 100);
        assert!(!player2.is_tapped_out());

        let _ = player2.bet_internal(PlayerState::Bet(100));
        assert!(!player2.is_tapped_out());
    }
}
