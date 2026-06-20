use crate::pokerbench::error::PokerBenchError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A self-describing poker action mirroring PokerBench's label vocabulary.
///
/// PokerBench solver labels (and legal-move tokens) are short strings: `fold`,
/// `check`, `call`, `bet N`, `raise N`, and `all in`. This enum is the parsed,
/// type-safe form. It is kept separate from any pkcore engine action type so the
/// `pokerbench` module stays additive and analysis-only.
///
/// `Bet` carries the bet size in chips; `Raise` carries the total-to amount in
/// chips. Both are in the dataset's native unit (big blinds — see
/// [`PB_BIG_BLIND`](crate::pokerbench::PB_BIG_BLIND)).
///
/// # Examples
/// ```
/// use std::str::FromStr;
/// use pkcore::pokerbench::PokerBenchAction;
///
/// assert_eq!(PokerBenchAction::from_str("bet 18"), Ok(PokerBenchAction::Bet(18)));
/// assert_eq!(PokerBenchAction::from_str("call"), Ok(PokerBenchAction::Call));
/// assert_eq!(PokerBenchAction::Raise(13).to_string(), "raise 13");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PokerBenchAction {
    /// Fold the hand.
    Fold,
    /// Check (only legal when facing no bet).
    Check,
    /// Call the outstanding bet.
    Call,
    /// Bet `n` chips into an unraised pot.
    Bet(u32),
    /// Raise to a total of `n` chips.
    Raise(u32),
    /// Move all-in.
    AllIn,
}

impl PokerBenchAction {
    /// The chip size carried by this action, if any.
    ///
    /// Returns `Some` for [`Bet`](PokerBenchAction::Bet) and
    /// [`Raise`](PokerBenchAction::Raise); `None` for the size-less actions
    /// (`Fold`/`Check`/`Call`/`AllIn`). Used by the size-error metric in
    /// [`score_action`](crate::pokerbench::score_action).
    ///
    /// # Examples
    /// ```
    /// use pkcore::pokerbench::PokerBenchAction;
    ///
    /// assert_eq!(PokerBenchAction::Bet(20).size(), Some(20));
    /// assert_eq!(PokerBenchAction::Fold.size(), None);
    /// ```
    #[must_use]
    pub fn size(self) -> Option<u32> {
        match self {
            PokerBenchAction::Bet(n) | PokerBenchAction::Raise(n) => Some(n),
            _ => None,
        }
    }

    /// Whether this action is the *same kind* as `other`, ignoring any size.
    ///
    /// `Bet(18)` and `Bet(20)` are the same kind; `Bet(18)` and `Raise(18)` are
    /// not. This is the action-accuracy comparison used by
    /// [`score_action`](crate::pokerbench::score_action).
    ///
    /// # Examples
    /// ```
    /// use pkcore::pokerbench::PokerBenchAction;
    ///
    /// assert!(PokerBenchAction::Bet(18).same_kind(PokerBenchAction::Bet(20)));
    /// assert!(!PokerBenchAction::Bet(18).same_kind(PokerBenchAction::Raise(18)));
    /// ```
    #[must_use]
    pub fn same_kind(self, other: PokerBenchAction) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&other)
    }
}

/// Parses a non-negative chip amount, tolerating a trailing decimal (`18` or
/// `18.0`), the form PokerBench uses for pot sizes. Shared by [`FromStr`] here
/// and by the loaders' `pot_size` parsing.
// The `as u32` is guarded by the `is_finite() && >= 0.0` filter and saturates on
// overflow (well-defined since Rust 1.45), so truncation/sign-loss can't produce
// an invalid value for the small integer chip counts PokerBench emits.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn parse_chips(token: &str) -> Result<u32, PokerBenchError> {
    token
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite() && *n >= 0.0)
        .map(|n| n.round() as u32)
        .ok_or_else(|| PokerBenchError::Action(token.to_string()))
}

impl FromStr for PokerBenchAction {
    type Err = PokerBenchError;

    /// Parses a PokerBench label or legal-move token into a `PokerBenchAction`.
    ///
    /// Accepts (case-insensitively, whitespace-trimmed): `fold`, `check`,
    /// `call`, `all in` / `allin` / `all-in`, `bet N` / `raise N`, and the
    /// dataset's bare to-amount form `Nbb` (a pre-flop raise to `N` big blinds),
    /// where `N` is a non-negative number.
    ///
    /// # Errors
    /// Returns [`PokerBenchError::Action`] if the token is not a recognized
    /// action or a sized action carries an unparsable amount.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowered = s.trim().to_lowercase();
        match lowered.as_str() {
            "fold" => Ok(PokerBenchAction::Fold),
            "check" => Ok(PokerBenchAction::Check),
            "call" => Ok(PokerBenchAction::Call),
            "all in" | "allin" | "all-in" => Ok(PokerBenchAction::AllIn),
            other => {
                if let Some(rest) = other.strip_prefix("bet ") {
                    Ok(PokerBenchAction::Bet(parse_chips(rest)?))
                } else if let Some(rest) = other.strip_prefix("raise ") {
                    Ok(PokerBenchAction::Raise(parse_chips(rest)?))
                } else if let Some(rest) = other.strip_suffix("bb") {
                    // Bare to-amount, e.g. "3.0bb": PokerBench's pre-flop raise label.
                    Ok(PokerBenchAction::Raise(parse_chips(rest)?))
                } else {
                    Err(PokerBenchError::Action(s.to_string()))
                }
            }
        }
    }
}

impl fmt::Display for PokerBenchAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PokerBenchAction::Fold => write!(f, "fold"),
            PokerBenchAction::Check => write!(f, "check"),
            PokerBenchAction::Call => write!(f, "call"),
            PokerBenchAction::Bet(n) => write!(f, "bet {n}"),
            PokerBenchAction::Raise(n) => write!(f, "raise {n}"),
            PokerBenchAction::AllIn => write!(f, "all in"),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod pokerbench__tests {
    use super::*;

    #[test]
    fn from_str_fold() {
        assert_eq!(PokerBenchAction::from_str("fold"), Ok(PokerBenchAction::Fold));
    }

    #[test]
    fn from_str_check_call() {
        assert_eq!(PokerBenchAction::from_str("check"), Ok(PokerBenchAction::Check));
        assert_eq!(PokerBenchAction::from_str("call"), Ok(PokerBenchAction::Call));
    }

    #[test]
    fn from_str_bet_with_size() {
        assert_eq!(PokerBenchAction::from_str("bet 18"), Ok(PokerBenchAction::Bet(18)));
    }

    #[test]
    fn from_str_raise_with_size() {
        assert_eq!(PokerBenchAction::from_str("raise 13"), Ok(PokerBenchAction::Raise(13)));
    }

    #[test]
    fn from_str_all_in_variants() {
        assert_eq!(PokerBenchAction::from_str("all in"), Ok(PokerBenchAction::AllIn));
        assert_eq!(PokerBenchAction::from_str("allin"), Ok(PokerBenchAction::AllIn));
        assert_eq!(PokerBenchAction::from_str("all-in"), Ok(PokerBenchAction::AllIn));
    }

    #[test]
    fn from_str_is_case_and_whitespace_insensitive() {
        assert_eq!(PokerBenchAction::from_str("  BET 42 "), Ok(PokerBenchAction::Bet(42)));
        assert_eq!(PokerBenchAction::from_str("CALL"), Ok(PokerBenchAction::Call));
    }

    #[test]
    fn from_str_tolerates_decimal_size() {
        assert_eq!(PokerBenchAction::from_str("bet 18.0"), Ok(PokerBenchAction::Bet(18)));
    }

    #[test]
    fn from_str_bare_bb_amount_is_raise() {
        assert_eq!(PokerBenchAction::from_str("3.0bb"), Ok(PokerBenchAction::Raise(3)));
        assert_eq!(PokerBenchAction::from_str("14.0bb"), Ok(PokerBenchAction::Raise(14)));
    }

    #[test]
    fn from_str_unknown_token_is_err() {
        assert_eq!(
            PokerBenchAction::from_str("shove everything"),
            Err(PokerBenchError::Action("shove everything".to_string()))
        );
    }

    #[test]
    fn from_str_bet_without_size_is_err() {
        assert!(PokerBenchAction::from_str("bet abc").is_err());
    }

    #[test]
    fn display_round_trips_through_from_str() {
        let actions = [
            PokerBenchAction::Fold,
            PokerBenchAction::Check,
            PokerBenchAction::Call,
            PokerBenchAction::Bet(24),
            PokerBenchAction::Raise(13),
            PokerBenchAction::AllIn,
        ];
        for action in actions {
            let rendered = action.to_string();
            assert_eq!(PokerBenchAction::from_str(&rendered), Ok(action));
        }
    }

    #[test]
    fn size_some_for_sized_actions() {
        assert_eq!(PokerBenchAction::Bet(20).size(), Some(20));
        assert_eq!(PokerBenchAction::Raise(55).size(), Some(55));
    }

    #[test]
    fn size_none_for_sizeless_actions() {
        assert_eq!(PokerBenchAction::Fold.size(), None);
        assert_eq!(PokerBenchAction::Check.size(), None);
        assert_eq!(PokerBenchAction::Call.size(), None);
        assert_eq!(PokerBenchAction::AllIn.size(), None);
    }

    #[test]
    fn same_kind_ignores_size() {
        assert!(PokerBenchAction::Bet(18).same_kind(PokerBenchAction::Bet(20)));
        assert!(PokerBenchAction::Raise(5).same_kind(PokerBenchAction::Raise(99)));
    }

    #[test]
    fn same_kind_distinguishes_variants() {
        assert!(!PokerBenchAction::Bet(18).same_kind(PokerBenchAction::Raise(18)));
        assert!(!PokerBenchAction::Fold.same_kind(PokerBenchAction::Check));
    }

    #[test]
    fn parse_chips_rejects_negative() {
        assert!(parse_chips("-5").is_err());
    }
}
