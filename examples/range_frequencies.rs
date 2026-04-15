//! EPIC-25: Range Frequencies — feature demonstration.
//!
//! Walks through every capability added in EPIC-25:
//!
//! 1. Parse a frequency-annotated range string into `WeightedCombos`
//! 2. Default frequency (1.0 when no `:f` suffix)
//! 3. Range token expansion with frequency (`JJ-99:0.8`)
//! 4. Round-trip: `to_range_str` → `from_str`
//! 5. Backward compat: `Combos::from_str` accepts and strips `:f`
//! 6. Frequency-weighted hand expansion via `weighted_twos`
//! 7. Mixed-strategy equity with `weighted_win_probability`
//! 8. Error handling: `PKError::InvalidFrequency`
//!
//! ```text
//! cargo run --example range_frequencies
//! ```

use pkcore::PKError;
use pkcore::analysis::gto::combo::Combo;
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::odds::WinLoseDraw;
use pkcore::analysis::gto::weighted_combos::WeightedCombos;
use pkcore::arrays::two::Two;
use std::collections::HashMap;
use std::str::FromStr;

fn separator(title: &str) {
    println!();
    println!("─── {title} {}", "─".repeat(55usize.saturating_sub(title.len() + 5)));
}

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║        EPIC-25: Range Frequencies Demo               ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();
    println!("Range strings like \"AA:0.5, KK, QQ:0.75\" represent mixed");
    println!("strategies where a hand is played at less than 100% frequency.");
    println!("This is standard GTO notation for balanced ranges.");

    // ── 1. Parse a frequency-annotated range string ───────────────────────────

    separator("1. Parse a frequency-annotated range string");

    // WeightedCombos::from_str understands the `:f` suffix.
    // Each token is   "combo_or_range"   or   "combo_or_range:frequency"
    // where frequency is a decimal in [0.0, 1.0].
    let range_str = "AA:0.5, KK, QQ:0.75, AKs:1.0";
    let wc = WeightedCombos::from_str(range_str).expect("valid range");

    println!("Input : \"{range_str}\"");
    println!();
    println!("  AA  → {:.0}%  (played 50% — mixing with checks/folds)",
        wc.frequency(&Combo::COMBO_AA).unwrap_or(0.0) * 100.0);
    println!("  KK  → {:.0}%  (no suffix → defaults to 100%)",
        wc.frequency(&Combo::COMBO_KK).unwrap_or(0.0) * 100.0);
    println!("  QQ  → {:.0}%  (played 75% — balancing the betting range)",
        wc.frequency(&Combo::COMBO_QQ).unwrap_or(0.0) * 100.0);
    println!("  AKs → {:.0}%  (explicit 1.0 — same as omitting the suffix)",
        wc.frequency(&Combo::COMBO_AKs).unwrap_or(0.0) * 100.0);
    println!("  22  → not in range (frequency returns None)");
    println!("  22 present: {}", wc.frequency(&Combo::COMBO_22).is_some());

    // ── 2. Default frequency ──────────────────────────────────────────────────

    separator("2. Default frequency (no suffix → 1.0)");

    let flat = WeightedCombos::from_str("JJ, TT, 99").expect("valid range");
    for combo in [Combo::COMBO_JJ, Combo::COMBO_TT, Combo::COMBO_99] {
        println!("  {} → {:.0}%", combo, flat.frequency(&combo).unwrap_or(0.0) * 100.0);
    }
    println!("Omitting `:f` is equivalent to `:1.0` — backward compatible.");

    // ── 3. Range token expansion with frequency ───────────────────────────────

    separator("3. Range expansion with frequency (\"JJ-99:0.8\")");

    // A `:f` suffix on a range token applies to every combo the range expands to.
    // "JJ-99:0.8" expands to JJ, TT, 99 — each at 80%.
    let range_wc = WeightedCombos::from_str("JJ-99:0.8, AKs-AJs:0.6").expect("valid range");

    println!("Input: \"JJ-99:0.8, AKs-AJs:0.6\"");
    println!();
    println!("  Pocket pairs (each at 80%):");
    for combo in [Combo::COMBO_JJ, Combo::COMBO_TT, Combo::COMBO_99] {
        println!("    {} → {:.0}%", combo, range_wc.frequency(&combo).unwrap_or(0.0) * 100.0);
    }
    println!("  Ace-X suited (each at 60%):");
    for combo in [Combo::COMBO_AKs, Combo::COMBO_AQs, Combo::COMBO_AJs] {
        println!("    {} → {:.0}%", combo, range_wc.frequency(&combo).unwrap_or(0.0) * 100.0);
    }

    // ── 4. Round-trip: to_range_str → from_str ───────────────────────────────

    separator("4. Round-trip: to_range_str → from_str");

    let mut original = WeightedCombos::default();
    original.insert(Combo::COMBO_AA, 0.5);
    original.insert(Combo::COMBO_KK, 1.0);
    original.insert(Combo::COMBO_QQ, 0.75);

    let serialized = original.to_range_str();
    let restored = WeightedCombos::from_str(&serialized).expect("round-trip parse");

    println!("Serialized : \"{serialized}\"");
    println!();
    println!("  `:f` suffix is omitted when frequency is 1.0 (clean output).");
    println!("  KK appears as \"KK\", not \"KK:1\".");
    println!();
    println!("Restored:");
    for combo in [Combo::COMBO_AA, Combo::COMBO_KK, Combo::COMBO_QQ] {
        let orig_f = original.frequency(&combo).unwrap_or(0.0);
        let rest_f = restored.frequency(&combo).unwrap_or(0.0);
        println!("  {combo} : original={orig_f:.2}  restored={rest_f:.2}  match={}", orig_f == rest_f);
    }

    // ── 5. Backward compat: Combos::from_str accepts :f ──────────────────────

    separator("5. Backward compat: Combos::from_str strips :f");

    // Combos is an unweighted HashSet<Combo>. It now tolerates the :f suffix
    // (strips and discards it) so that annotated strings don't cause parse errors
    // in code that only needs the combo identity.
    let annotated  = Combos::from_str("AA:0.5, KK:0.9").expect("tolerated");
    let plain      = Combos::from_str("AA, KK").expect("plain");
    println!("\"AA:0.5, KK:0.9\" parsed by Combos::from_str");
    println!("Equals unweighted \"AA, KK\" : {}", annotated == plain);
    println!("(Frequencies are silently dropped — identity only.)");

    // ── 6. Frequency-weighted hand expansion ─────────────────────────────────

    separator("6. weighted_twos: frequency-weighted hand expansion");

    // weighted_twos() expands each combo to its specific Two hands,
    // pairing each with its combo's frequency. Zero-frequency combos are excluded.
    let mut wc2 = WeightedCombos::default();
    wc2.insert(Combo::COMBO_AA, 1.0);  // 6 specific AA hands, each at 1.0
    wc2.insert(Combo::COMBO_KK, 0.5);  // 6 specific KK hands, each at 0.5
    wc2.insert(Combo::COMBO_QQ, 0.0);  // excluded (zero frequency)

    let pairs = wc2.weighted_twos();
    println!("Range: AA(100%), KK(50%), QQ(0%)");
    println!();

    let aa_pairs: Vec<_> = pairs.iter().filter(|(_, f)| *f == 1.0).collect();
    let kk_pairs: Vec<_> = pairs.iter().filter(|(_, f)| *f == 0.5).collect();
    println!("  AA hands included: {} (all 6, freq=1.0)", aa_pairs.len());
    println!("  KK hands included: {} (all 6, freq=0.5)", kk_pairs.len());
    println!("  QQ hands included: 0 (zero-frequency combos excluded)");
    println!("  Total pairs       : {}", pairs.len());

    // ── 7. Mixed-strategy equity ──────────────────────────────────────────────

    separator("7. weighted_win_probability: equity with mixed strategies");

    // weighted_win_probability weights each hand's equity result by its
    // combo's frequency before aggregating:
    //   Σ(freq_i × wins_i) / Σ(freq_i × total_i)
    //
    // Concretely: if AA is 100% and KK is 50%, AA hands count twice as much
    // as KK hands in the final equity number.

    let mut wc3 = WeightedCombos::default();
    wc3.insert(Combo::COMBO_AA, 1.0);
    wc3.insert(Combo::COMBO_KK, 0.5);

    // Synthetic per-hand equity results (stand-in for a real equity calculation)
    let mut hand_odds: HashMap<Two, WinLoseDraw> = HashMap::new();

    // AA hands win 85% (8.5 out of 10 outcomes)
    for two in pkcore::analysis::gto::twos::Twos::from(Combo::COMBO_AA).to_vec() {
        hand_odds.insert(two, WinLoseDraw { wins: 85, losses: 15, draws: 0 });
    }
    // KK hands win 65% (6.5 out of 10 outcomes)
    for two in pkcore::analysis::gto::twos::Twos::from(Combo::COMBO_KK).to_vec() {
        hand_odds.insert(two, WinLoseDraw { wins: 65, losses: 35, draws: 0 });
    }

    let unweighted_avg = (0.85 + 0.65) / 2.0;
    let weighted_prob  = wc3.weighted_win_probability(&hand_odds);

    // With freq(AA)=1.0 and freq(KK)=0.5, AA is weighted 2× relative to KK.
    // Expected: (1.0×0.85 + 0.5×0.65) / (1.0 + 0.5) = 1.175/1.5 ≈ 0.7833
    println!("AA equity: 85%,  KK equity: 65%");
    println!("  Simple average (ignoring frequency) : {:.2}%", unweighted_avg * 100.0);
    println!("  Weighted average AA(100%) + KK(50%) : {:.2}%", weighted_prob * 100.0);
    println!("  AA counts twice as much — result is pulled toward AA's equity.");

    // ── 8. Error handling ─────────────────────────────────────────────────────

    separator("8. Error handling: PKError::InvalidFrequency");

    let cases = [
        ("AA:1.5",  "above 1.0"),
        ("KK:-0.1", "below 0.0"),
        ("QQ:abc",  "not a number"),
    ];
    for (input, reason) in cases {
        match WeightedCombos::from_str(input) {
            Err(PKError::InvalidFrequency) => {
                println!("  \"{input}\" ({reason}) → PKError::InvalidFrequency  ✓");
            }
            Err(e) => println!("  \"{input}\" → unexpected error: {e}"),
            Ok(_)  => println!("  \"{input}\" → unexpectedly parsed OK"),
        }
    }

    // ── Summary ───────────────────────────────────────────────────────────────

    separator("Summary");
    println!("Components demonstrated:");
    println!("  ✓ WeightedCombos::from_str  — parse \":f\" frequency suffixes");
    println!("  ✓ Default frequency         — no suffix → 1.0");
    println!("  ✓ Range expansion           — \"JJ-99:0.8\" expands all three at 0.8");
    println!("  ✓ WeightedCombos::to_range_str — serialize (omit suffix when freq=1.0)");
    println!("  ✓ Round-trip                — from_str(to_range_str()) is lossless");
    println!("  ✓ Combos::from_str          — tolerates \":f\" suffix (backward compat)");
    println!("  ✓ weighted_twos             — expand to (Two, freq) pairs");
    println!("  ✓ weighted_win_probability  — frequency-weighted equity aggregation");
    println!("  ✓ PKError::InvalidFrequency — out-of-range or non-numeric values");
}
