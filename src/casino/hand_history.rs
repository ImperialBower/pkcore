//! Poker Hand History File Format
//!
//! This module provides structures for serializing and deserializing poker hand histories
//! into YAML format, following a standardized Poker Hand History specification.
//!
//! # Format
//!
//! The format captures all essential information about a poker hand including:
//! - Game metadata (table, blinds, game type)
//! - Player information and positions
//! - Hole cards dealt to each player
//! - Actions taken during each betting round
//! - Community cards (flop, turn, river)
//! - Showdown results and pot distribution
//!
//! # Example
//!
//! ```yaml
//! version: "1.0"
//! hand_id: "550e8400-e29b-41d4-a716-446655440000"
//! timestamp: "2026-02-19T10:30:00Z"
//! table:
//!   name: "Table 1"
//!   max_players: 6
//! game:
//!   variant: "NoLimitHoldem"
//!   blinds:
//!     small_blind: 50
//!     big_blind: 100
//!     ante: 0
//! players:
//!   - seat: 0
//!     name: "Player1"
//!     stack: 10000
//!     hole_cards: ["Ah", "Kh"]
//! actions:
//!   preflop:
//!     - seat: 0
//!       action: "raise"
//!       amount: 300
//! board:
//!   flop: ["Qh", "Jh", "Th"]
//!   turn: "9h"
//!   river: "8h"
//! results:
//!   - seat: 0
//!     winnings: 1500
//!     hand: "Royal Flush"
//! ```

use crate::card::Card;
use crate::casino::table::Table;
use crate::casino::table::event::TableAction;
use crate::games::GameType;
use crate::{PKError, Pile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Poker Hand History format version
pub const PHH_VERSION: &str = "1.0";

/// Complete poker hand history that can be serialized to/from YAML
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PokerHandHistory {
    /// Format version
    pub version: String,

    /// Unique identifier for this hand
    pub hand_id: String,

    /// ISO 8601 timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Table information
    pub table: TableInfo,

    /// Game configuration
    pub game: GameInfo,

    /// Players in the hand
    pub players: Vec<PlayerInfo>,

    /// Button position (seat number)
    pub button: u8,

    /// Actions taken during the hand
    pub actions: HandActions,

    /// Community cards
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board: Option<BoardCards>,

    /// Results and pot distribution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<HandResult>>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Table information
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TableInfo {
    /// Table name or identifier
    pub name: String,

    /// Maximum number of players
    pub max_players: u8,

    /// Table ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Game configuration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GameInfo {
    /// Game variant (`NoLimitHoldem`, PLO, etc.)
    pub variant: String,

    /// Blind structure
    pub blinds: BlindInfo,
}

/// Blind structure
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BlindInfo {
    pub small_blind: usize,
    pub big_blind: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ante: Option<usize>,
}

/// Player information
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlayerInfo {
    /// Seat number (0-indexed)
    pub seat: u8,

    /// Player name/handle
    pub name: String,

    /// Starting stack size
    pub stack: usize,

    /// Hole cards (if known/visible)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hole_cards: Option<Vec<String>>,

    /// Player ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_id: Option<String>,
}

/// Actions during all betting rounds
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct HandActions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preflop: Option<Vec<PlayerAction>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flop: Option<Vec<PlayerAction>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn: Option<Vec<PlayerAction>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub river: Option<Vec<PlayerAction>>,
}

/// Individual player action
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlayerAction {
    /// Seat number of acting player
    pub seat: u8,

    /// Action type (fold, call, bet, raise, check, all-in)
    pub action: String,

    /// Amount (for bets/raises)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<usize>,

    /// Final pot size after this action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_after: Option<usize>,
}

/// Community cards (board)
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BoardCards {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flop: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub river: Option<String>,
}

/// Hand result for a player
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HandResult {
    /// Seat number
    pub seat: u8,

    /// Player name
    pub name: String,

    /// Amount won (positive) or lost (negative)
    pub winnings: isize,

    /// Hand description (if shown at showdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand: Option<String>,

    /// Best 5-card hand (if shown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_hand: Option<Vec<String>>,
}

impl PokerHandHistory {
    /// Create a new poker hand history with default values
    #[must_use]
    pub fn new(hand_id: String) -> Self {
        Self {
            version: PHH_VERSION.to_string(),
            hand_id,
            timestamp: None,
            table: TableInfo {
                name: String::new(),
                max_players: 6,
                id: None,
            },
            game: GameInfo {
                variant: String::new(),
                blinds: BlindInfo {
                    small_blind: 0,
                    big_blind: 0,
                    ante: None,
                },
            },
            players: Vec::new(),
            button: 0,
            actions: HandActions::default(),
            board: None,
            results: None,
            metadata: None,
        }
    }

    /// Serialize to YAML string
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails
    pub fn to_yaml(&self) -> Result<String, PKError> {
        serde_yaml::to_string(self).map_err(|_| PKError::SerializationError)
    }

    /// Deserialize from YAML string
    ///
    /// # Errors
    ///
    /// Returns error if deserialization fails
    pub fn from_yaml(yaml: &str) -> Result<Self, PKError> {
        serde_yaml::from_str(yaml).map_err(|_| PKError::DeserializationError)
    }

    /// Save to YAML file
    ///
    /// # Errors
    ///
    /// Returns error if file write fails
    pub fn save_to_file(&self, path: &str) -> Result<(), PKError> {
        let yaml = self.to_yaml()?;
        std::fs::write(path, yaml).map_err(|_| PKError::IoError)
    }

    /// Load from YAML file
    ///
    /// # Errors
    ///
    /// Returns error if file read or deserialization fails
    pub fn load_from_file(path: &str) -> Result<Self, PKError> {
        let yaml = std::fs::read_to_string(path).map_err(|_| PKError::IoError)?;
        Self::from_yaml(&yaml)
    }
}

impl TryFrom<&Table> for PokerHandHistory {
    type Error = PKError;

    fn try_from(table: &Table) -> Result<Self, Self::Error> {
        let hand_id = table.id.to_string();

        // Extract table info
        let table_info = TableInfo {
            name: table.name.clone(),
            max_players: table.seats.size(),
            id: Some(table.id.to_string()),
        };

        // Extract game info
        let game_variant = match table.game {
            GameType::NoLimitHoldem => "NoLimitHoldem",
            GameType::PLO => "PLO",
            GameType::Razz => "Razz",
        };

        let game_info = GameInfo {
            variant: game_variant.to_string(),
            blinds: BlindInfo {
                small_blind: table.forced.small_blind,
                big_blind: table.forced.big_blind,
                ante: if table.forced.ante > 0 {
                    Some(table.forced.ante)
                } else {
                    None
                },
            },
        };

        // Extract player info
        let mut players = Vec::new();
        for (i, seat_cell) in table.seats.borrow_all().iter().enumerate() {
            let seat = seat_cell.borrow();
            let seat_num = u8::try_from(i).unwrap_or_default();

            // Get hole cards if dealt
            let hole_cards = if seat.cards.is_empty() {
                None
            } else {
                let cards: Vec<String> = seat.cards.cards().iter().map(ToString::to_string).collect();
                if cards.is_empty() { None } else { Some(cards) }
            };

            players.push(PlayerInfo {
                seat: seat_num,
                name: seat.player.handle.clone(),
                stack: seat.player.chips.count(),
                hole_cards,
                player_id: Some(seat.player.id.to_string()),
            });
        }

        // Extract board cards
        let board = if table.board.is_empty() {
            None
        } else {
            let cards_obj = table.board.cards();
            let cards: Vec<Card> = cards_obj.iter().copied().collect();
            let mut board_cards = BoardCards::default();

            if cards.len() >= 3 {
                board_cards.flop = Some(cards[0..3].iter().map(ToString::to_string).collect());
            }
            if cards.len() >= 4 {
                board_cards.turn = Some(cards[3].to_string());
            }
            if cards.len() >= 5 {
                board_cards.river = Some(cards[4].to_string());
            }

            Some(board_cards)
        };

        // Extract actions from event log
        let actions = extract_actions_from_log(&table.event_log);

        Ok(PokerHandHistory {
            version: PHH_VERSION.to_string(),
            hand_id,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            table: table_info,
            game: game_info,
            players,
            button: table.button.value(),
            actions,
            board,
            results: None, // Would need to be populated after hand completion
            metadata: None,
        })
    }
}

/// Extract actions from table event log
fn extract_actions_from_log(log: &crate::casino::table::event::TableLog) -> HandActions {
    let mut actions = HandActions::default();
    let mut preflop = Vec::new();
    let mut flop = Vec::new();
    let mut turn = Vec::new();
    let mut river = Vec::new();

    let mut current_phase = "preflop";

    for event in log.entries() {
        match event {
            TableAction::DealtFlop(_) => {
                current_phase = "flop";
            }
            TableAction::DealtTurn(_) => {
                current_phase = "turn";
            }
            TableAction::DealtRiver(_) => {
                current_phase = "river";
            }
            TableAction::Fold(seat) => {
                let action = PlayerAction {
                    seat,
                    action: "fold".to_string(),
                    amount: None,
                    pot_after: None,
                };
                push_action(&mut preflop, &mut flop, &mut turn, &mut river, current_phase, action);
            }
            TableAction::Call(seat, amount) => {
                let action = PlayerAction {
                    seat,
                    action: "call".to_string(),
                    amount: Some(amount),
                    pot_after: None,
                };
                push_action(&mut preflop, &mut flop, &mut turn, &mut river, current_phase, action);
            }
            TableAction::Raise(seat, amount) => {
                let action = PlayerAction {
                    seat,
                    action: "raise".to_string(),
                    amount: Some(amount),
                    pot_after: None,
                };
                push_action(&mut preflop, &mut flop, &mut turn, &mut river, current_phase, action);
            }
            TableAction::Bet(seat, amount) => {
                let action = PlayerAction {
                    seat,
                    action: "bet".to_string(),
                    amount: Some(amount),
                    pot_after: None,
                };
                push_action(&mut preflop, &mut flop, &mut turn, &mut river, current_phase, action);
            }
            TableAction::Check(seat) => {
                let action = PlayerAction {
                    seat,
                    action: "check".to_string(),
                    amount: None,
                    pot_after: None,
                };
                push_action(&mut preflop, &mut flop, &mut turn, &mut river, current_phase, action);
            }
            TableAction::AllIn(seat, amount) => {
                let action = PlayerAction {
                    seat,
                    action: "all-in".to_string(),
                    amount: Some(amount),
                    pot_after: None,
                };
                push_action(&mut preflop, &mut flop, &mut turn, &mut river, current_phase, action);
            }
            _ => {}
        }
    }

    actions.preflop = if preflop.is_empty() { None } else { Some(preflop) };
    actions.flop = if flop.is_empty() { None } else { Some(flop) };
    actions.turn = if turn.is_empty() { None } else { Some(turn) };
    actions.river = if river.is_empty() { None } else { Some(river) };

    actions
}

fn push_action(
    preflop: &mut Vec<PlayerAction>,
    flop: &mut Vec<PlayerAction>,
    turn: &mut Vec<PlayerAction>,
    river: &mut Vec<PlayerAction>,
    phase: &str,
    action: PlayerAction,
) {
    match phase {
        "preflop" => preflop.push(action),
        "flop" => flop.push(action),
        "turn" => turn.push(action),
        "river" => river.push(action),
        _ => {}
    }
}

/// Extension trait for Table to add hand history functionality
pub trait HandHistoryExt {
    /// Export the current table state as a poker hand history
    ///
    /// # Errors
    ///
    /// Returns error if conversion fails
    fn to_hand_history(&self) -> Result<PokerHandHistory, PKError>;

    /// Save the current hand to a YAML file
    ///
    /// # Errors
    ///
    /// Returns error if conversion or file write fails
    fn save_hand_history(&self, path: &str) -> Result<(), PKError>;
}

impl HandHistoryExt for Table {
    fn to_hand_history(&self) -> Result<PokerHandHistory, PKError> {
        PokerHandHistory::try_from(self)
    }

    fn save_hand_history(&self, path: &str) -> Result<(), PKError> {
        let history = self.to_hand_history()?;
        history.save_to_file(path)
    }
}

#[cfg(test)]
mod casino_hand_history_tests {
    use super::*;

    #[test]
    fn new_hand_history() {
        let history = PokerHandHistory::new("test-hand-001".to_string());
        assert_eq!(history.version, "1.0");
        assert_eq!(history.hand_id, "test-hand-001");
        assert_eq!(history.players.len(), 0);
    }

    #[test]
    fn serialize_deserialize_yaml() {
        let mut history = PokerHandHistory::new("test-001".to_string());
        history.table.name = "Test Table".to_string();
        history.game.variant = "NoLimitHoldem".to_string();
        history.game.blinds.small_blind = 50;
        history.game.blinds.big_blind = 100;

        let yaml = history.to_yaml().unwrap();
        let deserialized = PokerHandHistory::from_yaml(&yaml).unwrap();

        assert_eq!(history, deserialized);
    }

    #[test]
    fn player_action_serialization() {
        let action = PlayerAction {
            seat: 0,
            action: "raise".to_string(),
            amount: Some(300),
            pot_after: Some(500),
        };

        let yaml = serde_yaml::to_string(&action).unwrap();
        let deserialized: PlayerAction = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(action, deserialized);
    }
}
