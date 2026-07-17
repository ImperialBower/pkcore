//! EPIC-36 arena bench: rank YAML-configured `BotProfile`s by chips per 100
//! hands in a seeded, fixed-stack cash game.
//!
//! Each profile's graded `decision:` knobs (equity, ranges, pot_odds, exploit)
//! drive its play; strength is measured purely by arena result — no external
//! dataset is consulted.
//!
//! ```text
//! # Emit the reference weak / strong configs, then bench them:
//! cargo run --example bot_capability_bench -- --emit
//! cargo run --example bot_capability_bench -- --hands 20000 --seed 42 \
//!     data/bots/strong_all_on.yaml data/bots/weak_all_off.yaml
//! ```
//!
//! With no profile paths and no `--emit`, it benches the built-in strong-vs-weak
//! pair so the example runs out of the box.

use pkcore::bot::decision_config::{EquityMode, RangeMode};
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::{Player, Seat, Seats, Table};

const BUY_IN: usize = 10_000;
const SMALL_BLIND: usize = 50;
const BIG_BLIND: usize = 100;

/// gto base with every decision knob at its strong setting.
fn strong_profile() -> BotProfile {
    let mut p = BotProfile::gto();
    p.name = "strong_all_on".into();
    p.description = "gto base, all decision knobs on: exact equity, position-aware ranges, strict pot odds".into();
    p.decision.equity = EquityMode::Fast { samples: 1_000 };
    p.decision.ranges = RangeMode::PositionAware;
    p.decision.pot_odds.discipline = 1.0;
    p
}

/// gto base with every decision knob at its weak floor.
fn weak_profile() -> BotProfile {
    let mut p = BotProfile::gto();
    p.name = "weak_all_off".into();
    p.description = "gto base, all decision knobs off: hand-rank proxy, flat ranges, pot odds ignored".into();
    p.decision.equity = EquityMode::Off;
    p.decision.ranges = RangeMode::Flat;
    p.decision.pot_odds.discipline = 0.0;
    p
}

fn emit_configs() {
    for (path, profile) in [
        ("data/bots/strong_all_on.yaml", strong_profile()),
        ("data/bots/weak_all_off.yaml", weak_profile()),
    ] {
        match profile.to_file(path) {
            Ok(()) => println!("wrote {path}"),
            Err(e) => eprintln!("failed to write {path}: {e}"),
        }
    }
}

fn load_profiles(paths: &[String]) -> Vec<BotProfile> {
    paths
        .iter()
        .map(|p| BotProfile::from_file(p).unwrap_or_else(|e| panic!("failed to load {p}: {e}")))
        .collect()
}

fn bench(profiles: Vec<BotProfile>, hands: usize, seed: u64) {
    let seats = Seats::new(
        profiles
            .iter()
            .map(|p| Seat::new(Player::new_with_chips(p.name.clone(), BUY_IN)))
            .collect(),
    );
    let table = Table::nlh_from_seats(seats, ForcedBets::new(SMALL_BLIND, BIG_BLIND));
    let bots: Vec<(u8, BotProfile)> = profiles
        .iter()
        .enumerate()
        .map(|(i, p)| (u8::try_from(i).expect("seat index fits u8"), p.clone()))
        .collect();

    let mut sim = SimTable::with_rule_based(table, bots)
        .with_cash_mode(BUY_IN)
        .with_seed(seed);
    let result = sim.run_n_hands(hands).expect("bench run");

    let played = result.hands_played.max(1);
    println!(
        "\nchips/100 over {} hands (seed {seed}, cash buy-in {BUY_IN}):",
        result.hands_played
    );
    #[allow(clippy::cast_precision_loss)]
    let per_100 = |net: i64| net as f64 / (played as f64 / 100.0);
    let mut rows: Vec<(String, i64)> = profiles
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let seat = u8::try_from(i).expect("seat index fits u8");
            (p.name.clone(), *result.net_chips.get(&seat).unwrap_or(&0))
        })
        .collect();
    rows.sort_by_key(|r| std::cmp::Reverse(r.1));
    for (name, net) in rows {
        println!("  {name:<16} {:+10.1} chips/100", per_100(net));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--emit") {
        emit_configs();
        return;
    }

    let mut hands = 20_000usize;
    let mut seed = 42u64;
    let mut paths: Vec<String> = Vec::new();
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--hands" => hands = it.next().and_then(|v| v.parse().ok()).unwrap_or(hands),
            "--seed" => seed = it.next().and_then(|v| v.parse().ok()).unwrap_or(seed),
            _ => paths.push(arg),
        }
    }

    let profiles = if paths.is_empty() {
        println!("(no profile paths given; benching built-in strong vs weak)");
        vec![strong_profile(), weak_profile()]
    } else {
        load_profiles(&paths)
    };
    bench(profiles, hands, seed);
}
