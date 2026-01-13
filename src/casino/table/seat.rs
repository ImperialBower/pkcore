use crate::cards_cell::CardsCell;
use crate::casino::player::Player;
use crate::prelude::BoxedCards;
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

/// Want this to be a `BasicPile`, which is a vector
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Seat {
    pub player: Player,
    pub cards: BoxedCards,
}

impl Seat {
    #[must_use]
    pub fn new(player: Player) -> Self {
        Seat {
            player,
            cards: BoxedCards::default(),
        }
    }

    #[must_use]
    pub fn new_with_cards(player: Player, cards: BoxedCards) -> Self {
        Seat { player, cards }
    }

    #[must_use]
    pub fn discard_cards(&mut self) -> CardsCell {
        let boxed = self.cards.take();
        CardsCell::from(boxed)
    }

    pub fn is_active(&self) -> bool {
        self.player.state.is_active()
    }

    pub fn is_all_in(&self) -> bool {
        self.player.is_all_in()
    }

    pub fn is_empty(&self) -> bool {
        self.player.id == uuid::Uuid::nil()
    }

    pub fn is_in_hand(&self) -> bool {
        self.player.state.is_in_hand()
    }

    pub fn is_yet_to_act(&self) -> bool {
        self.player.state.is_yet_to_act()
    }

    #[must_use]
    pub fn is_yet_to_act_or_blind(&self) -> bool {
        self.player.state.is_yet_to_act_or_blind()
    }
}

impl std::fmt::Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cards: {}, Player: {}", self.cards, self.player)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__seat_tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    pub fn discard_cards() {
        let player = Player::new("Bob".to_string());
        let mut seat = Seat::new(player);
        seat.cards = boxed!("A♠ K♠");

        let discarded = seat.discard_cards();
        assert_eq!(discarded.to_string(), "A♠ K♠");
        assert_eq!(seat.cards.to_string(), "__ __");
    }

    #[test]
    pub fn is_empty() {
        let seat = Seat::default();
        assert!(seat.is_empty());

        let player = Player::new("Alice".to_string());
        let seat_with_player = Seat::new(player);
        assert!(!seat_with_player.is_empty());
    }
}
