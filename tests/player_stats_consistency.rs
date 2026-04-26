//! EPIC-26 Phase 5c — bot self-play stats consistency.
//!
//! Runs a multi-hand bot session with a [`StatsRegistry`] attached and
//! verifies that the resulting per-player VPIP rates land in the range you
//! would expect for each [`BotProfile`] style.  This is a smoke test that
//! styles are *actually differentiated* by the rule-based decider — not a
//! regression test on exact ratios.  Bands are deliberately loose to absorb
//! the thread-local RNG variability that `RuleBasedDecider` introduces
//! (see Unit B's regression-test discussion).

use pkcore::analysis::player_stats::StatsRegistry;
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
use uuid::Uuid;

/// Stacks chosen so per-hand losses cannot sum to a bust within `HANDS`.
/// Even an aggressive bot losing every hand at 50/100 blinds with full-pot
/// raises tops out near `HANDS * BIG_BLIND * pot_multiplier` ≈ 10M chips —
/// 1B leaves two orders of magnitude of headroom against bust.
const STARTING_CHIPS: usize = 1_000_000_000;
const SMALL_BLIND: usize = 50;
const BIG_BLIND: usize = 100;
const HANDS: usize = 100;
/// Minimum hands a survivor must see for their VPIP to be assertion-worthy.
/// At 1B starting chips against 50/100 blinds, every seated profile reliably
/// survives all 100 hands; the `maniac` opportunistic check at the bottom
/// handles the unlikely edge case anyway.
const MIN_HANDS_FOR_SURVIVOR_ASSERTION: u64 = 30;

/// Bot styles seated at the table, in seat order.
fn styles() -> Vec<(&'static str, BotProfile)> {
    vec![
        ("tight_passive", BotProfile::tight_passive()),
        ("loose_aggressive", BotProfile::loose_aggressive()),
        ("gto", BotProfile::gto()),
        ("maniac", BotProfile::maniac()),
        ("tight_aggressive", BotProfile::tight_aggressive()),
        ("loose_passive", BotProfile::loose_passive()),
    ]
}

#[test]
fn vpip_differentiates_styles_after_self_play() {
    let styles = styles();

    // Build seats and capture each player's Uuid *before* moving them into
    // the table so we can look up their stats afterwards.
    let mut seats: Vec<SeatNoCell> = Vec::with_capacity(styles.len());
    let mut style_to_uuid: Vec<(&'static str, Uuid)> = Vec::with_capacity(styles.len());
    for (name, _) in &styles {
        let player = PlayerNoCell::new_with_chips((*name).to_string(), STARTING_CHIPS);
        style_to_uuid.push((*name, player.id));
        seats.push(SeatNoCell::new(player));
    }
    let table = TableNoCell::nlh_from_seats(SeatsNoCell::new(seats), ForcedBets::new(SMALL_BLIND, BIG_BLIND));

    let bots: Vec<(u8, BotProfile)> = styles
        .iter()
        .enumerate()
        .map(|(i, (_, p))| (u8::try_from(i).expect("six seats fits u8"), p.clone()))
        .collect();

    let mut sim = SimTable::with_stats_registry(table, bots, StatsRegistry::new());
    let result = sim.run_n_hands(HANDS).expect("session must complete");
    assert!(
        result.hands_played * 2 >= HANDS,
        "session collapsed too early: only {} of {HANDS} hands played",
        result.hands_played
    );

    let stats = sim.stats().expect("registry attached");
    assert!(!stats.is_empty(), "registry should hold per-player stats");

    // Returns Some(vpip) for any seated player who survived long enough for
    // a stable read; None otherwise. Aggressive styles like `maniac` may
    // bust before reaching the threshold — we treat their early exit as
    // "no opinion," not as a test failure.
    let vpip_if_seasoned = |style: &str| -> Option<f64> {
        let (_, uuid) = style_to_uuid.iter().find(|(n, _)| *n == style)?;
        let ps = stats.get(*uuid)?;
        if ps.hands_dealt < MIN_HANDS_FOR_SURVIVOR_ASSERTION {
            return None;
        }
        ps.vpip()
    };

    // tight_passive and loose_aggressive both reliably survive: blind costs
    // are 50/100 against a 10M stack, so even sustained losses can't bust
    // them in 100 hands. These two are the load-bearing comparison.
    let tight_passive = vpip_if_seasoned("tight_passive").expect("tight_passive should survive 100 hands at 10M chips");
    let loose_aggressive =
        vpip_if_seasoned("loose_aggressive").expect("loose_aggressive should survive 100 hands at 10M chips");

    // EPIC-26 design rationale: tight_passive plays modestly, loose_aggressive
    // plays a wider range. Bands are deliberately wide to absorb thread-local
    // RNG variability — this is a differentiation smoke test, not a regression
    // on exact ratios.
    assert!(
        tight_passive < 0.45,
        "tight_passive VPIP should be modest, got {tight_passive:.3}"
    );
    assert!(
        loose_aggressive > 0.30,
        "loose_aggressive VPIP should be elevated, got {loose_aggressive:.3}"
    );

    // Strongest signal: relative ordering. If this ever flips, the deciders
    // have stopped honoring profile differences — a real regression.
    assert!(
        tight_passive < loose_aggressive,
        "tight_passive VPIP ({tight_passive:.3}) must be below loose_aggressive VPIP ({loose_aggressive:.3})"
    );

    // Opportunistic check: when the maniac survives long enough, their VPIP
    // should be even higher than loose_aggressive. Skipped silently when
    // they busted early — that's expected behavior, not a regression.
    if let Some(maniac) = vpip_if_seasoned("maniac") {
        assert!(
            maniac > loose_aggressive,
            "maniac VPIP ({maniac:.3}) should exceed loose_aggressive ({loose_aggressive:.3}) when maniac survives"
        );
    }
}

#[test]
fn registry_records_one_hand_per_active_seat() {
    // Wiring sanity check at the integration boundary: every active seat
    // gets exactly `result.hands_played` hands_dealt, with no double-counting
    // or skipping. Complements the unit test in `bot::sim::tests` by
    // exercising the full SimTable + HandHistory + StatsRegistry pipeline
    // end-to-end through the public API only.
    let mut seats: Vec<SeatNoCell> = Vec::new();
    let mut uuids: Vec<Uuid> = Vec::new();
    for name in ["A", "B", "C"] {
        let p = PlayerNoCell::new_with_chips(name.to_string(), STARTING_CHIPS);
        uuids.push(p.id);
        seats.push(SeatNoCell::new(p));
    }
    let table = TableNoCell::nlh_from_seats(SeatsNoCell::new(seats), ForcedBets::new(SMALL_BLIND, BIG_BLIND));
    let bots = vec![
        (0_u8, BotProfile::gto()),
        (1_u8, BotProfile::tight_passive()),
        (2_u8, BotProfile::loose_aggressive()),
    ];

    let mut sim = SimTable::with_stats_registry(table, bots, StatsRegistry::new());
    let result = sim.run_n_hands(25).expect("session must complete");

    let stats = sim.stats().expect("registry attached");
    assert_eq!(3, stats.len(), "three players must each have a stats entry");

    let hands_played = u64::try_from(result.hands_played).expect("fits u64");
    for uuid in &uuids {
        let ps = stats.get(*uuid).expect("each seat must have stats");
        assert_eq!(
            hands_played, ps.hands_dealt,
            "player {uuid} should have hands_dealt == hands_played"
        );
    }
}
