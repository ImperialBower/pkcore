use crate::bard::Bard;
use crate::cards::Cards;
use std::cell::RefCell;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub enum TableAction {
    #[default]
    Pause,
    PlayerSeated(u8, Uuid),
    SetButton(u8),
    MoveButton(u8),
    ShuffleNewDeck,
    ForcedBets,
    ForcedBetSmallBlind(u8, usize),
    ForcedBetBigBlind(u8, usize),
    BetAnteForced(u8, usize),
    Dealt(u8, Bard),
    ForceDealt(u8, Bard),
    Bet(u8, usize),
    Call(u8, usize),
    Raise(u8, usize),
    Fold(u8),
    Check(u8),
    TakePlayerCards(Bard),
    TakeBoardCards(Bard),
    InvalidAction,
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
            TableAction::PlayerSeated(seat, player_id) => {
                write!(f, "Player {player_id} is seated at Seat {seat}")
            }
            TableAction::SetButton(seat) => write!(f, "Set Button to Seat {seat}"),
            TableAction::MoveButton(seat) => write!(f, "Move Button to Seat {seat}"),
            TableAction::ShuffleNewDeck => write!(f, "Shuffle New Deck"),
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
            TableAction::Dealt(seat, cards) => write!(f, "Seat {seat} is dealt {}", Cards::from(*cards)),
            TableAction::ForceDealt(seat, cards) => {
                write!(f, "Seat {seat} is force-dealt {}", Cards::from(*cards))
            }
            TableAction::Bet(seat, amount) => write!(f, "Seat {seat} bets {amount}"),
            TableAction::Call(seat, amount) => write!(f, "Seat {seat} calls {amount}"),
            TableAction::Raise(seat, amount) => write!(f, "Seat {seat} raises to {amount}"),
            TableAction::Fold(seat) => write!(f, "Seat {seat} folds"),
            TableAction::Check(seat) => write!(f, "Seat {seat} checks"),
            TableAction::TakePlayerCards(cards) => write!(f, "Take player cards: {}", Cards::from(*cards)),
            TableAction::TakeBoardCards(cards) => write!(f, "Take board cards: {}", Cards::from(*cards)),
            TableAction::InvalidAction => write!(f, "Invalid Action"),
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

    pub fn log(&self, action: TableAction) {
        self.0.borrow_mut().push(action);
    }

    #[must_use]
    pub fn entries(&self) -> Vec<TableAction> {
        self.0.borrow().clone()
    }

    pub fn clear(&self) {
        self.0.borrow_mut().clear();
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
