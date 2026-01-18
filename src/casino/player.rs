use crate::casino::cashier::chips::Stack;
use crate::prelude::{PlayerState, PlayerStateCell};
use crate::util::name::Name;
use crate::{Agency, PKError};
use std::cell::Cell;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Player {
    pub id: Uuid,
    pub handle: String,
    pub chips: Stack,
    pub bet: Stack,
    pub chips_in_play: Cell<usize>,
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
            chips_in_play: Cell::new(0),
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
            chips_in_play: Cell::new(0),
            state: PlayerStateCell::default(),
        }
    }

    /// The control flow of the `bet_internal` is forgiving general to specific, but throwing errors
    /// when the specific isn't right. If you say all in, and you aren't all in, that's an error. Now
    /// the table mechanism can simply reject the bet or it can force the player to bet all it. It
    /// up to them.
    ///
    /// # Errors
    ///
    /// * `PKError::InsufficientChips` - if the player does not have enough chips to make the bet
    /// * `PKError::InvalidTableAction` - throws if the player is already all in when all in is passed..
    fn act_bet_internal(&self, bet_type: PlayerState) -> Result<usize, PKError> {
        if bet_type.amount() == 0 {
            log::warn!("InvalidAction: Player can't make a bet of zero.");
            Err(PKError::InvalidAction)
        } else if bet_type.amount() > self.total_chip_count() {
            log::warn!("InsufficientChips: Bet amount is greater than total chips.");
            Err(PKError::InsufficientChips)
        } else if !self.state.is_active() {
            log::warn!("InvalidTableAction: Player is not in hand.");
            Err(PKError::InvalidTableAction)
        } else {
            if self.bet.count() > 0 {
                log::debug!("Player has already bet {} this round.", self.bet.count());
            }
            // How many chips are there above what's already committed to the round?
            let additional_bet = bet_type.amount().saturating_sub(self.bet.count());

            // Throw an error if the result is 0, meaning they aren't betting anything.
            if additional_bet == 0 {
                log::warn!("InsufficientChips: Bet amount already placed.");
                return Err(PKError::InsufficientChips);
            }

            let bet_chips = self.chips.bet(additional_bet)?;
            self.bet.add_to(bet_chips);
            log::trace!(
                "Player {} bets {} for a total bet of {}",
                self.handle,
                additional_bet,
                self.bet.count()
            );

            if self.is_all_in() {
                log::debug!("Resetting PlayerState to AllIn");
                self.state.set(PlayerState::AllIn(self.bet.count()));
            } else {
                if matches!(bet_type, PlayerState::AllIn(_)) {
                    // If they aren't all in, throw an error.
                    log::warn!("InvalidTableAction: Player is already all in.");
                    return Err(PKError::InvalidTableAction);
                }
                self.state.set(bet_type);
            }

            self.chips_in_play.set(self.chips_in_play.get() + additional_bet);

            log::debug!("Player {} {}", self.state, self.handle);

            Ok(self.chips.count())
        }
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let player = Player::new_with_chips("The Russian".to_string(), 1_000);
    ///
    /// let bet = player.act_bet(500);
    /// assert!(bet.is_ok());
    /// assert_eq!(500, bet.unwrap());
    /// assert_eq!(1_000, player.total_chip_count());
    /// assert_eq!(500, player.chips.count());
    /// assert_eq!(500, player.bet.count());
    /// assert_eq!(PlayerState::Bet(500), player.state.get());
    ///
    /// let all_in_bet = player.act_all_in();
    /// assert!(all_in_bet.is_ok());
    /// assert_eq!(PlayerState::AllIn(1_000), player.state.get());
    /// assert_eq!(0, player.chips.count());
    /// assert_eq!(1_000, player.bet.count());
    ///
    /// let another_all_in_bet = player.act_all_in();
    /// assert!(another_all_in_bet.is_err());
    /// assert_eq!(PKError::InvalidTableAction, another_all_in_bet.unwrap_err());
    /// assert_eq!(1_000, player.bet.count());
    /// ```
    /// # Errors
    ///
    /// * `PKError::InvalidTableAction` - throws if the player is already all in.
    pub fn act_all_in(&self) -> Result<usize, PKError> {
        if self.is_all_in() {
            log::warn!("InvalidTableAction: Player is already all in.");
            return Err(PKError::InvalidTableAction);
        }
        let amount = self.total_chip_count();
        let _ = self.act_bet_internal(PlayerState::AllIn(amount))?;
        Ok(amount)
    }

    /// Working with cells this way is a completely different way of coding in `Rust`. It turns
    /// your natural instinct to make everything mutable on its head. When I first coded this
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
    /// let first_bet = player.act_bet(50);
    /// assert!(first_bet.is_ok());
    /// assert_eq!(950, first_bet.unwrap());
    ///
    /// let second_bet = player.act_bet(100);
    /// assert!(second_bet.is_ok());
    /// assert_eq!(900, second_bet.unwrap());
    ///
    /// // The 3rd bet has the same amount as the last, so throws an error.
    /// let third_bet = player.act_bet(100);
    /// assert!(!third_bet.is_ok());
    /// assert_eq!(PKError::InsufficientChips, third_bet.unwrap_err());
    ///
    /// assert_eq!(100, player.bet.count());
    /// assert_eq!(900, player.chips.count());
    /// ```
    ///
    /// # Errors
    ///
    /// * `PKError::InsufficientChips` - if the player does not have enough chips to make the bet
    pub fn act_bet(&self, amount: usize) -> Result<usize, PKError> {
        self.act_bet_internal(PlayerState::Bet(amount))
    }

    /// # Errors
    ///
    /// * `PKError::InsufficientChips` - if the player does not have enough chips to make the bet
    pub fn act_bet_blind(&self, amount: usize) -> Result<usize, PKError> {
        self.act_bet_internal(PlayerState::Blind(amount))
    }

    /// Removes and returns the chips from the player's bet stack and sets their state to `YetToAct`.
    ///
    /// # Errors
    ///
    /// * `PKError::InvalidTableAction` - throws if the player is not active in the hand.
    pub fn act_bring_it_in(&self) -> Result<Stack, PKError> {
        // if !self.state.is_active() {
        //     log::warn!("InvalidTableAction: Player is not active in the hand.");
        //     return Err(PKError::InvalidTableAction);
        // }
        if self.state.is_active() {
            self.state.set(PlayerState::YetToAct);
        }

        let player_bet = self.bet.takes();

        log::trace!("{} brings in {} chips", self.handle, player_bet);

        Ok(player_bet)
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let player = Player::new_with_chips("The Russian".to_string(), 1_000);
    ///
    /// let call = player.act_call(500);
    ///
    /// assert!(call.is_ok());
    /// assert_eq!(500, call.unwrap());
    /// assert_eq!(1_000, player.total_chip_count());
    /// assert_eq!(500, player.chips.count());
    /// assert_eq!(500, player.bet.count());
    /// assert_eq!(PlayerState::Call(500), player.state.get());
    /// ```
    ///
    /// # Errors
    ///
    ///  * `PKError::InsufficientChips` - if the player does not have enough chips to make the bet
    ///  * `PKError::InvalidTableAction` - throws if the player is not able to call.
    pub fn act_call(&self, amount: usize) -> Result<usize, PKError> {
        self.act_bet_internal(PlayerState::Call(amount))
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let player = Player::new_with_chips("The Russian".to_string(), 1_000);
    ///
    /// // Check
    /// let check = player.act_check();
    /// assert_eq!(PlayerState::Check(0), player.state.get());
    ///
    /// let folds = player.act_fold();
    /// // Now the check should return a `PKError::InvalidTableAction`
    /// let check = player.act_check();
    /// assert!(check.is_err());
    /// assert_eq!(PKError::InvalidTableAction, check.unwrap_err());
    /// ```
    /// # Errors
    ///
    /// * `PKError::InvalidTableAction` - throws if the player is already all in.
    pub fn act_check(&self) -> Result<usize, PKError> {
        if !self.state.is_active() {
            log::warn!("InvalidTableAction: Player is not active and cannot check.");
            return Err(PKError::InvalidTableAction);
        }
        self.state.set(PlayerState::Check(self.bet.count()));
        log::debug!("Player {} with {} checks", self.handle, self.bet.count());
        Ok(self.chips.count())
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let player = Player::new_with_chips("The Russian".to_string(), 1_000);
    ///
    /// // Bet and then fold.
    /// let bet = player.act_bet(500);
    /// let folds = player.act_fold();
    ///
    /// assert_eq!(500, player.total_chip_count());
    /// assert_eq!(500, player.chips.count());
    /// assert_eq!(0, player.bet.count());
    /// assert_eq!(PlayerState::Fold, player.state.get());
    ///
    /// // Trying to fold again should trigger a `PKError::InvalidTableAction`
    /// let folds_again = player.act_fold();
    /// assert!(folds_again.is_err());
    /// assert_eq!(PKError::InvalidTableAction, folds_again.unwrap_err());
    /// ```
    /// # Errors
    ///
    /// * `PKError::InvalidTableAction` - throws if the player is already all in.
    pub fn act_fold(&self) -> Result<Stack, PKError> {
        if !self.state.is_active() {
            log::warn!("InvalidTableAction: Player is not active and cannot fold.");
            return Err(PKError::InvalidTableAction);
        }
        self.state.set(PlayerState::Fold);
        Ok(self.bet.takes())
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let player = Player::new_with_chips("The Russian".to_string(), 1_000);
    ///
    /// let bet = player.act_bet(50);
    /// let raise = player.act_raise(100);
    ///
    /// assert!(bet.is_ok());
    /// assert!(raise.is_ok());
    /// assert_eq!(950, bet.unwrap());
    /// assert_eq!(900, raise.unwrap());
    /// assert_eq!(100, player.bet.count());
    /// assert_eq!(900, player.chips.count());
    /// assert_eq!(PlayerState::Raise(100), player.state.get());
    /// ```
    ///
    /// # Errors
    ///
    /// * `PKError::InsufficientChips` - if the player does not have enough chips to make the bet
    pub fn act_raise(&self, amount: usize) -> Result<usize, PKError> {
        self.act_bet_internal(PlayerState::Raise(amount))
    }

    /// The only difference between the different bets from a logic perspective is the stored state.
    ///
    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let player = Player::new_with_chips("The Russian".to_string(), 1_000);
    ///
    /// let bet = player.act_bet(50);
    /// assert!(bet.is_ok());
    /// assert_eq!(950, bet.unwrap());
    ///
    /// let raise = player.act_raise(100);
    /// assert!(raise.is_ok());
    /// assert_eq!(900, raise.unwrap());
    ///
    /// let reraise = player.act_reraise(150);
    /// assert!(reraise.is_ok());
    /// assert_eq!(850, reraise.unwrap());
    ///
    /// assert_eq!(150, player.bet.count());
    /// assert_eq!(850, player.chips.count());
    /// ```
    ///
    /// # Errors
    ///
    /// * `PKError::InsufficientChips` - if the player does not have enough chips to make the bet
    pub fn act_reraise(&self, amount: usize) -> Result<usize, PKError> {
        self.act_bet_internal(PlayerState::ReRaise(amount))
    }

    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    pub fn is_all_in(&self) -> bool {
        self.chips.count() == 0 && self.bet.count() > 0
    }

    pub fn is_check(&self) -> bool {
        self.state.is_check()
    }

    pub fn is_in_hand(&self) -> bool {
        self.state.is_in_hand()
    }

    pub fn is_tapped_out(&self) -> bool {
        self.chips.count() == 0 && self.bet.count() == 0
    }

    pub fn get_chips_in_play(&self) -> usize {
        self.chips_in_play.get()
    }

    pub fn has_bet(&self) -> bool {
        self.bet.count() > 0
    }

    pub fn lose_bet(&self) {}

    #[must_use]
    pub fn random(stack: usize) -> Player {
        Player {
            id: Uuid::new_v4(),
            handle: Name::generate(),
            chips: Stack::new(stack),
            bet: Stack::default(),
            chips_in_play: Cell::new(0),
            state: PlayerStateCell::default(),
        }
    }

    /// Returns the total count of the player that is in play.
    #[must_use]
    pub fn total_chip_count(&self) -> usize {
        self.chips.count() + self.bet.count()
    }
}

impl Agency for Player {
    fn can_act(&self) -> bool {
        self.state.can_act()
    }

    fn can_given(&self, next: &PlayerState) -> bool {
        self.state.can_given(next)
    }

    fn can_given_against(&self, next: &PlayerState, other: &PlayerState) -> bool {
        self.state.can_given_against(next, other)
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
    fn act_bet() {
        let player = Player::new_with_chips("The Russian".to_string(), 1_000);

        let did_bet = player.act_bet(100);

        assert!(did_bet.is_ok());
        assert_eq!(900, did_bet.unwrap());
        assert_eq!(PlayerState::Bet(100), player.state.get());
    }

    #[test]
    fn act_bring_it_in() {
        let player = Player::new_with_chips("The Russian".to_string(), 1_000);

        let _ = player.act_bet(100);
        let did_bring_it_in = player.act_bring_it_in();

        assert!(did_bring_it_in.is_ok());
        assert_eq!(Stack::new(100), did_bring_it_in.unwrap());
        assert_eq!(0, player.bet.count());
        assert_eq!(900, player.chips.count());
        assert_eq!(PlayerState::YetToAct, player.state.get());
    }

    #[test]
    fn act_check() {
        let player = Player::new_with_chips("The Russian".to_string(), 1_000);

        let did_blind = player.act_bet_blind(100);

        assert!(did_blind.is_ok());
        assert_eq!(900, did_blind.unwrap());
        assert_eq!(PlayerState::Blind(100), player.state.get());

        let did_check = player.act_check();

        assert!(did_check.is_ok());
        assert_eq!(900, did_check.unwrap());
        assert_eq!(PlayerState::Check(100), player.state.get());
    }

    #[test]
    fn get_chips_in_play() {
        let player = Player::new_with_chips("The Mouth".to_string(), 10_000);

        player.act_bet_blind(100).expect("Blind bet failed");
        assert_eq!(100, player.get_chips_in_play());

        player.act_bet(300).expect("Bet failed");
        assert_eq!(300, player.get_chips_in_play());

        player.act_raise(500).expect("Raise failed");
        assert_eq!(500, player.get_chips_in_play());

        player.act_reraise(1500).expect("Reraise failed");
        assert_eq!(1500, player.get_chips_in_play());

        player.act_call(2000).expect("Call failed");
        assert_eq!(2000, player.get_chips_in_play());

        player.act_bring_it_in().expect("Bring It failed");
        assert_eq!(2000, player.get_chips_in_play());

        player.act_bet(100).expect("Bet failed");
        assert_eq!(2100, player.get_chips_in_play());

        player.act_raise(300).expect("Raise failed");
        assert_eq!(2300, player.get_chips_in_play());

        player.act_reraise(500).expect("Reraise failed");
        assert_eq!(2500, player.get_chips_in_play());

        player.act_call(1000).expect("Call failed");
        assert_eq!(3000, player.get_chips_in_play());

        player.act_all_in().expect("All In failed");
        assert_eq!(10_000, player.get_chips_in_play());
    }

    #[test]
    fn is_all_in() {
        let player = Player::new_with_chips("All In Andy".to_string(), 500);
        assert!(!player.is_all_in());

        let _ = player.act_bet(500);
        assert!(player.is_all_in());
        assert_eq!(PlayerState::AllIn(500), player.state.get());
    }

    #[test]
    fn is_in_hand() {
        let player = Player::new_with_chips("All In Andy".to_string(), 500);
        assert!(player.is_in_hand());

        let _ = player.act_fold();
        assert!(!player.is_in_hand());
    }

    #[test]
    fn is_tapped_out() {
        let player = Player::new_with_chips("Tapped Out Tom".to_string(), 0);
        assert!(player.is_tapped_out());

        let player2 = Player::new_with_chips("Not Tapped Out Nancy".to_string(), 100);
        assert!(!player2.is_tapped_out());

        let _ = player2.act_bet(100);
        assert!(!player2.is_tapped_out());
    }
}
