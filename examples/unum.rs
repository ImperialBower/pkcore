//! EPIC-87 corpus verifier: round-trips every logged hand back out through
//! [`Unumable`] and reports what did not survive the trip.
//!
//! `cargo run --example unum`
//!
//! This is the only file in EPIC-87 that touches the filesystem — the writer
//! itself is pure `String` formatting, and the kernel keeps it that way.

use pkcore::PKError;
use pkcore::analysis::nubibus::Pluribus;
use pkcore::bard::Bard;
use pkcore::prelude::{Nubificus, Unumable};
use std::str::FromStr;

/// Re-orders each player's two hole cards high-to-low **in the raw text**, so
/// the comparison is against a line pkcore's normalizing [`Two`] could
/// actually have produced.
///
/// Deliberately built by string surgery over the original line rather than by
/// rendering the parsed form: an oracle that goes through the writer would
/// agree with the writer no matter how wrong the writer is.
fn canonicalize(line: &str) -> String {
    let fields: Vec<&str> = line.split(':').collect();
    if fields.len() != 6 {
        return line.to_string();
    }

    let (dealt, board) = match fields[3].split_once('/') {
        Some((dealt, board)) => (dealt, Some(board)),
        None => (fields[3], None),
    };

    let sorted: Vec<String> = dealt
        .split('|')
        .map(|pair| {
            if pair.len() != 4 {
                return pair.to_string();
            }
            let (first, second) = pair.split_at(2);
            match (
                pkcore::prelude::Card::from_str(first),
                pkcore::prelude::Card::from_str(second),
            ) {
                (Ok(one), Ok(two)) if two > one => format!("{second}{first}"),
                _ => pair.to_string(),
            }
        })
        .collect();

    let cards = match board {
        Some(board) => format!("{}/{}", sorted.join("|"), board),
        None => sorted.join("|"),
    };

    format!(
        "{}:{}:{}:{}:{}:{}",
        fields[0], fields[1], fields[2], cards, fields[4], fields[5]
    )
}

/// [`canonicalize`] plus the flop, sorted high-to-low.
///
/// Tier 2 needs this and Tier 1 must not have it. A hand exported from a
/// finished [`Table`] is rebuilt from `event_log`, where `DealtFlop` carries a
/// single `Bard` — a bitset — so the order of the three flop cards is gone by
/// construction. Tier 1 renders straight from the parsed `Board`, which keeps
/// it, and is held to the stricter oracle.
fn canonicalize_with_flop(line: &str) -> String {
    let canonical = canonicalize(line);
    let fields: Vec<&str> = canonical.split(':').collect();
    if fields.len() != 6 {
        return canonical;
    }

    let Some((dealt, board)) = fields[3].split_once('/') else {
        return canonical;
    };

    let mut streets: Vec<String> = board.split('/').map(str::to_string).collect();
    if let Some(flop) = streets.first_mut()
        && flop.len() == 6
    {
        // `Cards::from(Bard)` walks `Bard::DECK`, which runs highest bit
        // first — spades, then hearts, diamonds, clubs, descending by rank
        // inside each suit. Sorting the *raw* text by that same bit value
        // models the loss without going anywhere near the writer.
        let mut cards: Vec<String> = (0..3).map(|i| flop[i * 2..i * 2 + 2].to_string()).collect();
        cards.sort_by_key(|card| {
            std::cmp::Reverse(
                pkcore::prelude::Card::from_str(card)
                    .map(|card| Bard::from(card).as_u64())
                    .unwrap_or_default(),
            )
        });
        *flop = cards.concat();
    }

    format!(
        "{}:{}:{}:{}/{}:{}:{}",
        fields[0],
        fields[1],
        fields[2],
        dealt,
        streets.join("/"),
        fields[4],
        fields[5]
    )
}

#[derive(Default)]
struct Tally {
    hands: usize,
    byte_exact: usize,
    canonical_exact: usize,
    replay_failed: usize,
    hypothesis_agreed: usize,
    hypothesis_disagreed: usize,
    hypothesis_none: usize,
    half_chip: usize,
    tier2_replayed: usize,
    tier2_exact: usize,
    tier2_failed: usize,
    tier2_stalled: usize,
    tier2_half_chip: usize,
    tier2_other: usize,
}

fn main() -> Result<(), PKError> {
    let logs = Nubificus::get_log_files("data/pluribus/raw/")?;
    let mut tally = Tally::default();
    let mut first_failures: Vec<String> = Vec::new();
    let mut tier2_failures: Vec<String> = Vec::new();

    for log in &logs {
        for line in std::fs::read_to_string(log)?.lines() {
            if !line.starts_with("STATE:") {
                continue;
            }
            let Ok(hand) = Pluribus::from_str(line) else {
                continue;
            };
            tally.hands += 1;

            if line.split(':').nth(4).is_some_and(|field| field.contains('.')) {
                tally.half_chip += 1;
            }

            let events: Vec<_> = hand.actions.iter().copied().collect();
            let Ok(simulated) = hand.actions_to_pluribus() else {
                tally.replay_failed += 1;
                continue;
            };

            match Pluribus::divider_hypothesis(&events, hand.players.len()) {
                Some(guessed) if guessed == simulated => tally.hypothesis_agreed += 1,
                Some(_) => tally.hypothesis_disagreed += 1,
                None => tally.hypothesis_none += 1,
            }

            let Ok(rendered) = hand.try_to_pluribus() else {
                tally.replay_failed += 1;
                continue;
            };

            if rendered == line {
                tally.byte_exact += 1;
            }
            if rendered == canonicalize(line) {
                tally.canonical_exact += 1;
            } else if first_failures.len() < 5 {
                first_failures.push(format!("  in : {line}\n  out: {rendered}"));
            }

            // Tier 2: replay the hand through the engine, then ask the
            // finished table to write the line back out.
            let mut nubificus = Nubificus::try_from(&hand)?;
            if nubificus.play_hand().is_err() {
                tally.tier2_failed += 1;
                continue;
            }
            tally.tier2_replayed += 1;
            match Pluribus::try_from(&nubificus.table) {
                Ok(mut exported) => {
                    exported.index = hand.index;
                    // Chip conservation: a hand that actually finished pays
                    // out exactly what it took in, so the net column sums to
                    // zero. A non-zero sum means the pot was never awarded.
                    let conserved = exported.winnings.iter().sum::<isize>() == 0;
                    let rendered = exported.to_pluribus();
                    if rendered == canonicalize_with_flop(line) {
                        tally.tier2_exact += 1;
                    } else if !conserved {
                        tally.tier2_stalled += 1;
                    } else if line.split(':').nth(4).is_some_and(|f| f.contains('.')) {
                        tally.tier2_half_chip += 1;
                    } else {
                        tally.tier2_other += 1;
                        if tier2_failures.len() < 5 {
                            tier2_failures.push(format!("  in : {line}\n  out: {rendered}"));
                        }
                    }
                }
                Err(_) => tally.tier2_failed += 1,
            }
        }
    }

    println!(
        "EPIC-87 corpus round trip — {} files, {} hands",
        logs.len(),
        tally.hands
    );
    println!("  byte exact vs raw line      : {}", tally.byte_exact);
    println!("  exact vs canonicalized line : {}", tally.canonical_exact);
    println!("  replay failed               : {}", tally.replay_failed);
    println!("  half-chip payoff hands      : {}", tally.half_chip);
    println!("tier 2 — replay, then export from the finished table");
    println!("  replayed to completion       : {}", tally.tier2_replayed);
    println!("  exact vs canonicalized line : {}", tally.tier2_exact);
    println!("  failed                      : {}", tally.tier2_failed);
    println!("  stalled (pot never awarded) : {}", tally.tier2_stalled);
    println!("  half-chip payoff            : {}", tally.tier2_half_chip);
    println!("  other                       : {}", tally.tier2_other);
    println!("divider hypothesis vs re-simulation");
    println!("  agreed                      : {}", tally.hypothesis_agreed);
    println!("  disagreed                   : {}", tally.hypothesis_disagreed);
    println!("  no answer                   : {}", tally.hypothesis_none);

    if !first_failures.is_empty() {
        println!("\nfirst tier 1 divergences:");
        for failure in &first_failures {
            println!("{failure}");
        }
    }

    if !tier2_failures.is_empty() {
        println!("\nfirst tier 2 divergences:");
        for failure in &tier2_failures {
            println!("{failure}");
        }
    }

    Ok(())
}
