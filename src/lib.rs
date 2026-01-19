#![warn(clippy::pedantic, clippy::unwrap_used, clippy::expect_used)]
#![allow(
    non_upper_case_globals,
    clippy::unreadable_literal,
    clippy::iter_without_into_iter,
    clippy::module_inception,
    clippy::should_implement_trait,
    clippy::upper_case_acronyms,
    macro_expanded_macro_exports_accessed_by_absolute_paths
)]

//! # pkcore
//!
//! A comprehensive poker library for Texas Hold'em analysis, evaluation, and game simulation.
//!
//! ## Overview
//!
//! `pkcore` is a high-performance Rust library designed for serious poker analysis and game theory applications.
//! It provides tools for:
//! - Card and deck manipulation with efficient bit representations
//! - Hand evaluation (5-card, 7-card, and 8-or-better low hands)
//! - Heads-up preflop equity calculations with `SQLite` persistence
//! - Game theory optimal (GTO) range analysis and combo breakdowns
//! - Full game simulation with multi-round betting and pot management
//! - Suit-shift analysis for distinct hand pattern recognition
//!
//! ## Core Types
//!
//! ### Cards and Ranks
//!
//! - [`card::Card`] - A single playing card with rank and suit
//!   - Represented internally as a `u8` for efficient bitwise operations
//!   - Supports parsing from strings like "As" and "a♠" for (Ace of Spades) or "2h" for (Two of Hearts)
//!
//! - [`cards::Cards`] - An ordered, deduplicated collection of cards (hands, boards, etc.)
//!   - Built on `IndexSet` for O(1) lookups and preservation of insertion order
//!   - Implements [`Pile`] trait for common card collection operations
//!
//! - [`deck::Deck`] - A shuffleable 52-card deck
//!   - Supports drawing cards and checking remaining inventory
//!
//! - [`rank::Rank`] and [`suit::Suit`] - Card components
//!   - [`rank::Rank`]: Ace through King with utility methods for comparisons and bit patterns
//!   - [`suit::Suit`]: Spades, Hearts, Diamonds, Clubs with shift operations for distinct analysis

//! ## Analysis Module
//!
//! The [`analysis`] module provides sophisticated poker analysis tools:
//!
//! ### Hand Evaluation
//!
//! - [`analysis::evals::Evals`] - Complete hand evaluation results
//!   - 5-card hand rankings with Cactus Kev's evaluator
//!   - 7-card hand analysis (Texas Hold'em)
//!   - 8-or-better low hand qualification
//!   - Hand classifications (pair, two pair, trips, straight, flush, etc.)
//!
//! - [`analysis::the_nuts::TheNuts`] - Nut hand calculations
//!   - Determines the strongest possible hand given community cards
//!   - Supports both high and low hand analysis
//!
//! ### Preflop Equity
//!
//! - [`analysis::store::db::hup::HUPResult`] - Heads-up preflop equity results
//!   - Precomputed matchups between any two starting hands
//!   - Ability to compute odds against more than two players in a hand
//!   - `SQLite` persistence with the `Sqlable` trait
//!   - Configurable database path via `HUPS_DB_PATH` environment variable
//!   - Supports split pot scenarios
//!   - Efficient querying and bulk insertion
//!
//! ### GTO Analysis (WIP)
//!
//! - [`analysis::gto::combo::Combo`] - Individual hand combinations
//! - [`analysis::gto::combo_pairs::ComboPairs`] - Grouped combinations by rank pattern
//! - [`analysis::gto::twos::Twos`] - Two-card hand explosions for range analysis
//! - Range analysis with combo weighting and equity breakdowns
//! - See `examples/gto.rs` for a practical demonstration
//!
//! ## Game Simulation Module (WIP)
//!
//! The [`games`] module provides complete game simulation infrastructure:
//!
//! - **Table Management**: Multi-seat tables with position tracking
//! - **Betting Rounds**: Preflop, Flop, Turn, River with action tracking
//! - **Pot Calculation**: Side pots, main pots, and all-in scenarios
//! - **Player State**: Active, folded, all-in, or busted statuses
//! - **Action History**: Complete hand history tracking
//!
//! ## Key Traits
//!
//! ### Card Collection Operations
//!
//! - [`Pile`] - Common operations for card collections
//!   - Card containment checks
//!   - Combination generation (remaining cards, enumeration)
//!   - Rank and suit extraction
//!   - Uniqueness validation
//!   - Hand evaluation delegation
//!
//! ### Game State and Actions
//!
//! - [`Agency`] - Player action permissions based on game state
//!   - `can_act()` - Can player act at all?
//!   - `can_given()` - Can player do action X given previous action Y?
//!   - `can_given_against()` - Can player respond to opponent's action?
//!
//! - [`Betting`] - Chip management and wagering
//!   - `all_in()` - Wager all remaining chips
//!   - `bet()` - Wager specific amount
//!   - `wins()` - Collect winnings
//!   - Stack size queries
//!
//! ### Hand and Range Analysis
//!
//! - [`GTO`] - Range explosion and combo analysis
//!   - `explode()` - Convert range to all two-card combinations
//!   - `combo_pairs()` - Group combos by rank pattern
//!
//! ### Suit Permutation Analysis
//!
//! - [`SuitShift`] - Suit rotation operations
//!   - `shift_suit_up()` / `shift_suit_down()` - Rotate suits
//!   - `opposite()` - Find suit-shifted equivalent
//!   - Essential for analyzing distinct hand patterns
//!
//! - [`Shifty`] - Comprehensive shift analysis
//!   - `shifts()` - Generate all suit-shifted variants
//!   - `other_shifts()` - Get non-identity shifts
//!   - `is_shift()` - Check if two hands are suit-shifted versions
//!
//! ### Utility Traits
//!
//! - [`Forgiving`] - Graceful parsing with sensible defaults
//!   - `forgiving_from_str()` - Parse with fallback to default
//!   - Reduces error handling boilerplate
//!
//! - [`Plurable`] -  [Pluribus](https://en.wikipedia.org/wiki/Pluribus_(poker_bot)) AI log format parsing
//!   - `from_pluribus()` - Parse hand notation from Pluribus logs
//!
//! - [`SOK`] - Validation checks
//!   - `salright()` - Is this entity in a valid state?
//!
//! ## Constants and Metrics
//!
//! The crate defines comprehensive constants for poker mathematics,
//! based on [Cactus Kev's evaluator](https://suffe.cool/poker/evaluator.html):
//!
//! ### Hand Classification Counts
//!
//! | Hand Type | Unique | Distinct |
//! |-----------|--------|----------|
//! | Straight Flushes | 40 | 10 |
//! | Four of a Kind | 624 | 156 |
//! | Full Houses | 3,744 | 156 |
//! | Flushes | 5,108 | 1,277 |
//! | Straights | 10,200 | 10 |
//! | Three of a Kind | 54,912 | 858 |
//! | Two Pair | 123,552 | 858 |
//! | One Pair | 1,098,240 | 2,860 |
//! | High Card | 1,302,540 | 1,277 |
//!
//! ### Hold'em-Specific Counts
//!
//! - [`UNIQUE_5_CARD_HANDS`] = 2,598,960 - All possible 5-card hands
//! - [`DISTINCT_5_CARD_HANDS`] = 7,462 - Unique hand strengths
//! - [`UNIQUE_2_CARD_HANDS`] = 1,326 - All starting hand combinations
//! - [`DISTINCT_2_CARD_HANDS`] = 169 - Distinct starting hand patterns
//! - [`POSSIBLE_UNIQUE_HOLDEM_HUP_MATCHUPS`] = 1,624,350 - Heads-up preflop scenarios
//!
//! ### Starting Hand Metrics
//!
//! - [`UNIQUE_POCKET_PAIRS`] = 78 - All pocket pair combinations
//! - [`UNIQUE_SUITED_2_CARD_HANDS`] = 312 - All suited combinations
//! - [`UNIQUE_PER_RANK_2_CARD_HANDS`] = 198 - Combinations per specific rank
//! - [`UNIQUE_PER_SUIT_2_CARD_HANDS`] = 585 - Combinations per specific suit
//!
//! ## Error Handling
//!
//! The library uses a comprehensive [`PKError`] enum for all error conditions:
//!
//! - **Card Errors**: `BlankCard`, `InvalidCard`, `DuplicateCard`
//! - **Action Errors**: `InvalidAction`, `ActionIsntFinished`
//! - **Betting Errors**: `InsufficientChips`, `Busted`
//! - **Data Errors**: `NotDealt`, `AlreadyDealt`, `TooManyCards`
//! - **System Errors**: `DBConnectionError`, `SqlError`
//! - **Parsing Errors**: `InvalidCardNumber`, `InvalidRangeIndex`
//!
//! All errors implement `std::error::Error` and can be converted from `rusqlite::Error`.
//!
//! ## Database Integration
//!
//! Precomputed heads-up results can be stored in `SQLite`:
//!
//! - **Type**: [`analysis::store::db::hup::HUPResult`]
//! - **Trait**: Implements `Sqlable` for insert/query operations
//! - **Configuration**: Set `HUPS_DB_PATH` environment variable (default: `generated/hups.db`)
//! - **Persistence**: Efficient bulk loading and querying
//! - **Features**: Split pot tracking, win/loss counts
//!
//! Add to `.env`:
//! ```env
//! HUPS_DB_PATH=generated/hups.db
//! ```
//!
//! ## Examples
//!
//! ### Parsing and Evaluating a Hand at the Turn:
//!
//! `cargo run --example simple_eval_example`
//!
//! ```rust
//! use pkcore::prelude::*;
//!
//! // 1st player has A♠ K♥ while 2nd player has 8♦ K♣
//! let hands = HoleCards::from_str("A♠ KH 8♦ K♣").unwrap();
//! let board = Board::from_str("A♣ 8♥ 7♥ 9♠").unwrap();
//! let game = Game::new(hands, board);
//!
//! let case_evals = game.turn_case_evals();
//! let outs = Outs::from(&case_evals);
//!
//!
//! let player1_outs = outs.get(1).unwrap();
//! let player2_outs = outs.get(2).unwrap();
//!
//! // Show the outs for each player
//! println!("Player #1 has {} outs: {}", player1_outs.len(), player1_outs);
//! assert_eq!("K♠ Q♠ J♠ T♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ Q♥ J♥ T♥ 9♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ Q♣ J♣ T♣ 9♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣", player1_outs.to_string());
//! println!("Player #2 has {} outs: {}", player2_outs.len(), player2_outs);
//! assert_eq!("8♠ 8♣", player2_outs.to_string());
//!
//! // Show each players odds of winning against every possible card dealt as well
//! // as their best hand at the turn.
//! game.turn_display_odds().expect("TurnDisplayOdds");
//! ```
//!
//! ### Showing Remaining Card Combinations
//!
//! `cargo run --example simple_collections_example`
//!
//! ```rust
//! use pkcore::prelude::*;
//!
//! let hand: Cards = "As Ks".parse().unwrap();
//! let board: Cards = "Qs Js Ts".parse().unwrap();
//!
//! // Get remaining cards
//! let remaining = hand.remaining_after(&board);
//! println!("Cards left in deck: {}", remaining.len());
//! assert_eq!(47, remaining.len());
//!
//! // Generate combinations
//! for combo in hand.combinations_remaining(2) {
//!     println!("Possible combo: {}", Cards::from(combo));
//! }
//! ```
//!
//! ### Suit-Shift Analysis
//!
//! In order to speed up the brute force analysis of every possible result of two hands playing
//! against each other, I came up with the idea of suit-shifting.
//!
//! See `docs/EPIC-07_Transposition.md` for a detailed description of concept and rationale behind
//! suit shifting.
//!
//! `cargo run --example simple_suit_shift_example`
//!
//! ```rust
//! use pkcore::prelude::*;
//!
//! // Create a heads-up example where player 1 has A♠ K♠ and player 2 has A♥ K♥
//! let hand: SortedHeadsUp = "As Ks Ah kh".parse().unwrap();
//!
//! // Find all suit-shifted variants
//! let all_shifts = hand.shifts();
//! println!("Total variants: {}", all_shifts.len());
//! assert_eq!(6, all_shifts.len());
//!
//! for variant in all_shifts {
//!     println!("{}", variant);
//! }
//! ```
//!
//! returns:
//!
//! ```shell
//! Total variants: 6
//! A♠ K♠ - A♥ K♥
//! A♠ K♠ - A♦ K♦
//! A♥ K♥ - A♦ K♦
//! A♠ K♠ - A♣ K♣
//! A♦ K♦ - A♣ K♣
//! A♥ K♥ - A♣ K♣
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Card Operations**: O(1) lookups via bit representation
//! - **Hand Evaluation**: ~50ns per 5-card hand (Cactus Kev algorithm)
//! - **Combination Generation**: Lazy iteration without pre-allocation
//! - **Memory**: Minimal card representation (~4 bytes per card)
//! - **Database**: Pre-computed results enable instant lookups
//!
//! ## Compiler Features
//!
//! The crate uses Clippy's pedantic checking and forbids unsafe unwrap patterns:
//! - `#![warn(clippy::pedantic)]` - Strict code quality checks
//! - `#![warn(clippy::unwrap_used)]` - Unsafe unwrap detection
//! - `#![warn(clippy::expect_used)]` - Unsafe expect detection
//!
//! Several allowances are configured for practical implementation:
//! - `non_upper_case_globals` - Library constants use camelCase
//! - `clippy::upper_case_acronyms` - Pragmatic acronym naming
//! - Other pragmatic exceptions for macro expansions and inheritance patterns
//!
//! ## Philosophy
//!
//! `pkcore` is built on principles of:
//! - **Correctness**: Comprehensive error handling and validation
//! - **Performance**: Bit-level optimizations where appropriate
//! - **Clarity**: Descriptive naming and extensive documentation
//! - **Testability**: Extensive testing, with strict GitHub actions validation

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
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::combo_pairs::ComboPairs;
use crate::analysis::gto::twos::Twos;
use crate::prelude::PlayerState;
use crate::rank::Rank;
use crate::ranks::Ranks;
use crate::suit::Suit;
use rayon::iter::IterBridge;
use std::iter::Enumerate;
use std::str::FromStr;

#[macro_use]
pub mod macros;
pub mod analysis;
pub mod arrays;
pub mod bard;
pub mod card;
pub mod card_number;
pub mod cards;
pub mod cards_cell;
pub mod casino;
pub mod deck;
pub mod games;
mod lookups;
pub mod play;
pub mod prelude;
pub mod rank;
pub mod ranks;
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

pub const UNIQUE_2_CARD_HANDS: usize = 1_326;
pub const UNIQUE_SUITED_2_CARD_HANDS: usize = 312;
pub const UNIQUE_PER_RANK_2_CARD_HANDS: usize = 198; // 6 + (16 x 12) = 198
pub const DISTINCT_PER_RANK_2_CARD_HANDS: usize = 25; // 1 + (2 x 12) = 25

pub const UNIQUE_POCKET_PAIRS: usize = 78; // 13 x 6 = 78
pub const UNIQUE_NON_POCKET_PAIRS: usize = 1_248; // 13 x 6 = 78

pub const UNIQUE_PER_SUIT_2_CARD_HANDS: usize = 585; // TODO: Need to validate

pub const UNIQUE_PER_CARD_2_CARD_HANDS: usize = 198; // 6 + (16 x 12) = 198

pub const DISTINCT_2_CARD_HANDS: usize = 169;

pub const UNIQUE_5_CARD_HANDS: usize = 2_598_960;
pub const DISTINCT_5_CARD_HANDS: usize = 7_462;
pub const POSSIBLE_UNIQUE_HOLDEM_HUP_MATCHUPS: usize = 1_624_350;

// endregion

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub enum PKError {
    ActionIsntFinished,
    AlreadyDealt,
    BlankCard,
    Busted,
    CardNotFound,
    CardCast,
    DBConnectionError,
    DuplicateCard,
    DuplicateAction,
    Fubar,
    Incomplete,
    InvalidAction,
    InsufficientChips,
    InvalidBinaryFormat,
    InvalidCard,
    InvalidCardNumber,
    InvalidCardCount,
    InvalidComboIndex,
    InvalidHand,
    InvalidCardIndex,
    InvalidLength,
    InvalidPermutationIndex,
    InvalidPluribusIndex,
    InvalidPosition,
    InvalidRangeIndex,
    InvalidRankIndex,
    InvalidSeatNumber,
    InvalidShift,
    InvalidTableAction,
    Misaligned,
    NoBlankSlots,
    NoLow,
    NotDealt,
    NotEnoughCards,
    NotEnoughHands,
    PlayerOutOfHand,
    SqlError,
    TableFull,
    TooManyCards,
    TooManyHands,
    InvalidTwo,
}

impl Display for PKError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            PKError::ActionIsntFinished => "Action isn't finished Error",
            PKError::AlreadyDealt => "Already dealt Error",
            PKError::BlankCard => "Blank Card Error",
            PKError::Busted => "Player is out of chips",
            PKError::CardCast => "Card Cast Error",
            PKError::CardNotFound => "Card Not Found Error",
            PKError::DBConnectionError => "Unable to connect to DB",
            PKError::DuplicateCard => "Duplicate Card Error",
            PKError::DuplicateAction => "Duplicate Action Error",
            PKError::Fubar => "Unexpected Error",
            PKError::Incomplete => "Incomplete Error",
            PKError::InsufficientChips => "Insufficient chips Error",
            PKError::InvalidAction => "Invalid Action Error",
            PKError::InvalidBinaryFormat => "Invalid binary format Error",
            PKError::InvalidCard => "Invalid Card Error",
            PKError::InvalidCardNumber => "Invalid Card Number Error",
            PKError::InvalidCardCount => "Invalid Card Count Error",
            PKError::InvalidCardIndex => "Invalid Card Index Error",
            PKError::InvalidComboIndex => "Invalid Combo Index Error",
            PKError::InvalidHand => "Invalid Hand Error",
            PKError::InvalidLength => "Invalid Length Error",
            PKError::InvalidPermutationIndex => "Invalid Permutation Index Error",
            PKError::InvalidPluribusIndex => "Invalid Pluribus Index Error",
            PKError::InvalidPosition => "Invalid Position Error",
            PKError::InvalidRankIndex => "Invalid Rank Index Error",
            PKError::InvalidRangeIndex => "Invalid Range Index Error",
            PKError::InvalidSeatNumber => "Invalid Seat Number Error",
            PKError::InvalidShift => "Invalid Shift Error",
            PKError::InvalidTableAction => "Invalid Table Action Error",
            PKError::Misaligned => "Misaligned Error",
            PKError::NoBlankSlots => "No Blank Slots Error",
            PKError::NoLow => "No low hand possible Error",
            PKError::NotDealt => "Not Dealt Error",
            PKError::NotEnoughCards => "Not Enough Cards Error",
            PKError::NotEnoughHands => "Not Enough Hands Error",
            PKError::PlayerOutOfHand => "Player is out of hand Error",
            PKError::SqlError => "SQL Error",
            PKError::TableFull => "Table Full Error",
            PKError::TooManyCards => "Too Many Cards Error",
            PKError::TooManyHands => "Too Many Hands Error",
            PKError::InvalidTwo => "Invalid Two Error",
        };
        write!(f, "{msg}")
    }
}

impl Error for PKError {}

impl From<rusqlite::Error> for PKError {
    fn from(err: rusqlite::Error) -> Self {
        log::error!("{err}");
        PKError::DBConnectionError
    }
}

/// # Agency Trait
///
/// Player Agency Action perspectives
///
/// - CAN? Can they act at all? _The current function._
/// - CAN THIS IF THAT? Can they do something given what they did before?
/// - CAN GIVEN? Can they do something given what another player has done?
///
/// This trait is used to establish the contract for when an entity in a game can act, given the
/// `PlayerState` abd the state of the action in the game.
pub trait Agency {
    /// The perspective on this call is that given this `PlayerState` is any other action possible,
    /// regardless of any other player's state.
    ///
    /// - If `self's PlayerState` is not active, the player cannot act in the hand.
    /// - If `self's PlayerState` is all-in, the player cannot perform any other actions. Their hand and chips are locked.
    #[must_use]
    fn can_act(&self) -> bool;

    fn can_given(&self, next: &PlayerState) -> bool;

    fn can_given_against(&self, next: &PlayerState, other: &PlayerState) -> bool;
}

pub trait Betting {
    /// # Errors
    ///
    /// Returns `PKError::Busted` if there are no chips.
    fn all_in(&mut self) -> Result<Self, PKError>
    where
        Self: Sized;

    /// # Errors
    ///
    /// Returns `PKError::InsufficientChips` if there are insufficient chips.
    fn bet(&mut self, amount: usize) -> Result<Self, PKError>
    where
        Self: Sized;

    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    fn size(&self) -> usize;

    /// Adds the amount of Chips won to the stack. Returns the resulting stack size.
    fn wins(&mut self, winnings: Self) -> usize;
}

pub trait Forgiving: FromStr + Default {
    /// Idea stolen from my `CardPack.rs` library.
    ///
    /// DIARY:
    ///
    /// ```txt
    /// pub trait Forgiving {
    ///     Self::from_str(index).unwrap_or_else(|_| {
    ///         log::warn!("forgiving_from_str(): {index} is invalid. Returning empty Pile.");
    ///         Self::default()
    ///    })
    /// }
    /// ```
    ///
    /// So I ask `CoPilot` how I get the code to compile, and it tells me that I need to
    /// make sure that the implementing struct implements `FromStr` and `Default`. DUMBASS!!!
    /// What TF is your code trying to do? What TWO MOTHER FUCKING THINGS does `Self` require?
    /// Take a minute... use that planet sized brain of yours. Why yes, if you need to use your
    /// structs `from_str`, which comes from implementing the `FromStr` trait, and the `default`,
    /// which requires the... **DRUM ROLL PLEASE** the FUCKING DEFAULT TRAIT **DUN DUN DUNNNN**,
    /// so yes, you need to make sure your implementer needs to have those traits implemented.
    ///
    /// As the great
    /// [Ben Stern](https://www.legacy.com/news/celebrity-deaths/ben-stern-2022-howard-sterns-father/)
    /// once said: "I told you not to be stupid, you moron."
    ///
    /// AI is making you dumber than you already clearly are.
    ///
    /// Perhaps I should implement this trait on myself.
    #[must_use]
    fn forgiving_from_str(index: &str) -> Self {
        Self::from_str(index).unwrap_or_else(|_| {
            log::warn!("forgiving_from_str(): {index} is invalid. Returning empty Pile.");
            Self::default()
        })
    }
}

pub trait GTO {
    fn combo_pairs(&self) -> ComboPairs {
        let twos = self.explode();
        let mut cps = ComboPairs::default();

        for two in twos.into_iter() {
            let combo = Combo::from(two);
            cps.add(combo, two);
        }
        cps
    }

    fn explode(&self) -> Twos;
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

    fn card_at(self, index: usize) -> Option<Card>;

    fn cards(&self) -> Cards {
        Cards::from(self.to_vec())
    }

    /// Will this work? Can I create a self referential clean? Only one want to find out...
    ///
    /// *NARRATOR:* _The answer is yes._
    #[must_use]
    fn clean(&self) -> Self;

    /// > This is a bit of a hack. I'm not sure if I should be doing this. I'm going to try it and
    ///
    /// Why TF is copilot using this tone for its suggested documentation???
    ///
    /// This method takes a `Pile` of `Cards` and does a bitwise OR on all of the `Cards` in the
    /// `pile`, returning a single `u32`.
    ///
    /// The initial goal of this method is for use in 8 or Better hand evals where all that matters
    /// us the `Rank` of the cards.
    #[must_use]
    fn collapse(&self) -> u32 {
        self.to_vec().iter().fold(0, |acc, card| acc | card.as_u32())
    }

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
        log::debug!("Pile.combinations_after(k: {k} cards: {cards})");
        self.remaining_after(cards).combinations(k)
    }

    fn combinations_remaining(&self, k: usize) -> Combinations<IntoIter<Card>> {
        log::debug!("Pile.combinations_after(k: {k})");
        self.remaining().combinations(k)
    }

    fn par_combinations_remaining(&self, k: usize) -> IterBridge<Combinations<IntoIter<Card>>> {
        log::debug!("Pile.combinations_after(k: {k})");
        self.remaining().par_combinations(k)
    }

    /// Tried refactoring this as `self.cards().index_set().contains(card)`, but it broke a lot of
    /// negative tests, since it just stripped out `Card::BLANK`.
    fn contains(&self, card: &Card) -> bool {
        self.to_vec().contains(card)
    }

    fn contains_blank(&self) -> bool {
        self.contains(&Card::BLANK)
    }

    fn enumerate_after(&self, k: usize, cards: &Cards) -> Enumerate<Combinations<IntoIter<Card>>> {
        log::info!("Pile.enumerate_after(k: {k} cards: {cards})");
        self.remaining_after(cards).combinations(k).enumerate()
    }

    fn enumerate_remaining(&self, k: usize) -> Enumerate<Combinations<IntoIter<Card>>> {
        log::info!("Pile.enumerate_after(k: {k})");
        self.combinations_remaining(k).enumerate()
    }

    fn get_rank_bits(&self) -> u16 {
        self.ranks().sum_or()
    }

    fn how_many(&self, cards: &Cards) -> usize {
        cards.to_vec().iter().filter(|card| self.contains(card)).count()
    }

    fn common(&self, cards: &Cards) -> Cards {
        // let v = self.to_vec().iter().filter(|card| {
        //     let contains = cards.contains(card);
        //     println!("{} contains {}? {}", self.cards(), card, contains);
        //
        //     contains
        //
        // }).cloned().collect::<Vec<Card>>();
        //
        // Cards::from(v)

        Cards::from(
            cards
                .to_vec()
                .iter()
                .filter(|card| self.contains(card))
                .copied()
                .collect::<Vec<Card>>(),
        )
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
        log::debug!("Pile.remaining_after(cards: {cards})");
        let mut held = self.cards();
        held.insert_all(cards);
        Cards::deck_minus(&held)
    }

    /// Returns the `Ranks` vector struct for the `Pile`.
    fn ranks(&self) -> Ranks {
        Ranks::from(self.to_vec().iter().map(Card::get_rank).collect::<Vec<Rank>>())
    }

    fn ranks_index(&self) -> String {
        self.to_vec()
            .iter()
            .map(|card| card.get_rank().to_char())
            .collect::<String>()
    }

    fn suits(&self) -> HashSet<Suit> {
        self.to_vec().iter().map(Card::get_suit).collect::<HashSet<Suit>>()
    }

    fn swap(&mut self, index: usize, card: Card) -> Option<Card>;

    fn the_nuts(&self) -> TheNuts;

    fn to_eight_or_better_bits(&self) -> u8 {
        self.cards()
            .iter()
            .fold(0, |acc, card| acc | card.get_rank().to_eight_or_better_lo_bit() | acc)
    }

    fn evals(&self) -> Evals {
        self.the_nuts().to_evals()
    }

    fn to_vec(&self) -> Vec<Card>;
}

/// The name of this trait is a pun on pluribus, which is the name of the poker AI group.
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
        Self: Display,
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
