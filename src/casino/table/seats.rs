use crate::PKError;
use crate::card::Card;
use crate::cards::Cards;
use crate::cards_cell::CardsCell;
use crate::casino::cashier::chips::Stack;
use crate::casino::table::seat::{Seat, SeatCell};
use log;
use std::cell::{Ref, RefMut};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Seats(Box<[SeatCell]>);

impl Seats {
    pub const DEFAULT_NUMBER_SEATS: u8 = 6;
    pub const MAX_NUMBER_SEATS: u8 = 10;

    /// How frackin' cool is this `into_boxed_slice` pattern?! I'm going to need to play with this.
    #[must_use]
    pub fn new(seats: Vec<Seat>) -> Self {
        let seat_cells: Vec<SeatCell> = seats.into_iter().map(SeatCell::new).collect();
        Seats(seat_cells.into_boxed_slice())
    }

    /// # Errors
    ///
    /// `PKError::InvalidSeatNumber` error if the `seat_number` is not valid.
    pub fn act_all_in(&self, seat_number: u8) -> Result<usize, PKError> {
        if let Some(seat) = self.get_seat_mut(seat_number) {
            let total_chips = seat.player.act_all_in()?;
            Ok(total_chips)
        } else {
            log::error!("Failed to find seat #{seat_number} for betting");
            Err(PKError::InvalidSeatNumber)
        }
    }

    /// # Errors
    ///
    /// `PKError::InvalidSeatNumber` error if the `seat_number` is not valid.
    pub fn act_bet(&self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        if let Some(seat) = self.get_seat_mut(seat_number) {
            let remaining = seat.player.act_bet(amount)?;
            Ok(remaining)
        } else {
            log::error!("Failed to find seat #{seat_number} for betting");
            Err(PKError::InvalidSeatNumber)
        }
    }

    /// # Errors
    ///
    /// `PKError::InvalidSeatNumber` error if the `seat_number` is not valid.
    pub fn act_raise(&self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        if let Some(seat) = self.get_seat_mut(seat_number) {
            let remaining = seat.player.act_raise(amount)?;
            Ok(remaining)
        } else {
            log::error!("Failed to find seat #{seat_number} for raising");
            Err(PKError::InvalidSeatNumber)
        }
    }

    /// # Errors
    ///
    /// `PKError::InvalidSeatNumber` error if the `seat_number` is not valid.
    pub fn act_call(&self, seat_number: u8) -> Result<(usize, usize), PKError> {
        let to_call = self.current_bet();
        if let Some(seat) = self.get_seat_mut(seat_number) {
            let remaining = seat.player.act_call(to_call)?;
            drop(seat);
            Ok((to_call, remaining))
        } else {
            log::error!("Failed to find seat #{seat_number} for calling");
            Err(PKError::InvalidSeatNumber)
        }
    }

    /// # Errors
    ///
    /// `PKError::InvalidTableAction` error if the player cannot check.
    /// `PKError::InvalidSeatNumber` error if the `seat_number` is not valid.
    pub fn act_check(&self, seat_number: u8) -> Result<usize, PKError> {
        let current_bet = self.current_bet();
        if let Some(seat) = self.get_seat_mut(seat_number) {
            // if seat.player.bet.count() < current_bet || seat.is_yet_to_act() {
            if seat.player.bet.count() < current_bet {
                log::error!(
                    "Seat #{seat_number} cannot check; current bet is {} but seat's bet is {}",
                    current_bet,
                    seat.player.bet.count()
                );
                return Err(PKError::InvalidTableAction);
            }
            seat.player.act_check()?;
            let remaining = seat.player.chips.count();
            drop(seat);
            Ok(remaining)
        } else {
            log::error!("Failed to find seat #{seat_number} for checking");
            Err(PKError::InvalidSeatNumber)
        }
    }

    /// # Errors
    ///
    /// `PKError::InvalidSeatNumber` error if the `seat_number` is not valid.
    pub fn act_fold(&self, seat_number: u8) -> Result<Stack, PKError> {
        if let Some(seat) = self.get_seat_mut(seat_number) {
            let remaining = seat.player.act_fold()?;
            drop(seat);
            Ok(remaining)
        } else {
            log::error!("Failed to find seat #{seat_number} for folding");
            Err(PKError::InvalidSeatNumber)
        }
    }

    /// # Errors
    ///
    /// `PKError::InvalidSeatNumber` error if the `seat_number` is not valid.
    pub fn act_forced_bet(&self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        if let Some(seat) = self.get_seat_mut(seat_number) {
            let ramaining = seat.player.act_bet_blind(amount)?;
            drop(seat);
            Ok(ramaining)
        } else {
            log::error!("Failed to act forced bet for seat #{seat_number}");
            Err(PKError::InvalidSeatNumber)
        }
    }

    #[must_use]
    pub fn are_dealt(&self) -> bool {
        for seat_cell in &self.0 {
            let seat = seat_cell.borrow();
            if !seat.cards.is_dealt() {
                return false;
            }
        }
        true
    }

    /// Assigns a `Seat` to the given index, returning the old `Seat`.
    ///
    /// # Errors
    ///
    /// This will return a `PKError::TableFull` error if the `seat_number` is not one of the
    /// available seats.
    pub fn assign(&self, seat_number: usize, seat: Seat) -> Result<Seat, PKError> {
        if seat_number >= self.size() as usize {
            return Err(PKError::TableFull);
        }
        Ok(self.0[seat_number].replace(seat))
    }

    #[must_use]
    pub fn borrow(&self, index: usize) -> Option<Ref<'_, Seat>> {
        self.0.get(index).map(|seat_cell| seat_cell.borrow())
    }

    #[must_use]
    pub fn borrow_all(&self) -> &[SeatCell] {
        &self.0
    }

    #[must_use]
    pub fn borrow_mut(&self, index: usize) -> Option<RefMut<'_, Seat>> {
        self.0.get(index).map(|seat_cell| seat_cell.borrow_mut())
    }

    /// Removes and returns the chips from all the player's bet stack and sets their state to `YetToAct`.
    ///
    /// # Errors
    ///
    /// * `PKError::InvalidTableAction` - throws if a player is not active in the hand.
    pub fn bring_it_in(&self) -> Result<Stack, PKError> {
        if !self.is_betting_complete() {
            return Err(PKError::ActionIsntFinished);
        }

        let collected = Stack::default();
        for (i, seat) in self.borrow_all().iter().enumerate() {
            if !seat.borrow().player.has_bet() {
                continue;
            }
            let chips = seat.borrow_mut().player.act_bring_it_in()?;
            log::trace!("Seat #{i} brought in {} chips.", chips.count());
            collected.add_to(chips);
        }

        log::info!("Bringing in {}.", collected.count());

        Ok(collected)
    }

    #[must_use]
    pub fn cards_string(&self) -> String {
        let mut seat_strings = Vec::new();
        for seat_cell in &self.0 {
            let seat = seat_cell.borrow();
            seat_strings.push(seat.cards.to_string());
        }
        seat_strings.join(", ")
    }

    #[must_use]
    pub fn chips_in_play(&self) -> usize {
        let mut total = 0;
        for seat_cell in &self.0 {
            let seat = seat_cell.borrow();
            total += seat.player.bet.count();
        }
        total
    }

    #[must_use]
    pub fn count_active_in_hand(&self) -> usize {
        let mut count = 0;
        for seat_cell in &self.0 {
            let seat = seat_cell.borrow();
            if seat.is_active() {
                count += 1;
            }
        }
        count
    }

    #[must_use]
    pub fn count_able_to_bet_in_hand(&self) -> usize {
        let mut count = 0;
        for seat_cell in &self.0 {
            let seat = seat_cell.borrow();
            if seat.is_active() && !seat.is_all_in() {
                count += 1;
            }
        }
        count
    }

    #[must_use]
    pub fn count_cards_in_play(&self) -> usize {
        let mut count = 0;
        for seat_cell in &self.0 {
            let seat = seat_cell.borrow();
            count += seat.cards.len();
        }
        count
    }

    /// Returns the number of cards that have actually been dealt to the players.
    ///
    /// ```
    /// use pkcore::cards_cell::CardsCell;
    /// use pkcore::casino::table::seats::Seats;
    /// use pkcore::util::data::TestData;
    ///
    /// // Seat eight players without any cards.
    /// let seats = Seats::try_from(TestData::the_hand_players()).unwrap();
    /// assert_eq!(0, seats.count_cards_dealt());
    /// assert_eq!(16, seats.count_cards_in_play());
    ///
    /// let deck = CardsCell::deck().shuffle();
    ///
    /// while seats.count_cards_dealt() != seats.count_cards_in_play() {
    ///     if let Ok(card) = deck.draw_one() {
    ///        seats.deal_card(2, card).unwrap();
    ///     }
    /// }
    ///
    /// assert_eq!(16, seats.count_cards_dealt());
    /// ```
    #[must_use]
    pub fn count_cards_dealt(&self) -> usize {
        let mut count = 0;
        for seat_cell in &self.0 {
            let seat = seat_cell.borrow();
            count += seat.cards.number_of_dealt_cards();
        }
        count
    }

    #[must_use]
    pub fn current_bet(&self) -> usize {
        self.borrow_all()
            .iter()
            .map(|s| s.borrow().player.bet.count())
            .max()
            .unwrap_or_default()
    }

    /// ```
    /// use pkcore::prelude::*;
    /// use pkcore::util::data::TestData;
    ///
    /// let seats = Seats::try_from(TestData::the_hand_players()).unwrap();
    ///
    /// assert!(seats.deal_card(2, Card::ACE_HEARTS).is_ok());
    /// assert_eq!("__ __, __ __, A♥ __, __ __, __ __, __ __, __ __, __ __", seats.cards_string());
    ///
    /// assert!(seats.deal_card(2, Card::KING_SPADES).is_ok());
    /// assert_eq!("__ __, __ __, A♥ __, K♠ __, __ __, __ __, __ __, __ __", seats.cards_string());
    ///
    /// assert!(seats.deal_card(2, Card::QUEEN_DIAMONDS).is_ok());
    /// assert!(seats.deal_card(2, Card::JACK_CLUBS).is_ok());
    /// assert!(seats.deal_card(2, Card::TEN_HEARTS).is_ok());
    /// assert!(seats.deal_card(2, Card::NINE_SPADES).is_ok());
    /// assert!(seats.deal_card(2, Card::EIGHT_DIAMONDS).is_ok());
    /// assert!(seats.deal_card(2, Card::SEVEN_CLUBS).is_ok());
    /// assert_eq!("8♦ __, 7♣ __, A♥ __, K♠ __, Q♦ __, J♣ __, T♥ __, 9♠ __", seats.cards_string());
    ///
    /// assert!(seats.deal_card(2, Card::SIX_HEARTS).is_ok());
    /// assert_eq!("8♦ __, 7♣ __, A♥ 6♥, K♠ __, Q♦ __, J♣ __, T♥ __, 9♠ __", seats.cards_string());
    ///
    /// assert!(seats.deal_card(2, Card::FOUR_SPADES).is_ok());
    /// assert!(seats.deal_card(2, Card::TREY_DIAMONDS).is_ok());
    /// assert!(seats.deal_card(2, Card::DEUCE_CLUBS).is_ok());
    /// assert!(seats.deal_card(2, Card::ACE_SPADES).is_ok());
    /// assert!(seats.deal_card(2, Card::KING_HEARTS).is_ok());
    /// assert!(seats.deal_card(2, Card::QUEEN_CLUBS).is_ok());
    /// assert!(seats.deal_card(2, Card::JACK_DIAMONDS).is_ok());
    /// assert_eq!("8♦ Q♣, 7♣ J♦, A♥ 6♥, K♠ 4♠, Q♦ 3♦, J♣ 2♣, T♥ A♠, 9♠ K♥", seats.cards_string());
    ///
    /// assert_eq!(PKError::AlreadyDealt, seats.deal_card(2, Card::DEUCE_DIAMONDS).unwrap_err());
    /// ```
    ///
    /// # Errors
    ///
    /// /// This will return a `PKError::AlreadyDealt` error if all seats have already been dealt
    pub fn deal_card(&self, utg: usize, card: Card) -> Result<(), PKError> {
        let seat_count = self.size() as usize;

        // Find the minimum number of dealt cards to determine current round
        let min_dealt = self
            .0
            .iter()
            .map(|s| s.borrow().cards.number_of_dealt_cards())
            .min()
            .unwrap_or(0);

        for i in 0..seat_count {
            let seat_index = (utg + i) % seat_count;
            let seat_cell = self.0.get(seat_index).ok_or(PKError::TableFull)?;

            if seat_cell.borrow().cards.number_of_dealt_cards() == min_dealt
                && seat_cell.borrow_mut().cards.deal(card).is_ok()
            {
                return Ok(());
            }
        }
        Err(PKError::AlreadyDealt)
    }

    #[must_use]
    pub fn first_yet_to_act(&self, utg: u8) -> Option<u8> {
        for seat in self.iter_from(utg) {
            if seat.player.state.is_yet_to_act() {
                return self.get_seat_number_from_handle(&seat.player.handle);
            }
        }
        None
    }

    #[must_use]
    pub fn first_yet_to_bet(&self, utg: u8) -> Option<u8> {
        for seat in self.iter_from(utg) {
            if seat.player.state.is_yet_to_act_or_blind() {
                return self.get_seat_number_from_handle(&seat.player.handle);
            }
        }
        None
    }

    #[must_use]
    pub fn get_seat_number_from_handle(&self, handle: &str) -> Option<u8> {
        for (i, seat_cell) in self.0.iter().enumerate() {
            let seat = seat_cell.borrow();
            if seat.player.handle == handle {
                return u8::try_from(i).ok();
            }
        }
        None
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&SeatCell> {
        self.0.get(index)
    }

    #[must_use]
    pub fn get_seat(&self, index: u8) -> Option<Ref<'_, Seat>> {
        let seat_cell = self.0.get(index as usize)?;
        Some(seat_cell.borrow())
    }

    #[must_use]
    pub fn get_seat_mut(&self, index: u8) -> Option<RefMut<'_, Seat>> {
        let seat_cell = self.0.get(index as usize)?;
        match seat_cell.try_borrow_mut() {
            Ok(seat) => Some(seat),
            Err(e) => {
                log::error!("Failed to borrow seat #{index} mutably: {e}");
                None
            }
        }
    }

    /// This tests if every player has done something, even if it's a forced bet. This is used
    /// to check to see if action needs has come back around to the blinds.
    #[must_use]
    pub fn has_everyone_acted(&self) -> bool {
        self.first_yet_to_act(0).is_none()
    }

    #[must_use]
    pub fn has_everyone_bet(&self) -> bool {
        self.first_yet_to_bet(0).is_none()
    }

    #[must_use]
    pub fn is_active(&self, seat_id: u8) -> bool {
        if let Some(seat) = self.get_seat(seat_id) {
            seat.is_active()
        } else {
            false
        }
    }

    /// Checks if equilibrium has been reached in the betting round.
    #[must_use]
    pub fn is_betting_complete(&self) -> bool {
        if self.count_active_in_hand() <= 1 {
            return true;
        }
        let current_bet = self.current_bet();

        for seat_cell in &self.0 {
            let seat = seat_cell.borrow();
            if seat.player.state.is_yet_to_act_or_blind() {
                return false;
            }
            if seat.is_active() && seat.player.bet.count() != current_bet {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn is_seat_in_hand(&self, seat_number: u8) -> bool {
        if let Some(seat) = self.get_seat(seat_number) {
            seat.is_in_hand()
        } else {
            false
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// NOTE: I have no idea why I wrote this.
    #[must_use]
    pub fn is_to_utg_preflop(&self) -> bool {
        for seat_cell in &self.0 {
            let seat = seat_cell.borrow();
            if seat.player.state.is_yet_to_act() {
                return true;
            }
        }
        false
    }

    pub fn iter(&self) -> std::slice::Iter<'_, SeatCell> {
        self.0.iter()
    }

    /// ```
    /// use pkcore::prelude::*;
    /// use pkcore::util::data::TestData;
    ///
    /// let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
    ///
    /// let gus = seats.next_to_act(3).unwrap();
    /// assert_eq!(3, gus);
    ///
    /// seats.get_seat_mut(gus).unwrap().player.state.set(PlayerState::Bet(100));
    ///
    /// // let daniel = seats.next_to_act(3).unwrap();
    /// // assert_eq!("Gus Hansen", daniel.player.handle);
    ///
    /// ```
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::Fubar` if no one is found to act next.
    pub fn next_to_act(&self, utg: u8) -> Result<u8, PKError> {
        // if self.is_betting_complete() {
        //     return Ok(utg);
        // }

        let current_bet = self.current_bet();

        // The logic flow is different if we're still waiting for the blinds to act.
        let everyone_has_bet = self.has_everyone_bet();

        for seat in self.iter_from(utg) {
            let state = &seat.player.state;

            if !seat.is_in_hand() || seat.is_all_in() {
                continue;
            }

            if state.is_blind() {
                return Ok(self.get_seat_number_from_handle(&seat.player.handle).unwrap_or(0));
            }

            if state.is_yet_to_act() {
                return Ok(self.get_seat_number_from_handle(&seat.player.handle).unwrap_or(0));
            }

            if state.is_check() && current_bet == 0 {
                continue;
            }

            if state.is_in_hand() && everyone_has_bet && state.get().amount() < current_bet {
                return Ok(self.get_seat_number_from_handle(&seat.player.handle).unwrap_or(0));
            }
        }

        // Edge case where all action is complete for the round, but the bets haven't been
        // brought in.
        for seat in self.iter_from(utg) {
            if !seat.is_in_hand() || seat.is_all_in() {
                continue;
            }
            return Ok(self.get_seat_number_from_handle(&seat.player.handle).unwrap_or(0));
        }

        Err(PKError::InvalidSeatNumber)
    }

    /// Clears the `PlayerState` for all the seats.
    pub fn reset_state(&self) {
        for seat_cell in &self.0 {
            let seat = seat_cell.borrow_mut();
            seat.player.state.reset();
        }
    }

    #[must_use]
    pub fn size(&self) -> u8 {
        if let Ok(size) = u8::try_from(self.0.len()) {
            size
        } else {
            log::error!("Seat size conversion error");
            0
        }
    }

    /// Takes all the cards from all the seats and returns them as a single `CardsCell`.
    ///
    /// ```
    /// use pkcore::casino::table::seats::Seats;
    /// use pkcore::util::data::TestData;
    ///
    /// let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
    /// let cards = seats.take_cards();
    /// assert_eq!(cards.to_string(), "T♠ 2♥ 8♠ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♣ 4♦ 7♣ 2♣");
    ///
    /// // Now, they should all be empty.
    /// let cards = seats.take_cards();
    /// assert_eq!(cards.to_string(), "");
    /// ```
    #[must_use]
    pub fn take_cards(&self) -> CardsCell {
        let cards = CardsCell::default();
        for seat_cell in &self.0 {
            let mut seat = seat_cell.borrow_mut();
            if !seat.is_empty() {
                let seat_cards = Cards::from(seat.cards.take());
                cards.insert_all(seat_cards);
            }
        }
        cards
    }

    #[must_use]
    pub fn total_chip_count(&self) -> usize {
        let mut total = 0;
        for seat_cell in &self.0 {
            let seat = seat_cell.borrow();
            if !seat.is_empty() {
                total += seat.player.total_chip_count();
            }
        }
        total
    }

    /// Iterate indices starting at `start`, wrapping once through all seats.
    pub fn indices_from(&self, start: u8) -> impl Iterator<Item = usize> + '_ {
        let len = self.0.len();
        (0..len).map(move |offset| (start as usize + offset) % len)
    }

    /// The original version of this function was completely flawed. It assumed that the value of
    /// to call was whatever the highest bet was.
    #[must_use]
    pub fn to_call(&self, player: u8) -> usize {
        let highest_bet = self.current_bet();

        if let Some(seat) = self.get_seat(player) {
            highest_bet.saturating_sub(seat.player.bet.count())
        } else {
            0
        }
    }

    // region iterators

    /// Iterate immutably over seats starting at `start`, wrapping through all seats.
    pub fn iter_from(&self, start: u8) -> impl Iterator<Item = Ref<'_, Seat>> {
        self.indices_from(start).map(|i| self.0[i].borrow())
    }

    /// Iterate mutably over seats starting at `start`, wrapping through all seats.
    /// Note: avoid holding on to the returned `RefMut` across iterations.
    pub fn iter_from_mut(&self, start: u8) -> impl Iterator<Item = RefMut<'_, Seat>> {
        self.indices_from(start).map(|i| self.0[i].borrow_mut())
    }

    /// Run a closure for each seat starting at `start`, passing (index, &Seat).
    /// Each borrow is dropped before the next iteration.
    pub fn for_each_from<F>(&self, start: u8, mut f: F)
    where
        F: FnMut(usize, &Seat),
    {
        for i in self.indices_from(start) {
            let seat_ref = self.0[i].borrow();
            f(i, &seat_ref);
        }
    }

    // endregion
}

impl Default for Seats {
    fn default() -> Self {
        let mut seats = Vec::with_capacity(Self::DEFAULT_NUMBER_SEATS as usize);
        for _ in 0..Self::DEFAULT_NUMBER_SEATS {
            seats.push(Seat::default());
        }
        Self::new(seats)
    }
}

impl std::fmt::Display for Seats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, seat) in self.0.iter().enumerate() {
            if seat.is_empty() {
                writeln!(f, "Seat {i}: __________")?;
            } else {
                writeln!(f, "Seat {i}: {seat}")?;
            }
        }
        Ok(())
    }
}

/// TODO: Why do I need these?
impl From<Box<[SeatCell; 6]>> for Seats {
    fn from(value: Box<[SeatCell; 6]>) -> Self {
        Self(value)
    }
}

impl From<Box<[SeatCell; 7]>> for Seats {
    fn from(value: Box<[SeatCell; 7]>) -> Self {
        Self(value)
    }
}

impl From<Box<[SeatCell; 8]>> for Seats {
    fn from(value: Box<[SeatCell; 8]>) -> Self {
        Self(value)
    }
}

impl From<Box<[SeatCell; 9]>> for Seats {
    fn from(value: Box<[SeatCell; 9]>) -> Self {
        Self(value)
    }
}

impl TryFrom<Vec<Seat>> for Seats {
    type Error = PKError;

    fn try_from(value: Vec<Seat>) -> Result<Self, Self::Error> {
        if value.len() > Self::MAX_NUMBER_SEATS as usize {
            return Err(PKError::TableFull);
        }
        Ok(Self::new(value))
    }
}

/// TODO: This feels like stupid over architecting.
impl TryFrom<Vec<SeatCell>> for Seats {
    type Error = PKError;

    fn try_from(value: Vec<SeatCell>) -> Result<Self, Self::Error> {
        if value.len() > Self::MAX_NUMBER_SEATS as usize {
            return Err(PKError::TableFull);
        }
        Ok(Self(value.into_boxed_slice()))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__seats_tests {
    use super::*;
    use crate::casino::game::ForcedBets;
    use crate::casino::table::Table;
    use crate::prelude::*;
    use crate::util::data::TestData;

    #[test]
    fn act_all_in() {
        let seats = Seats::try_from(TestData::min_seats()).unwrap();

        let all_in = seats.act_all_in(0).unwrap();
        assert_eq!(1_000_000, all_in);
    }

    #[test]
    fn all_players_have_acted() {
        let seats = Seats::try_from(TestData::min_seats()).unwrap();
        assert!(!seats.is_betting_complete());

        for seat_cell in seats.borrow_all() {
            let seat = seat_cell.borrow();
            seat.player.state.set(PlayerState::Check(100));
        }

        assert!(seats.is_betting_complete());
    }

    #[test]
    fn assign() {
        let seats = Seats::default();
        let antonio_esfandiari = Seat {
            player: crate::casino::player::Player::new_with_chips("Antonio Esfandari".to_string(), 1_000_000),
            cards: boxed!("A♦ Q♣"),
        };

        let old_seat = seats.assign(1, antonio_esfandiari.clone()).unwrap();

        assert_eq!(old_seat, Seat::default());

        let seat = seats.get(1).unwrap();

        assert_eq!(&SeatCell::new(antonio_esfandiari), seat);
    }

    #[test]
    fn bring_it_in() {
        let seats = Seats::new(TestData::min_players());

        seats.act_forced_bet(1, 50).expect("Should be able to act");
        seats.act_forced_bet(2, 100).expect("Should be able to act");
        seats.act_fold(0).expect("Should be able to act");
        seats.act_call(1).expect("Should be able to act");
        seats.act_check(2).expect("Should be able to act");

        println!("{seats}");

        assert_eq!(Stack::new(200), seats.bring_it_in().unwrap());
    }

    #[test]
    fn count_cards_in_play() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
        assert_eq!(16, seats.count_cards_in_play());
    }

    #[test]
    fn current_bet() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
        let _ = seats.act_forced_bet(0, 100);
        assert_eq!(100, seats.current_bet());

        let _ = seats.act_forced_bet(1, 200);
        assert_eq!(200, seats.current_bet());

        let _ = seats.act_bet(2, 400);
        assert_eq!(400, seats.current_bet());

        let _ = seats.act_bet(0, 400);
        assert_eq!(400, seats.current_bet());
    }

    #[test]
    fn first_yet_to_act() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
        seats.act_forced_bet(1, 50).expect("Should be able to act");
        seats.act_forced_bet(2, 100).expect("Should be able to act");

        assert_eq!(3, seats.first_yet_to_act(1).unwrap());
    }

    #[test]
    fn get_seat_number_from_handle() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();

        assert_eq!(3, seats.get_seat_number_from_handle("Gus Hansen").unwrap());
        assert_eq!(4, seats.get_seat_number_from_handle("Daniel Negreanu").unwrap());
        assert!(seats.get_seat_number_from_handle("The Russian").is_none())
    }

    #[test]
    fn has_everyone_acted() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
        assert!(!seats.has_everyone_acted());

        for seat_cell in seats.borrow_all() {
            let seat = seat_cell.borrow();
            seat.player.state.set(PlayerState::Call(100));
        }

        assert!(seats.has_everyone_acted());
    }

    #[test]
    fn next_to_act() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();

        let seat = seats.next_to_act(3).unwrap();
        assert_eq!(3, seat);

        seats
            .get_seat_mut(seat)
            .unwrap()
            .player
            .state
            .set(PlayerState::Check(100));
        let seat = seats.next_to_act(3).unwrap();
        assert_eq!(4, seat);

        seats
            .get_seat_mut(seat)
            .unwrap()
            .player
            .state
            .set(PlayerState::Bet(100));
        let seat = seats.next_to_act(3).unwrap();
        assert_eq!(5, seat);
    }

    #[test]
    fn next_to_act__long_play_defect() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
        let pot = Stack::default();

        seats.act_forced_bet(1, 50).expect("Should be able to act");
        seats.act_forced_bet(2, 100).expect("Should be able to act");

        assert_eq!(3, seats.next_to_act(3).unwrap());
        seats.act_bet(3, 2100).expect("Should be able to act");

        assert_eq!(4, seats.next_to_act(3).unwrap());
        seats.act_raise(4, 5000).expect("Should be able to act");

        assert_eq!(5, seats.next_to_act(3).unwrap());
        seats.act_fold(5).expect("Should be able to fold");

        assert_eq!(6, seats.next_to_act(3).unwrap());
        seats.act_fold(6).expect("Should be able to fold");

        assert_eq!(7, seats.next_to_act(3).unwrap());
        seats.act_fold(7).expect("Should be able to fold");

        assert_eq!(0, seats.next_to_act(3).unwrap());
        seats.act_fold(0).expect("Should be able to call");

        assert_eq!(1, seats.next_to_act(3).unwrap());
        pot.add_to(seats.act_fold(1).unwrap());

        assert_eq!(2, seats.next_to_act(3).unwrap());
        pot.add_to(seats.act_fold(2).unwrap());

        assert_eq!(3, seats.next_to_act(3).unwrap());
        seats.act_call(3).expect("Should be able to call");

        assert!(seats.is_betting_complete());
        pot.add_to(seats.bring_it_in().unwrap());
        assert_eq!(Stack::new(10150), pot);

        assert_eq!(3, seats.next_to_act(1).unwrap());
        assert_eq!(3, seats.next_to_act(2).unwrap());
        assert_eq!(3, seats.next_to_act(3).unwrap());
        assert_eq!(3, seats.next_to_act(5).unwrap());
        assert_eq!(3, seats.next_to_act(6).unwrap());
        assert_eq!(3, seats.next_to_act(7).unwrap());
        assert_eq!(3, seats.next_to_act(0).unwrap());
    }

    #[test]
    fn reset_state() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        let _ = table.act_forced_bets();
        let _seat0_folded_amount = table.act_fold(0).unwrap();
        let _seat1_folded_amount = table.act_fold(1).unwrap();

        table.seats.reset_state();

        for seat in table.seats.borrow_all() {
            let seat = seat.borrow();
            assert_eq!(PlayerState::YetToAct, seat.player.state.get());
        }
    }

    #[test]
    fn seat() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
        // Gab the seat, change the player's handle, and then return it.
        let mut seat = seats.get_seat_mut(0).unwrap();
        assert_eq!("Doyle Brunson", seat.player.handle);
        seat.player.handle = "Texas Dolly".to_string();
        drop(seat);

        let seat = seats.get_seat_mut(0).unwrap();

        assert_eq!("Texas Dolly", seat.player.handle);
    }

    #[test]
    fn get() {
        let seats = Seats::default();
        let seat = seats.get(0).unwrap();
        let gus_hansen = Seat {
            player: crate::casino::player::Player::new_with_chips("Gus Hansen".to_string(), 1_000_000),
            cards: boxed!("5♦ 5♣"),
        };

        assert!(seat.is_empty());

        seat.swap(&SeatCell::new(gus_hansen));

        assert!(!seat.is_empty());

        let seat = seats.get(0).unwrap();
        assert!(!seat.is_empty());

        print!("{seats}");
    }

    #[test]
    fn iter_from_wraps_in_order() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
        let order: Vec<usize> = seats.indices_from(6).take(4).collect();
        assert_eq!(order, vec![6, 7, 0, 1]);

        // Sanity: collect a couple of handles starting at 6
        let handles: Vec<String> = seats.iter_from(6).take(2).map(|s| s.player.handle.clone()).collect();
        assert_eq!(handles.len(), 2);
    }

    #[test]
    fn iter_from_wraps_confirm_next_to_act() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();

        let utg: u8 = 3;

        if let Some(_seat_utg) = seats.get_seat(utg) {
            let order: Vec<usize> = seats.indices_from(utg + 1).take((seats.size() - 1) as usize).collect();
            assert_eq!(order, vec![4, 5, 6, 7, 0, 1, 2]);
            for i in order.iter() {
                let seat = seats.get_seat(*i as u8).unwrap();
                if seat.player.state.is_yet_to_act() {
                    // assert_eq!(seat.player.handle, "Gus Hansen");
                    break;
                }
            }
        }
    }

    #[test]
    fn iterror() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();

        let gus = seats.next_to_act(3).unwrap();
        assert_eq!(3, gus);
        //
        seats.get_seat_mut(gus).unwrap().player.state.set(PlayerState::Bet(100));
        //
        let _daniel = seats.next_to_act(3).unwrap();
        // assert_eq!(4, daniel);
    }

    /// Matches test in `Table`
    #[test]
    fn validate__tiny_table() {
        let seats = Seats::try_from(TestData::min_seats()).unwrap();

        let _ = seats.act_forced_bet(1, 50);
        let _ = seats.act_forced_bet(2, 100);

        // Seat 0 and 2 cannot check because they have not put enough money in the pot.
        assert!(!seats.is_betting_complete());
        assert_eq!(PKError::InvalidTableAction, seats.act_check(1).unwrap_err());
        assert_eq!(50, seats.to_call(1));
        assert_eq!(PKError::InvalidTableAction, seats.act_check(0).unwrap_err());
        assert_eq!(100, seats.to_call(0));
        // Seat 1 can check because their big blind is the highest bet.
        assert_eq!(0, seats.to_call(2));
        assert_eq!(999_900, seats.act_check(2).unwrap());

        assert_eq!(100, seats.to_call(0));
        assert_eq!((100, 999_900), seats.act_call(0).unwrap());
        assert_eq!(0, seats.to_call(0));
        assert_eq!(PKError::InsufficientChips, seats.act_call(0).unwrap_err());
        assert!(!seats.is_betting_complete());
    }
}
