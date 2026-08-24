//! Action vocabularies for the table engines.
//!
//! [`PlayerAction`] is the canonical player-decision type for the engine's
//! transition surface: what
//! [`Table::legal_actions`](crate::casino::table::Table::legal_actions)
//! reports and [`Table::apply_action`](crate::casino::table::Table::apply_action)
//! consumes; it is also the decision type bot deciders produce (via the
//! re-export `crate::bot::player_action::PlayerAction`). It has no feature
//! requirement — the transition surface is a feature-free kernel boundary.
//!
//! [`TableAction`] is the event-log entry type.
//! [`Table`](crate::casino::table::Table) records hand history as a plain
//! `Vec<TableAction>`. EPIC-83 retired the interior-mutable `TableLog` that
//! the second engine used to keep alongside it.

use crate::bard::Bard;
use crate::cards::Cards;
use crate::prelude::PlayerState;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::Uuid;

/// A player's chosen action at their turn.
///
/// Reported by
/// [`Table::legal_actions`](crate::casino::table::Table::legal_actions)
/// and applied via
/// [`Table::apply_action`](crate::casino::table::Table::apply_action);
/// also the value bot deciders produce.
///
/// # Examples
///
/// ```
/// use pkcore::casino::action::PlayerAction;
///
/// let action = PlayerAction::Bet(200);
/// assert_eq!(action, PlayerAction::Bet(200));
/// assert_eq!(action.to_string(), "Bet(200)");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAction {
    /// Discard hole cards and exit the hand.
    Fold,
    /// Pass without betting (only legal when no bet faces the player).
    Check,
    /// Match the current bet to stay in the hand.
    Call,
    /// Open a bet of `n` chips (only legal when no bet is outstanding).
    Bet(usize),
    /// Re-open the bet to `n` chips total (must exceed the current bet by at
    /// least the minimum raise increment).
    Raise(usize),
    /// Commit all remaining chips.
    AllIn,
}

impl std::fmt::Display for PlayerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fold => write!(f, "Fold"),
            Self::Check => write!(f, "Check"),
            Self::Call => write!(f, "Call"),
            Self::Bet(n) => write!(f, "Bet({n})"),
            Self::Raise(n) => write!(f, "Raise({n})"),
            Self::AllIn => write!(f, "AllIn"),
        }
    }
}

/// An entry in a table's event log — everything that happens during a hand,
/// from `TableOpen` through `ResetTable`.
///
/// [`Table`](crate::casino::table::Table) records hand history as a sequence
/// of these, in its `event_log: Vec<TableAction>` field.
///
/// # Examples
///
/// ```
/// use pkcore::casino::action::TableAction;
///
/// let bet = TableAction::Bet(3, 200);
/// assert_eq!("Seat 3 bets 200", bet.to_string());
/// assert_eq!(Some(200), bet.get_amount());
/// assert_eq!(Some(3), bet.get_seat());
/// ```
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
#[non_exhaustive] // 0.2.0: this is a serialized wire enum; adding a variant must stay non-breaking.
pub enum TableAction {
    #[default]
    Pause,
    TableOpen(Uuid),
    PlayerSeated(u8, Uuid),
    NewHand,
    ShuffleDeck,
    SetButton(u8),
    MoveButton(u8),
    ForcedBets,
    ForcedBet(u8, usize),
    ForcedBetSmallBlind(u8, usize),
    ForcedBetBigBlind(u8, usize),
    BetAnteForced(u8, usize),
    /// EPIC-32: stud-family bring-in post. The seat with the lowest
    /// (Stud Hi) or highest (Razz) upcard on 3rd street pays this amount.
    /// Distinct from `BringItIn(usize)` which consolidates bets at the
    /// end of a betting street.
    StudBringInPost(u8, usize),
    DealingXCards(u8),
    Dealt(u8, Bard),
    DealtFlop(Bard),
    DealtTurn(Bard),
    DealtRiver(Bard),
    DealtPlayers,
    ForceDealt(u8, Bard),
    BringItIn(usize),
    ActionTo(u8),
    Check(u8),
    Bet(u8, usize),
    Call(u8, usize),
    Raise(u8, usize),
    AllIn(u8, usize),
    Fold(u8),
    PotSize(usize),
    SplitPots(),
    MainPot(usize),
    SidePot(usize),
    SplitPot(u8, usize), // (number of winners, pot size)
    MuckCards(Bard),
    MuckPlayerCards(u8, Bard),
    TakePlayerCards(u8, Bard),
    TakeBoardCards(Bard),
    ClosesTheAction(u8),
    CloseItOut(usize),
    EndHand,
    /// The hand could not be completed and was unwound: every chip committed
    /// to it was returned to the stack it came from. Carries the total
    /// refunded (`DEFECT_019`).
    HandAborted(usize),
    ResetTable,
    Showdown(u8),
    PlayerMucksCards(u8), // At a showdown one player mucks their cards rather than show them.
    AllFoldedTo(u8),
    PlayerWinsSidePot(u8, usize),
    PlayerWinsMainPot(u8, usize),
    PlayerLosesSidePot(u8, usize),
    PlayerLosesMainPot(u8, usize),
    PlayerWins(u8, Uuid, Bard, usize, usize), // (seat, player_id, winning_hand, amount_won, in_showdown)
    PlayerLoses(u8, Uuid, Bard, usize),       // (seat, player_id, winning_hand, amount_lost, in_showdown)
    InvalidAction,
    InvalidPlayerAction(u8, PlayerState),
    NotEnoughCards,
    TooManyCards,
    InvalidSeatNumber,
    DeckPassesAudit,
    /// Chip conservation check at end of hand.
    ///
    /// `(expected, actual)` — only logged when the counts differ.
    ChipAuditFailed(usize, usize),
}

impl TableAction {
    #[must_use]
    pub fn commentary(&self, name: &str) -> String {
        match self {
            TableAction::ForcedBetSmallBlind(_, amount) => format!("{name} posts {amount} small blind"),
            TableAction::ForcedBetBigBlind(_, amount) => format!("{name} posts {amount} big blind"),
            TableAction::BringItIn(amount) => format!("Brings in {amount}"),
            TableAction::Bet(_, amount) => format!("{name} bets {amount}"),
            TableAction::Call(_, amount) => format!("{name} calls {amount}"),
            TableAction::Raise(_, amount) => format!("{name} raises to {amount}"),
            TableAction::AllIn(_, _) => format!("{name} goes all in."),
            TableAction::Fold(_) => format!("{name} folds"),
            TableAction::Check(_) => format!("{name} checks"),
            TableAction::PotSize(amount) => format!("{amount} pots size"),
            TableAction::MainPot(amount) => format!("Main pot {amount}"),
            TableAction::SidePot(amount) => format!("Side pot {amount}"),
            TableAction::SplitPot(number, size) => format!("{size} split pot between {number} players"),
            TableAction::Dealt(_, bard) => format!("{name} dealt {}", Cards::from(*bard)),
            TableAction::DealtFlop(bard) => format!("Flop is {}", Cards::from(*bard)),
            TableAction::DealtTurn(bard) => format!("Turn is {}", Cards::from(*bard)),
            TableAction::DealtRiver(bard) => format!("River is {}", Cards::from(*bard)),
            TableAction::EndHand => "Hand over.".to_string(),
            TableAction::ResetTable => "Table reset for next hand.".to_string(),
            TableAction::PlayerMucksCards(_u8) => format!("{name} mucks their cards."),
            TableAction::AllFoldedTo(_) => format!("Everyone folds to {name}."),
            TableAction::PlayerWinsSidePot(seat, winnings) => {
                format!("{name} (Seat {seat}) wins side pot of {winnings}")
            }
            TableAction::PlayerWinsMainPot(seat, winnings) => {
                format!("{name} (Seat {seat}) wins main pot of {winnings}")
            }
            TableAction::PlayerLosesSidePot(seat, losses) => {
                format!("{name} (Seat {seat}) loses side pot of {losses}")
            }
            TableAction::PlayerLosesMainPot(seat, losses) => {
                format!("{name} (Seat {seat}) loses main pot of {losses}")
            }
            TableAction::PlayerWins(seat, _, bard, winnings, pot_size) => {
                format!(
                    "{name} (Seat {seat}) wins {winnings} of a pot of {pot_size} with {}",
                    Cards::from(*bard)
                )
            }
            TableAction::PlayerLoses(seat, _, bard, losses) => {
                format!("{name} (Seat {seat}) loses {losses} with {}", Cards::from(*bard))
            }
            _ => self.to_string(),
        }
    }

    /// Mirrors a [`TableAction::PlayerWins`] into the matching
    /// [`TableAction::PlayerLoses`], carrying the seat, player id and hand
    /// across and dropping the pot-size field the losing variant does not
    /// carry.
    ///
    /// Returns `None` for every other variant — only a win has a loss to
    /// mirror. (`DEFECT_023`: this method used to be an unconditional
    /// `unimplemented!()`.)
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bard::Bard;
    /// use pkcore::casino::action::TableAction;
    /// use uuid::Uuid;
    ///
    /// let id = Uuid::nil();
    /// let win = TableAction::PlayerWins(3, id, Bard::default(), 500, 1_200);
    ///
    /// assert_eq!(
    ///     Some(TableAction::PlayerLoses(3, id, Bard::default(), 500)),
    ///     win.generate_player_loses()
    /// );
    /// assert_eq!(None, TableAction::Fold(3).generate_player_loses());
    /// ```
    #[must_use]
    pub fn generate_player_loses(&self) -> Option<TableAction> {
        match self {
            TableAction::PlayerWins(seat, player_id, hand, amount, _pot_size) => {
                Some(TableAction::PlayerLoses(*seat, *player_id, *hand, *amount))
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn get_amount(&self) -> Option<usize> {
        match self {
            TableAction::ForcedBet(_, amount)
            | TableAction::ForcedBetSmallBlind(_, amount)
            | TableAction::ForcedBetBigBlind(_, amount)
            | TableAction::BetAnteForced(_, amount)
            | TableAction::StudBringInPost(_, amount)
            | TableAction::BringItIn(amount)
            | TableAction::Bet(_, amount)
            | TableAction::Call(_, amount)
            | TableAction::Raise(_, amount)
            | TableAction::AllIn(_, amount)
            | TableAction::CloseItOut(amount)
            | TableAction::PotSize(amount)
            | TableAction::MainPot(amount)
            | TableAction::SidePot(amount)
            | TableAction::PlayerWins(_, _, _, amount, _)
            | TableAction::PlayerLoses(_, _, _, amount)
            | TableAction::PlayerWinsSidePot(_, amount)
            | TableAction::PlayerWinsMainPot(_, amount)
            | TableAction::PlayerLosesSidePot(_, amount)
            | TableAction::PlayerLosesMainPot(_, amount) => Some(*amount),
            _ => None,
        }
    }

    /// Returns the seat number for the `TableAction`, if there is one.
    #[must_use]
    pub fn get_seat(&self) -> Option<u8> {
        match self {
            TableAction::PlayerSeated(seat, _)
            | TableAction::SetButton(seat)
            | TableAction::MoveButton(seat)
            | TableAction::ForcedBetSmallBlind(seat, _)
            | TableAction::ForcedBetBigBlind(seat, _)
            | TableAction::BetAnteForced(seat, _)
            | TableAction::StudBringInPost(seat, _)
            | TableAction::Dealt(seat, _)
            | TableAction::ForceDealt(seat, _)
            | TableAction::ActionTo(seat)
            | TableAction::Check(seat)
            | TableAction::Bet(seat, _)
            | TableAction::Call(seat, _)
            | TableAction::Raise(seat, _)
            | TableAction::AllIn(seat, _)
            | TableAction::Fold(seat)
            | TableAction::AllFoldedTo(seat)
            | TableAction::PlayerWins(seat, _, _, _, _)
            | TableAction::PlayerLoses(seat, _, _, _)
            | TableAction::MuckPlayerCards(seat, _)
            | TableAction::InvalidPlayerAction(seat, _)
            | TableAction::TakePlayerCards(seat, _) => Some(*seat),

            _ => None,
        }
    }

    #[must_use]
    pub fn is_player_action(&self) -> bool {
        matches!(
            self,
            TableAction::Bet(_, _)
                | TableAction::Call(_, _)
                | TableAction::Raise(_, _)
                | TableAction::Fold(_)
                | TableAction::Check(_)
                | TableAction::AllIn(_, _)
                | TableAction::ClosesTheAction(_)
        )
    }

    #[must_use]
    pub fn is_result(&self) -> bool {
        matches!(
            self,
            Self::PlayerWins(_, _, _, _, _)
                | Self::PlayerLoses(_, _, _, _)
                | Self::PlayerWinsMainPot(_, _)
                | Self::PlayerWinsSidePot(_, _)
                | Self::PlayerLosesMainPot(_, _)
                | Self::PlayerLosesSidePot(_, _)
        )
    }
}

impl Display for TableAction {
    /// ```
    /// use pkcore::bard::Bard;
    /// use pkcore::casino::action::TableAction;
    /// use std::str::FromStr;
    ///
    /// let dealt = TableAction::Dealt(1, Bard::from_str("AS KS").unwrap());
    ///
    /// assert_eq!("Seat 1 is dealt A♠ K♠", dealt.to_string())
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableAction::Pause => write!(f, "Pause"),
            TableAction::TableOpen(table_id) => write!(f, "Table {table_id} is now open"),
            TableAction::PlayerSeated(seat, player_id) => {
                write!(f, "Player {player_id} is seated at Seat {seat}")
            }
            TableAction::NewHand => write!(f, "New Hand"),
            TableAction::ShuffleDeck => write!(f, "Shuffle Deck"),
            TableAction::SetButton(seat) => write!(f, "Set Button to Seat {seat}"),
            TableAction::MoveButton(seat) => write!(f, "Move Button to Seat {seat}"),
            TableAction::ForcedBets => write!(f, "Forced Bets"),
            TableAction::ForcedBet(seat, amount) => {
                write!(f, "Seat {seat} puts in forced bet of {amount}")
            }
            TableAction::ForcedBetSmallBlind(seat, amount) => {
                write!(f, "Seat {seat} puts in Small Blind of {amount}")
            }
            TableAction::ForcedBetBigBlind(seat, amount) => {
                write!(f, "Seat {seat} puts in Big Blind of {amount}")
            }
            TableAction::BetAnteForced(seat, amount) => {
                write!(f, "Seat {seat} Antes {amount}")
            }
            TableAction::StudBringInPost(seat, amount) => {
                write!(f, "Seat {seat} posts {amount} bring-in")
            }
            TableAction::DealingXCards(x) => write!(f, "Dealing out {x} cards"),
            TableAction::Dealt(seat, cards) => write!(f, "Seat {seat} is dealt {}", Cards::from(*cards)),
            TableAction::DealtFlop(cards) => write!(f, "Flop is {}", Cards::from(*cards)),
            TableAction::DealtTurn(cards) => write!(f, "Turn is {}", Cards::from(*cards)),
            TableAction::DealtRiver(cards) => write!(f, "River is {}", Cards::from(*cards)),
            TableAction::DealtPlayers => write!(f, "Dealt Players"),
            TableAction::ForceDealt(seat, cards) => {
                write!(f, "Seat {seat} is force-dealt {}", Cards::from(*cards))
            }
            TableAction::BringItIn(amount) => write!(f, "Brings in {amount}"),
            TableAction::ActionTo(seat) => write!(f, "Action to Seat {seat}"),
            TableAction::Check(seat) => write!(f, "Seat {seat} checks"),
            TableAction::Bet(seat, amount) => write!(f, "Seat {seat} bets {amount}"),
            TableAction::Call(seat, amount) => write!(f, "Seat {seat} calls {amount}"),
            TableAction::Raise(seat, amount) => write!(f, "Seat {seat} raises to {amount}"),
            TableAction::AllIn(seat, amount) => write!(f, "Seat {seat} goes all in with {amount}"),
            TableAction::Fold(seat) => write!(f, "Seat {seat} folds"),
            TableAction::PotSize(amount) => write!(f, "Pot size is {amount}"),
            TableAction::SplitPots() => write!(f, "Split Pots"),
            TableAction::MainPot(amount) => write!(f, "Main Pot is {amount}"),
            TableAction::SidePot(amount) => write!(f, "Side Pot is {amount}"),
            TableAction::SplitPot(number, size) => {
                write!(f, "{size} split pot between {number} players")
            }
            TableAction::MuckCards(cards) => write!(f, "Muck cards: {}", Cards::from(*cards)),
            TableAction::MuckPlayerCards(seat, cards) => {
                write!(f, "Muck player {seat}'s cards: {}", Cards::from(*cards))
            }
            TableAction::TakePlayerCards(seat, cards) => {
                write!(f, "Take player {seat}'s cards: {}", Cards::from(*cards))
            }
            TableAction::TakeBoardCards(cards) => write!(f, "Take board cards: {}", Cards::from(*cards)),
            TableAction::ClosesTheAction(seat) => write!(f, "Seat {seat} closes the action"),
            TableAction::CloseItOut(amount) => write!(f, "Close out the hand with a {amount} pot"),
            TableAction::EndHand => write!(f, "End Hand"),
            TableAction::Showdown(seat) => write!(f, "{seat} seats in showdown"),
            TableAction::PlayerMucksCards(seat) => write!(f, "Seat {seat} mucks their cards"),
            TableAction::AllFoldedTo(seat) => write!(f, "All folded to Seat {seat}"),
            TableAction::PlayerWinsSidePot(seat, amount) => {
                write!(f, "Seat {seat} wins side pot of {amount}")
            }
            TableAction::PlayerWinsMainPot(seat, amount) => {
                write!(f, "Seat {seat} wins main pot of {amount}")
            }
            TableAction::PlayerLosesSidePot(seat, amount) => {
                write!(f, "Seat {seat} loses side pot of {amount}")
            }
            TableAction::PlayerLosesMainPot(seat, amount) => {
                write!(f, "Seat {seat} loses main pot of {amount}")
            }
            TableAction::PlayerWins(seat, player_id, winning_hand, amount_won, pot_size) => write!(
                f,
                "Seat {seat} (Player {player_id}) wins {amount_won} of a pot of {pot_size} with {}",
                Cards::from(*winning_hand)
            ),
            TableAction::PlayerLoses(seat, player_id, losing_hand, amount_lost) => write!(
                f,
                "Seat {seat} (Player {player_id}) loses {amount_lost} with {}",
                Cards::from(*losing_hand)
            ),
            TableAction::InvalidAction => write!(f, "Invalid Action"),
            TableAction::InvalidPlayerAction(seat, action) => write!(f, "Invalid action by Seat {seat}: {action}"),
            TableAction::NotEnoughCards => write!(f, "Not enough cards to deal"),
            TableAction::TooManyCards => write!(f, "Too many cards to deal"),
            TableAction::InvalidSeatNumber => write!(f, "Invalid Seat Number"),
            TableAction::DeckPassesAudit => write!(f, "Deck passes audit"),
            TableAction::ChipAuditFailed(expected, actual) => {
                write!(f, "Chip audit failed: expected {expected} chips, found {actual}")
            }
            TableAction::HandAborted(refunded) => {
                write!(f, "Hand aborted; {refunded} returned to players")
            }
            TableAction::ResetTable => write!(f, "Table reset for next hand"),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    /// `DEFECT_023`: `generate_player_loses` was an unconditional
    /// `unimplemented!()` on a `#[must_use]` public method.
    #[test]
    fn generate_player_loses__mirrors_a_win() {
        let id = Uuid::nil();
        let win = TableAction::PlayerWins(3, id, Bard::default(), 500, 1_200);

        assert_eq!(
            Some(TableAction::PlayerLoses(3, id, Bard::default(), 500)),
            win.generate_player_loses()
        );
    }

    #[test]
    fn generate_player_loses__none_when_not_a_win() {
        assert_eq!(None, TableAction::Fold(3).generate_player_loses());
    }

    #[test]
    fn test_player_action_fold() {
        let a = PlayerAction::Fold;
        assert_eq!(a, PlayerAction::Fold);
    }

    #[test]
    fn test_player_action_check() {
        assert_eq!(PlayerAction::Check, PlayerAction::Check);
    }

    #[test]
    fn test_player_action_call() {
        assert_eq!(PlayerAction::Call, PlayerAction::Call);
    }

    #[test]
    fn test_player_action_bet() {
        let a = PlayerAction::Bet(300);
        assert_eq!(a, PlayerAction::Bet(300));
        assert_ne!(a, PlayerAction::Bet(200));
    }

    #[test]
    fn test_player_action_raise() {
        let a = PlayerAction::Raise(600);
        assert_eq!(a, PlayerAction::Raise(600));
    }

    #[test]
    fn test_player_action_all_in() {
        assert_eq!(PlayerAction::AllIn, PlayerAction::AllIn);
    }

    #[test]
    fn test_player_action_clone_copy() {
        let a = PlayerAction::Bet(100);
        let b = a; // Copy
        let c = a.clone(); // Clone
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    // P9j.6 — the six-variant Display test lost when src/bot/player_action.rs was
    // deleted (only a one-variant doctest survived). Restored here on the canonical
    // type so every arm of the Display impl is covered.
    #[test]
    fn player_action_display_all_six_variants() {
        assert_eq!("Fold", PlayerAction::Fold.to_string());
        assert_eq!("Check", PlayerAction::Check.to_string());
        assert_eq!("Call", PlayerAction::Call.to_string());
        assert_eq!("Bet(200)", PlayerAction::Bet(200).to_string());
        assert_eq!("Raise(600)", PlayerAction::Raise(600).to_string());
        assert_eq!("AllIn", PlayerAction::AllIn.to_string());
    }

    // Moved here with `TableAction` when the celled event log was retired.
    #[test]
    fn is_result() {
        assert!(TableAction::PlayerWins(0, Uuid::nil(), Bard::default(), 100, 200).is_result());
        assert!(TableAction::PlayerLoses(0, Uuid::nil(), Bard::default(), 100).is_result());
        assert!(TableAction::PlayerWinsMainPot(0, 100).is_result());
        assert!(TableAction::PlayerWinsSidePot(0, 100).is_result());
        assert!(TableAction::PlayerLosesMainPot(0, 100).is_result());
        assert!(TableAction::PlayerLosesSidePot(0, 100).is_result());
        assert!(!TableAction::Bet(0, 100).is_result());
    }
}
