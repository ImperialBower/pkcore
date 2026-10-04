//! The one `World` every scenario starts from.
//!
//! cucumber-rs builds a fresh `PokerWorld` per scenario, so nothing leaks
//! between them. Each feature area owns its own slice of state; a scenario
//! only ever touches the slices its steps need.

use std::collections::HashMap;

use cucumber::World;
use pkcore::analysis::equity::{EquityOptions, EquityReport, PlayerSpec};
use pkcore::analysis::gto::twos::Twos;
use pkcore::casino::winnings::Winnings;
use pkcore::prelude::*;

#[derive(Debug, Default, World)]
pub struct PokerWorld {
    // ── cards ─────────────────────────────────────────────────────────────
    pub card: Option<Result<Card, PKError>>,
    pub cards: Option<Result<Cards, PKError>>,

    // ── hand evaluation ───────────────────────────────────────────────────
    /// The most recent evaluation, the subject of "the hand is …".
    pub eval: Option<Result<Eval, PKError>>,
    /// Evaluations kept under a name for head-to-head comparisons.
    pub named: HashMap<String, Eval>,

    // ── ranges ────────────────────────────────────────────────────────────
    pub range: Option<Result<Twos, PKError>>,

    // ── equity ────────────────────────────────────────────────────────────
    pub players: Vec<PlayerSpec>,
    pub board: Board,
    pub opts: EquityOptions,
    pub report: Option<Result<EquityReport, PKError>>,
    /// The report before the most recent one, for repeatability checks.
    pub previous_report: Option<Result<EquityReport, PKError>>,

    // ── table ─────────────────────────────────────────────────────────────
    pub table: Option<Table>,
    /// Chips on the table when it was set up; conservation is checked against it.
    pub chips_at_start: usize,
    pub last_action: Option<Result<usize, PKError>>,
    pub winnings: Option<Result<Winnings, PKError>>,
}

impl PokerWorld {
    /// The table a scenario set up. Panics (fails the step) when there is none,
    /// which can only mean the feature file is missing its `Given`.
    pub fn table(&self) -> &Table {
        self.table
            .as_ref()
            .expect("no table — the scenario needs a `Given … table` step")
    }

    pub fn table_mut(&mut self) -> &mut Table {
        self.table
            .as_mut()
            .expect("no table — the scenario needs a `Given … table` step")
    }

    /// The seat number of the player with this handle.
    pub fn seat_of(&self, name: &str) -> u8 {
        let table = self.table();
        (0..table.seats.size())
            .find(|&i| table.seats.get_seat(i).is_some_and(|s| s.player.handle == name))
            .unwrap_or_else(|| panic!("no player named {name:?} at the table"))
    }

    pub fn eval(&self) -> &Eval {
        match self.eval.as_ref().expect("no hand has been evaluated") {
            Ok(eval) => eval,
            Err(e) => panic!("the hand could not be evaluated: {e:?}"),
        }
    }

    pub fn report(&self) -> &EquityReport {
        match self.report.as_ref().expect("no equity has been calculated") {
            Ok(report) => report,
            Err(e) => panic!("the equity calculation failed: {e:?}"),
        }
    }
}

/// Reduces a phrase to lowercase letters and digits, so the feature files can
/// say "four of a kind" or "Aces over Kings" and still match the `Debug` names
/// `FourOfAKind` and `AcesOverKings`.
pub fn squash(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}
