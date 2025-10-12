use std::cell::RefCell;
use crate::cards_cell::CardsCell;
use crate::casino::player::Player;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SeatCell(RefCell<Seat>);

impl SeatCell {
    #[must_use]
    pub fn new(seat: Seat) -> Self {
        Self(RefCell::new(seat))
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, Seat> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, Seat> {
        self.0.borrow_mut()
    }

    pub fn replace(&self, seat: Seat) -> Seat {
        self.0.replace(seat)
    }

    pub fn into_inner(self) -> Seat {
        self.0.into_inner()
    }

    pub fn get_mut(&mut self) -> &mut Seat {
        self.0.get_mut()
    }

    pub fn take(&self) -> Seat {
        self.0.take()
    }

    pub fn swap(&self, other: &SeatCell) {
        self.0.swap(&other.0);
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
    pub cards: CardsCell,
}

impl Seat {
    #[must_use]
    pub fn new(player: Player) -> Self {
        Seat {
            player,
            cards: CardsCell::default(),
        }
    }

    #[must_use]
    pub fn new_with_cards(player: Player, cards: CardsCell) -> Self {
        Seat { player, cards }
    }

    pub fn is_empty(&self) -> bool {
        self.player.id == uuid::Uuid::nil()
    }
}

impl std::fmt::Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cards: {}, Player: {}", self.cards, self.player)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table_tests {
    use super::*;

    #[test]
    pub fn is_empty() {
        let seat = Seat::default();
        assert!(seat.is_empty());

        let player = Player::new("Alice".to_string());
        let seat_with_player = Seat::new(player);
        assert!(!seat_with_player.is_empty());
    }
}