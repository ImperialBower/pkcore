use clap::Parser;
use pkcore::prelude::*;

/// Shows all the unique suit-shifted variants of heads up pair of hands.
/// Each variant is evaluated as the same strength from an odds perspective.
///
/// See `docs/EPIC-07_Transposition.md` for a detailed description of concept and rationale behind
/// suit shifting.
///
/// ## Using default hand
/// `cargo run --example simple_suit_shift_example`
///
/// ## Specify custom hands
///
/// A smothered example is one where both cards share the same suits:
/// cargo run --example simple_suit_shift_example -- -c "2s 2h 3s 3h"
///
/// A covered example is one where they share one suit:
/// `cargo run --example simple_suit_shift_example -- -c "2s 2h 3s 3d"`
///
/// And here's an example where they share no suits:
/// `cargo run --example simple_suit_shift_example -- -c "8s 7h 7c 3d"`
///
/// ## View help
/// cargo run --example simple_suit_shift_example -- --help
#[derive(Parser, Debug)]
#[command(name = "Suit Shift Example")]
#[command(about = "Generate suit-shifted variants of a heads-up matchup", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "As Ks Ah Kh")]
    cards: String,
}

fn main() {
    let args = Args::parse();

    let hand: SortedHeadsUp = args.cards.parse().unwrap_or_else(|e| {
        eprintln!("Error parsing hand '{}': {}", args.cards, e);
        std::process::exit(1);
    });

    // Find all suit-shifted variants
    let all_shifts = hand.shifts();
    println!("Total variants: {}", all_shifts.len());

    for variant in all_shifts {
        println!("{}", variant);
    }
}
