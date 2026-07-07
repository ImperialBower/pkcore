use crate::casino::table_celled::seats::seat::Seat;
use std::cell::{BorrowMutError, Ref, RefCell, RefMut};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SeatCell(RefCell<Seat>);

impl SeatCell {
    #[must_use]
    pub fn new(seat: Seat) -> Self {
        Self(RefCell::new(seat))
    }

    pub fn borrow(&self) -> Ref<'_, Seat> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, Seat> {
        self.0.borrow_mut()
    }

    pub fn replace(&self, seat: Seat) -> Seat {
        self.0.replace(seat)
    }

    pub fn into_inner(self) -> Seat {
        self.0.into_inner()
    }

    pub fn is_clear(&self) -> bool {
        self.borrow().is_clear()
    }

    pub fn is_in_hand(&self) -> bool {
        self.borrow().player.state.is_in_hand()
    }

    pub fn is_yet_to_act(&self) -> bool {
        self.borrow().player.state.is_yet_to_act()
    }

    pub fn get_mut(&mut self) -> &mut Seat {
        self.0.get_mut()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    pub fn swap(&self, other: &SeatCell) {
        self.0.swap(&other.0);
    }

    pub fn take(&self) -> Seat {
        self.0.take()
    }

    /// # Errors
    ///
    /// This will return a `BorrowError` if the `RefCell` is already mutably borrowed.
    pub fn try_borrow(&self) -> Result<Ref<'_, Seat>, std::cell::BorrowError> {
        self.0.try_borrow()
    }

    /// # Errors
    ///
    /// This will return a `BorrowMutError` error if the `RefCell` is already borrowed.
    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, Seat>, BorrowMutError> {
        self.0.try_borrow_mut()
    }
}

impl std::fmt::Display for SeatCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let internal = self.0.borrow();
        write!(f, "{internal}")
    }
}
