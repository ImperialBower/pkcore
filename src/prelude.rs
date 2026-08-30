//! Prelude module for pkcore
//!
//! Import commonly used traits, types, and constants with:
//! ```
//! use pkcore::prelude::*;
//! ```

pub use std::str::FromStr;
pub use wincounter::PlayerFlag;
pub use wincounter::results::WinResults;
pub use wincounter::win::Win;
pub use wincounter::wins::Wins;

pub use crate::macros;

pub use crate::analysis::case_eval::CaseEval;
pub use crate::analysis::case_evals::CaseEvals;
pub use crate::analysis::eval::Eval;
pub use crate::analysis::evals::Evals;
pub use crate::analysis::gto::combo::Combo;
pub use crate::analysis::gto::combo_pairs::ComboPairs;
pub use crate::analysis::gto::combos::Combos;
pub use crate::analysis::nubibus::Nubificus;
pub use crate::analysis::nubibus::Pluribus;
pub use crate::analysis::outs::Outs;
pub use crate::analysis::the_nuts::TheNuts;
pub use crate::arrays::five::Five;
pub use crate::arrays::five::hands::Hands;
pub use crate::arrays::four::Four;
pub use crate::arrays::matchups::masked::Masked;
pub use crate::arrays::matchups::masks::Mask;
pub use crate::arrays::matchups::masks::rank_mask::RankMask;
pub use crate::arrays::matchups::masks::suit_mask::SuitMask;
pub use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
pub use crate::arrays::seven::Seven;
pub use crate::arrays::six::Six;
pub use crate::arrays::sliced::*;
pub use crate::arrays::three::Three;
pub use crate::arrays::two::Two;
pub use crate::bard::Bard;
pub use crate::boxed;
pub use crate::card::Card;
pub use crate::cards;
pub use crate::cards::Cards;
pub use crate::cards_cell::CardsCell;
pub use crate::casino;
pub use crate::casino::action::TableAction;
pub use crate::casino::equity::seat_equity::SeatEquity;
pub use crate::casino::equity::seatbit::Seatbit;
pub use crate::casino::equity::table_equity::TableEquity;
pub use crate::casino::game::ForcedBets;
pub use crate::casino::state::*;
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
pub use crate::util::Percentage;
pub use crate::util::Util;
pub use crate::util::data::TestData;
pub use crate::util::name::Name;
pub use crate::util::terminal::Terminal;

// Re-export core traits
pub use crate::{Agency, Betting, Forgiving, GTO, PKError, Pile, Plurable, SOK, Shifty, SuitShift, Unumable};

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

// play
pub use crate::play::game::StreetEquity;
#[cfg(feature = "equity")]
pub use crate::play::stages::deal_eval::DealEval;
pub use crate::play::stages::flop_eval::FlopEval;
pub use crate::play::stages::river_eval::RiverEval;
pub use crate::play::stages::turn_eval::TurnEval;

// games
pub use crate::games::omaha::OmahaHigh;

// games (EPIC-29 Phase 10 — variant foundation)
pub use crate::games::betting_structure::{BetTier, BettingStructure};
pub use crate::games::street::{StreetDescriptor, StreetIndex};
pub use crate::games::{GameFamily, GamePhase, GameType};

// play (EPIC-29 Phase 10 — visibility-aware hole-card model)
pub use crate::play::hole_card::HoleCard;
pub use crate::play::seat_hand::SeatHand;
pub use crate::play::visibility::Visibility;

// casino
pub use crate::casino::cashier::chips::Stack;
pub use crate::casino::dealer::Dealer;
pub use crate::casino::manager::TableManager;
pub use crate::casino::position::Positions;
pub use crate::casino::table::{BettingState, Player, SNAPSHOT_VERSION, Seat, SeatState, Seats, Table, TableState};
pub use crate::casino::winnings::{PotWin, Winnings};

// analysis/store
#[cfg(all(feature = "store", not(target_arch = "wasm32")))]
pub use crate::analysis::store::bcm::binary_card_map::{FiveBCM, SevenFiveBCM};
pub use crate::analysis::store::db::hup::HUPResult;
#[cfg(all(feature = "store", not(target_arch = "wasm32")))]
pub use crate::analysis::store::db::sqlite::Connect;
pub use crate::analysis::store::heads_up::HUP;

// analysis
pub use crate::analysis::ev::Ev;
pub use crate::analysis::eval::SevenEval;
pub use crate::analysis::gto::odds::WinLoseDraw;
pub use crate::analysis::gto::vs::Versus;
pub use crate::analysis::hand_rank::HandRank;
pub use crate::analysis::player_wins::PlayerWins;
pub use crate::analysis::pot_odds::PotOdds;
pub use crate::analysis::range_equity::RangeEquity;

// GTO solver
pub use crate::analysis::gto::game_tree::{GameTree, NodeId, TerminalNode};
pub use crate::analysis::gto::regret::RegretAccumulator;
pub use crate::analysis::gto::solver::{Solver, SolverResult};
#[cfg(not(target_arch = "wasm32"))]
pub use crate::analysis::gto::solver_cache::SolverCache;
pub use crate::analysis::gto::solver_config::{BetSize, BetSizings, SolverConfig};
pub use crate::analysis::gto::strategy_profile::{ActionFrequencies, StrategyProfile};
pub use crate::analysis::gto::twos::Twos;
pub use crate::analysis::gto::weighted_combos::WeightedCombos;

// hand history
pub use crate::hand_history::{
    Action, ActionType, AnalysisContext, FORMAT_VERSION, FlopStreet, HandCollection, HandHistory, HandMeta,
    HandVariant, Outcome, PlayerEntry, PostedBlind, PreflopStreet, ResultEntry, RiverStreet, Stakes, Streets,
    TableInfo, TurnStreet,
};
// The YAML I/O error type only exists with the `hand-histories` feature.
#[cfg(feature = "hand-histories")]
pub use crate::hand_history::HandHistoryError;

// player stats (player-stats feature)
#[cfg(feature = "player-stats")]
pub use crate::analysis::player_stats::{Confidence, PlayerStats, StatsRegistry};
#[cfg(feature = "player-stats")]
pub use crate::bot::exploit::ExploitConfig;
#[cfg(feature = "player-stats")]
pub use crate::bot::exploitative_decider::ExploitativeDecider;

// player stats persistence (player-stats-persistence feature)
#[cfg(feature = "player-stats-persistence")]
pub use crate::analysis::player_stats_store::{PlayerStatsStore, YamlPlayerStatsStore};

// bot training (bot-training feature)
#[cfg(feature = "bot-training")]
pub use crate::bot::training::{ExploitTrainer, GenerationRecord, TrainingConfig, TrainingResult};

// casino
pub use crate::casino::action::PlayerAction;
pub use crate::casino::principal::Principal;
pub use crate::casino::session::{PokerSession, SeatView, SessionState, SessionStep, SessionView};

// bot
pub use crate::bot::betting_strategy::BettingStrategy;
pub use crate::bot::decision_config::{
    DecisionConfig, EquityMode, ExploitMode, PotOddsConfig, PreflopCharts, RangeMode, Toggle,
};
pub use crate::bot::playbook::{Playbook, PlaybookEntry};
pub use crate::bot::position_ranges::{ActionRanges, PositionRanges};
pub use crate::bot::positional_betting::PositionalBetting;
pub use crate::bot::profile::{BotProfile, PlayStyle};
pub use crate::bot::range_strategy::RangeStrategy;
pub use crate::bot::table_size::TableSize;
pub use crate::bot::weighted_range::{ComboWeight, WeightedRange};
