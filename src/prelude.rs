//! Prelude module for pkcore
//!
//! Import commonly used traits, types, and constants with:
//! ```
//! use pkcore::prelude::*;
//! ```

pub use std::str::FromStr;

pub use crate::macros;

pub use crate::analysis::evals::Evals;
pub use crate::analysis::gto::combo::Combo;
pub use crate::analysis::gto::combo_pairs::ComboPairs;
pub use crate::analysis::outs::Outs;
pub use crate::analysis::the_nuts::TheNuts;
pub use crate::arrays::sliced::*;
pub use crate::arrays::two::Two;

pub use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
pub use crate::bard::Bard;
pub use crate::boxed;
pub use crate::card::Card;
pub use crate::cards;
pub use crate::cards::Cards;
pub use crate::cards_cell::CardsCell;
pub use crate::casino;
pub use crate::casino::game::ForcedBets;
pub use crate::casino::player::Player;
pub use crate::casino::state::*;
pub use crate::casino::table;
pub use crate::casino::table::Table;
pub use crate::casino::table::event::TableAction;
pub use crate::casino::table::event::TableLog;
pub use crate::casino::table::seat::Seat;
pub use crate::casino::table::seats::Seats;
pub use crate::cc;
pub use crate::deck;
pub use crate::deck::Deck;
pub use crate::deck_cell;
pub use crate::play::board::Board;
pub use crate::play::game::Game;
pub use crate::play::hole_cards::HoleCards;
pub use crate::rank::Rank;
pub use crate::ranks::Ranks;
pub use crate::suit::Suit;
pub use crate::util::data::TestData;

// Re-export core traits
pub use crate::{Agency, Betting, Forgiving, GTO, PKError, Pile, Plurable, SOK, Shifty, SuitShift};

// Re-export all constants
pub use crate::{
    DISTINCT_2_CARD_HANDS, DISTINCT_5_CARD_HANDS, DISTINCT_FLUSH, DISTINCT_FOUR_OF_A_KIND, DISTINCT_FULL_HOUSES,
    DISTINCT_HIGH_CARD, DISTINCT_ONE_PAIR, DISTINCT_PER_RANK_2_CARD_HANDS, DISTINCT_STRAIGHT,
    DISTINCT_STRAIGHT_FLUSHES, DISTINCT_THREE_OF_A_KIND, DISTINCT_TWO_PAIR, POSSIBLE_UNIQUE_HOLDEM_HUP_MATCHUPS,
    UNIQUE_2_CARD_HANDS, UNIQUE_5_CARD_HANDS, UNIQUE_FLUSH, UNIQUE_FOUR_OF_A_KIND, UNIQUE_FULL_HOUSES,
    UNIQUE_HIGH_CARD, UNIQUE_NON_POCKET_PAIRS, UNIQUE_ONE_PAIR, UNIQUE_PER_CARD_2_CARD_HANDS,
    UNIQUE_PER_RANK_2_CARD_HANDS, UNIQUE_PER_SUIT_2_CARD_HANDS, UNIQUE_POCKET_PAIRS, UNIQUE_STRAIGHT,
    UNIQUE_STRAIGHT_FLUSHES, UNIQUE_SUITED_2_CARD_HANDS, UNIQUE_THREE_OF_A_KIND, UNIQUE_TWO_PAIR,
};
