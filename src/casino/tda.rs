//! TDA 2024 rule primitives.
//!
//! Everything here is pure: it takes the facts a rule turns on and returns the
//! answer, so it can be tested without building a table at all. The module
//! predates EPIC-83, when a rule had to be implemented once for two engines;
//! the separation still earns its keep as a testable rule layer.
//!
//! TDA rules used by permission of the Poker TDA, <http://www.pokertda.com>,
//! all rights reserved.

use crate::arrays::five::Five;
use crate::games::GameFamily;
use std::cmp::Reverse;

/// How many seats it takes to reach `seat` walking left (clockwise) from
/// `button`. The seat immediately to the button's left scores 0, so **lower is
/// nearer**. The button itself scores `seat_count - 1`, which is correct: going
/// left, it is the last seat reached rather than the first.
///
/// `seat_count` of 0 is treated as 1 so the modulus cannot divide by zero.
#[must_use]
pub(crate) fn seats_left_of_button(button: u8, seat: u8, seat_count: u8) -> usize {
    let count = usize::from(seat_count).max(1);
    (usize::from(seat) + count - (usize::from(button) % count) - 1) % count
}

/// TDA 2024 Rule 20-B — the high card by suit of a winning 5-card hand, as a
/// `(rank, suit)` sort key.
///
/// Rank leads and suit breaks the tie, which is what "high card by suit" means:
/// the highest card, with the suit deciding only when two winners hold the same
/// rank. Suit order is [`Suit`](crate::suit::Suit)'s own — spades, hearts,
/// diamonds, clubs — the bridge ranking the TDA uses. Cards are unique, so this
/// key can never tie between two seats.
#[must_use]
pub(crate) fn high_card_by_suit(hand: &Five) -> (u8, u8) {
    hand.iter()
        .map(|card| (card.get_rank() as u8, card.get_suit() as u8))
        .max()
        .unwrap_or((0, 0))
}

/// TDA 2024 Rule 20 — the order in which tied winners take odd chips
/// (`DEFECT_011`).
///
/// > First, odd chips will be broken into the smallest denomination in play.
/// > **A)** Board games with 2 or more high or low hands: the odd chip goes to
/// > the **first seat left of the button**. **B)** Stud, razz, and if 2 or more
/// > high or low hands in stud/8: the odd chip goes to the **high card by
/// > suit** in the player's 5-card winning hand.
///
/// Returns `winners` sorted by that precedence, best claim first. `hand_of` is
/// consulted only for the stud family, so a board game never pays for an eval
/// it does not need.
///
/// The rule's first clause does not apply to pkcore: chips are modelled as
/// integers, so there is no denomination to break.
///
/// **Case C (hi/lo split — the odd chip in the total pot goes to the high side)
/// is deliberately absent, because it is unreachable.** pkcore ships no hi/lo
/// variant and [`GameFamily`] has no split-pot arm. When one lands, it belongs
/// here.
#[must_use]
pub(crate) fn odd_chip_order<F>(winners: &[u8], family: GameFamily, button: u8, seat_count: u8, hand_of: F) -> Vec<u8>
where
    F: Fn(u8) -> Five,
{
    let mut ordered = winners.to_vec();
    match family {
        // 20-B: the button is not consulted in stud at all — the cards decide.
        GameFamily::StudHi | GameFamily::Razz => {
            ordered.sort_by_key(|&seat| Reverse(high_card_by_suit(&hand_of(seat))));
        }
        // 20-A: board games. Sorting by distance walking left from the button
        // puts the first seat to its left at index 0.
        GameFamily::Holdem | GameFamily::Omaha => {
            ordered.sort_by_key(|&seat| seats_left_of_button(button, seat, seat_count));
        }
    }
    ordered
}

/// Splits `total` chips into `by` roughly equal shares, remainder one chip at a
/// time to the **last** shares.
///
/// Pure arithmetic with no domain context: it cannot see the button or the
/// cards, and should not learn to. [`pair_shares`] is what decides which seat
/// each share belongs to.
#[must_use]
pub(crate) fn divvy_up(total: usize, by: usize) -> Vec<usize> {
    match by {
        0 | 1 => vec![total],
        _ => {
            let share = total / by;
            let remainder = total % by;
            (0..by)
                .map(|i| if i >= by - remainder { share + 1 } else { share })
                .collect()
        }
    }
}

/// Splits `total` among tied `winners`, placing odd chips by TDA 2024 Rule 20.
/// Returns `(seat, share)` pairs.
///
/// Pairs are returned in Rule 20 order, best claim first. [`divvy_up`] puts the
/// remainder on the *last* shares, so the share list is reversed before pairing:
/// that lands the extra chips on the *first* seats in Rule 20 precedence, which
/// is what the rule asks for when more than one odd chip is left over.
#[must_use]
pub(crate) fn pair_shares<F>(
    total: usize,
    winners: &[u8],
    family: GameFamily,
    button: u8,
    seat_count: u8,
    hand_of: F,
) -> Vec<(u8, usize)>
where
    F: Fn(u8) -> Five,
{
    let ordered = odd_chip_order(winners, family, button, seat_count, hand_of);
    let mut shares = divvy_up(total, ordered.len());
    shares.reverse();
    ordered.into_iter().zip(shares).collect()
}

#[allow(non_snake_case)]
#[cfg(test)]
mod casino__tda_tests {
    use super::*;
    use crate::arrays::five::Five;
    use crate::cards::Cards;
    use std::str::FromStr;

    fn five(index: &str) -> Five {
        Five::try_from(Cards::from_str(index).expect("valid cards")).expect("five cards")
    }

    /// The seat immediately left of the button is nearest; the button itself is
    /// furthest, because going left it is reached last.
    #[test]
    fn seats_left_of_button__orders_from_the_button_leftward() {
        assert_eq!(0, seats_left_of_button(7, 0, 8));
        assert_eq!(2, seats_left_of_button(7, 2, 8));
        assert_eq!(5, seats_left_of_button(7, 5, 8));
        assert_eq!(7, seats_left_of_button(7, 7, 8), "the button is reached last");
    }

    /// The walk wraps, so a low seat number is not automatically nearer.
    #[test]
    fn seats_left_of_button__wraps_past_the_last_seat() {
        assert_eq!(1, seats_left_of_button(3, 5, 8));
        assert_eq!(6, seats_left_of_button(3, 2, 8));
    }

    #[test]
    fn seats_left_of_button__zero_seat_count_does_not_divide_by_zero() {
        assert_eq!(0, seats_left_of_button(0, 0, 0));
    }

    #[test]
    fn high_card_by_suit__ranks_by_rank_then_suit() {
        let spade_ace = five("A♠ K♦ Q♦ J♦ T♦");
        let heart_ace = five("A♥ K♣ Q♣ J♣ T♣");
        assert!(high_card_by_suit(&spade_ace) > high_card_by_suit(&heart_ace));
    }

    /// Rank leads: a low spade does not beat a high club.
    #[test]
    fn high_card_by_suit__rank_outweighs_suit() {
        let ace_high = five("A♣ K♣ Q♣ J♣ 9♣");
        let king_high = five("K♠ Q♠ J♠ T♠ 8♠");
        assert!(high_card_by_suit(&ace_high) > high_card_by_suit(&king_high));
    }

    #[test]
    fn odd_chip_order__board_game_orders_from_the_button() {
        let blank = |_| five("A♠ K♠ Q♠ J♠ T♠");
        assert_eq!(vec![2, 5], odd_chip_order(&[2, 5], GameFamily::Holdem, 7, 8, blank));
        assert_eq!(vec![5, 2], odd_chip_order(&[2, 5], GameFamily::Holdem, 3, 8, blank));
    }

    /// Stud ignores the button entirely: the same button that reversed the
    /// board-game order above leaves the stud order untouched.
    #[test]
    fn odd_chip_order__stud_ignores_the_button() {
        let hand_of = |seat| match seat {
            2 => five("A♠ K♦ Q♦ J♦ T♦"),
            _ => five("A♥ K♣ Q♣ J♣ T♣"),
        };
        assert_eq!(vec![2, 5], odd_chip_order(&[2, 5], GameFamily::StudHi, 7, 8, hand_of));
        assert_eq!(vec![2, 5], odd_chip_order(&[2, 5], GameFamily::StudHi, 3, 8, hand_of));
    }

    #[test]
    fn divvy_up__remainder_lands_on_the_last_shares() {
        assert_eq!(vec![100], divvy_up(100, 1));
        assert_eq!(vec![50, 50], divvy_up(100, 2));
        assert_eq!(vec![33, 33, 34], divvy_up(100, 3));
        assert_eq!(vec![100], divvy_up(100, 0));
    }

    /// The whole point: 175 splits 87/88, and the 88 goes to the seat nearest
    /// the button's left rather than to the highest seat number.
    #[test]
    fn pair_shares__odd_chip_follows_the_button() {
        let blank = |_| five("A♠ K♠ Q♠ J♠ T♠");
        assert_eq!(
            vec![(2, 88), (5, 87)],
            pair_shares(175, &[2, 5], GameFamily::Holdem, 7, 8, blank)
        );
        assert_eq!(
            vec![(5, 88), (2, 87)],
            pair_shares(175, &[2, 5], GameFamily::Holdem, 3, 8, blank)
        );
    }

    /// Two odd chips go to the two nearest seats, in order — not both to one
    /// seat and not to the two highest seat numbers.
    #[test]
    fn pair_shares__multiple_odd_chips_walk_left_from_the_button() {
        let blank = |_| five("A♠ K♠ Q♠ J♠ T♠");
        let shares = pair_shares(101, &[1, 2, 3], GameFamily::Holdem, 0, 4, blank);
        assert_eq!(vec![(1, 34), (2, 34), (3, 33)], shares);
    }

    #[test]
    fn pair_shares__single_winner_takes_everything() {
        let blank = |_| five("A♠ K♠ Q♠ J♠ T♠");
        assert_eq!(vec![(3, 175)], pair_shares(175, &[3], GameFamily::Holdem, 7, 8, blank));
    }
}
