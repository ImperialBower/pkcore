//! Audit a [`HandCollection`] YAML file for structural validity and game-engine
//! consistency.
//!
//! Two validation layers are applied to every hand:
//!
//! 1. **Bridge checks** — card string fields (`hole_cards`, `board`, `best_hand`)
//!    are parsed through pkcore's card types.  A failure here means the YAML
//!    contains a malformed card string.
//!
//! 2. **Replay** — the full action sequence is replayed through the game engine
//!    and the resulting chip counts are compared against the recorded `net` values.
//!    A failure here means the engine disagrees with what was recorded — the most
//!    common symptom of action-logging bugs in pkdealer / pkarena0-web.
//!
//! **Usage:**
//! ```text
//! # audit a specific file
//! cargo run --features hand-histories --example yaml_audit -- path/to/session.yaml
//!
//! # audit the most recent YAML in generated/
//! cargo run --features hand-histories --example yaml_audit
//! ```
//!
//! Exits 0 if every hand passes both layers, 1 if any hand fails.

use pkcore::hand_history::{HandCollection, HandHistory, PlayerEntry};
use std::path::PathBuf;
use std::process;

fn main() {
    let path = resolve_path();
    let yaml = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error: cannot read {}: {e}", path.display());
        process::exit(1);
    });
    let collection = HandCollection::from_yaml(&yaml).unwrap_or_else(|e| {
        eprintln!("error: cannot parse YAML in {}: {e}", path.display());
        process::exit(1);
    });

    let n = collection.len();
    println!(
        "=== YAML Audit: {}  ({} hand{}) ===",
        path.display(),
        n,
        if n == 1 { "" } else { "s" }
    );
    if let Some(ver) = collection.pkcore_version.as_str().split('+').next() {
        println!("    pkcore {}  format v{}", ver, collection.format_version);
    }
    println!();

    let mut passed = 0usize;
    for hand in collection.hands() {
        if audit_hand(hand) {
            passed += 1;
        }
    }

    println!();
    if passed == n {
        println!("=== Summary: {passed}/{n} passed ✓ ===");
    } else {
        println!("=== Summary: {passed}/{n} passed — {} FAILED ===", n - passed);
    }

    if passed < n {
        process::exit(1);
    }
}

// ── Per-hand audit ─────────────────────────────────────────────────────────────

/// Returns `true` if the hand passes both validation layers.
fn audit_hand(hand: &HandHistory) -> bool {
    let id = &hand.hand.id;
    let mut failures: Vec<String> = Vec::new();

    // ── Layer 1: bridge checks ────────────────────────────────────────────────

    // Board
    if hand.board.is_some() {
        if let Err(e) = hand.to_board() {
            failures.push(format!("board: invalid card string — {e}"));
        }
    }

    // Hole cards
    for p in &hand.players {
        if p.hole_cards.is_some() {
            if let Err(e) = p.to_two() {
                failures.push(format!(
                    "seat {} ({}): invalid hole_cards {:?} — {e}",
                    p.seat,
                    p.name,
                    p.hole_cards.as_deref().unwrap_or("")
                ));
            }
        }
    }

    // Best hands in results
    if let Some(results) = &hand.results {
        for r in results {
            if r.best_hand.is_some() {
                if let Err(e) = r.to_five() {
                    failures.push(format!(
                        "seat {} result: invalid best_hand {:?} — {e}",
                        r.seat,
                        r.best_hand.as_deref().unwrap_or("")
                    ));
                }
            }
        }
    }

    // ── Layer 2: replay ───────────────────────────────────────────────────────

    if failures.is_empty() {
        match hand.replay() {
            Err(e) => {
                failures.push(format!("replay error: {e}"));
            }
            Ok(result) if !result.is_consistent => {
                // Cross-reference replayed stacks with recorded net P&L to show
                // the exact per-seat delta.
                failures.push("chip mismatch (replayed vs recorded):".to_string());
                for (seat, replayed_chips) in &result.final_stacks {
                    if let Some(player) = hand.players.iter().find(|p| p.seat == *seat) {
                        let recorded_net = hand
                            .results
                            .as_deref()
                            .and_then(|rs| rs.iter().find(|r| r.seat == *seat))
                            .and_then(|r| r.net);

                        let mismatch_line = format_mismatch(player, *replayed_chips, recorded_net);
                        if let Some(line) = mismatch_line {
                            failures.push(line);
                        }
                    }
                }
            }
            Ok(_) => {} // consistent
        }
    }

    // ── Output ────────────────────────────────────────────────────────────────

    if failures.is_empty() {
        println!("  {id:<40}  PASS");
        true
    } else {
        println!("  {id:<40}  FAIL");
        for f in &failures {
            println!("      {f}");
        }
        false
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Formats one seat's chip mismatch.  Returns `None` when the stacks match within
/// the ±1 rounding tolerance (i.e. no problem to report for this seat).
fn format_mismatch(player: &PlayerEntry, replayed: usize, recorded_net: Option<f64>) -> Option<String> {
    let Some(net) = recorded_net else {
        // No net recorded — can't compare; skip.
        return None;
    };

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    let expected = (player.stack + net).max(0.0).round() as usize;

    if expected.abs_diff(replayed) <= 1 {
        return None; // within rounding tolerance
    }

    #[allow(clippy::cast_possible_wrap)]
    let delta = replayed as isize - expected as isize;
    Some(format!(
        "      seat {} ({:<20})  expected {:>6}  replayed {:>6}  Δ{:+}",
        player.seat, player.name, expected, replayed, delta
    ))
}

// ── File resolution ────────────────────────────────────────────────────────────

fn resolve_path() -> PathBuf {
    if let Some(arg) = std::env::args().nth(1) {
        return PathBuf::from(arg);
    }
    most_recent_yaml("generated").unwrap_or_else(|| {
        eprintln!("error: no YAML file given and no YAML files found in generated/.");
        eprintln!("  Usage: cargo run --features hand-histories --example yaml_audit -- <file.yaml>");
        process::exit(1);
    })
}

fn most_recent_yaml(dir: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "yaml"))
        .filter_map(|e| {
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((mtime, e.path()))
        })
        .max_by_key(|(mtime, _)| *mtime)
        .map(|(_, path)| path)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pkcore::hand_history::PlayerEntry;

    fn make_player(seat: u8, name: &str, stack: f64) -> PlayerEntry {
        PlayerEntry {
            seat,
            player_id: None,
            name: name.to_string(),
            stack,
            hole_cards: None,
            posted: None,
        }
    }

    #[test]
    fn format_mismatch_within_tolerance() {
        let player = make_player(0, "Alice", 1000.0);
        // expected = 1000 + 50 = 1050; replayed = 1050 — exact match
        assert!(format_mismatch(&player, 1050, Some(50.0)).is_none());
    }

    #[test]
    fn format_mismatch_within_one_chip_rounding() {
        let player = make_player(0, "Alice", 1000.0);
        // expected = 1050, replayed = 1051 — within ±1 rounding tolerance
        assert!(format_mismatch(&player, 1051, Some(50.0)).is_none());
    }

    #[test]
    fn format_mismatch_reports_real_delta() {
        let player = make_player(1, "Bob", 1000.0);
        // expected = 900, replayed = 850 — Δ-50, should report
        let msg = format_mismatch(&player, 850, Some(-100.0));
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("Bob"));
        assert!(msg.contains("900"));
        assert!(msg.contains("850"));
        assert!(msg.contains("-50"));
    }

    #[test]
    fn format_mismatch_no_recorded_net_returns_none() {
        let player = make_player(0, "Alice", 1000.0);
        assert!(format_mismatch(&player, 1000, None).is_none());
    }

    #[test]
    fn most_recent_yaml_missing_dir() {
        assert!(most_recent_yaml("/nonexistent/dir/xyzzy").is_none());
    }
}
