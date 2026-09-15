//! The entropy-free kernel, run for real.
//!
//! The rest of the suite needs the `entropy` feature, because it builds its
//! tables with `Player::new`-style conveniences that mint random ids. This
//! target uses only the seeded and id-taking APIs, so it compiles against
//! `--no-default-features` and proves the kernel build plays deterministic
//! poker (`docs/KERNEL_PURITY_AUDIT.md` §1, fix 8). It is an integration test, not a
//! colocated one, for exactly that reason: the lib's unit-test binary cannot be
//! built without `entropy`.
//!
//! `make test-kernel` runs it as
//! `cargo test --no-default-features --test kernel_determinism`. The tests
//! without a `cfg` also pass with `entropy` on.

use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::action::PlayerAction;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::session::PokerSession;
use pkcore::casino::table::{Player, Seat, Seats, Table};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use uuid::Uuid;

fn heads_up_table() -> Table {
    let seats = Seats::new(vec![
        Seat::new(Player::with_id(Uuid::from_u128(1), "Alice".to_string(), 2_000)),
        Seat::new(Player::with_id(Uuid::from_u128(2), "Bob".to_string(), 2_000)),
    ]);
    Table::nlh_from_seats_with_id(seats, ForcedBets::new(10, 20), Uuid::from_u128(42))
}

fn stacks(table: &Table) -> Vec<usize> {
    table.seats.0.iter().map(|seat| seat.player.chips).collect()
}

#[test]
fn a_seeded_session_plays_the_same_hand_twice() {
    let play = || {
        let mut session = PokerSession::new(heads_up_table());
        let winnings = session
            .run_hand_with(&mut SmallRng::seed_from_u64(7), |_table, _seat| PlayerAction::Call)
            .unwrap();
        (
            winnings,
            session.table.event_log.clone(),
            session.table.board.to_string(),
        )
    };
    assert_eq!(play(), play());
}

#[test]
fn a_seeded_sim_plays_the_same_hands_twice() {
    let run = || {
        let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::tight_passive())];
        let mut sim = SimTable::with_rule_based(heads_up_table(), bots).with_seed(42);
        let result = sim.run_n_hands(10).unwrap();
        (
            result.hands_played,
            result.net_chips.get(&0).copied(),
            result.net_chips.get(&1).copied(),
        )
    };
    assert_eq!(run(), run());
}

#[cfg(not(feature = "entropy"))]
#[test]
fn without_entropy_an_unseeded_sim_is_deterministic() {
    let run = || {
        let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::tight_passive())];
        let mut sim = SimTable::with_rule_based(heads_up_table(), bots);
        sim.run_n_hands(10).unwrap().net_chips.get(&0).copied()
    };
    assert_eq!(run(), run());
}

#[cfg(not(feature = "entropy"))]
#[test]
fn without_entropy_the_same_seed_as_default_seed_gives_the_same_run() {
    let run = |sim: SimTable| {
        let mut sim = sim;
        sim.run_n_hands(10).unwrap().net_chips.get(&0).copied()
    };
    let bots = || vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::tight_passive())];
    assert_eq!(
        run(SimTable::with_rule_based(heads_up_table(), bots())),
        run(SimTable::with_rule_based(heads_up_table(), bots()).with_seed(SimTable::DEFAULT_SEED)),
    );
}

#[cfg(all(feature = "equity", not(feature = "entropy")))]
#[test]
fn without_entropy_a_seedless_equity_request_is_deterministic() {
    use pkcore::analysis::equity::{EquityRequest, PlayerSpec};
    use pkcore::arrays::two::Two;
    let counts = || {
        let report = EquityRequest::new(vec![
            PlayerSpec::Exact(Two::HAND_AS_AH),
            PlayerSpec::Exact(Two::HAND_KS_KH),
        ])
        .compute()
        .unwrap();
        report.players.iter().map(|p| (p.wins, p.ties)).collect::<Vec<_>>()
    };
    assert_eq!(counts(), counts());
}

#[cfg(not(feature = "entropy"))]
#[test]
fn without_entropy_the_commentary_faces_are_fixed() {
    use pkcore::util::terminal::Terminal;
    assert_eq!(Terminal::random_happy(), Terminal::random_happy());
    assert_eq!(Terminal::random_sad(), Terminal::random_sad());
}

#[test]
fn a_played_hand_conserves_chips() {
    let mut session = PokerSession::new(heads_up_table());
    let before: usize = stacks(&session.table).iter().sum();
    session
        .run_hand_with(&mut SmallRng::seed_from_u64(3), |_table, _seat| PlayerAction::Call)
        .unwrap();
    assert_eq!(before, stacks(&session.table).iter().sum::<usize>());
}
