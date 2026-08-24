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
    /// Table-level `bet` immediately **after** this seat last voluntarily
    /// acted on the current street (TDA 2024 Rule 47-A, `DEFECT_010`).
    ///
    /// Paired with [`PlayerState`](crate::prelude::PlayerState) it answers
    /// "is this seat facing at least a full raise since it last acted?", which
    /// is what decides whether a short all-in re-opens the betting for it.
    ///
    /// Post-action, not pre-action: a player who raised to 300 must be
    /// measured against 300, not against the 100 they faced before raising.
    /// Forced posts (blinds, antes, the stud bring-in) do not set it.
    ///
    /// Reset to `0` alongside `PlayerState::YetToAct` at the street boundary.
    pub bet_level_when_last_acted: usize,
}

impl Default for Seat {
    fn default() -> Self {
        Seat {
            player: Player::default(),
            cards: BoxedCards::blanks(2),
            hand: SeatHand::new(0),
            bet_level_when_last_acted: 0,
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
            bet_level_when_last_acted: 0,
        }
    }

    /// Creates a seat for `player` holding `cards`.
    ///
    /// The companion to [`new`](Seat::new), which allocates blank slots
    /// instead. Like `new`, the `SeatHand` is numbered `0`; callers that need
    /// a real ring index set it after seating.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::arrays::sliced::BoxedCards;
    /// use pkcore::casino::table::{Player, Seat};
    /// use pkcore::prelude::Forgiving;
    ///
    /// let player = Player::new_with_chips("Sam".to_string(), 500);
    /// let seat = Seat::new_with_cards(player, BoxedCards::forgiving_from_str("A♠ K♦"));
    ///
    /// assert_eq!(2, seat.cards.len());
    /// ```
    #[must_use]
    pub fn new_with_cards(player: Player, cards: BoxedCards) -> Self {
        Seat {
            player,
            cards,
            hand: SeatHand::new(0),
            bet_level_when_last_acted: 0,
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

/// Seats a player by name, with no chips and blank cards.
///
/// The shape a hand history gives you: names, but no stack until the log says
/// what it was. Mirrors the celled `Seat::from(String)`.
impl From<String> for Seat {
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::Seat;
    ///
    /// let seat = Seat::from("Ann".to_string());
    /// assert_eq!("Ann", seat.player.handle);
    /// assert_eq!(0, seat.player.chips);
    /// ```
    fn from(handle: String) -> Self {
        Seat::new(Player::new(handle))
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
    use crate::prelude::Forgiving;

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

    #[test]
    fn seat_new_with_cards_holds_the_given_cards() {
        let player = Player::new_with_chips("Dealt".to_string(), 500);

        let seat = Seat::new_with_cards(player, boxed!("A♠ K♦"));

        assert_eq!(boxed!("A♠ K♦"), seat.cards);
        assert_eq!("Dealt", seat.player.handle);
    }

    #[test]
    fn seat_new_with_cards_accepts_blank_slots() {
        // `Dealer::new` builds an empty ring this way.
        let seat = Seat::new_with_cards(Player::default(), BoxedCards::blanks(2));

        assert!(seat.is_empty());
        assert_eq!(2, seat.cards.len());
    }

    #[test]
    fn seat_discard_cards_hands_the_cards_back_and_clears_the_hand() {
        // Ported from the retired celled `Seat` suite (EPIC-83 Phase 3d).
        // Both stores must clear: `cards` is what the table reads, `hand` is
        // what the visibility-aware stud paths read.
        let mut seat = Seat::new_with_cards(Player::new_with_chips("Dealt".to_string(), 500), boxed!("A♠ K♦"));
        seat.hand
            .push(crate::Card::ACE_SPADES, crate::play::visibility::Visibility::Down);

        let discarded = seat.discard_cards();

        assert_eq!(2, discarded.len(), "both cards come back");
        assert!(!seat.cards.is_dealt(), "the seat is holding nothing");
        assert!(seat.hand.is_empty(), "the visibility-aware hand is cleared too");
    }

    #[test]
    fn seat_from_a_name_seats_a_chipless_player() {
        let seat = Seat::from("Ann".to_string());

        assert_eq!("Ann", seat.player.handle);
        assert_eq!(0, seat.player.chips, "a name alone buys no chips");
        assert!(!seat.is_empty());
    }
}
