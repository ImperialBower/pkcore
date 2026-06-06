//! `equity` — compute multi-way Texas hold'em equity from the command line.
//!
//! Each `--player` (`-p`) seat is one of:
//! * exact hole cards, e.g. `"As Kh"`
//! * a range, e.g. `"KK+,AKs"`
//! * `random` (or `?`) for an unknown opponent drawn from the deck
//!
//! The optional `--board` (`-b`) takes 3, 4, or 5 community cards; omit it for
//! pre-flop. Exact enumeration is used when the runout space is small, otherwise
//! seeded Monte Carlo (see `--samples` / `--seed`).
//!
//! Requires the `equity` feature. Use `--release` for Monte Carlo runs — hand
//! evaluation is many times slower in a debug build, so a large pre-flop sample
//! can take minutes unoptimized.
//!
//! ```text
//! # AA vs KK on a dry flop (exact, 990 runouts — fast even in debug)
//! cargo run --features equity --example equity -- -p "As Ah" -p "Ks Kh" -b "7d 8c 2s"
//!
//! # AK vs a KK+ range vs a random hand, pre-flop (Monte Carlo — use --release)
//! cargo run --release --features equity --example equity -- -p "Ah Kh" -p "KK+" -p random --samples 200000 --seed 7
//!
//! # A three-way all-in on the turn (exact, 42 runouts)
//! cargo run --features equity --example equity -- -p "Js Jd" -p "Ac Kc" -p "7h 7s" -b "2c 9d Th 5s"
//! ```

use clap::Parser;
use pkcore::analysis::equity::{EquityOptions, EquityRequest, PlayerSpec, compute};
use pkcore::analysis::gto::combos::Combos;
use pkcore::arrays::two::Two;
use pkcore::play::board::Board;
use std::str::FromStr;
use std::time::Instant;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Compute multi-way Texas hold'em equity", long_about = None)]
struct Args {
    /// A seat's holding: exact cards ("As Kh"), a range ("KK+,AKs"), or "random".
    /// Repeat for each of the 2–10 seats.
    #[clap(short = 'p', long = "player", required = true)]
    players: Vec<String>,

    /// Community cards, e.g. "9c 6d 5h". Omit for pre-flop.
    #[clap(short = 'b', long, default_value = "")]
    board: String,

    /// Monte Carlo sample cap (used when not enumerating exactly).
    #[clap(short = 's', long)]
    samples: Option<u64>,

    /// Enumerate exactly when the runout count is at or below this value.
    #[clap(long)]
    exact_threshold: Option<u64>,

    /// RNG seed for reproducible Monte Carlo runs.
    #[clap(long)]
    seed: Option<u64>,
}

/// Parses one `--player` argument into a [`PlayerSpec`].
///
/// Tries, in order: the literal `random`/`?`, then exact [`Two`] cards, then a
/// [`Combos`] range. The order matters — `"AA"` is not a valid pair of specific
/// cards, so it falls through to a range, while `"As Ah"` is matched as exact.
fn parse_player(s: &str) -> Result<PlayerSpec, String> {
    let trimmed = s.trim();
    if trimmed.eq_ignore_ascii_case("random") || trimmed == "?" {
        return Ok(PlayerSpec::Random);
    }
    if let Ok(two) = Two::from_str(trimmed) {
        return Ok(PlayerSpec::Exact(two));
    }
    if let Ok(combos) = Combos::from_str(trimmed) {
        return Ok(PlayerSpec::Range(combos));
    }
    Err(format!(
        "could not parse {trimmed:?} as cards (\"As Kh\"), a range (\"KK+\"), or \"random\""
    ))
}

fn main() {
    let args = Args::parse();

    let players: Vec<PlayerSpec> = match args.players.iter().map(|s| parse_player(s)).collect() {
        Ok(specs) => specs,
        Err(message) => {
            eprintln!("error: {message}");
            std::process::exit(2);
        }
    };

    let board = if args.board.trim().is_empty() {
        Board::default()
    } else {
        match Board::from_str(&args.board) {
            Ok(board) => board,
            Err(e) => {
                eprintln!("error: invalid board {:?}: {e:?}", args.board);
                std::process::exit(2);
            }
        }
    };

    let mut opts = EquityOptions::default();
    if let Some(samples) = args.samples {
        opts.max_samples = samples;
    }
    if let Some(threshold) = args.exact_threshold {
        opts.exact_threshold = threshold;
    }
    opts.seed = args.seed;

    let request = EquityRequest { players, board, opts };

    let start = Instant::now();
    let report = match compute(&request) {
        Ok(report) => report,
        Err(e) => {
            eprintln!("error: {e:?}");
            std::process::exit(1);
        }
    };
    let elapsed = start.elapsed();

    let board_label = if args.board.trim().is_empty() {
        "(pre-flop)".to_string()
    } else {
        args.board.clone()
    };
    println!("Board:  {board_label}");
    println!(
        "Method: {:?}   Cases: {}   Elapsed: {elapsed:?}",
        report.method, report.samples
    );
    println!();
    println!(
        "{:<3} {:<16} {:>7} {:>7} {:>9}",
        "#", "Player", "Win%", "Tie%", "Equity%"
    );
    println!("{}", "-".repeat(45));
    for (i, (label, pe)) in args.players.iter().zip(report.players.iter()).enumerate() {
        println!(
            "{i:<3} {label:<16} {:>7.2} {:>7.2} {:>9.2}",
            pe.win * 100.0,
            pe.tie * 100.0,
            pe.equity * 100.0
        );
    }
}
