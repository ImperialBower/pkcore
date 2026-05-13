//! Integration test: bot-selfplay session → YAML round-trip → replay validation.
//!
//! Runs a small bot-selfplay session, serializes the hand collection to YAML,
//! deserializes it, replays every hand through [`HandHistory::replay`], and
//! asserts that the replayed chip counts match the recorded results.
//!
//! The test is marked `#[ignore]` because it runs the full game engine (several
//! hands of NLH with 3 bots) and is therefore slower than a unit test.
//!
//! Run explicitly with:
//! ```text
//! cargo test --test replay_consistency -- --include-ignored
//! ```

use pkcore::bot::profile::BotProfile;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::session::PokerSession;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
use pkcore::games::betting_structure::BettingStructure;
use pkcore::hand_history::{HandCollection, HandHistory, HandVariant};

const SB: usize = 50;
const BB: usize = 100;
const STARTING_CHIPS: usize = 5_000;
const NUM_HANDS: usize = 10;
const SOURCE: &str = "replay_consistency_test";

#[test]
#[ignore = "runs a full bot-selfplay session; use --include-ignored to enable"]
fn test_bot_selfplay_replay_roundtrip() {
    let profile_names = ["gto", "tight_passive", "loose_aggressive"];
    let profiles: Vec<BotProfile> = profile_names
        .iter()
        .map(|n| {
            BotProfile::from_file(format!("data/bots/{n}.yaml"))
                .unwrap_or_else(|e| panic!("failed to load {n}.yaml: {e}"))
        })
        .collect();

    let seats_vec: Vec<SeatNoCell> = profiles
        .iter()
        .map(|p| SeatNoCell::new(PlayerNoCell::new_with_chips(p.name.clone(), STARTING_CHIPS)))
        .collect();
    let table = TableNoCell::nlh_from_seats(SeatsNoCell::new(seats_vec), ForcedBets::new(SB, BB));
    let mut session = PokerSession::new(table);
    let mut rng = rand::rng();
    let mut collection = HandCollection::new();

    for hand_num in 1..=NUM_HANDS {
        session.eliminate_busted();
        if session.count_funded() < 2 {
            break;
        }

        let button = session.table.button;

        // Capture starting stacks before forced bets.
        let stacks: Vec<(u8, String, usize)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session
                    .table
                    .seats
                    .get_seat(i)
                    .filter(|s| !s.is_empty())
                    .map(|s| (i, s.player.handle.clone(), s.player.chips))
            })
            .collect();

        // Record log length before this hand so we can slice out only this hand's events.
        let event_log_start = session.table.event_log.len();

        // Shuffles deck, posts forced bets, deals hole cards.
        session
            .start_hand()
            .unwrap_or_else(|e| panic!("start_hand failed on hand {hand_num}: {e}"));

        // Capture hole cards immediately after deal.
        let hole_cards: Vec<(u8, Option<String>)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session.table.seats.get_seat(i).filter(|s| !s.is_empty()).map(|s| {
                    (
                        i,
                        if s.cards.has_cards() {
                            Some(s.cards.sorted_display())
                        } else {
                            None
                        },
                    )
                })
            })
            .collect();

        let player_snapshot: Vec<(u8, String, usize, Option<String>)> = stacks
            .iter()
            .map(|(seat, name, stack)| {
                let hole = hole_cards.iter().find(|(s, _)| s == seat).and_then(|(_, h)| h.clone());
                (*seat, name.clone(), *stack, hole)
            })
            .collect();

        // Run all bot actions. PokerSession::next_actor handles street advancement.
        // If a bot's chosen action is invalid (e.g. insufficient chips at a low
        // stack), fall back to AllIn (when facing a bet) or Check.
        while let Some(seat) = session.next_actor() {
            use pkcore::casino::action::PlayerAction;
            let action = profiles[seat as usize].decide(&session.table, seat, &mut rng);
            if session.apply_action(seat, action).is_err() {
                let fallback = if session.table.to_call(seat) > 0 {
                    PlayerAction::AllIn
                } else {
                    PlayerAction::Check
                };
                let _ = session.apply_action(seat, fallback);
            }
        }

        let board_str = session.table.board.to_string();
        let event_log = session.table.event_log[event_log_start..].to_vec();
        let winnings = session
            .end_hand()
            .unwrap_or_else(|e| panic!("end_hand failed on hand {hand_num}: {e}"));

        let ending_stacks: Vec<(u8, usize)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session
                    .table
                    .seats
                    .get_seat(i)
                    .filter(|s| !s.is_empty())
                    .map(|s| (i, s.player.chips))
            })
            .collect();

        let history = HandHistory::from_table_state(
            hand_num,
            0,
            button,
            &ForcedBets::new(SB, BB),
            &player_snapshot,
            &board_str,
            &winnings,
            &event_log,
            &ending_stacks,
            SOURCE,
            session.shuffled_deck_str.clone(),
        );
        collection.push(history);
        session.table.button_up();
    }

    assert!(!collection.is_empty(), "at least one hand should have been played");

    // Serialize → deserialize round-trip.
    let yaml = collection.to_yaml().expect("serialization should succeed");
    let loaded = HandCollection::from_yaml(&yaml).expect("deserialization should succeed");

    assert_eq!(
        collection.len(),
        loaded.len(),
        "hand count should survive YAML round-trip"
    );

    // Replay every hand and verify consistency.
    for (idx, result) in loaded.replay_all().into_iter().enumerate() {
        let replay =
            result.unwrap_or_else(|e| panic!("replay() returned error for hand {}: {e}", loaded.hands()[idx].hand.id));
        assert!(
            replay.is_consistent,
            "replay mismatch for hand {}: final stacks = {:?}",
            loaded.hands()[idx].hand.id,
            replay.final_stacks
        );
    }
}

// EPIC-30 Phase 11: FLHE round-trip test. Records 10 hands of Fixed-Limit
// Hold'em, serializes to YAML, deserializes, and replays every hand
// through the engine — asserting that the recorded `betting_structure`
// reconstructs an FLHE replay table (not an NLHE one) and that chip
// totals match.

const FL_SMALL_BET: usize = 100;
const FL_BIG_BET: usize = 200;
const FL_RAISE_CAP: u8 = 3;
const FLHE_SOURCE: &str = "replay_consistency_flhe";

#[test]
#[ignore = "runs a full FLHE bot session; use --include-ignored to enable"]
fn test_flhe_bot_selfplay_replay_roundtrip() {
    let profile_names = ["tight_aggressive_flhe", "loose_passive_flhe"];
    let profiles: Vec<BotProfile> = profile_names
        .iter()
        .map(|n| {
            BotProfile::from_file(format!("data/bots/flhe/{n}.yaml"))
                .unwrap_or_else(|e| panic!("failed to load flhe/{n}.yaml: {e}"))
        })
        .collect();

    let seats_vec: Vec<SeatNoCell> = profiles
        .iter()
        .map(|p| SeatNoCell::new(PlayerNoCell::new_with_chips(p.name.clone(), STARTING_CHIPS)))
        .collect();
    let table =
        TableNoCell::limit_holdem_from_seats(SeatsNoCell::new(seats_vec), FL_SMALL_BET, FL_BIG_BET, FL_RAISE_CAP);
    let table_betting = table.betting;
    let mut session = PokerSession::new(table);
    let mut rng = rand::rng();
    let mut collection = HandCollection::new();

    for hand_num in 1..=NUM_HANDS {
        session.eliminate_busted();
        if session.count_funded() < 2 {
            break;
        }

        let button = session.table.button;
        let stacks: Vec<(u8, String, usize)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session
                    .table
                    .seats
                    .get_seat(i)
                    .filter(|s| !s.is_empty())
                    .map(|s| (i, s.player.handle.clone(), s.player.chips))
            })
            .collect();

        let event_log_start = session.table.event_log.len();

        session
            .start_hand()
            .unwrap_or_else(|e| panic!("FLHE start_hand failed on hand {hand_num}: {e}"));

        let hole_cards: Vec<(u8, Option<String>)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session.table.seats.get_seat(i).filter(|s| !s.is_empty()).map(|s| {
                    (
                        i,
                        if s.cards.has_cards() {
                            Some(s.cards.sorted_display())
                        } else {
                            None
                        },
                    )
                })
            })
            .collect();

        let player_snapshot: Vec<(u8, String, usize, Option<String>)> = stacks
            .iter()
            .map(|(seat, name, stack)| {
                let hole = hole_cards.iter().find(|(s, _)| s == seat).and_then(|(_, h)| h.clone());
                (*seat, name.clone(), *stack, hole)
            })
            .collect();

        while let Some(seat) = session.next_actor() {
            use pkcore::casino::action::PlayerAction;
            let action = profiles[seat as usize].decide(&session.table, seat, &mut rng);
            if session.apply_action(seat, action).is_err() {
                let fallback = if session.table.to_call(seat) > 0 {
                    PlayerAction::AllIn
                } else {
                    PlayerAction::Check
                };
                let _ = session.apply_action(seat, fallback);
            }
        }

        let board_str = session.table.board.to_string();
        let event_log = session.table.event_log[event_log_start..].to_vec();
        let winnings = session
            .end_hand()
            .unwrap_or_else(|e| panic!("FLHE end_hand failed on hand {hand_num}: {e}"));

        let ending_stacks: Vec<(u8, usize)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session
                    .table
                    .seats
                    .get_seat(i)
                    .filter(|s| !s.is_empty())
                    .map(|s| (i, s.player.chips))
            })
            .collect();

        let history = HandHistory::from_table_state(
            hand_num,
            0,
            button,
            &session.table.forced,
            &player_snapshot,
            &board_str,
            &winnings,
            &event_log,
            &ending_stacks,
            FLHE_SOURCE,
            session.shuffled_deck_str.clone(),
        )
        .with_betting_structure(table_betting);
        collection.push(history);
        session.table.button_up();
    }

    assert!(!collection.is_empty(), "at least one FLHE hand should play");

    // Confirm every recorded hand carries the FixedLimit structure.
    for h in collection.hands() {
        assert!(
            matches!(h.table.betting_structure, BettingStructure::FixedLimit { .. }),
            "every recorded FLHE hand must carry betting_structure: FixedLimit"
        );
    }

    let yaml = collection.to_yaml().expect("FLHE YAML serialize");
    let loaded = HandCollection::from_yaml(&yaml).expect("FLHE YAML deserialize");

    // Confirm the structure survives the round-trip.
    for h in loaded.hands() {
        assert!(
            matches!(h.table.betting_structure, BettingStructure::FixedLimit { .. }),
            "FLHE betting_structure must survive YAML round-trip"
        );
    }

    for (idx, result) in loaded.replay_all().into_iter().enumerate() {
        let replay =
            result.unwrap_or_else(|e| panic!("FLHE replay error for hand {}: {e}", loaded.hands()[idx].hand.id));
        assert!(
            replay.is_consistent,
            "FLHE replay mismatch for hand {}: final stacks = {:?}",
            loaded.hands()[idx].hand.id,
            replay.final_stacks,
        );
    }
}

// EPIC-31 Phase 9: PLO round-trip test. Records 10 hands of Pot-Limit
// Omaha (Hi), serializes to YAML, deserializes, and replays every hand —
// asserting that the recorded variant + betting_structure reconstructs a
// PLO replay table (uses `OmahaHigh` for showdown, not Hold'em's
// `Seven::eval`) and that chip totals match.

const PLO_SMALL_BLIND: usize = 5;
const PLO_BIG_BLIND: usize = 10;
const PLO_SOURCE: &str = "replay_consistency_plo";

#[test]
#[ignore = "runs a full PLO bot session; use --include-ignored to enable"]
fn test_plo_bot_selfplay_replay_roundtrip() {
    let profile_names = ["loose_aggressive_plo", "tight_aggressive_plo"];
    let profiles: Vec<BotProfile> = profile_names
        .iter()
        .map(|n| {
            BotProfile::from_file(format!("data/bots/plo/{n}.yaml"))
                .unwrap_or_else(|e| panic!("failed to load plo/{n}.yaml: {e}"))
        })
        .collect();

    let seats_vec: Vec<SeatNoCell> = profiles
        .iter()
        .map(|p| SeatNoCell::new(PlayerNoCell::new_with_chips(p.name.clone(), STARTING_CHIPS)))
        .collect();
    let table = TableNoCell::plo_from_seats(SeatsNoCell::new(seats_vec), (PLO_SMALL_BLIND, PLO_BIG_BLIND));
    let table_betting = table.betting;
    let mut session = PokerSession::new(table);
    let mut rng = rand::rng();
    let mut collection = HandCollection::new();

    for hand_num in 1..=NUM_HANDS {
        session.eliminate_busted();
        if session.count_funded() < 2 {
            break;
        }

        let button = session.table.button;
        let stacks: Vec<(u8, String, usize)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session
                    .table
                    .seats
                    .get_seat(i)
                    .filter(|s| !s.is_empty())
                    .map(|s| (i, s.player.handle.clone(), s.player.chips))
            })
            .collect();

        let event_log_start = session.table.event_log.len();

        session
            .start_hand()
            .unwrap_or_else(|e| panic!("PLO start_hand failed on hand {hand_num}: {e}"));

        let hole_cards: Vec<(u8, Option<String>)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session.table.seats.get_seat(i).filter(|s| !s.is_empty()).map(|s| {
                    (
                        i,
                        if s.cards.has_cards() {
                            Some(s.cards.sorted_display())
                        } else {
                            None
                        },
                    )
                })
            })
            .collect();

        let player_snapshot: Vec<(u8, String, usize, Option<String>)> = stacks
            .iter()
            .map(|(seat, name, stack)| {
                let hole = hole_cards.iter().find(|(s, _)| s == seat).and_then(|(_, h)| h.clone());
                (*seat, name.clone(), *stack, hole)
            })
            .collect();

        while let Some(seat) = session.next_actor() {
            use pkcore::casino::action::PlayerAction;
            let action = profiles[seat as usize].decide(&session.table, seat, &mut rng);
            if session.apply_action(seat, action).is_err() {
                let fallback = if session.table.to_call(seat) > 0 {
                    PlayerAction::AllIn
                } else {
                    PlayerAction::Check
                };
                let _ = session.apply_action(seat, fallback);
            }
        }

        let board_str = session.table.board.to_string();
        let event_log = session.table.event_log[event_log_start..].to_vec();
        let winnings = session
            .end_hand()
            .unwrap_or_else(|e| panic!("PLO end_hand failed on hand {hand_num}: {e}"));

        let ending_stacks: Vec<(u8, usize)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session
                    .table
                    .seats
                    .get_seat(i)
                    .filter(|s| !s.is_empty())
                    .map(|s| (i, s.player.chips))
            })
            .collect();

        let history = HandHistory::from_table_state(
            hand_num,
            0,
            button,
            &session.table.forced,
            &player_snapshot,
            &board_str,
            &winnings,
            &event_log,
            &ending_stacks,
            PLO_SOURCE,
            session.shuffled_deck_str.clone(),
        )
        .with_variant(HandVariant::Omaha)
        .with_betting_structure(table_betting);
        collection.push(history);
        session.table.button_up();
    }

    assert!(!collection.is_empty(), "at least one PLO hand should play");

    for h in collection.hands() {
        assert_eq!(
            HandVariant::Omaha,
            h.hand.game,
            "every recorded PLO hand must carry HandVariant::Omaha"
        );
        assert_eq!(
            BettingStructure::PotLimit,
            h.table.betting_structure,
            "every recorded PLO hand must carry betting_structure: PotLimit"
        );
    }

    let yaml = collection.to_yaml().expect("PLO YAML serialize");
    let loaded = HandCollection::from_yaml(&yaml).expect("PLO YAML deserialize");

    for h in loaded.hands() {
        assert_eq!(HandVariant::Omaha, h.hand.game, "variant survives round-trip");
        assert_eq!(
            BettingStructure::PotLimit,
            h.table.betting_structure,
            "betting_structure survives round-trip"
        );
    }

    for (idx, result) in loaded.replay_all().into_iter().enumerate() {
        let replay =
            result.unwrap_or_else(|e| panic!("PLO replay error for hand {}: {e}", loaded.hands()[idx].hand.id));
        assert!(
            replay.is_consistent,
            "PLO replay mismatch for hand {}: final stacks = {:?}",
            loaded.hands()[idx].hand.id,
            replay.final_stacks,
        );
    }
}

// EPIC-32 Phase 12: Seven-Card Stud Hi live-play smoke test. Exercises
// the full Stud machinery — antes, bring-in by lowest upcard, 5 streets
// with per-card visibility (Down/Down/Up/Up/Up/Up/Down), best-visible-
// hand action order on 4th+, fixed-limit small→big tier transition,
// showdown via `Seven::eval`. Records hand history with explicit
// per-card visibility array and verifies YAML round-trip preserves the
// fields.
//
// **Not a replay round-trip** (unlike the NLHE/FLHE/PLO tests). Stud
// replay correctness requires incremental card injection — the bring-in
// seat selection and best-visible-hand action order both depend on the
// per-street visible-card set, which doesn't survive a one-shot
// `inject_hole_cards` that loads all 7 cards at once. Implementing
// incremental Stud injection (or extending Streets to carry Stud
// 3rd/4th/5th/6th/7th action lists) is a v1.1 polish item.

const STUD_ANTE: usize = 2;
const STUD_BRING_IN: usize = 5;
const STUD_SMALL_BET: usize = 20;
const STUD_BIG_BET: usize = 40;
const STUD_SOURCE: &str = "replay_consistency_stud_hi";

#[test]
#[ignore = "runs a full Stud Hi bot session; use --include-ignored to enable"]
fn test_stud_hi_bot_selfplay_replay_roundtrip() {
    let profile_names = ["tight_aggressive_stud_hi", "loose_passive_stud_hi"];
    let profiles: Vec<BotProfile> = profile_names
        .iter()
        .map(|n| {
            BotProfile::from_file(format!("data/bots/stud_hi/{n}.yaml"))
                .unwrap_or_else(|e| panic!("failed to load stud_hi/{n}.yaml: {e}"))
        })
        .collect();

    let seats_vec: Vec<SeatNoCell> = profiles
        .iter()
        .map(|p| SeatNoCell::new(PlayerNoCell::new_with_chips(p.name.clone(), STARTING_CHIPS)))
        .collect();
    let table = TableNoCell::stud_hi_from_seats(
        SeatsNoCell::new(seats_vec),
        STUD_ANTE,
        STUD_BRING_IN,
        STUD_SMALL_BET,
        STUD_BIG_BET,
    );
    let table_betting = table.betting;
    let mut session = PokerSession::new(table);
    let mut rng = rand::rng();
    let mut collection = HandCollection::new();

    for hand_num in 1..=NUM_HANDS {
        session.eliminate_busted();
        if session.count_funded() < 2 {
            break;
        }

        let button = session.table.button;
        let stacks: Vec<(u8, String, usize)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session
                    .table
                    .seats
                    .get_seat(i)
                    .filter(|s| !s.is_empty())
                    .map(|s| (i, s.player.handle.clone(), s.player.chips))
            })
            .collect();

        let event_log_start = session.table.event_log.len();

        session
            .start_hand()
            .unwrap_or_else(|e| panic!("Stud start_hand failed on hand {hand_num}: {e}"));

        // For Stud, record hole_cards in DEALING order (not sorted) so
        // visibility array indices align. seat.hand.iter() iterates in
        // dealing order (push order).
        let hole_and_visibility: Vec<(u8, Option<String>, Option<Vec<String>>)> =
            (0..session.table.seats.0.len() as u8)
                .filter_map(|i| {
                    let seat = session.table.seats.get_seat(i)?;
                    if seat.is_empty() {
                        return None;
                    }
                    // At this point (right after start_hand) Stud has 3
                    // cards dealt to each seat. We'll capture the FULL
                    // 7-card sequence at end-of-hand instead; here we
                    // record an empty placeholder.
                    Some((i, None, None))
                })
                .collect();

        let player_snapshot: Vec<(u8, String, usize, Option<String>)> = stacks
            .iter()
            .map(|(seat, name, stack)| {
                let hole = hole_and_visibility
                    .iter()
                    .find(|(s, _, _)| s == seat)
                    .and_then(|(_, h, _)| h.clone());
                (*seat, name.clone(), *stack, hole)
            })
            .collect();

        while let Some(seat) = session.next_actor() {
            use pkcore::casino::action::PlayerAction;
            let action = profiles[seat as usize].decide(&session.table, seat, &mut rng);
            if session.apply_action(seat, action).is_err() {
                let fallback = if session.table.to_call(seat) > 0 {
                    PlayerAction::AllIn
                } else {
                    PlayerAction::Check
                };
                let _ = session.apply_action(seat, fallback);
            }
        }

        // After end of betting, capture the final 7-card sequence + per-
        // card visibility for each seat. Walk seat.hand in dealing order.
        let dealing_order_records: Vec<(u8, String, Vec<String>)> =
            (0..session.table.seats.0.len() as u8)
                .filter_map(|i| {
                    let seat = session.table.seats.get_seat(i)?;
                    if seat.is_empty() || !seat.cards.has_cards() {
                        return None;
                    }
                    let cards: Vec<String> = seat
                        .hand
                        .iter()
                        .map(|hc| hc.card().to_string())
                        .collect();
                    let visibility: Vec<String> = seat
                        .hand
                        .iter()
                        .map(|hc| if hc.is_up() { "up".to_string() } else { "down".to_string() })
                        .collect();
                    Some((i, cards.join(" "), visibility))
                })
                .collect();

        let board_str = session.table.board.to_string();
        let event_log = session.table.event_log[event_log_start..].to_vec();
        let winnings = session
            .end_hand()
            .unwrap_or_else(|e| panic!("Stud end_hand failed on hand {hand_num}: {e}"));

        let ending_stacks: Vec<(u8, usize)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session
                    .table
                    .seats
                    .get_seat(i)
                    .filter(|s| !s.is_empty())
                    .map(|s| (i, s.player.chips))
            })
            .collect();

        // Rebuild player_snapshot with the actual dealing-order cards
        // captured above. We need to do this BEFORE from_table_state so
        // the recorded hole_cards field carries the right values.
        let player_snapshot_final: Vec<(u8, String, usize, Option<String>)> = player_snapshot
            .iter()
            .map(|(seat, name, stack, _)| {
                let hole = dealing_order_records
                    .iter()
                    .find(|(s, _, _)| s == seat)
                    .map(|(_, cards, _)| cards.clone());
                (*seat, name.clone(), *stack, hole)
            })
            .collect();

        let mut history = HandHistory::from_table_state(
            hand_num,
            0,
            button,
            &session.table.forced,
            &player_snapshot_final,
            &board_str,
            &winnings,
            &event_log,
            &ending_stacks,
            STUD_SOURCE,
            session.shuffled_deck_str.clone(),
        )
        .with_variant(HandVariant::Stud)
        .with_betting_structure(table_betting);

        // Attach per-card visibility for each player in dealing order.
        for player in &mut history.players {
            if let Some((_, _, visibility)) = dealing_order_records
                .iter()
                .find(|(s, _, _)| *s == player.seat)
            {
                player.hole_cards_visibility = Some(visibility.clone());
            }
        }

        collection.push(history);
        session.table.button_up();
    }

    assert!(!collection.is_empty(), "at least one Stud hand should play");

    for h in collection.hands() {
        assert_eq!(
            HandVariant::Stud,
            h.hand.game,
            "every recorded Stud hand must carry HandVariant::Stud"
        );
        // Every player with hole_cards must have visibility in dealing
        // order (Down/Down/Up/Up/Up/Up/Down for a complete 7-card hand).
        for p in &h.players {
            if let (Some(cards), Some(visibility)) = (&p.hole_cards, &p.hole_cards_visibility) {
                let card_count = cards.split_whitespace().count();
                assert_eq!(
                    card_count,
                    visibility.len(),
                    "hole_cards/visibility length mismatch for hand {} seat {}",
                    h.hand.id,
                    p.seat,
                );
            }
        }
    }

    let yaml = collection.to_yaml().expect("Stud YAML serialize");
    let loaded = HandCollection::from_yaml(&yaml).expect("Stud YAML deserialize");

    for h in loaded.hands() {
        assert_eq!(HandVariant::Stud, h.hand.game, "variant survives round-trip");
        for p in &h.players {
            // Visibility array must survive YAML round-trip.
            if p.hole_cards.is_some() {
                assert!(
                    p.hole_cards_visibility.is_some(),
                    "Stud visibility array must round-trip"
                );
            }
        }
    }

    // Replay round-trip intentionally NOT exercised here — see test-level
    // comment for the incremental-injection gap. The session-level
    // chip-conservation is already validated by `end_hand`'s audit at
    // each hand.
}
