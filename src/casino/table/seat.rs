//! [`Seat`] — a seat at a [`Table`](super::Table): a [`Player`] plus their cards.

use super::Player;
use crate::Pile;
use crate::arrays::sliced::BoxedCards;
use crate::cards::Cards;
use crate::play::seat_hand::SeatHand;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

/// A single seat at the table holding a [`Player`] and their hole cards.
///
/// Replaces `SeatCell(RefCell<Seat>)` with a plain struct whose fields are
/// directly mutable via `&mut self`.
///
/// # Examples
///
/// ```
/// use pkcore::casino::table::{Player, Seat};
///
/// let player = Player::new_with_chips("Oliver".to_string(), 1_000);
/// let seat = Seat::new(player);
/// assert!(!seat.is_empty());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Seat {
    pub player: Player,
    pub cards: BoxedCards,
    /// Visibility-aware per-seat hand introduced by EPIC-29 Phase 5.
    /// Populated in parallel with `cards` for NLHE/PLO (every card
    /// `Visibility::Down`); stud-family variants (EPIC-32/33) will use
    /// this field as the source of truth for per-card visibility.
    pub hand: SeatHand,
}

impl Default for Seat {
    fn default() -> Self {
        Seat {
            player: Player::default(),
            cards: BoxedCards::blanks(2),
            hand: SeatHand::new(0),
        }
    }
}

impl Seat {
    /// Creates a seat for `player` with two blank card slots.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{Player, Seat};
    ///
    /// let seat = Seat::new(Player::new_with_chips("Pat".to_string(), 500));
    /// assert!(!seat.is_empty());
    /// ```
    #[must_use]
    pub fn new(player: Player) -> Self {
        Seat {
            player,
            cards: BoxedCards::blanks(2),
            hand: SeatHand::new(0),
        }
    }

    /// True when no player is seated (nil UUID / empty handle).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.player.id == Uuid::default() || self.player.handle.is_empty()
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.is_empty() && self.player.is_active()
    }

    #[must_use]
    pub fn is_all_in(&self) -> bool {
        self.player.is_all_in()
    }

    #[must_use]
    pub fn is_in_hand(&self) -> bool {
        !self.is_empty() && self.player.is_in_hand()
    }

    #[must_use]
    pub fn is_yet_to_act(&self) -> bool {
        self.player.state.is_yet_to_act()
    }

    #[must_use]
    pub fn is_yet_to_act_or_blind(&self) -> bool {
        self.player.state.is_yet_to_act_or_blind()
    }

    #[must_use]
    pub fn is_clear(&self) -> bool {
        self.player.is_clear()
    }

    /// Discards the player's cards, returning them as `Cards`. Clears
    /// both the legacy `cards: BoxedCards` storage and the new
    /// visibility-aware `hand: SeatHand`.
    pub fn discard_cards(&mut self) -> Cards {
        let cards = self.cards.cards();
        let _ = self.cards.take();
        self.hand.clear();
        cards
    }
}

impl Display for Seat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "[Empty]")
        } else {
            write!(f, "Cards: {}, Player: {}", self.cards, self.player)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__seat_tests {
    use super::*;

    #[test]
    fn seat_new() {
        let player = Player::new_with_chips("Seat0".to_string(), 1_000);
        let seat = Seat::new(player);
        assert!(!seat.is_empty());
        assert!(seat.is_in_hand());
    }

    #[test]
    fn seat_default_is_empty() {
        let seat = Seat::default();
        assert!(seat.is_empty());
    }
}
