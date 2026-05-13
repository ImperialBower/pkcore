//! Per-seat, variable-size, visibility-aware hole-card collection.
//!
//! `SeatHand` is the per-seat hand storage used by the table engine. It
//! holds 2 cards for Hold'em, 4 for Omaha, and up to 7 for Stud/Razz (built
//! up across streets). Internally it wraps `Vec<HoleCard>`; its API surface
//! mirrors the ergonomics of the fixed-size `arrays/{Two, Four, Seven}`
//! types (`len`, `iter`, `sort`, indexing, `Display`) so consumers can use
//! it interchangeably with the existing collections.
//!
//! For bridging into the existing evaluator API, `SeatHand` provides
//! [`SeatHand::as_two`], [`SeatHand::as_four`], and [`SeatHand::as_seven`]
//! conversion helpers. Each returns `Some` only when the count matches the
//! target shape; this is the load-bearing migration path that keeps every
//! existing fixed-size evaluator working unchanged once variant code starts
//! storing cards in `SeatHand`.

use crate::arrays::four::Four;
use crate::arrays::seven::Seven;
use crate::arrays::sliced::BoxedCards;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::play::hole_card::HoleCard;
use crate::play::visibility::Visibility;
use std::fmt::{Display, Formatter};
use std::ops::Index;

const HOLE_CAPACITY_HINT: usize = 7;

/// A seat's hand of hole cards, with per-card visibility.
///
/// # Examples
///
/// ```
/// use pkcore::card::Card;
/// use pkcore::play::seat_hand::SeatHand;
/// use pkcore::play::visibility::Visibility;
///
/// let mut hand = SeatHand::new(0);
/// hand.push(Card::ACE_SPADES, Visibility::Down);
/// hand.push(Card::KING_SPADES, Visibility::Down);
/// assert_eq!(2, hand.len());
/// assert!(hand.as_two().is_some());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeatHand {
    seat: u8,
    cards: Vec<HoleCard>,
}

impl SeatHand {
    /// Constructs an empty `SeatHand` for the given seat, pre-allocated for
    /// the largest variant (Stud's 7 cards).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let hand = SeatHand::new(3);
    /// assert_eq!(3, hand.seat());
    /// assert!(hand.is_empty());
    /// ```
    #[must_use]
    pub fn new(seat: u8) -> Self {
        SeatHand {
            seat,
            cards: Vec::with_capacity(HOLE_CAPACITY_HINT),
        }
    }

    /// Constructs an empty `SeatHand` with a caller-specified capacity hint.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let hand = SeatHand::with_capacity(0, 2);
    /// assert!(hand.is_empty());
    /// ```
    #[must_use]
    pub fn with_capacity(seat: u8, cap: usize) -> Self {
        SeatHand {
            seat,
            cards: Vec::with_capacity(cap),
        }
    }

    /// Returns the seat index this hand belongs to.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// assert_eq!(5, SeatHand::new(5).seat());
    /// ```
    #[must_use]
    pub fn seat(&self) -> u8 {
        self.seat
    }

    /// Number of cards currently in the hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    /// use pkcore::play::visibility::Visibility;
    ///
    /// let mut hand = SeatHand::new(0);
    /// assert_eq!(0, hand.len());
    /// hand.push(Card::ACE_SPADES, Visibility::Down);
    /// assert_eq!(1, hand.len());
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// True if the hand has no cards.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// assert!(SeatHand::new(0).is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Returns an iterator over the [`HoleCard`]s (preserving visibility).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    /// use pkcore::play::visibility::Visibility;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.push(Card::ACE_SPADES, Visibility::Down);
    /// hand.push(Card::KING_SPADES, Visibility::Up);
    /// assert_eq!(2, hand.iter().count());
    /// ```
    pub fn iter(&self) -> std::slice::Iter<'_, HoleCard> {
        self.cards.iter()
    }

    /// Returns the underlying slice of [`HoleCard`]s.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    /// use pkcore::play::visibility::Visibility;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.push(Card::ACE_SPADES, Visibility::Down);
    /// assert_eq!(1, hand.as_slice().len());
    /// ```
    #[must_use]
    pub fn as_slice(&self) -> &[HoleCard] {
        &self.cards
    }

    /// Sorts the hand descending by card rank (highest first), mirroring
    /// the `arrays/*` `sort_in_place` semantics. Visibility is preserved
    /// with each card.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    /// use pkcore::play::visibility::Visibility;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.push(Card::KING_SPADES, Visibility::Down);
    /// hand.push(Card::ACE_SPADES, Visibility::Down);
    /// hand.sort();
    /// assert_eq!(Card::ACE_SPADES, hand.iter().next().unwrap().card());
    /// ```
    pub fn sort(&mut self) {
        self.cards.sort_by_key(|c| std::cmp::Reverse(c.card()));
    }

    /// Returns a new `SeatHand` sorted descending, without mutating self.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    /// use pkcore::play::visibility::Visibility;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.push(Card::KING_SPADES, Visibility::Down);
    /// hand.push(Card::ACE_SPADES, Visibility::Down);
    /// let sorted = hand.sorted();
    /// assert_eq!(Card::ACE_SPADES, sorted.iter().next().unwrap().card());
    /// ```
    #[must_use]
    pub fn sorted(&self) -> Self {
        let mut copy = self.clone();
        copy.sort();
        copy
    }

    /// Appends a card with explicit visibility. The primary dealing
    /// primitive; used by the engine to grow the hand across streets.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    /// use pkcore::play::visibility::Visibility;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.push(Card::ACE_SPADES, Visibility::Up);
    /// assert!(hand.iter().next().unwrap().is_up());
    /// ```
    pub fn push(&mut self, card: Card, visibility: Visibility) {
        self.cards.push(HoleCard::new(card, visibility));
    }

    /// Appends one or more cards with `Visibility::Down`. Convenience for
    /// Hold'em/Omaha preflop dealing and Stud's 3rd-street hole cards.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.extend_down([Card::ACE_SPADES, Card::KING_SPADES]);
    /// assert_eq!(2, hand.len());
    /// assert!(hand.iter().all(|c| c.is_down()));
    /// ```
    pub fn extend_down<I>(&mut self, cards: I)
    where
        I: IntoIterator<Item = Card>,
    {
        for c in cards {
            self.push(c, Visibility::Down);
        }
    }

    /// Appends one or more cards with `Visibility::Up`. Convenience for
    /// Stud/Razz upcard dealing.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.extend_up([Card::ACE_SPADES]);
    /// assert!(hand.iter().next().unwrap().is_up());
    /// ```
    pub fn extend_up<I>(&mut self, cards: I)
    where
        I: IntoIterator<Item = Card>,
    {
        for c in cards {
            self.push(c, Visibility::Up);
        }
    }

    /// Iterator over visibility-stripped cards. Used by analysis routines
    /// that don't care about visibility.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.extend_down([Card::ACE_SPADES, Card::KING_SPADES]);
    /// let cards: Vec<Card> = hand.cards().collect();
    /// assert_eq!(2, cards.len());
    /// ```
    pub fn cards(&self) -> impl Iterator<Item = Card> + '_ {
        self.cards.iter().map(HoleCard::card)
    }

    /// Iterator over face-up cards only.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.extend_down([Card::ACE_SPADES]);
    /// hand.extend_up([Card::KING_SPADES]);
    /// let up: Vec<Card> = hand.up_cards().collect();
    /// assert_eq!(1, up.len());
    /// assert_eq!(Card::KING_SPADES, up[0]);
    /// ```
    pub fn up_cards(&self) -> impl Iterator<Item = Card> + '_ {
        self.cards.iter().filter(|hc| hc.is_up()).map(HoleCard::card)
    }

    /// Iterator over face-down cards only.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.extend_down([Card::ACE_SPADES]);
    /// hand.extend_up([Card::KING_SPADES]);
    /// let down: Vec<Card> = hand.down_cards().collect();
    /// assert_eq!(1, down.len());
    /// assert_eq!(Card::ACE_SPADES, down[0]);
    /// ```
    pub fn down_cards(&self) -> impl Iterator<Item = Card> + '_ {
        self.cards.iter().filter(|hc| hc.is_down()).map(HoleCard::card)
    }

    /// Bridge to the existing `arrays::two::Two` fixed-size type. Returns
    /// `Some` only when the hand has exactly 2 cards (NLHE / FLHE shape).
    /// Visibility is dropped — the evaluator API is visibility-blind.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.extend_down([Card::ACE_SPADES, Card::KING_SPADES]);
    /// assert!(hand.as_two().is_some());
    /// hand.extend_down([Card::QUEEN_SPADES]);
    /// assert!(hand.as_two().is_none());
    /// ```
    #[must_use]
    pub fn as_two(&self) -> Option<Two> {
        if self.cards.len() != 2 {
            return None;
        }
        Some(Two::from([self.cards[0].card(), self.cards[1].card()]))
    }

    /// Bridge to `arrays::four::Four` (PLO shape). Returns `Some` only
    /// when the hand has exactly 4 cards.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.extend_down([
    ///     Card::ACE_SPADES, Card::KING_SPADES,
    ///     Card::QUEEN_SPADES, Card::JACK_SPADES,
    /// ]);
    /// assert!(hand.as_four().is_some());
    /// ```
    #[must_use]
    pub fn as_four(&self) -> Option<Four> {
        if self.cards.len() != 4 {
            return None;
        }
        Some(Four::from([
            self.cards[0].card(),
            self.cards[1].card(),
            self.cards[2].card(),
            self.cards[3].card(),
        ]))
    }

    /// Bridge to `arrays::seven::Seven` (Stud / Razz shape). Returns
    /// `Some` only when the hand has exactly 7 cards.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.extend_down([
    ///     Card::ACE_SPADES, Card::KING_SPADES,
    ///     Card::QUEEN_SPADES, Card::JACK_SPADES,
    ///     Card::TEN_SPADES, Card::NINE_SPADES,
    ///     Card::EIGHT_SPADES,
    /// ]);
    /// assert!(hand.as_seven().is_some());
    /// ```
    #[must_use]
    pub fn as_seven(&self) -> Option<Seven> {
        if self.cards.len() != 7 {
            return None;
        }
        Some(Seven::from([
            self.cards[0].card(),
            self.cards[1].card(),
            self.cards[2].card(),
            self.cards[3].card(),
            self.cards[4].card(),
            self.cards[5].card(),
            self.cards[6].card(),
        ]))
    }

    /// Bridges to the existing [`BoxedCards`] type. Visibility is dropped.
    /// Used by hand-history serialization paths that pre-date this epic.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.extend_down([Card::ACE_SPADES, Card::KING_SPADES]);
    /// let boxed = hand.as_boxed_cards();
    /// assert_eq!(2, boxed.len());
    /// ```
    #[must_use]
    pub fn as_boxed_cards(&self) -> BoxedCards {
        let cards: Vec<Card> = self.cards.iter().map(HoleCard::card).collect();
        BoxedCards::from(cards)
    }

    /// Empties the hand. Used by the engine between hands.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.extend_down([Card::ACE_SPADES]);
    /// hand.clear();
    /// assert!(hand.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.cards.clear();
    }
}

impl Index<usize> for SeatHand {
    type Output = HoleCard;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cards[index]
    }
}

impl Display for SeatHand {
    /// Renders all cards space-separated; down cards appear in `[brackets]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::seat_hand::SeatHand;
    /// use pkcore::play::visibility::Visibility;
    ///
    /// let mut hand = SeatHand::new(0);
    /// hand.push(Card::ACE_SPADES, Visibility::Down);
    /// hand.push(Card::KING_SPADES, Visibility::Up);
    /// assert_eq!("[A♠] K♠", hand.to_string());
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = self.cards.iter().map(ToString::to_string).collect();
        write!(f, "{}", parts.join(" "))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__seat_hand__tests {
    use super::*;

    #[test]
    fn new_is_empty_at_correct_seat() {
        let hand = SeatHand::new(3);
        assert_eq!(3, hand.seat());
        assert_eq!(0, hand.len());
        assert!(hand.is_empty());
    }

    #[test]
    fn push_and_iter() {
        let mut hand = SeatHand::new(0);
        hand.push(Card::ACE_SPADES, Visibility::Down);
        hand.push(Card::KING_SPADES, Visibility::Up);
        let collected: Vec<HoleCard> = hand.iter().copied().collect();
        assert_eq!(2, collected.len());
        assert!(collected[0].is_down());
        assert!(collected[1].is_up());
    }

    #[test]
    fn extend_down_and_up() {
        let mut hand = SeatHand::new(0);
        hand.extend_down([Card::ACE_SPADES, Card::KING_SPADES]);
        hand.extend_up([Card::QUEEN_SPADES]);
        assert_eq!(3, hand.len());
        assert_eq!(2, hand.down_cards().count());
        assert_eq!(1, hand.up_cards().count());
    }

    #[test]
    fn cards_strips_visibility() {
        let mut hand = SeatHand::new(0);
        hand.extend_down([Card::ACE_SPADES]);
        hand.extend_up([Card::KING_SPADES]);
        let cards: Vec<Card> = hand.cards().collect();
        assert_eq!(vec![Card::ACE_SPADES, Card::KING_SPADES], cards);
    }

    #[test]
    fn as_two_only_at_len_2() {
        let mut hand = SeatHand::new(0);
        assert!(hand.as_two().is_none());
        hand.extend_down([Card::ACE_SPADES]);
        assert!(hand.as_two().is_none());
        hand.extend_down([Card::KING_SPADES]);
        assert!(hand.as_two().is_some());
        hand.extend_down([Card::QUEEN_SPADES]);
        assert!(hand.as_two().is_none());
    }

    #[test]
    fn as_four_only_at_len_4() {
        let mut hand = SeatHand::new(0);
        hand.extend_down([
            Card::ACE_SPADES,
            Card::KING_SPADES,
            Card::QUEEN_SPADES,
            Card::JACK_SPADES,
        ]);
        assert!(hand.as_four().is_some());
        assert!(hand.as_two().is_none());
        assert!(hand.as_seven().is_none());
    }

    #[test]
    fn as_seven_only_at_len_7() {
        let mut hand = SeatHand::new(0);
        hand.extend_down([
            Card::ACE_SPADES,
            Card::KING_SPADES,
            Card::QUEEN_SPADES,
            Card::JACK_SPADES,
            Card::TEN_SPADES,
            Card::NINE_SPADES,
            Card::EIGHT_SPADES,
        ]);
        assert!(hand.as_seven().is_some());
        assert!(hand.as_two().is_none());
        assert!(hand.as_four().is_none());
    }

    #[test]
    fn as_boxed_cards_preserves_order() {
        let mut hand = SeatHand::new(0);
        hand.extend_down([Card::ACE_SPADES, Card::KING_SPADES]);
        let boxed = hand.as_boxed_cards();
        assert_eq!(2, boxed.len());
    }

    #[test]
    fn sort_descending_preserves_visibility() {
        let mut hand = SeatHand::new(0);
        hand.push(Card::KING_SPADES, Visibility::Up);
        hand.push(Card::ACE_SPADES, Visibility::Down);
        hand.sort();
        let cards: Vec<HoleCard> = hand.iter().copied().collect();
        assert_eq!(Card::ACE_SPADES, cards[0].card());
        assert!(cards[0].is_down());
        assert_eq!(Card::KING_SPADES, cards[1].card());
        assert!(cards[1].is_up());
    }

    #[test]
    fn sorted_does_not_mutate() {
        let mut hand = SeatHand::new(0);
        hand.push(Card::KING_SPADES, Visibility::Down);
        hand.push(Card::ACE_SPADES, Visibility::Down);
        let _ = hand.sorted();
        // Original order unchanged.
        assert_eq!(Card::KING_SPADES, hand[0].card());
    }

    #[test]
    fn clear_empties_the_hand() {
        let mut hand = SeatHand::new(0);
        hand.extend_down([Card::ACE_SPADES, Card::KING_SPADES]);
        hand.clear();
        assert!(hand.is_empty());
    }

    #[test]
    fn index_access() {
        let mut hand = SeatHand::new(0);
        hand.push(Card::ACE_SPADES, Visibility::Down);
        assert_eq!(Card::ACE_SPADES, hand[0].card());
    }

    #[test]
    fn display_mixed_visibility() {
        let mut hand = SeatHand::new(0);
        hand.push(Card::ACE_SPADES, Visibility::Down);
        hand.push(Card::KING_SPADES, Visibility::Up);
        let s = hand.to_string();
        assert!(s.contains('['));
        assert!(s.contains(']'));
    }

    #[test]
    fn with_capacity_constructor() {
        let hand = SeatHand::with_capacity(2, 4);
        assert_eq!(2, hand.seat());
        assert!(hand.is_empty());
    }
}
