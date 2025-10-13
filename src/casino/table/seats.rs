use crate::PKError;
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
    pub(crate) fn new(seats: Vec<Seat>) -> Self {
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
    pub fn get(&self, index: usize) -> Option<&SeatCell> {
        self.0.get(index)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn seat(&self, index: usize) -> Option<RefMut<'_, Seat>> {
        let seat_cell = self.0.get(index)?;
        match seat_cell.try_borrow_mut() {
            Ok(seat) => Some(seat),
            Err(e) => {
                log::error!("Failed to borrow seat #{index} mutably: {e}");
                None
            }
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
            let seat = seat_cell.borrow_mut();
            if !seat.is_empty() {
                cards.insert_all(seat.cards.take());
            }
        }
        cards
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
    use crate::cards_cell::CardsCell;
    use crate::util::data::TestData;
    use std::str::FromStr;

    #[test]
    fn assign() {
        let seats = Seats::default();
        let antonio_esfandiari = Seat {
            player: crate::casino::player::Player::new_with_chips("Antonio Esfandari".to_string(), 1_000_000),
            cards: CardsCell::from_str("A♦ Q♣").unwrap(),
        };

        let old_seat = seats.assign(1, antonio_esfandiari.clone()).unwrap();

        assert_eq!(old_seat, Seat::default());

        let seat = seats.get(1).unwrap();

        assert_eq!(&SeatCell::new(antonio_esfandiari), seat);
    }

    #[test]
    fn seat() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
        // Gab the seat, change the player's handle, and then return it.
        let mut seat = seats.seat(0).unwrap();
        assert_eq!("Doyle Brunson", seat.player.handle);
        seat.player.handle = "Texas Dolly".to_string();
        drop(seat);

        let seat = seats.seat(0).unwrap();

        assert_eq!("Texas Dolly", seat.player.handle);
    }

    #[test]
    fn get() {
        let seats = Seats::default();
        let seat = seats.get(0).unwrap();
        let gus_hansen = Seat {
            player: crate::casino::player::Player::new_with_chips("Gus Hansen".to_string(), 1_000_000),
            cards: CardsCell::from_str("5♦ 5♣").unwrap(),
        };

        assert!(seat.is_empty());

        seat.swap(&SeatCell::new(gus_hansen));

        assert!(!seat.is_empty());

        let seat = seats.get(0).unwrap();
        assert!(!seat.is_empty());

        print!("{seats}");
    }
}
