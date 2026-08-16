//! Integration test: 1000-hand bot marathon with all 8 standard profiles.
#![allow(non_snake_case)]
//!
//! Plays 1000 hands of NLH with all 8 bot profiles on 8 seats. After every
//! hand the test:
//!   - builds a [`HandHistory`] and pushes it to a running [`HandCollection`]
//!   - serializes the hand to YAML, deserializes it, and replays it via
//!     [`HandCollection::replay_all`], asserting `is_consistent` is true
//!
//! The built-in chip audit in [`end_hand`] (`PKError::ChipAuditFailed`) is
//! exercised on every hand automatically.
//!
//! On any error the full collection is serialized to a YAML string and
//! included in the panic message so the failing hand can be audited offline.
//!
//! Run with:
//! ```text
//! cargo test --test bot_marathon -- --include-ignored --nocapture
//! ```

use pkcore::bot::profile::BotProfile;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::session::PokerSession;
use pkcore::casino::table::{Player, Seat, Seats, Table};
use pkcore::hand_history::{HandCollection, HandHistory};

const SB: usize = 50;
const BB: usize = 100;
/// 10,000 BB each — makes busting in ≤1000 hands essentially impossible.
const STARTING_CHIPS: usize = 1_000_000;
const NUM_HANDS: usize = 1_000;
const SOURCE: &str = "bot_marathon";
const PROGRESS_INTERVAL: usize = 100;

/// Serializes the full hand collection to YAML, writes it to a file (so CI can
/// upload it as an artifact), and panics with a summary message.
///
/// The output path is controlled by the `MARATHON_DUMP_PATH` environment
/// variable (default: `marathon_failure.yaml`), letting the CI workflow point
/// the artifact upload step at the right location.
///
/// Returns `!` so it can be used in `unwrap_or_else` closures.
fn dump_and_panic(hand_num: usize, context: &str, msg: String, collection: &HandCollection) -> ! {
    let yaml = collection
        .to_yaml()
        .unwrap_or_else(|e| format!("(YAML serialization also failed: {e})"));
    let path = std::env::var("MARATHON_DUMP_PATH").unwrap_or_else(|_| "marathon_failure.yaml".to_string());
    let _ = std::fs::write(&path, &yaml);
    panic!(
        "bot_marathon FAILED at hand {hand_num} [{context}]: {msg}\n\
         (YAML written to {path} — download the CI artifact if the log is truncated)"
    );
}

/// Validates the most recently pushed hand in `collection` via a single-hand
/// YAML round-trip and replay consistency check.
///
/// On any failure calls [`dump_and_panic`] with the full collection so the
/// entire hand history up to the failure point is available for debugging.
fn validate_last_hand(hand_num: usize, collection: &HandCollection) {
    let mut single = HandCollection::new();
    single.push(collection.hands().last().unwrap().clone());

    let yaml = single
        .to_yaml()
        .unwrap_or_else(|e| dump_and_panic(hand_num, "to_yaml", e.to_string(), collection));
    let loaded = HandCollection::from_yaml(&yaml)
        .unwrap_or_else(|e| dump_and_panic(hand_num, "from_yaml", e.to_string(), collection));

    for (idx, result) in loaded.replay_all().into_iter().enumerate() {
        let replay = result.unwrap_or_else(|e| {
            dump_and_panic(
                hand_num,
                "replay",
                format!("{}: {e}", loaded.hands()[idx].hand.id),
                collection,
            )
        });
        if !replay.is_consistent {
            dump_and_panic(
                hand_num,
                "consistency",
                format!("{} final_stacks={:?}", loaded.hands()[idx].hand.id, replay.final_stacks),
                collection,
            );
        }
    }
}

/// Plays 1000 hands of NLH using all 8 standard bot profiles, validating chip
/// audit and YAML replay consistency after every hand.
///
/// On any error the full game YAML is included in the panic message for
/// offline debugging. Run with:
///
/// ```text
/// cargo test --test bot_marathon -- --include-ignored --nocapture
/// ```
#[test]
#[ignore = "marathon: 1000 hands with 8 bots; use --include-ignored"]
fn bot_marathon__1000_hands_without_error() {
    let profiles = BotProfile::default_profiles();

    let seats_vec: Vec<Seat> = profiles
        .iter()
        .map(|p| Seat::new(Player::new_with_chips(p.name.clone(), STARTING_CHIPS)))
        .collect();
    let table = Table::nlh_from_seats(Seats::new(seats_vec), ForcedBets::new(SB, BB));
    let mut session = PokerSession::new(table);
    let mut rng = rand::rng();
    let mut collection = HandCollection::new();
    let mut hands_played = 0usize;

    for hand_num in 1..=NUM_HANDS {
        session.eliminate_busted();
        if session.count_funded() < 2 {
            break;
        }

        let button = session.table.button;
        let event_log_start = session.table.event_log.len();

        // Capture stacks BEFORE start_hand() — act_forced_bets() modifies chips.
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

        if let Err(e) = session.start_hand() {
            dump_and_panic(hand_num, "start_hand", e.to_string(), &collection);
        }

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

        // Bot action loop. There is deliberately NO fallback here: a bot that
        // returns an action the engine rejects is a defect in the bot, and this
        // is the one place a long, blind-escalating run happens. The AllIn/Check
        // fallback that used to sit here is what hid DEFECT_007 for three months
        // and two releases — it turned the harness that would have caught the
        // defect into the harness that concealed it.
        while let Some(seat) = session.next_actor() {
            let profile = &profiles[seat as usize % profiles.len()];
            let action = profile.decide(&session.table, seat, &mut rng);
            if let Err(e) = session.apply_action(seat, action) {
                dump_and_panic(
                    hand_num,
                    "apply_action",
                    format!(
                        "seat {seat} returned {action:?}, which the engine rejected: {e} \
                         (to_call={} min_raise_to={} raise_bounds={:?})",
                        session.table.to_call(seat),
                        session.table.min_raise_to(),
                        session.table.raise_bounds(seat),
                    ),
                    &collection,
                );
            }
        }

        let board_str = session.table.board.to_string();
        let event_log = session.table.event_log[event_log_start..].to_vec();

        // end_hand() includes the chip conservation audit internally.
        let winnings = match session.end_hand() {
            Ok(w) => w,
            Err(e) => dump_and_panic(hand_num, "end_hand", e.to_string(), &collection),
        };

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

        validate_last_hand(hand_num, &collection);

        session.table.button_up();

        hands_played = hand_num;

        if hand_num % PROGRESS_INTERVAL == 0 {
            println!("bot_marathon: {hand_num}/{NUM_HANDS} hands complete");
        }
    }

    assert!(
        hands_played >= NUM_HANDS,
        "marathon ended early after only {hands_played} hands (needed {NUM_HANDS})"
    );

    println!("bot_marathon: complete — {hands_played} hands played without error");
}
