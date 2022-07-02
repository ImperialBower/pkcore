#![warn(clippy::pedantic)]
#![allow(clippy::unreadable_literal)]

use crate::card::Card;
use crate::cards::Cards;
use crate::hand_rank::the_nuts::TheNuts;
use indexmap::set::IntoIter;
use itertools::Combinations;
use std::iter::Enumerate;

pub mod analysis;
pub mod arrays;
pub mod card;
pub mod card_number;
pub mod cards;
pub mod hand_rank;
mod lookups;
pub mod play;
pub mod rank;
pub mod suit;
pub mod util;

// region CONSTANTS

pub const UNIQUE_5_CARD_HANDS: usize = 2_598_960;

// endregion

#[derive(Debug, Eq, PartialEq)]
pub enum PKError {
    BlankCard,
    DuplicateCard,
    Fubar,
    Incomplete,
    InvalidBinaryFormat,
    InvalidCard,
    InvalidCardNumber,
    InvalidCardCount,
    InvalidIndex,
    NotEnoughCards,
    TooManyCards,
}

pub trait Pile {
    /// This code is cribbed from [`oli_obk`](https://stackoverflow.com/a/46766782/1245251).
    fn are_unique(&self) -> bool {
        let v = self.to_vec();
        !(1..v.len()).any(|i| v[i..].contains(&v[i - 1]))
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

    fn contains(&self, card: &Card) -> bool {
        self.to_vec().contains(card)
    }

    fn contains_blank(&self) -> bool {
        self.contains(&Card::BLANK)
    }

    fn enumerate_after(&self, k: usize, cards: &Cards) -> Enumerate<Combinations<IntoIter<Card>>> {
        log::debug!("Pile.enumerate_after(k: {} cards: {})", k, cards);
        self.remaining_after(cards).combinations(k).enumerate()
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
        held.add(cards);
        Cards::deck_minus(&held)
    }

    fn the_nuts(&self) -> TheNuts;

    fn to_vec(&self) -> Vec<Card>;
}

// https://en.wikipedia.org/wiki/Se%C3%B1or_Wences#Catchphrases
/// The more I think about this, the more I feel like this is me avoiding the best practice
/// of returning `Result` and `Option`. I'm worried about speed, but that's probably Knuth's
/// dreaded [premature optimization](http://wiki.c2.com/?PrematureOptimization).
pub trait SOK {
    fn salright(&self) -> bool;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
