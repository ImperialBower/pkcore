//! EPIC-26 Phase 1 back-compat: pre-`player_id` YAML files must still parse
//! and re-serialize stably.
//!
//! `Action.player_id` and `PlayerEntry.player_id` were added with
//! `#[serde(default, skip_serializing_if = "Option::is_none")]`, so legacy
//! files (no `player_id` anywhere) should:
//!   1. Deserialize without error, leaving `player_id` as `None`.
//!   2. Re-serialize without injecting a `player_id` key.
//!   3. Re-deserialize the result to a value equal to the first parse.

#![cfg(feature = "hand-histories")]

use pkcore::hand_history::{HandCollection, HandHistory};

const PKARENA_SESSION_YAML: &str = include_str!("../data/hands/legacy/pkarena0-session_2026-04-15.yaml");
const THE_HAND_YAML: &str = include_str!("../data/hands/the_hand.yaml");

#[test]
fn legacy_collection_round_trips_without_player_id() {
    let original = HandCollection::from_yaml(PKARENA_SESSION_YAML).expect("legacy session YAML should parse");

    for hand in original.hands() {
        for player in &hand.players {
            assert!(
                player.player_id.is_none(),
                "legacy YAML should not carry player_id (seat {})",
                player.seat
            );
        }
        if let Some(streets) = &hand.streets {
            for street_actions in [
                streets.preflop.as_ref().map(|s| &s.actions),
                streets.flop.as_ref().map(|s| &s.actions),
                streets.turn.as_ref().map(|s| &s.actions),
                streets.river.as_ref().map(|s| &s.actions),
            ]
            .into_iter()
            .flatten()
            {
                for action in street_actions {
                    assert!(
                        action.player_id.is_none(),
                        "legacy YAML action should not carry player_id (seat {})",
                        action.seat
                    );
                }
            }
        }
    }

    let re_serialized = original.to_yaml().expect("collection should re-serialize");
    assert!(
        !re_serialized.contains("player_id"),
        "round-tripped YAML must not introduce a player_id field"
    );

    let reparsed = HandCollection::from_yaml(&re_serialized).expect("re-serialized YAML should parse");
    assert_eq!(original, reparsed, "round-tripped HandCollection should equal original");
}

#[test]
fn legacy_hand_history_round_trips_without_player_id() {
    let original = HandHistory::from_yaml(THE_HAND_YAML).expect("legacy hand YAML should parse");

    for player in &original.players {
        assert!(player.player_id.is_none());
    }

    let re_serialized = original.to_yaml().expect("hand history should re-serialize");
    assert!(!re_serialized.contains("player_id"));

    let reparsed = HandHistory::from_yaml(&re_serialized).expect("re-serialized YAML should parse");
    assert_eq!(original, reparsed);
}
