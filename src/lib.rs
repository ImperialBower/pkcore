#![warn(clippy::pedantic)]
#![allow(
    clippy::unreadable_literal,
    clippy::iter_without_into_iter,
    clippy::should_implement_trait
)]

extern crate core;

use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use analysis::evals::Evals;
use analysis::the_nuts::TheNuts;
use indexmap::set::IntoIter;
use itertools::Combinations;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::Hash;

use crate::casino::cashier::chips::Chips;
use crate::suit::Suit;
use rayon::iter::IterBridge;
use std::iter::Enumerate;

pub mod analysis;
pub mod arrays;
pub mod bard;
pub mod card;
pub mod card_number;
pub mod cards;
pub mod casino;
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
pub const DISTINCT_2_CARD_HANDS: usize = 1_326;
pub const DISTINCT_SUITED_2_CARD_HANDS: usize = 312;
pub const NON_EQUIVALENT_2_CARD_HANDS: usize = 169;
pub const UNIQUE_5_CARD_HANDS: usize = 2_598_960;
pub const DISTINCT_5_CARD_HANDS: usize = 7_462;
pub const POSSIBLE_UNIQUE_HOLDEM_HUP_MATCHUPS: usize = 1_624_350;

// endregion

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum PKError {
    AlreadyDealt,
    BlankCard,
    Busted,
    CardCast,
    DBConnectionError,
    Duplicate,
    Fubar,
    Incomplete,
    InsufficientChips,
    InvalidBinaryFormat,
    InvalidCard,
    InvalidCardNumber,
    InvalidCardCount,
    InvalidHand,
    InvalidIndex,
    InvalidPluribusIndex,
    InvalidPosition,
    NotDealt,
    NotEnoughCards,
    NotEnoughHands,
    PlayerOutOfHand,
    SqlError,
    TooManyCards,
    TooManyHands,
}

pub trait Betting {
    /// # Errors
    ///
    /// Returns `PKError::Busted` if there are no chips.
    fn all_in(&mut self) -> Result<Chips, PKError>;

    /// # Errors
    ///
    /// Returns `PKError::InsufficientChips` if there are insufficient chips.
    fn bet(&mut self, amount: usize) -> Result<Chips, PKError>;

    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    fn size(&self) -> usize;

    /// Adds the amount of Chips won to the stack. Returns the resulting stack size.
    fn wins(&mut self, winnings: Chips) -> usize;
}

/// The name of this trait is a pun om pluribus, which is the name of the poker AI group.
pub trait Plurable {
    /// Converts a part of the Pluribus log format
    ///
    /// # Errors
    ///
    /// Throws a `PKError` if the string isn't formatted correctly or the length isn't correct.
    fn from_pluribus(s: &str) -> Result<Self, PKError>
    where
        Self: Sized;
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

    fn par_combinations_remaining(&self, k: usize) -> IterBridge<Combinations<IntoIter<Card>>> {
        log::debug!("Pile.combinations_after(k: {})", k);
        self.remaining().par_combinations(k)
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

pub trait Shifty {
    #[must_use]
    fn is_shift(&self, other: Box<Self>) -> bool
    where
        Self: Sized,
        Self: Eq,
        Self: Hash,
    {
        self.shifts().contains(other.borrow())
    }

    /// ```txt
    /// #[must_use]
    ///     fn other_shifts(&self) -> HashSet<Self>
    ///     where
    ///         Self: Sized,
    ///         Self: Eq,
    ///         Self: Hash,
    ///         Self: std::fmt::Display,
    ///     {
    ///         let mut hs = HashSet::new();
    ///         let original = *self;
    ///         let mut shifted = *self;
    ///         /// Tbe original version of this section has a flaw. It adds itself back if there is a gap. We
    ///         /// Need to fix that.
    ///         //
    ///         /// ```
    ///         /// for _ in 1..=3 {
    ///         ///   shifty = shifty.shift_suit_up();
    ///         ///   hs.insert(shifty);
    ///         /// }
    ///         /// ````
    ///         for _ in 1..=3 {
    ///             shifted = shifted.shift_suit_up();
    ///             if shifted != original {
    ///                 hs.insert(shifted);
    ///             }
    ///         }
    ///
    ///         hs
    ///     }
    /// ```
    #[must_use]
    fn other_shifts(&self) -> HashSet<Self>
    where
        Self: Sized,
        Self: Eq,
        Self: Hash,
        Self: std::fmt::Display,
    {
        let mut shifts = self.shifts();
        shifts.remove(self);
        shifts
    }

    /// Returns a `HashSet` of the possible suit shifts. I'm thinking that I want to add this to the
    /// `SuitShift` trait. This would require that the trait would need Copy as a
    /// [supertrait](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-supertraits-to-require-one-traits-functionality-within-another-trait).
    ///
    /// I've never used a supertrait before. This should be fun.
    ///
    /// Firs, let's implement it on `SuitShift` without changing anything, and then we'll see if
    /// we can make this method apply to any struct that implements the trait.
    ///
    /// Adding the supertrait was easy:
    ///
    /// ```txt
    /// pub trait SuitShift: Copy {
    ///     #[must_use]
    ///     fn shift_suit_down(&self) -> Self;
    ///
    ///     #[must_use]
    ///     fn shift_suit_up(&self) -> Self;
    ///
    ///     #[must_use]
    ///     fn opposite(&self) -> Self;
    /// }
    /// ```
    ///
    /// But that won't work:
    ///
    /// ```txt
    /// error[E0277]: the trait bound `Cards: std::marker::Copy` is not satisfied
    ///    --> src/cards.rs:640:20
    ///     |
    /// 640 | impl SuitShift for Cards {
    ///     |                    ^^^^^ the trait `std::marker::Copy` is not implemented for `Cards`
    ///     |
    /// note: required by a bound in `SuitShift`
    ///    --> src/lib.rs:183:22
    ///     |
    /// 183 | pub trait SuitShift: Copy {
    ///     |                      ^^^^ required by this bound in `SuitShift`
    /// ```
    ///
    /// `Cards` doesn't implement `Copy`, and since it's an `IndexSet`, it isn't going to. Back to
    /// the drawing board.
    ///
    /// How about we create a trait called `Shifty`, and make `SuitShift` its supertrait? Something like:
    ///
    /// ```txt
    /// use std::collections::HashSet;
    /// use pkcore::SuitShift;
    /// pub trait Shifty: SuitShift {
    ///     #[must_use]
    ///     fn shifts(&self) -> HashSet<Self>;
    /// }
    /// ```
    ///
    /// Nope. Strike two!
    ///
    /// ```txt
    /// error[E0277]: the size for values of type `Self` cannot be known at compilation time
    ///    --> src/arrays/matchups/sorted_heads_up.rs:151:25
    ///     |
    /// 9   |     fn shifts(&self) -> HashSet<Self>;
    ///     |                         ^^^^^^^^^^^^^ doesn't have a size known at compile-time
    ///     |
    /// note: required by a bound in `HashSet`
    ///    --> ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/set.rs:106:20
    ///     |
    /// 106 | pub struct HashSet<T, S = RandomState> {
    ///     |                    ^ required by this bound in `HashSet`
    /// help: consider further restricting `Self`
    ///     |
    /// 9   |     fn shifts(&self) -> HashSet<Self> where Self: Sized;
    ///     |                                       +++++++++++++++++
    ///
    /// error: aborting due to previous error
    /// ```
    ///
    ///  Wonder if its recommendations will work?
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use pkcore::SuitShift;
    /// pub trait Shifty: SuitShift {
    ///     #[must_use]
    ///     fn shifts(&self) -> HashSet<Self> where Self: Sized;
    /// }
    /// ```
    ///
    /// 💥! That compiles! But... will it actually work?
    ///
    /// First, we'll need to rewrite shifts into the trait, and then swap it out inside here.
    ///
    /// ```txt
    /// use std::collections::HashSet;
    /// use pkcore::SuitShift;
    /// pub trait Shifty: SuitShift {
    ///     #[must_use]
    ///     fn shifts(&self) -> HashSet<Self> where Self: Sized {
    ///         let mut hs = HashSet::new();
    ///         let mut shifty = *self;
    ///         hs.insert(shifty);
    ///         for _ in 1..=3 {
    ///             shifty = shifty.shift_suit_up();
    ///             hs.insert(shifty);
    ///         }
    ///
    ///         hs
    ///     }
    /// }
    /// ```
    ///
    /// Nope... but we're getting closer...
    ///
    /// ```txt
    /// error[E0277]: the trait bound `Self: std::cmp::Eq` is not satisfied
    ///    --> src/lib.rs:200:12
    ///     |
    /// 200 |         hs.insert(shifty);
    ///     |            ^^^^^^ the trait `std::cmp::Eq` is not implemented for `Self`
    ///     |
    /// note: required by a bound in `HashSet::<T, S>::insert`
    ///    --> ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/set.rs:428:8
    ///     |
    /// 428 |     T: Eq + Hash,
    ///     |        ^^ required by this bound in `HashSet::<T, S>::insert`
    /// ...
    /// 887 |     pub fn insert(&mut self, value: T) -> bool {
    ///     |            ------ required by a bound in this associated function
    /// help: consider further restricting `Self`
    /// ```
    ///
    /// Adding that we get
    ///
    /// ```txt
    /// error[E0277]: the trait bound `Self: Hash` is not satisfied
    ///    --> src/lib.rs:200:12
    ///     |
    /// 200 |         hs.insert(shifty);
    ///     |            ^^^^^^ the trait `Hash` is not implemented for `Self`
    ///     |
    /// note: required by a bound in `HashSet::<T, S>::insert`
    ///    --> ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/set.rs:428:13
    ///     |
    /// 428 |     T: Eq + Hash,
    ///     |             ^^^^ required by this bound in `HashSet::<T, S>::insert`
    /// ...
    /// 887 |     pub fn insert(&mut self, value: T) -> bool {
    ///     |            ------ required by a bound in this associated function
    /// help: consider further restricting `Self`
    ///     |
    /// 197 |     fn shifts(&self) -> HashSet<Self> where Self: Sized, Self: std::cmp::Eq, Self: Hash {
    ///     |                                                                            ++++++++++++
    /// ```
    ///
    /// Still no.
    ///
    /// ```txt
    /// error[E0507]: cannot move out of `*self` which is behind a shared reference
    ///    --> src/lib.rs:200:26
    ///     |
    /// 200 |         let mut shifty = *self;
    ///     |                          ^^^^^ move occurs because `*self` has type `Self`, which does not implement the `Copy` trait
    ///     |
    /// help: consider removing the dereference here
    ///     |
    /// 200 -         let mut shifty = *self;
    /// 200 +         let mut shifty = self;
    /// ```
    ///
    /// So let's add the Copy trait as a supertrait of `Shifty`.
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use std::hash::Hash;
    /// use pkcore::SuitShift;
    /// pub trait Shifty: SuitShift + Copy {
    ///     #[must_use]
    ///     fn shifts(&self) -> HashSet<Self> where Self: Sized, Self: std::cmp::Eq, Self: Hash {
    ///         let mut hs = HashSet::new();
    ///         let mut shifty = *self;
    ///         hs.insert(shifty);
    ///         for _ in 1..=3 {
    ///             shifty = shifty.shift_suit_up();
    ///             hs.insert(shifty);
    ///         }
    ///
    ///         hs
    ///     }
    /// }
    /// ```
    ///
    /// 💥💥💥! It compiles! We're back in business. Still, we don't know if it will actually work.
    /// Let's swap out the function for the trait and see what transpires, shall we.
    ///
    /// ```txt
    /// error[E0599]: no method named `shifts` found for reference `&SortedHeadsUp` in the current scope
    ///    --> examples/hup.rs:277:34
    ///     |
    /// 277 |         let possible_sorts = shu.shifts();
    ///     |                                  ^^^^^^
    ///     |
    ///     = help: items from traits can only be used if the trait is in scope
    /// help: the following trait is implemented but not in scope; perhaps add a `use` for it:
    ///     |
    /// 1   + use pkcore::Shifty;
    ///     |
    /// help: there is a method with a similar name
    ///     |
    /// 277 |         let possible_sorts = shu.old_shifts();
    ///     |                                  ~~~~~~~~~~
    /// ```
    ///
    /// Gonna need to import the trait for the code that was using our `shifts()` method.
    ///
    /// Tests pass... let's see if `examples/hup.rs` still does its magic.
    ///
    /// Still works, although to be fair we've never ran it through an entire run. I'm going to
    /// check it out in a different location from this point and let hup run to see what happens
    /// when we get all the way to the end. At the same time I'm going to refactor the code so
    /// that it works on a smaller sample so we can get faster feedback.
    ///
    /// Here's the original version of the function for reference:
    ///
    /// ```txt
    ///     #[must_use]
    ///     pub fn shifts(&self) -> HashSet<Self> {
    ///         let mut v = HashSet::new();
    ///         let mut shifty = *self;
    ///         v.insert(shifty);
    ///
    ///         for _ in 1..=3 {
    ///             shifty = shifty.shift_suit_up();
    ///             v.insert(shifty);
    ///         }
    ///
    ///         v
    ///     }
    /// ```
    ///
    /// ## UPDATE: Type X DEFECT
    ///
    /// We're going to retire all of the trait implementations. They are based on
    /// flawed logic, that simply rotating the suits will return all the shifts.
    ///
    /// ```txt
    /// #[must_use]
    ///     fn shifts(&self) -> HashSet<Self>
    ///     where
    ///         Self: Sized,
    ///         Self: Eq,
    ///         Self: Hash,
    ///         Self: std::fmt::Display,
    ///     {
    ///         let mut hs = HashSet::new();
    ///         let shifty = *self;
    ///         hs.insert(shifty);
    ///         hs.extend(self.other_shifts());
    ///         hs
    ///     }
    /// ```
    fn shifts(&self) -> HashSet<Self>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
