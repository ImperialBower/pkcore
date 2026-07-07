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
use pkcore::casino::table::{Player, Seat, Seats, Table};
use uuid::Uuid;

/// Deep stacks reduce — but don't eliminate — the chance an aggressive bot
/// busts inside `HANDS`. `RuleBasedDecider` sizes raises as a fraction of the
/// pot, and the pot can grow multiplicatively across a multi-bet sequence
/// (3-bet, 4-bet, 5-bet across multiple streets), so a single big-pot loss
/// can swing tens to hundreds of millions even at deep stacks. Conservative
/// styles (`tight_passive`, `loose_passive`) reliably survive; loose
/// aggressive styles (`loose_aggressive`, `maniac`) sometimes bust early —
/// `vpip_if_seasoned` treats their early exit as "no opinion" rather than
/// a test failure.
const STARTING_CHIPS: usize = 1_000_000_000;
const SMALL_BLIND: usize = 50;
const BIG_BLIND: usize = 100;
const HANDS: usize = 100;
/// Minimum hands a survivor must see for their VPIP to be assertion-worthy.
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
    let mut seats: Vec<Seat> = Vec::with_capacity(styles.len());
    let mut style_to_uuid: Vec<(&'static str, Uuid)> = Vec::with_capacity(styles.len());
    for (name, _) in &styles {
        let player = Player::new_with_chips((*name).to_string(), STARTING_CHIPS);
        style_to_uuid.push((*name, player.id));
        seats.push(Seat::new(player));
    }
    let table = Table::nlh_from_seats(Seats::new(seats), ForcedBets::new(SMALL_BLIND, BIG_BLIND));

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
    // a stable read; None otherwise. Aggressive styles (`loose_aggressive`,
    // `maniac`) may bust before reaching the threshold when a multi-bet pot
    // war drains their stack in a single hand — we treat early exit as
    // "no opinion," not a test failure.
    let vpip_if_seasoned = |style: &str| -> Option<f64> {
        let (_, uuid) = style_to_uuid.iter().find(|(n, _)| *n == style)?;
        let ps = stats.get(*uuid)?;
        if ps.hands_dealt < MIN_HANDS_FOR_SURVIVOR_ASSERTION {
            return None;
        }
        ps.vpip()
    };

    // tight_passive folds frequently and reliably survives all 100 hands;
    // it's the test's load-bearing anchor.
    let tight_passive =
        vpip_if_seasoned("tight_passive").expect("tight_passive should survive 100 hands at deep stacks");

    // EPIC-26 design rationale: tight_passive plays modestly. Bands are
    // deliberately wide to absorb thread-local RNG variability — this is a
    // differentiation smoke test, not a regression on exact ratios.
    assert!(
        tight_passive < 0.45,
        "tight_passive VPIP should be modest, got {tight_passive:.3}"
    );

    // Opportunistic comparisons: any aggressive style that survived long
    // enough must read higher than tight_passive. Strongest signal of the
    // suite — if relative ordering ever flips, the deciders have stopped
    // honoring profile differences. Each style is checked independently so
    // the test still validates differentiation when one aggressive style
    // busts early.
    let mut survivors_checked = 0;
    for style in ["loose_aggressive", "maniac", "tight_aggressive"] {
        if let Some(vpip) = vpip_if_seasoned(style) {
            assert!(
                tight_passive < vpip,
                "tight_passive VPIP ({tight_passive:.3}) must be below {style} VPIP ({vpip:.3})"
            );
            survivors_checked += 1;
        }
    }
    assert!(
        survivors_checked >= 1,
        "at least one aggressive style must survive long enough to validate differentiation"
    );
}

#[test]
fn registry_records_one_hand_per_active_seat() {
    // Wiring sanity check at the integration boundary: every active seat
    // gets exactly `result.hands_played` hands_dealt, with no double-counting
    // or skipping. Complements the unit test in `bot::sim::tests` by
    // exercising the full SimTable + HandHistory + StatsRegistry pipeline
    // end-to-end through the public API only.
    let mut seats: Vec<Seat> = Vec::new();
    let mut uuids: Vec<Uuid> = Vec::new();
    for name in ["A", "B", "C"] {
        let p = Player::new_with_chips(name.to_string(), STARTING_CHIPS);
        uuids.push(p.id);
        seats.push(Seat::new(p));
    }
    let table = Table::nlh_from_seats(Seats::new(seats), ForcedBets::new(SMALL_BLIND, BIG_BLIND));
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
