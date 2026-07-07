//! Betting-rule abstraction orthogonal to [`crate::games::GameType`].
//!
//! `BettingStructure` lifts the inline min-raise / max-raise math out of
//! [`crate::casino::table::Table::act_raise`] into an enum
//! dispatch so Fixed-Limit (EPIC-30) and Pot-Limit (EPIC-31) variants can
//! plug in without forking the betting loop. The `NoLimit` arm preserves
//! today's NLHE behavior verbatim: `min_raise` returns the previous raise
//! increment if any, else the big blind; `max_raise` returns the player's
//! stack (effectively unlimited).
//!
//! Phase 1 of EPIC-29 introduces the type with NLHE-correct semantics on
//! the `NoLimit` arm and runnable placeholder semantics on the `PotLimit`
//! and `FixedLimit` arms. Phase 7 wires it into `TableNoCell::act_raise`.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Per-street betting tier for variants that use small-bet / big-bet
/// fixed increments (Limit Hold'em, Limit Stud).
///
/// In Hold'em variants, preflop and flop use `Small`; turn and river use
/// `Big`. In Stud variants, 3rd–4th streets use `Small`; 5th onward use
/// `Big`. No-Limit and Pot-Limit games ignore this tier; pass `Small` as
/// a default.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum BetTier {
    /// Small-bet increment (early streets).
    #[default]
    Small,
    /// Big-bet increment (later streets).
    Big,
}

/// Betting structure: no-limit, pot-limit, or fixed-limit. Orthogonal to
/// the game family ([`crate::games::GameFamily`]).
///
/// # Examples
///
/// ```
/// use pkcore::games::betting_structure::BettingStructure;
///
/// let nl = BettingStructure::NoLimit;
/// // First raise on the street: increment is the big blind.
/// assert_eq!(100, nl.min_raise(0, 100));
/// // Subsequent raise: increment is the previous raise.
/// assert_eq!(200, nl.min_raise(200, 100));
/// ```
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum BettingStructure {
    /// No-Limit: any size between min-raise (last raise or BB) and stack.
    #[default]
    NoLimit,
    /// Pot-Limit: maximum raise = pot after call. Used by Pot-Limit Omaha.
    PotLimit,
    /// Fixed-Limit: small/big-bet tiered increments with a raise cap.
    FixedLimit {
        /// Small-bet increment (used on early streets).
        small_bet: usize,
        /// Big-bet increment (used on later streets).
        big_bet: usize,
        /// Maximum number of raises permitted per street (typical: 3).
        raise_cap: u8,
    },
}

impl BettingStructure {
    /// Minimum legal raise increment (delta above the current bet) given
    /// the previous raise size on this street.
    ///
    /// - `NoLimit`: matches today's `TableNoCell::min_raise` inline math:
    ///   returns `last_raise` if non-zero, else `big_blind`.
    /// - `PotLimit`: same min-raise rule as `NoLimit`; pot-limit only caps
    ///   the *maximum*, not the minimum.
    /// - `FixedLimit`: returns the tier's fixed increment regardless of
    ///   `last_raise`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::betting_structure::{BettingStructure, BetTier};
    ///
    /// // No-Limit: first raise on the street → BB.
    /// assert_eq!(100, BettingStructure::NoLimit.min_raise(0, 100));
    /// // No-Limit: subsequent raise → previous raise increment.
    /// assert_eq!(200, BettingStructure::NoLimit.min_raise(200, 100));
    ///
    /// // Fixed-Limit: ignores last_raise; returns tier increment.
    /// let fl = BettingStructure::FixedLimit { small_bet: 100, big_bet: 200, raise_cap: 3 };
    /// assert_eq!(100, fl.min_raise_for_tier(0, BetTier::Small));
    /// assert_eq!(200, fl.min_raise_for_tier(0, BetTier::Big));
    /// ```
    #[must_use]
    pub fn min_raise(&self, last_raise: usize, big_blind: usize) -> usize {
        match self {
            BettingStructure::NoLimit | BettingStructure::PotLimit => {
                if last_raise > 0 {
                    last_raise
                } else {
                    big_blind
                }
            }
            BettingStructure::FixedLimit { small_bet, .. } => *small_bet,
        }
    }

    /// Tier-aware min-raise for fixed-limit structures. Equivalent to
    /// [`Self::min_raise`] for No-Limit / Pot-Limit (the tier is ignored).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::betting_structure::{BettingStructure, BetTier};
    ///
    /// let fl = BettingStructure::FixedLimit { small_bet: 100, big_bet: 200, raise_cap: 3 };
    /// assert_eq!(100, fl.min_raise_for_tier(0, BetTier::Small));
    /// assert_eq!(200, fl.min_raise_for_tier(0, BetTier::Big));
    /// ```
    #[must_use]
    pub fn min_raise_for_tier(&self, last_raise: usize, tier: BetTier) -> usize {
        match self {
            BettingStructure::FixedLimit { small_bet, big_bet, .. } => match tier {
                BetTier::Small => *small_bet,
                BetTier::Big => *big_bet,
            },
            _ => self.min_raise(last_raise, 0),
        }
    }

    /// Maximum legal raise *amount* (absolute, not delta) relative to the
    /// player's stack.
    ///
    /// - `NoLimit`: returns `stack` (player can move all-in).
    /// - `PotLimit`: returns `min(stack, current_bet + pot_after_call)`
    ///   where `pot_after_call = pot + call_amount`.
    /// - `FixedLimit`: returns the single legal raise-to for the tier —
    ///   `current_bet + tier_increment`, except when `current_bet` is a
    ///   partial forced bet below one full bet (the stud bring-in), where it
    ///   returns one full `tier_increment` (completion). Capped at `stack`.
    ///   In fixed-limit this equals the *minimum* legal raise, so there is
    ///   exactly one legal amount.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::betting_structure::{BettingStructure, BetTier};
    ///
    /// let nl = BettingStructure::NoLimit;
    /// // No-Limit: max raise = player's full stack.
    /// assert_eq!(5_000, nl.max_raise(1_000, 100, 50, 5_000, BetTier::Small));
    ///
    /// let pl = BettingStructure::PotLimit;
    /// // Pot-Limit: max raise = current_bet + pot + call_amount, capped at stack.
    /// // pot=1000, current_bet=100, my_committed=0 → call=100, max=100+1000+100=1200
    /// assert_eq!(1_200, pl.max_raise(1_000, 100, 0, 5_000, BetTier::Small));
    ///
    /// let fl = BettingStructure::FixedLimit { small_bet: 20, big_bet: 40, raise_cap: 4 };
    /// // Completion: only the 5 bring-in is in, so the raise-to is one full
    /// // small bet (20), not 5 + 20.
    /// assert_eq!(20, fl.max_raise(0, 5, 0, 5_000, BetTier::Small));
    /// // Once a full bet is in, the raise-to steps by the tier increment.
    /// assert_eq!(40, fl.max_raise(0, 20, 0, 5_000, BetTier::Small));
    /// ```
    #[must_use]
    pub fn max_raise(&self, pot: usize, current_bet: usize, my_committed: usize, stack: usize, tier: BetTier) -> usize {
        match self {
            BettingStructure::NoLimit => stack,
            BettingStructure::PotLimit => {
                let call_amount = current_bet.saturating_sub(my_committed);
                let pot_max = current_bet.saturating_add(pot).saturating_add(call_amount);
                pot_max.min(stack)
            }
            BettingStructure::FixedLimit { small_bet, big_bet, .. } => {
                let increment = match tier {
                    BetTier::Small => *small_bet,
                    BetTier::Big => *big_bet,
                };
                // The sole legal fixed-limit raise amount is the completion-aware
                // target — shared with `min_raise_to` via `completion_raise_to`
                // so min and max cannot drift (audit P9j.2).
                Self::completion_raise_to(current_bet, increment).min(stack)
            }
        }
    }

    /// The completion-aware minimum raise-*to* target given the bet currently on
    /// the table and one full `increment` for the street.
    ///
    /// With only a partial forced bet (a stud bring-in) below one full increment
    /// in front of the actor, the raise **completes** to one full increment;
    /// otherwise it steps *by* the increment on top of the current bet. This one
    /// rule is shared by
    /// [`TableNoCell::min_raise_to`](crate::casino::table::Table::min_raise_to)
    /// and the fixed-limit arm of [`Self::max_raise`], so the minimum and the
    /// (fixed-limit) maximum are computed from the same source and cannot drift
    /// (audit P9j.2).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// // Only a 5 bring-in is in: completion targets one full 20 small bet.
    /// assert_eq!(20, BettingStructure::completion_raise_to(5, 20));
    /// // A full bet is already in: step up by the increment.
    /// assert_eq!(40, BettingStructure::completion_raise_to(20, 20));
    /// ```
    #[must_use]
    pub fn completion_raise_to(current_bet: usize, increment: usize) -> usize {
        if current_bet < increment {
            increment
        } else {
            current_bet.saturating_add(increment)
        }
    }

    /// True if the per-street raise cap has been reached. No-Limit and
    /// Pot-Limit have no cap.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// assert!(!BettingStructure::NoLimit.cap_reached(100));
    /// let fl = BettingStructure::FixedLimit { small_bet: 100, big_bet: 200, raise_cap: 3 };
    /// assert!(!fl.cap_reached(2));
    /// assert!(fl.cap_reached(3));
    /// ```
    #[must_use]
    pub fn cap_reached(&self, raises_this_street: u8) -> bool {
        match self {
            BettingStructure::NoLimit | BettingStructure::PotLimit => false,
            BettingStructure::FixedLimit { raise_cap, .. } => raises_this_street >= *raise_cap,
        }
    }

    /// True if this is the `NoLimit` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// assert!(BettingStructure::NoLimit.is_no_limit());
    /// assert!(!BettingStructure::PotLimit.is_no_limit());
    /// ```
    #[must_use]
    pub fn is_no_limit(&self) -> bool {
        matches!(self, BettingStructure::NoLimit)
    }
}

impl Display for BettingStructure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BettingStructure::NoLimit => write!(f, "No-Limit"),
            BettingStructure::PotLimit => write!(f, "Pot-Limit"),
            BettingStructure::FixedLimit {
                small_bet,
                big_bet,
                raise_cap,
            } => write!(f, "Fixed-Limit ({small_bet}/{big_bet}, cap {raise_cap})"),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod games__betting_structure__tests {
    use super::*;

    // ---- No-Limit min_raise matches the inline NLHE math exactly. ----

    #[test]
    fn no_limit_min_raise_first_raise_returns_big_blind() {
        // Mirrors TableNoCell::min_raise at table_no_cell.rs:1572-1578
        // when raise_increment == 0: returns forced.big_blind.
        let nl = BettingStructure::NoLimit;
        assert_eq!(100, nl.min_raise(0, 100));
    }

    #[test]
    fn no_limit_min_raise_subsequent_returns_last_raise() {
        // Mirrors TableNoCell::min_raise when raise_increment > 0.
        let nl = BettingStructure::NoLimit;
        assert_eq!(200, nl.min_raise(200, 100));
        assert_eq!(50, nl.min_raise(50, 100));
    }

    #[test]
    fn no_limit_max_raise_is_stack() {
        let nl = BettingStructure::NoLimit;
        assert_eq!(5_000, nl.max_raise(1_000, 100, 50, 5_000, BetTier::Small));
        // Stack of 0 → no raise possible.
        assert_eq!(0, nl.max_raise(1_000, 100, 50, 0, BetTier::Small));
    }

    #[test]
    fn no_limit_cap_never_reached() {
        let nl = BettingStructure::NoLimit;
        assert!(!nl.cap_reached(0));
        assert!(!nl.cap_reached(100));
        assert!(!nl.cap_reached(u8::MAX));
    }

    // ---- Pot-Limit ----

    #[test]
    fn pot_limit_min_raise_same_as_no_limit() {
        let pl = BettingStructure::PotLimit;
        assert_eq!(100, pl.min_raise(0, 100));
        assert_eq!(200, pl.min_raise(200, 100));
    }

    #[test]
    fn pot_limit_max_raise_is_current_bet_plus_pot_plus_call() {
        // pot=1000, current_bet=100, my_committed=0
        // call_amount = 100, max = 100 + 1000 + 100 = 1200
        let pl = BettingStructure::PotLimit;
        assert_eq!(1_200, pl.max_raise(1_000, 100, 0, 5_000, BetTier::Small));
    }

    #[test]
    fn pot_limit_max_raise_capped_at_stack() {
        let pl = BettingStructure::PotLimit;
        // Same math but stack is smaller than the pot-limit ceiling.
        assert_eq!(500, pl.max_raise(1_000, 100, 0, 500, BetTier::Small));
    }

    #[test]
    fn pot_limit_my_committed_reduces_call_amount() {
        // pot=1000, current_bet=200, my_committed=100
        // call_amount = 100, max = 200 + 1000 + 100 = 1300
        let pl = BettingStructure::PotLimit;
        assert_eq!(1_300, pl.max_raise(1_000, 200, 100, 5_000, BetTier::Small));
    }

    // ---- Fixed-Limit ----

    #[test]
    fn fixed_limit_min_raise_by_tier() {
        let fl = BettingStructure::FixedLimit {
            small_bet: 100,
            big_bet: 200,
            raise_cap: 3,
        };
        assert_eq!(100, fl.min_raise_for_tier(999, BetTier::Small));
        assert_eq!(200, fl.min_raise_for_tier(999, BetTier::Big));
    }

    #[test]
    fn fixed_limit_max_raise_adds_tier_to_current_bet() {
        let fl = BettingStructure::FixedLimit {
            small_bet: 100,
            big_bet: 200,
            raise_cap: 3,
        };
        // current_bet=200, tier=Big → max = 200+200 = 400
        assert_eq!(400, fl.max_raise(0, 200, 0, 5_000, BetTier::Big));
    }

    #[test]
    fn fixed_limit_max_raise_capped_at_stack() {
        let fl = BettingStructure::FixedLimit {
            small_bet: 100,
            big_bet: 200,
            raise_cap: 3,
        };
        assert_eq!(250, fl.max_raise(0, 200, 0, 250, BetTier::Big));
    }

    #[test]
    fn fixed_limit_cap_at_threshold() {
        let fl = BettingStructure::FixedLimit {
            small_bet: 100,
            big_bet: 200,
            raise_cap: 3,
        };
        assert!(!fl.cap_reached(0));
        assert!(!fl.cap_reached(2));
        assert!(fl.cap_reached(3));
        assert!(fl.cap_reached(99));
    }

    // ---- Display + Default + is_no_limit ----

    #[test]
    fn default_is_no_limit() {
        assert_eq!(BettingStructure::NoLimit, BettingStructure::default());
    }

    #[test]
    fn is_no_limit() {
        assert!(BettingStructure::NoLimit.is_no_limit());
        assert!(!BettingStructure::PotLimit.is_no_limit());
        let fl = BettingStructure::FixedLimit {
            small_bet: 100,
            big_bet: 200,
            raise_cap: 3,
        };
        assert!(!fl.is_no_limit());
    }

    #[test]
    fn display_no_limit() {
        assert_eq!("No-Limit", BettingStructure::NoLimit.to_string());
    }

    #[test]
    fn display_pot_limit() {
        assert_eq!("Pot-Limit", BettingStructure::PotLimit.to_string());
    }

    #[test]
    fn display_fixed_limit() {
        let fl = BettingStructure::FixedLimit {
            small_bet: 100,
            big_bet: 200,
            raise_cap: 3,
        };
        assert_eq!("Fixed-Limit (100/200, cap 3)", fl.to_string());
    }

    #[test]
    fn bet_tier_default_is_small() {
        assert_eq!(BetTier::Small, BetTier::default());
    }
}
