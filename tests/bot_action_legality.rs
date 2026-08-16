//! Integration test: every action a bot returns must be legal (DEFECT_007).
#![allow(non_snake_case)]
//!
//! `PokerSession::run_hand` propagates `apply_action` failures with `?` and
//! offers no seam to substitute a legal action. So the only way pkcore's own
//! bots compose with pkcore's own driver is if `BotProfile::decide` never
//! returns an action the engine rejects.
//!
//! This harness deliberately does **not** carry the AllIn/Check fallback that
//! `tests/bot_marathon.rs` uses — the fallback is what hid DEFECT_007 for three
//! months. Every `apply_action` result is asserted.
//!
//! Stakes escalate every few hands so stacks fall below one legal raise, which
//! is the region the defect lives in. All four betting shapes pkcore ships are
//! covered — No-Limit, Pot-Limit, Fixed-Limit (whose raise cap is a second way
//! a raise can be illegal) and Seven-Card Stud (whose minimum is a completion,
//! not a step).

use pkcore::bot::profile::BotProfile;
use pkcore::casino::action::PlayerAction;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::session::PokerSession;
use pkcore::casino::table::{Player, Seat, Seats, Table};
use rand::SeedableRng;
use rand::rngs::SmallRng;

/// Asserts the bot chose an aggressive action of a *kind* the engine advertises.
///
/// Acceptance alone is too weak a bar. `Table::act_bet` accepts a `Bet` where
/// `legal_actions` says `Raise` (the big-blind option) and then records the
/// wrong raise increment, so the hand plays on with a corrupted betting ladder
/// and nothing ever errors. This checks the variant, which is what
/// `legal_actions` is actually promising.
///
/// Only `Bet` and `Raise` are checked. `Call` is deliberately excluded:
/// `apply_action` documents that it degrades a `Call` to a `Check` when the bet
/// is already matched, so a `Call` outside `legal_actions` is intended.
fn assert_action_kind_is_advertised(table: &Table, seat: u8, action: PlayerAction, context: &str) {
    let legal = table.legal_actions(seat);
    let advertised = match action {
        PlayerAction::Bet(_) => legal.iter().any(|a| matches!(a, PlayerAction::Bet(_))),
        PlayerAction::Raise(_) => legal.iter().any(|a| matches!(a, PlayerAction::Raise(_))),
        _ => true,
    };
    assert!(
        advertised,
        "{context}: seat {seat} returned {action:?}, but the engine advertises {legal:?}"
    );
}

/// The betting structures a bot can be sat down at. Each has its own legality
/// rules, so each is its own exposure surface for DEFECT_007-shaped bugs.
#[derive(Clone, Copy, Debug)]
enum Structure {
    NoLimit,
    PotLimit,
    FixedLimit,
}

/// Nine short seats against escalating stakes — a tournament shape that drives
/// every stack below the minimum raise.
fn escalating_session(structure: Structure, seed: u64) -> (PokerSession, SmallRng) {
    let seats = Seats::new(
        (0..9)
            .map(|i| Seat::new(Player::new_with_chips(format!("bot{i}"), 2_000)))
            .collect(),
    );
    let table = match structure {
        Structure::NoLimit => Table::nlh_from_seats(seats, ForcedBets::new(25, 50)),
        Structure::PotLimit => Table::plo_from_seats(seats, (25, 50)),
        // raise_cap 4 is the standard limit-holdem cap: bet, raise, re-raise, cap.
        Structure::FixedLimit => Table::limit_holdem_from_seats(seats, 50, 100, 4),
    };
    (PokerSession::new(table), SmallRng::seed_from_u64(seed))
}

/// Drives hands with no fallback and asserts every bot action is accepted.
fn run_without_fallback(structure: Structure, seed: u64, hands: usize) {
    let profiles = BotProfile::default_profiles();
    let (mut session, mut rng) = escalating_session(structure, seed);

    for hand in 0..hands {
        // Escalate every 10 hands: 25/50 -> 50/100 -> 100/200 -> ...
        let level = hand / 10;
        let small = 25usize << level.min(6);
        session.set_blinds(ForcedBets::new(small, small * 2));

        session.eliminate_busted();
        if session.count_funded() < 2 {
            return;
        }
        if session.start_hand().is_err() {
            return;
        }

        while let Some(seat) = session.next_actor() {
            let profile = &profiles[seat as usize % profiles.len()];
            let action = profile.decide(&session.table, seat, &mut rng);
            let to_call = session.table.to_call(seat);
            let chips = session
                .table
                .seats
                .get_seat(seat)
                .map_or(0, |s| s.player.total_chip_count());
            let min_raise_to = session.table.min_raise_to();
            let bounds = session.table.raise_bounds(seat);

            assert_action_kind_is_advertised(&session.table, seat, action, &format!("{structure:?} hand {hand}"));
            assert!(
                session.apply_action(seat, action).is_ok(),
                "{structure:?} hand {hand} seat {seat}: engine rejected {action:?} \
                 (to_call={to_call} total_chips={chips} \
                 min_raise_to={min_raise_to} raise_bounds={bounds:?})"
            );
        }

        assert!(session.end_hand().is_ok(), "hand {hand}: end_hand failed");
    }
}

/// Seven-Card Stud is the one family where `min_raise_to` is *not*
/// `current_bet + min_raise`: a bring-in below one full small bet is
/// **completed** to the small bet rather than raised on top of. Bots must size
/// against the completion rule, so stud gets its own harness — stakes are fixed
/// and stacks shrink instead of blinds growing.
///
/// Seven seats, not eight: eight players need 56 cards for seven streets and a
/// 52-card deck cannot supply them, so an 8-handed hand stalls with
/// `PKError::ActionNotFinished` at `end_hand`. That is a dealing gap, not a
/// betting one — see `docs/defects/DEFECT_007_decider_subminimum_raise.md`.
fn run_stud_without_fallback(seed: u64, hands: usize) {
    let profiles = BotProfile::default_profiles();
    let seats = Seats::new(
        (0..7)
            .map(|i| Seat::new(Player::new_with_chips(format!("bot{i}"), 300)))
            .collect(),
    );
    // ante 2, bring-in 5, small bet 20, big bet 40 against 300-chip stacks —
    // shallow enough that the completion boundary is reached inside a hand.
    let table = Table::stud_hi_from_seats(seats, 2, 5, 20, 40);
    let mut session = PokerSession::new(table);
    let mut rng = SmallRng::seed_from_u64(seed);

    for hand in 0..hands {
        session.eliminate_busted();
        if session.count_funded() < 2 {
            return;
        }
        if session.start_hand().is_err() {
            return;
        }

        while let Some(seat) = session.next_actor() {
            let profile = &profiles[seat as usize % profiles.len()];
            let action = profile.decide(&session.table, seat, &mut rng);
            let min_raise_to = session.table.min_raise_to();
            let bounds = session.table.raise_bounds(seat);

            assert_action_kind_is_advertised(&session.table, seat, action, &format!("stud hand {hand}"));
            assert!(
                session.apply_action(seat, action).is_ok(),
                "stud hand {hand} seat {seat}: engine rejected {action:?} \
                 (bet={} min_raise_to={min_raise_to} raise_bounds={bounds:?})",
                session.table.bet
            );
        }

        if let Err(e) = session.end_hand() {
            panic!("stud hand {hand}: end_hand failed: {e}");
        }
    }
}

#[test]
fn stud_bots_never_return_an_action_the_engine_rejects() {
    for seed in 0..25 {
        run_stud_without_fallback(seed, 120);
    }
}

#[test]
fn no_limit_bots_never_return_an_action_the_engine_rejects() {
    for seed in 0..25 {
        run_without_fallback(Structure::NoLimit, seed, 120);
    }
}

#[test]
fn pot_limit_bots_never_return_an_action_the_engine_rejects() {
    for seed in 0..25 {
        run_without_fallback(Structure::PotLimit, seed, 120);
    }
}

#[test]
fn fixed_limit_bots_never_return_an_action_the_engine_rejects() {
    for seed in 0..25 {
        run_without_fallback(Structure::FixedLimit, seed, 120);
    }
}
