use crate::PKError;
use crate::bard::Bard;
use crate::cards::Cards;
use crate::casino::table::seats::Seats;
use std::cell::RefCell;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
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
    ForcedBetSmallBlind(u8, usize),
    ForcedBetBigBlind(u8, usize),
    BetAnteForced(u8, usize),
    DealingXCards(u8),
    Dealt(u8, Bard),
    DealtPlayers,
    ForceDealt(u8, Bard),
    ActionTo(u8),
    Check(u8),
    Bet(u8, usize),
    Call(u8, usize),
    Raise(u8, usize),
    AllIn(u8, usize),
    Fold(u8),
    MuckCards(Bard),
    MuckPlayerCards(u8, Bard),
    TakePlayerCards(u8, Bard),
    TakeBoardCards(Bard),
    InvalidAction,
    Error(PKError),
}

impl TableAction {
    #[must_use]
    pub fn commentary(&self, name: &str) -> String {
        match self {
            TableAction::ForcedBetSmallBlind(_, amount) => format!("{name} posts {amount} small blind"),
            TableAction::ForcedBetBigBlind(_, amount) => format!("{name} posts {amount} big blind"),
            TableAction::Bet(_, amount) => format!("{name} bets {amount}"),
            TableAction::Call(_, amount) => format!("{name} calls {amount}"),
            TableAction::Raise(_, amount) => format!("{name} raises to {amount}"),
            TableAction::AllIn(_, _) => format!("{name} goes all in."),
            TableAction::Fold(_) => format!("{name} folds"),
            TableAction::Check(_) => format!("{name} checks"),
            TableAction::Dealt(_, bard) => format!("{name} dealt {}", Cards::from(*bard)),
            _ => self.to_string(),
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
            | TableAction::Dealt(seat, _)
            | TableAction::ForceDealt(seat, _)
            | TableAction::ActionTo(seat)
            | TableAction::Check(seat)
            | TableAction::Bet(seat, _)
            | TableAction::Call(seat, _)
            | TableAction::Raise(seat, _)
            | TableAction::AllIn(seat, _)
            | TableAction::Fold(seat)
            | TableAction::MuckPlayerCards(seat, _)
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
                | TableAction::Dealt(_, _)
        )
    }
}

impl Display for TableAction {
    /// ```
    /// use pkcore::bard::Bard;
    /// use pkcore::casino::table::event::TableAction;
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
            TableAction::ForcedBetSmallBlind(seat, amount) => {
                write!(f, "Seat {seat} puts in Small Blind of {amount}")
            }
            TableAction::ForcedBetBigBlind(seat, amount) => {
                write!(f, "Seat {seat} puts in Big Blind of {amount}")
            }
            TableAction::BetAnteForced(seat, amount) => {
                write!(f, "Seat {seat} Antes {amount}")
            }
            TableAction::DealingXCards(x) => write!(f, "Dealing {x} cards to each player"),
            TableAction::Dealt(seat, cards) => write!(f, "Seat {seat} is dealt {}", Cards::from(*cards)),
            TableAction::DealtPlayers => write!(f, "Dealt Players"),
            TableAction::ForceDealt(seat, cards) => {
                write!(f, "Seat {seat} is force-dealt {}", Cards::from(*cards))
            }
            TableAction::ActionTo(seat) => write!(f, "Action to Seat {seat}"),
            TableAction::Check(seat) => write!(f, "Seat {seat} checks"),
            TableAction::Bet(seat, amount) => write!(f, "Seat {seat} bets {amount}"),
            TableAction::Call(seat, amount) => write!(f, "Seat {seat} calls {amount}"),
            TableAction::Raise(seat, amount) => write!(f, "Seat {seat} raises to {amount}"),
            TableAction::AllIn(seat, amount) => write!(f, "Seat {seat} goes all in with {amount}"),
            TableAction::Fold(seat) => write!(f, "Seat {seat} folds"),
            TableAction::MuckCards(cards) => write!(f, "Muck cards: {}", Cards::from(*cards)),
            TableAction::MuckPlayerCards(seat, cards) => {
                write!(f, "Muck player {seat}'s cards: {}", Cards::from(*cards))
            }
            TableAction::TakePlayerCards(seat, cards) => {
                write!(f, "Take player {seat}'s cards: {}", Cards::from(*cards))
            }
            TableAction::TakeBoardCards(cards) => write!(f, "Take board cards: {}", Cards::from(*cards)),
            TableAction::InvalidAction => write!(f, "Invalid Action"),
            TableAction::Error(err) => write!(f, "Error: {err}"),
        }
    }
}

#[derive(Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct TableLog(RefCell<Vec<TableAction>>);

impl TableLog {
    #[must_use]
    pub fn new() -> Self {
        Self(RefCell::new(Vec::new()))
    }

    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    pub fn commentary(&self, seats: &Seats, index: usize) -> Option<String> {
        let player: String = match seats.get_seat(index) {
            None => return None,
            Some(s) => s.player.handle.clone(),
        };

        let last = self.last()?;

        match last {
            TableAction::Bet(_, amount) => Some(format!("{player} bets {amount}")),
            TableAction::Call(_, amount) => Some(format!("{player} calls {amount}")),
            TableAction::Raise(_, amount) => Some(format!("{player} raises to {amount}")),
            TableAction::Fold(_) => Some(format!("{player} folds")),
            TableAction::Check(_) => Some(format!("{player} checks")),
            _ => Some(last.to_string()),
        }
    }

    pub fn last(&self) -> Option<TableAction> {
        self.0.borrow().last().copied()
    }

    pub fn log(&self, action: TableAction) {
        self.0.borrow_mut().push(action);
    }

    #[must_use]
    pub fn entries(&self) -> Vec<TableAction> {
        self.0.borrow().iter().copied().collect()
    }
}

impl Display for TableLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let internal = self.0.borrow();
        let lines: Vec<String> = internal
            .iter()
            .enumerate()
            .map(|(i, action)| format!("{}: {}", i + 1, action))
            .collect();
        write!(f, "{}", lines.join("\n"))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__log_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn display() {
        let log = TableLog::new();

        log.log(TableAction::PlayerSeated(0, Uuid::nil()));
        log.log(TableAction::PlayerSeated(1, Uuid::nil()));
        log.log(TableAction::ForcedBetSmallBlind(0, 50));
        log.log(TableAction::ForcedBetBigBlind(1, 100));
        log.log(TableAction::Dealt(0, Bard::from_str("AS KS").unwrap()));
        log.log(TableAction::Dealt(1, Bard::from_str("KD KC").unwrap()));

        assert_eq!(
            "1: Player 00000000-0000-0000-0000-000000000000 is seated at Seat 0\n2: Player 00000000-0000-0000-0000-000000000000 is seated at Seat 1\n3: Seat 0 puts in Small Blind of 50\n4: Seat 1 puts in Big Blind of 100\n5: Seat 0 is dealt A♠ K♠\n6: Seat 1 is dealt K♦ K♣",
            log.to_string()
        );
    }
}
