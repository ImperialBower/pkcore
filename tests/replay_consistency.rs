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
use pkcore::hand_history::{HandCollection, HandHistory};
use uuid::Uuid;

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
        let stacks: Vec<(u8, String, usize, Uuid)> = (0..session.table.seats.0.len() as u8)
            .filter_map(|i| {
                session
                    .table
                    .seats
                    .get_seat(i)
                    .filter(|s| !s.is_empty())
                    .map(|s| (i, s.player.handle.clone(), s.player.chips, s.player.id))
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

        let player_snapshot: Vec<(u8, String, usize, Option<String>, Option<Uuid>)> = stacks
            .iter()
            .map(|(seat, name, stack, id)| {
                let hole = hole_cards.iter().find(|(s, _)| s == seat).and_then(|(_, h)| h.clone());
                (*seat, name.clone(), *stack, hole, Some(*id))
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
