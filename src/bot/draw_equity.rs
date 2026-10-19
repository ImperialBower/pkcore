//! Draw-aware hand strength for the `outs` knob (EPIC-39 Phase 3b).
//!
//! The hand-rank proxy the decider falls back on when `equity: off` is a
//! *snapshot of made strength*: `1 - hand_rank_value / 7462`. A four-flush is
//! not a made hand, so the proxy scores it below one pair — it rates an
//! open-ended straight draw at `0.0021`, below total air at `0.1635`.
//!
//! This module counts the hero's **outs** and converts them to equity. An out
//! is not merely a card that improves the hand; it is a card that improves it
//! *past what the villain is likely holding*, so the count depends on the
//! villain's estimated range.

use crate::Pile;
use crate::analysis::gto::combos::Combos;
use crate::analysis::gto::twos::Twos;
use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::six::Six;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::cards::Cards;

/// Draw-aware hand strength for the decider. A unit struct that groups this
/// module's functions; it holds no state.
///
/// # Examples
///
/// ```
/// use pkcore::bot::draw_equity::DrawEquity;
///
/// assert_eq!(DrawEquity, DrawEquity::default());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawEquity;

impl DrawEquity {
    /// Counts the hero's true outs on a flop or turn against `range`.
    ///
    /// The range is reduced to a single yardstick — the **median** made-hand
    /// strength its holdings have on this board — and a card counts as an out when
    /// it lifts the hero past that yardstick. This is why the same hand has more
    /// outs against a loose range than a tight one: the bar is lower.
    ///
    /// A hand already ahead of the yardstick has no outs, so the count is `0`.
    ///
    /// Returns `None` unless the board holds three or four cards, or when no
    /// holding in the range survives the dead cards.
    ///
    /// # Domain
    ///
    /// - **Role:** cohesive mechanism
    /// - **Invariants:**
    ///   - `None` unless the board holds 3 or 4 cards
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::arrays::two::Two;
    /// use pkcore::bot::draw_equity::DrawEquity;
    /// use pkcore::cards::Cards;
    /// use std::str::FromStr;
    ///
    /// // A♥K♥ on 7♥2♥3♦ draws to the nut flush: nine hearts plus three aces
    /// // and three kings for top pair — the textbook fifteen outs.
    /// let outs = DrawEquity::outs_against(
    ///     Two::from_str("AH KH").unwrap(),
    ///     &Cards::from_str("7H 2H 3D").unwrap(),
    ///     &Combos::from_str("QQ+, AK").unwrap(),
    /// )
    /// .unwrap();
    /// assert_eq!(15, outs);
    /// ```
    #[must_use]
    pub fn outs_against(hero: Two, board: &Cards, range: &Combos) -> Option<usize> {
        if !matches!(board.len(), 3 | 4) {
            return None;
        }
        let yardstick = Self::range_median_strength(board, range, hero)?;
        // DEFECT_032: an out takes the hero from behind to ahead. A hand already
        // ahead of the yardstick has none; its made strength speaks for it.
        if Self::made_strength(hero, board)? < yardstick {
            return Some(0);
        }
        let unseen = Self::unseen_cards(hero, board);

        Some(
            unseen
                .iter()
                .filter_map(|card| Self::hero_strength_with(hero, board, *card))
                // A *lower* `hand_rank_value` is a stronger hand: 1 is a royal flush.
                .filter(|improved| *improved < yardstick)
                .count(),
        )
    }

    /// Converts an out count to equity with the classic rule of four and two:
    /// each out is worth about 4% with two cards to come and 2% with one.
    ///
    /// Capped at `1.0`, since the rule overshoots badly for very large counts.
    ///
    /// # Domain
    ///
    /// - **Role:** cohesive mechanism
    /// - **Invariants:**
    ///   - result is at most `1.0`
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::draw_equity::DrawEquity;
    ///
    /// assert!((DrawEquity::outs_equity(9, 2) - 0.36).abs() < 1e-9); // flop
    /// assert!((DrawEquity::outs_equity(9, 1) - 0.18).abs() < 1e-9); // turn
    /// ```
    #[must_use]
    pub fn outs_equity(outs: usize, cards_to_come: usize) -> f64 {
        let per_out = if cards_to_come >= 2 { 0.04 } else { 0.02 };
        #[allow(clippy::cast_precision_loss)]
        let raw = outs as f64 * per_out;
        raw.min(1.0)
    }

    /// The median made-hand strength the range holds on this board.
    fn range_median_strength(board: &Cards, range: &Combos, hero: Two) -> Option<u16> {
        let mut strengths: Vec<u16> = Twos::from(range)
            .to_vec()
            .into_iter()
            .filter(|villain| !Self::collides(*villain, hero, board))
            .filter_map(|villain| Self::made_strength(villain, board))
            .collect();
        if strengths.is_empty() {
            return None;
        }
        strengths.sort_unstable();
        Some(strengths[strengths.len() / 2])
    }

    /// Best five-card value for a holding on this board, before the next card.
    fn made_strength(villain: Two, board: &Cards) -> Option<u16> {
        let combined = format!("{villain} {board}");
        match board.len() {
            3 => combined.parse::<Five>().ok().map(|hand| hand.hand_rank_value()),
            4 => combined.parse::<Six>().ok().map(|hand| hand.hand_rank_value()),
            _ => None,
        }
    }

    /// Best value the hero reaches once `card` lands.
    fn hero_strength_with(hero: Two, board: &Cards, card: Card) -> Option<u16> {
        let combined = format!("{hero} {board} {card}");
        match board.len() {
            3 => combined.parse::<Six>().ok().map(|hand| hand.hand_rank_value()),
            4 => combined.parse::<Seven>().ok().map(|hand| hand.hand_rank_value()),
            _ => None,
        }
    }

    /// Every card still unaccounted for.
    fn unseen_cards(hero: Two, board: &Cards) -> Cards {
        let mut deck = Cards::deck();
        deck.remove(&hero.first());
        deck.remove(&hero.second());
        for card in board {
            deck.remove(card);
        }
        deck
    }

    /// Returns `true` when a villain holding cannot coexist with the known cards.
    fn collides(villain: Two, hero: Two, board: &Cards) -> bool {
        let dead = [hero.first(), hero.second()];
        dead.contains(&villain.first())
            || dead.contains(&villain.second())
            || board.contains(&villain.first())
            || board.contains(&villain.second())
    }
}

/// Deprecated: use [`DrawEquity::outs_against`].
///
/// # Examples
///
/// ```
/// # #![allow(deprecated)]
/// use pkcore::analysis::gto::combos::Combos;
/// use pkcore::arrays::two::Two;
/// use pkcore::bot::draw_equity::outs_against;
/// use pkcore::cards::Cards;
/// use std::str::FromStr;
///
/// let outs = outs_against(
///     Two::from_str("AH KH").unwrap(),
///     &Cards::from_str("7H 2H 3D").unwrap(),
///     &Combos::from_str("QQ+, AK").unwrap(),
/// );
/// assert_eq!(Some(15), outs);
/// ```
#[deprecated(since = "0.16.0", note = "use `DrawEquity::outs_against`")]
#[must_use]
pub fn outs_against(hero: Two, board: &Cards, range: &Combos) -> Option<usize> {
    DrawEquity::outs_against(hero, board, range)
}

/// Deprecated: use [`DrawEquity::outs_equity`].
///
/// # Examples
///
/// ```
/// # #![allow(deprecated)]
/// use pkcore::bot::draw_equity::outs_equity;
///
/// assert!((outs_equity(9, 2) - 0.36).abs() < 1e-9);
/// ```
#[deprecated(since = "0.16.0", note = "use `DrawEquity::outs_equity`")]
#[must_use]
pub fn outs_equity(outs: usize, cards_to_come: usize) -> f64 {
    DrawEquity::outs_equity(outs, cards_to_come)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__draw_equity_tests {
    use super::*;
    use std::str::FromStr;

    fn two(s: &str) -> Two {
        Two::from_str(s).expect("a legal hand")
    }

    fn cards(s: &str) -> Cards {
        Cards::from_str(s).expect("legal cards")
    }

    fn range(s: &str) -> Combos {
        Combos::from_str(s).expect("a legal range")
    }

    /// A♥K♥ on 7♥2♥3♦ is the textbook fifteen-out hand: nine hearts complete
    /// the nut flush, and three aces plus three kings make a top pair that
    /// still beats the range's median holding.
    #[test]
    fn counts_the_nut_flush_draw_with_two_overcards_as_fifteen_outs() {
        let outs = DrawEquity::outs_against(two("AH KH"), &cards("7H 2H 3D"), &range("QQ+, AK"))
            .expect("a flop hand resolves");
        assert_eq!(
            15, outs,
            "nine flush outs plus six top-pair outs is the classic fifteen"
        );
    }

    /// The same two cards on a board with only *one* heart hold a backdoor
    /// draw, not a flush draw — four hearts is not four-to-a-flush. All that
    /// is left is pairing: three aces and three kings.
    #[test]
    fn counts_only_the_pairing_outs_when_the_flush_draw_is_backdoor() {
        let outs = DrawEquity::outs_against(two("AH KH"), &cards("7H 2C 3D"), &range("QQ+, AK"))
            .expect("a flop hand resolves");
        assert_eq!(6, outs, "three aces and three kings, no flush draw");
    }

    /// J♣4♦ on A♥K♦9♣ is air against a premium range — pairing the jack or
    /// the four still loses to every hand in `QQ+, AK`.
    #[test]
    fn finds_almost_nothing_for_air_against_a_premium_range() {
        let outs = DrawEquity::outs_against(two("JC 4D"), &cards("AH KD 9C"), &range("QQ+, AK"))
            .expect("a flop hand resolves");
        assert!(
            outs <= 3,
            "air against a premium range should have almost no real outs, got {outs}"
        );
    }

    /// The same air has far more outs against a range that is mostly air too.
    #[test]
    fn the_same_hand_has_more_outs_against_a_wide_range() {
        let board = cards("AH KD 9C");
        let vs_premium = DrawEquity::outs_against(two("JC 4D"), &board, &range("QQ+, AK")).expect("resolves");
        let vs_wide = DrawEquity::outs_against(two("JC 4D"), &board, &range("22+, A2s+, K2s+, Q2s+, J2s+, T2s+, 32s+"))
            .expect("resolves");
        assert!(
            vs_wide > vs_premium,
            "a wider range makes more improvements good: {vs_wide} vs {vs_premium}"
        );
    }

    /// DEFECT_032: bottom set is already ahead of a wide range, so no card is
    /// an out. The old count took every card that kept it ahead.
    #[test]
    fn a_hand_already_ahead_of_the_range_has_no_outs() {
        let wide = range("22+, A2s+, K2s+, Q2s+, J2s+, T2s+, 32s+");
        let flop = DrawEquity::outs_against(two("2S 2D"), &cards("2H 7C KD"), &wide).expect("a flop hand resolves");
        assert_eq!(0, flop, "a set on the flop has no outs");
        let turn = DrawEquity::outs_against(two("2S 2D"), &cards("2H 7C KD 9S"), &wide).expect("a turn hand resolves");
        assert_eq!(0, turn, "a set on the turn has no outs");
    }

    #[test]
    fn outs_are_none_when_the_board_is_not_a_flop_or_turn() {
        assert!(DrawEquity::outs_against(two("AH KH"), &Cards::default(), &range("QQ+")).is_none());
        assert!(DrawEquity::outs_against(two("AH KH"), &cards("7H 2C 3D 4S 5S"), &range("QQ+")).is_none());
    }

    /// The classic rule of four and two.
    #[test]
    fn outs_convert_to_equity_by_the_four_and_two_rule() {
        assert!(
            (DrawEquity::outs_equity(9, 2) - 0.36).abs() < 1e-9,
            "nine outs on the flop is 36%"
        );
        assert!(
            (DrawEquity::outs_equity(9, 1) - 0.18).abs() < 1e-9,
            "nine outs on the turn is 18%"
        );
        assert!((DrawEquity::outs_equity(0, 2) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn outs_equity_never_exceeds_certainty() {
        assert!(DrawEquity::outs_equity(40, 2) <= 1.0, "the rule must be capped");
    }

    #[test]
    fn draw_equity_is_a_stateless_unit_struct() {
        assert_eq!(DrawEquity, DrawEquity::default());
        assert_eq!("DrawEquity", format!("{DrawEquity:?}"));
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_functions_match_the_methods() {
        let (hero, board, villains) = (two("AH KH"), cards("7H 2H 3D"), range("QQ+, AK"));
        assert_eq!(
            DrawEquity::outs_against(hero, &board, &villains),
            outs_against(hero, &board, &villains)
        );
        assert!((DrawEquity::outs_equity(9, 2) - outs_equity(9, 2)).abs() < 1e-12);
    }
}
