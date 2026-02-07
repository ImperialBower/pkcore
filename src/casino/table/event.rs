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
    ForcedBet(u8, usize),
    ForcedBetSmallBlind(u8, usize),
    ForcedBetBigBlind(u8, usize),
    BetAnteForced(u8, usize),
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
    Showdown(u8),
    PlayerMucksCards(u8), // At a showdown one player mucks their cards rather than show them.
    AllFoldedTo(u8),
    PlayerWinsSidePot(u8, usize),
    PlayerWinsMainPot(u8, usize),
    PlayerLosesSidePot(u8, usize),
    PlayerLosesMainPot(u8, usize),
    PlayerWins(u8, Uuid, Bard, usize), // (seat, player_id, winning_hand, amount_won, in_showdown)
    PlayerLoses(u8, Uuid, Bard, usize), // (seat, player_id, winning_hand, amount_lost, in_showdown)
    InvalidAction,
    Error(PKError),
    DeckPassesAudit,
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
            TableAction::PlayerWins(seat, _, bard, winnings) => {
                format!("{name} (Seat {seat}) wins {winnings} with {}", Cards::from(*bard))
            }
            TableAction::PlayerLoses(seat, _, bard, losses) => {
                format!("{name} (Seat {seat}) loses {losses} with {}", Cards::from(*bard))
            }
            _ => self.to_string(),
        }
    }

    #[must_use]
    pub fn generate_player_loses(&self) -> TableAction {
        todo!()
    }

    #[must_use]
    pub fn get_ammount(&self) -> Option<usize> {
        match self {
            TableAction::ForcedBet(_, amount)
            | TableAction::ForcedBetSmallBlind(_, amount)
            | TableAction::ForcedBetBigBlind(_, amount)
            | TableAction::BetAnteForced(_, amount)
            | TableAction::BringItIn(amount)
            | TableAction::Bet(_, amount)
            | TableAction::Call(_, amount)
            | TableAction::Raise(_, amount)
            | TableAction::AllIn(_, amount)
            | TableAction::CloseItOut(amount)
            | TableAction::PotSize(amount)
            | TableAction::MainPot(amount)
            | TableAction::SidePot(amount)
            | TableAction::PlayerWins(_, _, _, amount)
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
            | TableAction::PlayerWins(seat, _, _, _)
            | TableAction::PlayerLoses(seat, _, _, _)
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
                | TableAction::ClosesTheAction(_)
        )
    }

    #[must_use]
    pub fn is_result(&self) -> bool {
        matches!(
            self,
            Self::PlayerWins(_, _, _, _)
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
            TableAction::PlayerWins(seat, player_id, winning_hand, amount_won) => write!(
                f,
                "Seat {seat} (Player {player_id}) wins {amount_won} with {}",
                Cards::from(*winning_hand)
            ),
            TableAction::PlayerLoses(seat, player_id, losing_hand, amount_lost) => write!(
                f,
                "Seat {seat} (Player {player_id}) loses {amount_lost} with {}",
                Cards::from(*losing_hand)
            ),
            TableAction::InvalidAction => write!(f, "Invalid Action"),
            TableAction::Error(err) => write!(f, "Error: {err}"),
            TableAction::DeckPassesAudit => write!(f, "Deck passes audit"),
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

    pub fn commentary(&self, seats: &Seats, index: u8) -> Option<String> {
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

    #[must_use]
    pub fn entries(&self) -> Vec<TableAction> {
        self.0.borrow().iter().copied().collect()
    }

    pub fn get(&self, index: usize) -> Option<TableAction> {
        self.0.borrow().get(index).copied()
    }

    pub fn iter_reverse(&self) -> impl Iterator<Item = TableAction> {
        self.0.borrow().iter().rev().copied().collect::<Vec<_>>().into_iter()
    }

    pub fn last(&self) -> Option<TableAction> {
        self.0.borrow().last().copied()
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let log = TableLog::new();
    /// log.log(TableAction::Bet(0, 200));
    /// log.log(TableAction::Raise(1, 400));
    ///
    /// let last_player_action = log.last_player_action().unwrap();
    ///
    /// assert_eq!(last_player_action, TableAction::Raise(1, 400));
    /// ```
    pub fn last_player_action(&self) -> Option<TableAction> {
        self.iter_reverse().find(|&action| action.is_player_action())
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn log(&self, action: TableAction) {
        self.0.borrow_mut().push(action);
    }

    pub fn result_actions(&self) -> Vec<TableAction> {
        self.0
            .borrow()
            .iter()
            .filter(|action| action.is_result())
            .copied()
            .collect()
    }

    pub fn is_results_only(&self) -> bool {
        let internal = self.0.borrow();
        internal.iter().all(TableAction::is_result)
    }

    pub fn iter(&self) -> std::vec::IntoIter<TableAction> {
        <&Self as IntoIterator>::into_iter(self)
    }

    #[must_use]
    pub fn results_only(&self) -> TableLog {
        let results: Vec<TableAction> = self.result_actions();
        TableLog::from(results)
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

impl From<Vec<TableAction>> for TableLog {
    fn from(actions: Vec<TableAction>) -> Self {
        Self(RefCell::new(actions))
    }
}

/// # Diary
///
/// One of the things that I am trying to make myself do is type out the suggestions that
/// `CoPi` makes. Composers like Mozart learned how to compose my copying and rearranging
/// compositions from other composers. In his case, taking the sonatas of J.C.Bach and turning
/// them into piano concertos. For many other composers, they transcribed the keyboard sonatas
/// of Scarlatti and turned them into chamber concertos. (See Charles Avison)
impl IntoIterator for TableLog {
    type Item = TableAction;
    type IntoIter = std::vec::IntoIter<TableAction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_inner().into_iter()
    }
}

impl IntoIterator for &TableLog {
    type Item = TableAction;
    type IntoIter = std::vec::IntoIter<TableAction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.borrow().iter().copied().collect::<Vec<_>>().into_iter()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__log_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn last_player_action() {
        let log = TableLog::new();
        log.log(TableAction::PlayerSeated(0, Uuid::nil()));
        log.log(TableAction::PlayerSeated(1, Uuid::nil()));
        log.log(TableAction::ForcedBetSmallBlind(0, 50));
        log.log(TableAction::ForcedBetBigBlind(1, 100));
        log.log(TableAction::Dealt(0, Bard::from_str("AS KS").unwrap()));
        log.log(TableAction::Dealt(1, Bard::from_str("KD KC").unwrap()));
        log.log(TableAction::ActionTo(0));
        log.log(TableAction::Bet(0, 200));
        log.log(TableAction::ActionTo(1));
        log.log(TableAction::Call(1, 200));
        log.log(TableAction::NewHand);

        let last_player_action = log.last_player_action().unwrap();

        assert_eq!(last_player_action, TableAction::Call(1, 200));
        assert!(TableLog::new().last_player_action().is_none());
    }

    #[test]
    fn is_result() {
        assert!(TableAction::PlayerWins(0, Uuid::nil(), Bard::default(), 100).is_result());
        assert!(TableAction::PlayerLoses(0, Uuid::nil(), Bard::default(), 100).is_result());
        assert!(TableAction::PlayerWinsMainPot(0, 100).is_result());
        assert!(TableAction::PlayerWinsSidePot(0, 100).is_result());
        assert!(TableAction::PlayerLosesMainPot(0, 100).is_result());
        assert!(TableAction::PlayerLosesSidePot(0, 100).is_result());
        assert!(!TableAction::Bet(0, 100).is_result());
    }

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

    #[test]
    fn result_actions() {
        let log = TableLog::new();

        log.log(TableAction::PlayerSeated(0, Uuid::nil()));
        log.log(TableAction::PlayerSeated(1, Uuid::nil()));
        log.log(TableAction::PlayerWins(
            0,
            Uuid::nil(),
            Bard::from_str("AS KS").unwrap(),
            100,
        ));
        log.log(TableAction::PlayerLoses(
            1,
            Uuid::nil(),
            Bard::from_str("KD KC").unwrap(),
            100,
        ));

        let results = log.result_actions();

        assert_eq!(results.len(), 2);
        assert_eq!(
            results,
            vec![
                TableAction::PlayerWins(0, Uuid::nil(), Bard::from_str("AS KS").unwrap(), 100),
                TableAction::PlayerLoses(1, Uuid::nil(), Bard::from_str("KD KC").unwrap(), 100)
            ]
        );
    }

    #[test]
    fn results_only() {
        let log = TableLog::new();

        log.log(TableAction::PlayerSeated(0, Uuid::nil()));
        log.log(TableAction::PlayerSeated(1, Uuid::nil()));
        log.log(TableAction::PlayerWins(
            0,
            Uuid::nil(),
            Bard::from_str("AS KS").unwrap(),
            100,
        ));
        log.log(TableAction::PlayerLoses(
            1,
            Uuid::nil(),
            Bard::from_str("KD KC").unwrap(),
            100,
        ));

        let results = log.results_only();

        assert_eq!(results.len(), 2);
        assert_eq!(
            results,
            TableLog::from(vec![
                TableAction::PlayerWins(0, Uuid::nil(), Bard::from_str("AS KS").unwrap(), 100),
                TableAction::PlayerLoses(1, Uuid::nil(), Bard::from_str("KD KC").unwrap(), 100)
            ])
        );
    }
}
