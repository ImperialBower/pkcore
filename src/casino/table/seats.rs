use crate::casino::table::seat::{Seat, SeatCell};
use crate::PKError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Seats(Box<[SeatCell]>);

impl Seats {
    pub const DEFAULT_NUMBER_SEATS: usize = 6;
    pub const MAX_NUMBER_SEATS: usize = 9;

    /// How frackin' cool is this `into_boxed_slice` pattern?! I'm going to need to play with this.
    #[must_use]
    pub(crate) fn new(seats: Vec<Seat>) -> Self {
        let seat_cells: Vec<SeatCell> = seats.into_iter().map(SeatCell::new).collect();
        Seats(seat_cells.into_boxed_slice())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&SeatCell> {
        self.0.get(index)
    }

    #[must_use]
    pub fn borrow_all(&self) -> &[SeatCell] {
        &self.0
    }

    #[must_use]
    pub fn borrow(&self, index: usize) -> Option<std::cell::Ref<'_, Seat>> {
        self.0.get(index).map(|seat_cell| seat_cell.borrow())
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
        if value.len() > Self::MAX_NUMBER_SEATS {
            return Err(PKError::TableFull);
        }
        Ok(Self::new(value))
    }
}

impl TryFrom<Vec<SeatCell>> for Seats {
    type Error = PKError;

    fn try_from(value: Vec<SeatCell>) -> Result<Self, Self::Error> {
        if value.len() > Self::MAX_NUMBER_SEATS {
            return Err(PKError::TableFull);
        }
        Ok(Self(value.into_boxed_slice()))
    }
}

impl Default for Seats {
    fn default() -> Self {
        let mut seats = Vec::with_capacity(Self::DEFAULT_NUMBER_SEATS);
        for _ in 0..Self::DEFAULT_NUMBER_SEATS {
            seats.push(Seat::default());
        }
        Self::new(seats)
    }
}

impl std::fmt::Display for Seats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, seat) in self.0.iter().enumerate() {
            writeln!(f, "Seat {}: {}", i + 1, seat)?;
        }
        Ok(())
    }
}