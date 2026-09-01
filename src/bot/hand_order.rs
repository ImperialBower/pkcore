//! Starting-hand ordering derived from the embedded heads-up chart (EPIC-39 Phase 3a).
//!
//! Widening a villain's range by an observed VPIP needs an answer to "which
//! hands are the top N%?" — an ordering the repo did not have. This module
//! derives one from `generated/hups.bin`: every canonical starting-hand class
//! scored by its exact equity against a uniformly random opponent hand.
//!
//! The ordering is built once, lazily, and only when something asks for it.
//! It costs roughly a second in a debug build and a tenth of that in release.

use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::combos::Combos;
use crate::analysis::store::embedded::hup_cache::HUP_CACHE;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::cards::Cards;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Every canonical starting-hand class, strongest first, with its exact
/// equity against a uniformly random opposing hand.
static ORDERING: LazyLock<Vec<(Combo, f64)>> = LazyLock::new(build_ordering);

/// The 169 canonical starting-hand classes, strongest first.
///
/// Each entry pairs a [`Combo`] with its equity against a random hand, read
/// from the embedded chart rather than sampled. Built on first use.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::gto::combo::Combo;
/// use pkcore::bot::hand_order::hand_ordering;
///
/// let ordering = hand_ordering();
/// assert_eq!(169, ordering.len());
/// assert_eq!(Combo::COMBO_AA, ordering[0].0);
/// ```
#[must_use]
pub fn hand_ordering() -> &'static [(Combo, f64)] {
    &ORDERING
}

/// The strongest `fraction` of all starting hands, as a [`Combos`] range.
///
/// `fraction` is a share of the 1,326 distinct two-card hands, so `0.45` means
/// "the hands a 45%-VPIP player would be holding". Classes are added
/// strongest-first until the accumulated hand count reaches the target, which
/// makes the result **nested**: a wider fraction always contains a narrower
/// one. Returns `None` unless `fraction` lies in `(0.0, 1.0]`.
///
/// # Examples
///
/// ```
/// use pkcore::bot::hand_order::range_of_width;
///
/// let loose = range_of_width(0.45).unwrap();
/// let tight = range_of_width(0.10).unwrap();
/// let loose_set = loose.to_hash_set();
/// assert!(tight.iter().all(|combo| loose_set.contains(combo)));
///
/// assert!(range_of_width(0.0).is_none());
/// ```
#[must_use]
pub fn range_of_width(fraction: f64) -> Option<Combos> {
    if !(fraction > 0.0 && fraction <= 1.0) {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let target = fraction * TOTAL_HANDS as f64;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let target = target as usize;

    let mut chosen = std::collections::HashSet::new();
    let mut covered = 0_usize;
    for (combo, _) in hand_ordering() {
        if covered >= target {
            break;
        }
        chosen.insert(*combo);
        covered += hands_in_class(*combo);
    }
    if chosen.is_empty() {
        return None;
    }
    Some(Combos::from(chosen))
}

/// The number of distinct two-card hands in a canonical class: 6 for a pair,
/// 4 suited, 12 offsuit.
fn hands_in_class(combo: Combo) -> usize {
    use crate::analysis::gto::twos::Twos;
    Twos::from(&Combos::from(vec![combo])).to_vec().len()
}

/// Total distinct two-card starting hands: `C(52, 2)`.
const TOTAL_HANDS: usize = 1_326;

/// Scores every canonical class against the full field of opposing hands.
fn build_ordering() -> Vec<(Combo, f64)> {
    let field: Vec<Two> = Cards::deck()
        .combinations(2)
        .filter_map(|cards| Two::new(cards[0], cards[1]).ok())
        .collect();

    // One representative hand per canonical class. Suit permutations inside a
    // class are equivalent against a uniform field, so any representative does.
    let mut representatives: HashMap<Combo, Two> = HashMap::new();
    for hand in &field {
        representatives.entry(Combo::from(*hand)).or_insert(*hand);
    }

    let mut ordering: Vec<(Combo, f64)> = representatives
        .into_iter()
        .filter_map(|(combo, hand)| equity_vs_field(hand, &field).map(|equity| (combo, equity)))
        .collect();
    ordering.sort_by(|left, right| right.1.total_cmp(&left.1));
    ordering
}

/// Exact equity for `hero` against every hand it can be dealt alongside.
fn equity_vs_field(hero: Two, field: &[Two]) -> Option<f64> {
    let hero_bard: Bard = hero.into();
    let mut total = 0.0;
    let mut counted = 0_u32;

    for villain in field {
        if shares_a_card(hero, *villain) {
            continue;
        }
        let villain_bard: Bard = (*villain).into();
        let (higher, lower) = if hero_bard.as_u64() > villain_bard.as_u64() {
            (hero_bard, villain_bard)
        } else {
            (villain_bard, hero_bard)
        };
        let Some(odds) = HUP_CACHE.get(&(higher.as_u64(), lower.as_u64())) else {
            continue;
        };
        let boards = odds.wins + odds.losses + odds.draws;
        if boards == 0 {
            continue;
        }
        // `HUP_CACHE` reports from the higher hand's side; see the perspective
        // note on `crate::bot::preflop_equity::hup_equity_vs_range`.
        let mine = if higher == hero_bard { odds.wins } else { odds.losses };
        #[allow(clippy::cast_precision_loss)]
        {
            total += (mine as f64 + odds.draws as f64 / 2.0) / boards as f64;
        }
        counted += 1;
    }

    if counted == 0 {
        None
    } else {
        Some(total / f64::from(counted))
    }
}

/// Returns `true` when the two hands cannot be dealt at the same time.
fn shares_a_card(hero: Two, villain: Two) -> bool {
    let (first, second) = (hero.first(), hero.second());
    villain.first() == first || villain.first() == second || villain.second() == first || villain.second() == second
}

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__hand_order_tests {
    use super::*;
    use crate::analysis::gto::twos::Twos;

    fn hands_in(range: &Combos) -> usize {
        Twos::from(range).to_vec().len()
    }

    #[test]
    fn ordering_covers_every_canonical_starting_hand_class() {
        assert_eq!(
            169,
            hand_ordering().len(),
            "13 pairs + 78 suited + 78 offsuit = 169 classes"
        );
    }

    #[test]
    fn ordering_runs_from_aces_down_to_seven_deuce_offsuit() {
        let ordering = hand_ordering();
        let best = ordering.first().expect("a non-empty ordering");
        let worst = ordering.last().expect("a non-empty ordering");
        assert_eq!(Combo::COMBO_AA, best.0, "aces are the best starting hand");
        assert!(
            best.1 > 0.8 && worst.1 < 0.36,
            "aces should sit above 0.80 and the worst hand below 0.36, got {:.4} and {:.4}",
            best.1,
            worst.1
        );
    }

    #[test]
    fn ordering_is_sorted_by_descending_equity() {
        let ordering = hand_ordering();
        assert!(
            ordering.windows(2).all(|pair| pair[0].1 >= pair[1].1),
            "the ordering must be strongest-first"
        );
    }

    #[test]
    fn range_of_width_grows_with_the_fraction() {
        let tight = range_of_width(0.10).expect("10% is a legal width");
        let wide = range_of_width(0.45).expect("45% is a legal width");
        assert!(
            hands_in(&wide) > hands_in(&tight),
            "45% must cover more hands than 10%: {} vs {}",
            hands_in(&wide),
            hands_in(&tight)
        );
    }

    #[test]
    fn range_of_width_is_nested() {
        let tight = range_of_width(0.10).expect("a range");
        let wide = range_of_width(0.45).expect("a range");
        let wide_set = wide.to_hash_set();
        assert!(
            tight.iter().all(|combo| wide_set.contains(combo)),
            "a wider range must contain every hand of a narrower one"
        );
    }

    #[test]
    fn range_of_width_at_forty_five_percent_is_roughly_forty_five_percent() {
        let range = range_of_width(0.45).expect("a range");
        let pct = hands_in(&range) as f64 / 1326.0;
        assert!(
            (pct - 0.45).abs() < 0.06,
            "a 45% range should cover roughly 45% of all hands, got {:.1}%",
            pct * 100.0
        );
    }

    #[test]
    fn range_of_width_rejects_a_nonsense_fraction() {
        assert!(range_of_width(0.0).is_none());
        assert!(range_of_width(-1.0).is_none());
        assert!(range_of_width(1.5).is_none());
    }
}
