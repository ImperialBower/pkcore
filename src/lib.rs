#![warn(clippy::pedantic)]
#![allow(clippy::unreadable_literal)]

use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use analysis::evals::Evals;
use analysis::the_nuts::TheNuts;
use indexmap::set::IntoIter;
use itertools::Combinations;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;

use crate::suit::Suit;
use std::iter::Enumerate;

pub mod analysis;
pub mod arrays;
pub mod bard;
pub mod card;
pub mod card_number;
pub mod cards;
pub mod deck;
mod lookups;
pub mod play;
pub mod rank;
pub mod suit;
pub mod util;

// region CONSTANTS

/// See Cactus Kev's explanation of [unique vs. distinct](https://suffe.cool/poker/evaluator.html)
/// Poker hands.
/// TODO: Write on demand tests (ignore) to validate these numbers against our code.
pub const UNIQUE_STRAIGHT_FLUSHES: i32 = 40;
pub const DISTINCT_STRAIGHT_FLUSHES: i32 = 10;
pub const UNIQUE_FOUR_OF_A_KIND: i32 = 624;
pub const DISTINCT_FOUR_OF_A_KIND: i32 = 156;
pub const UNIQUE_FULL_HOUSES: i32 = 3_744;
pub const DISTINCT_FULL_HOUSES: i32 = 156;
pub const UNIQUE_FLUSH: i32 = 5_108;
pub const DISTINCT_FLUSH: i32 = 1_277;
pub const UNIQUE_STRAIGHT: i32 = 10_200;
pub const DISTINCT_STRAIGHT: i32 = 10;
pub const UNIQUE_THREE_OF_A_KIND: i32 = 54912;
pub const DISTINCT_THREE_OF_A_KIND: i32 = 858;
pub const UNIQUE_TWO_PAIR: i32 = 123_552;
pub const DISTINCT_TWO_PAIR: i32 = 858;
pub const UNIQUE_ONE_PAIR: i32 = 1_098_240;
pub const DISTINCT_ONE_PAIR: i32 = 2_860;
pub const UNIQUE_HIGH_CARD: i32 = 1_302_540;
pub const DISTINCT_HIGH_CARD: i32 = 1_277;
pub const UNIQUE_5_CARD_HANDS: usize = 2_598_960;
pub const DISTINCT_5_CARD_HANDS: usize = 7_462;
pub const POSSIBLE_UNIQUE_HOLDEM_HUP_MATCHUPS: usize = 1_624_350;

// endregion

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum PKError {
    BlankCard,
    CardCast,
    Duplicate,
    Fubar,
    Incomplete,
    InvalidBinaryFormat,
    InvalidCard,
    InvalidCardNumber,
    InvalidCardCount,
    InvalidHand,
    InvalidIndex,
    NotDealt,
    NotEnoughCards,
    NotEnoughHands,
    SqlError,
    TooManyCards,
    TooManyHands,
}

pub trait Pile {
    /// This code is cribbed from [`oli_obk`](https://stackoverflow.com/a/46766782/1245251).
    fn are_unique(&self) -> bool {
        let v = self.to_vec();
        !(1..v.len()).any(|i| v[i..].contains(&v[i - 1]))
    }

    fn bard(&self) -> Bard {
        Bard::from(self.to_vec())
    }

    fn cards(&self) -> Cards {
        Cards::from(self.to_vec())
    }

    /// Will this work? Can I create a self referential clean? Only one want to find out...
    ///
    /// *NARRATOR:* _The answer is yes._
    #[must_use]
    fn clean(&self) -> Self;

    /// If I can move logic to a trait that can be automatically reusable by other implementations
    /// that I do it. A strict TDD person could argue that you shouldn't do this unless you have
    /// a need for more than one use case that demands it. As an anti-fundamentalist, when I see
    /// these moments of beauty, I do them. It simplifies my code, and I have a good enough feel
    /// for the domain at this point to know that it will come in handy later.
    ///
    /// On the clock, you will have a lot of these programming theological debates. I generally let
    /// them win. You learn a lot trying to walk in a fundamentalist's shoes. The have a clarity of
    /// purpose that is cleansing. How can you understand when to bend the rules, if you haven't
    /// tried living with them? A lot of times, when pairing with someone who hasn't had much
    /// experience I will play by TDD
    /// [Queensbury rules](https://en.wikipedia.org/wiki/Marquess_of_Queensberry_Rules) so that they
    /// will have a good understanding of the technique. In times of darkness, test driving is one
    /// of your most trusted tools; much more important that the understanding of any specific
    /// programming language.
    ///
    /// **Breakdown strict TDD**
    fn combinations_after(&self, k: usize, cards: &Cards) -> Combinations<IntoIter<Card>> {
        log::debug!("Pile.combinations_after(k: {} cards: {})", k, cards);
        self.remaining_after(cards).combinations(k)
    }

    fn combinations_remaining(&self, k: usize) -> Combinations<IntoIter<Card>> {
        log::debug!("Pile.combinations_after(k: {})", k);
        self.remaining().combinations(k)
    }

    fn contains(&self, card: &Card) -> bool {
        self.to_vec().contains(card)
    }

    fn contains_blank(&self) -> bool {
        self.contains(&Card::BLANK)
    }

    fn enumerate_after(&self, k: usize, cards: &Cards) -> Enumerate<Combinations<IntoIter<Card>>> {
        log::info!("Pile.enumerate_after(k: {} cards: {})", k, cards);
        self.remaining_after(cards).combinations(k).enumerate()
    }

    fn enumerate_remaining(&self, k: usize) -> Enumerate<Combinations<IntoIter<Card>>> {
        log::info!("Pile.enumerate_after(k: {})", k);
        self.combinations_remaining(k).enumerate()
    }

    /// This feels like the best name for this functionality. If a `Pile` doesn't contain
    /// a blank card, and all of the cards are unique, that it has been dealt.
    fn is_dealt(&self) -> bool {
        self.are_unique() && !self.contains_blank()
    }

    fn remaining(&self) -> Cards {
        log::debug!("Pile.remaining()");
        Cards::deck_minus(&self.cards())
    }

    fn remaining_after(&self, cards: &Cards) -> Cards {
        log::debug!("Pile.remaining_after(cards: {})", cards);
        let mut held = self.cards();
        held.insert_all(cards);
        Cards::deck_minus(&held)
    }

    fn suits(&self) -> HashSet<Suit> {
        self.to_vec()
            .iter()
            .map(card::Card::get_suit)
            .collect::<HashSet<Suit>>()
    }

    fn the_nuts(&self) -> TheNuts;

    fn evals(&self) -> Evals {
        self.the_nuts().to_evals()
    }

    fn to_bard(&self) -> Bard {
        Bard::from(self.cards())
    }

    fn to_vec(&self) -> Vec<Card>;
}

// https://en.wikipedia.org/wiki/Se%C3%B1or_Wences#Catchphrases
/// The more I think about this, the more I feel like this is me avoiding the best practice
/// of returning `Result` and `Option`. I'm worried about speed, but that's probably Knuth's
/// dreaded [premature optimization](http://wiki.c2.com/?PrematureOptimization).
pub trait SOK {
    fn salright(&self) -> bool;
}

/// Spades to Hearts to Diamonds to Clubs.
pub trait SuitShift {
    #[must_use]
    fn shift_suit_down(&self) -> Self;

    #[must_use]
    fn shift_suit_up(&self) -> Self;

    /// I don't trust this concept. Up and down are straightforward, but not this
    /// I need to do a deep dive into unique and distinct patterns.
    #[must_use]
    fn opposite(&self) -> Self;
}

pub trait Shifty: SuitShift + Copy {
    #[must_use]
    fn other_shifts(&self) -> HashSet<Self>
    where
        Self: Sized,
        Self: std::cmp::Eq,
        Self: Hash,
        Self: std::fmt::Display,
    {
        let mut hs = HashSet::new();
        let original = *self;
        let mut shifted = *self;
        // Tbe original version of this section has a flaw. It adds itself back if there is a gap. We
        // Need to fix that.
        //
        // ```
        // for _ in 1..=3 {
        //   shifty = shifty.shift_suit_up();
        //   hs.insert(shifty);
        // }
        // ````
        for _ in 1..=3 {
            shifted = shifted.shift_suit_up();
            if shifted != original {
                hs.insert(shifted);
            }
        }

        hs
    }

    #[must_use]
    fn shifts(&self) -> HashSet<Self>
    where
        Self: Sized,
        Self: std::cmp::Eq,
        Self: Hash,
        Self: std::fmt::Display,
    {
        let mut hs = HashSet::new();
        let shifty = *self;
        hs.insert(shifty);
        hs.extend(self.other_shifts());
        hs
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
