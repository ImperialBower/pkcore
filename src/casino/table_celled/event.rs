//! [`TableLog`] — the interior-mutability event log used by
//! [`TableCelled`](crate::casino::table_celled::TableCelled).
//!
//! The entry type itself, [`TableAction`], lives in
//! [`casino::action`](crate::casino::action) since both table engines share it.

use crate::casino::action::TableAction;
use crate::casino::table_celled::seats::SeatsCell;
use std::cell::RefCell;
use std::fmt::Display;

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

    pub fn commentary(&self, seats: &SeatsCell, index: u8) -> Option<String> {
        let player: String = seats.get_seat(index)?.player.handle.clone();

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

    /// Returns the first occurrence of a specific `TableAction` variant that matches the predicate.
    ///
    /// ```
    /// use pkcore::casino::action::TableAction;
    /// use pkcore::casino::table_celled::event::TableLog;
    ///
    /// let log = TableLog::new();
    /// log.log(TableAction::Bet(0, 200));
    /// log.log(TableAction::Raise(1, 400));
    /// log.log(TableAction::Fold(2));
    ///
    /// let raise_action = log.find_action(|action| matches!(action, TableAction::Raise(_, _)));
    /// assert_eq!(raise_action, Some(TableAction::Raise(1, 400)));
    /// ```
    pub fn find_action<F>(&self, predicate: F) -> Option<TableAction>
    where
        F: Fn(&TableAction) -> bool,
    {
        self.0.borrow().iter().find(|&action| predicate(action)).copied()
    }

    /// ```
    /// use pkcore::casino::action::TableAction;
    /// use pkcore::casino::table_celled::event::TableLog;
    ///
    /// let log = TableLog::new();
    /// log.log(TableAction::ForcedBetBigBlind(0, 500));
    /// log.log(TableAction::Bet(1, 200));
    ///
    /// assert_eq!(TableAction::ForcedBetBigBlind(0, 500), log.get_action_big_blind().unwrap());
    /// ```
    pub fn get_action_big_blind(&self) -> Option<TableAction> {
        self.find_action(|action| matches!(action, TableAction::ForcedBetBigBlind(_, _)))
    }

    /// ```
    /// use pkcore::casino::action::TableAction;
    /// use pkcore::casino::table_celled::event::TableLog;
    ///
    /// let log = TableLog::new();
    /// log.log(TableAction::ForcedBetSmallBlind(0, 500));
    /// log.log(TableAction::Bet(1, 200));
    ///
    /// assert_eq!(TableAction::ForcedBetSmallBlind(0, 500), log.get_action_small_blind().unwrap());
    /// ```
    pub fn get_action_small_blind(&self) -> Option<TableAction> {
        self.find_action(|action| matches!(action, TableAction::ForcedBetSmallBlind(_, _)))
    }

    pub fn has_player_action(&self) -> bool {
        self.0.borrow().iter().any(TableAction::is_player_action)
    }

    /// ```
    /// use pkcore::casino::action::TableAction;
    /// use pkcore::casino::table_celled::event::TableLog;
    ///
    /// let log = TableLog::new();
    /// assert!(!log.have_posted_blinds());
    ///
    /// log.log(TableAction::ForcedBetSmallBlind(0, 500));
    /// assert!(!log.have_posted_blinds());
    ///
    /// log.log(TableAction::ForcedBetBigBlind(0, 500));
    /// assert!(log.have_posted_blinds());
    /// ```
    pub fn have_posted_blinds(&self) -> bool {
        self.get_action_big_blind().is_some() && self.get_action_small_blind().is_some()
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
mod casino__table_celled__log_tests {
    use super::*;
    use crate::bard::Bard;
    use std::str::FromStr;
    use uuid::Uuid;

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
            200,
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
                TableAction::PlayerWins(0, Uuid::nil(), Bard::from_str("AS KS").unwrap(), 100, 200),
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
            200,
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
                TableAction::PlayerWins(0, Uuid::nil(), Bard::from_str("AS KS").unwrap(), 100, 200),
                TableAction::PlayerLoses(1, Uuid::nil(), Bard::from_str("KD KC").unwrap(), 100)
            ])
        );
    }
}
