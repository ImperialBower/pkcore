use crate::pokerbench::action::PokerBenchAction;
use crate::pokerbench::scenario::PokerBenchScenario;

/// A predicted action's score against the solver-optimal label.
///
/// Produced by [`score_action`]. `ev_loss` is a hook for a future solver-equity
/// metric and is currently always `None`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActionScore {
    /// The predicted and optimal action *kinds* match
    /// (fold/check/call/bet/raise/all-in), ignoring size.
    pub type_match: bool,
    /// For a sized optimal action: `|predicted − label|` as a fraction of the
    /// pot. A predicted action with no size (fold/check/call/all-in) counts as
    /// size `0`. `None` when the optimal action itself carries no size.
    pub size_error: Option<f64>,
    /// Optional solver-equity EV-loss; filled when the equity path is enabled.
    /// Always `None` for now (EPIC-43 open decision #3).
    pub ev_loss: Option<f64>,
}

/// Scores a predicted [`PokerBenchAction`] against a scenario's optimal label.
///
/// `type_match` is exact-kind agreement. `size_error` is the pot-normalized chip
/// distance, reported only when the optimal action carries a size; a size-less
/// prediction against a sized optimal is treated as size `0` (so e.g. folding
/// when the solver bets registers the full bet as the error). The pot is
/// floored at the big blind (then `1`) to avoid division by zero.
///
/// # Examples
/// ```
/// use pkcore::pokerbench::{score_action, PokerBenchAction, PokerBenchScenario, PokerBenchSplit};
/// use pkcore::casino::table::position::Position;
///
/// let scenario = PokerBenchScenario {
///     instruction: String::new(),
///     hero: Position::BB,
///     board: vec![],
///     hole: vec![],
///     pot: 6,
///     to_call: 2,
///     big_blind: 1,
///     stacks: vec![(Position::BTN, 100), (Position::BB, 100)],
///     history: vec![],
///     legal: vec![PokerBenchAction::Fold, PokerBenchAction::Call],
///     optimal: PokerBenchAction::Call,
///     split: PokerBenchSplit::Preflop,
/// };
/// let score = score_action(&scenario, PokerBenchAction::Call);
/// assert!(score.type_match);
/// assert_eq!(score.size_error, None); // optimal Call has no size
/// ```
#[must_use]
pub fn score_action(scenario: &PokerBenchScenario, predicted: PokerBenchAction) -> ActionScore {
    let optimal = scenario.optimal;
    let type_match = predicted.same_kind(optimal);

    let size_error = optimal.size().map(|opt| {
        let pred = predicted.size().unwrap_or(0);
        let denom = if scenario.pot == 0 {
            scenario.big_blind.max(1)
        } else {
            scenario.pot
        };
        (f64::from(pred) - f64::from(opt)).abs() / f64::from(denom)
    });

    ActionScore {
        type_match,
        size_error,
        ev_loss: None,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod pokerbench__tests {
    use super::*;
    use crate::casino::table::position::Position;
    use crate::pokerbench::scenario::PokerBenchSplit;

    fn scenario_with(optimal: PokerBenchAction, pot: u32) -> PokerBenchScenario {
        PokerBenchScenario {
            instruction: String::new(),
            hero: Position::BB,
            board: vec![],
            hole: vec![],
            pot,
            to_call: 2,
            big_blind: 1,
            stacks: vec![(Position::BTN, 100), (Position::BB, 100)],
            history: vec![],
            legal: vec![],
            optimal,
            split: PokerBenchSplit::Preflop,
        }
    }

    #[test]
    fn type_match_true_on_same_kind() {
        let s = scenario_with(PokerBenchAction::Call, 6);
        assert!(score_action(&s, PokerBenchAction::Call).type_match);
    }

    #[test]
    fn type_match_false_on_different_kind() {
        let s = scenario_with(PokerBenchAction::Call, 6);
        assert!(!score_action(&s, PokerBenchAction::Fold).type_match);
    }

    #[test]
    fn type_match_ignores_size() {
        let s = scenario_with(PokerBenchAction::Bet(18), 24);
        assert!(score_action(&s, PokerBenchAction::Bet(20)).type_match);
    }

    #[test]
    fn size_error_none_for_sizeless_optimal() {
        let s = scenario_with(PokerBenchAction::Call, 6);
        assert_eq!(score_action(&s, PokerBenchAction::Raise(10)).size_error, None);
    }

    #[test]
    fn size_error_is_pot_normalized() {
        let s = scenario_with(PokerBenchAction::Bet(18), 24);
        let score = score_action(&s, PokerBenchAction::Bet(12));
        // |12 - 18| / 24 = 0.25
        assert!((score.size_error.unwrap() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn size_error_treats_sizeless_prediction_as_zero() {
        let s = scenario_with(PokerBenchAction::Bet(18), 24);
        let score = score_action(&s, PokerBenchAction::Fold);
        // |0 - 18| / 24 = 0.75
        assert!((score.size_error.unwrap() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn size_error_floors_pot_to_avoid_div_by_zero() {
        let s = scenario_with(PokerBenchAction::Bet(2), 0);
        let score = score_action(&s, PokerBenchAction::Bet(0));
        // denom floored to big_blind (1): |0 - 2| / 1 = 2.0
        assert!((score.size_error.unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn ev_loss_is_none_for_now() {
        let s = scenario_with(PokerBenchAction::Bet(18), 24);
        assert_eq!(score_action(&s, PokerBenchAction::Bet(18)).ev_loss, None);
    }
}
