use crate::PKError;
use crate::card::Card;
use crate::cards::Cards;
use crate::cards_cell::CardsCell;
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
    pub fn get(&self, index: usize) -> Option<&SeatCell> {
        self.0.get(index)
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

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn get_seat(&self, index: usize) -> Option<Ref<'_, Seat>> {
        let seat_cell = self.0.get(index)?;
        Some(seat_cell.borrow())
    }

    #[must_use]
    pub fn get_seat_mut(&self, index: usize) -> Option<RefMut<'_, Seat>> {
        let seat_cell = self.0.get(index)?;
        match seat_cell.try_borrow_mut() {
            Ok(seat) => Some(seat),
            Err(e) => {
                log::error!("Failed to borrow seat #{index} mutably: {e}");
                None
            }
        }
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
    fn count_cards_in_play() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
        assert_eq!(16, seats.count_cards_in_play());
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
}
