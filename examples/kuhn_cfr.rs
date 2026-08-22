//! CFR convergence demo for Kuhn poker.
//!
//! This example is an educational walkthrough of how Counterfactual Regret
//! Minimization (CFR) discovers the Nash equilibrium of Kuhn poker from scratch.
//!
//! It shows:
//!
//! 1. **The analytical Nash equilibrium** — the closed-form solution parameterized
//!    by a single bluff-frequency `alpha ∈ [0, 1/3]`.
//! 2. **CFR strategy snapshots** — the current average strategy at logarithmically
//!    spaced iteration milestones (1, 10, 100, 1 000, 10 000).
//! 3. **Exploitability decay** — the Nash gap shrinking toward zero as CFR
//!    accumulates experience across all 6 possible deals.
//! 4. **Convergence comparison** — how close the learned strategy is to the
//!    analytical solution for each of the 12 info sets.
//!
//! # Tip
//!
//! Run with `--release` for faster output at higher iteration counts:
//!
//! ```bash
//! cargo run --release --example kuhn_cfr
//! ```
//!
//! # Usage
//!
//! ```bash
//! cargo run --example kuhn_cfr
//! ```

use pkcore::games::kuhn::{KuhnAction, KuhnCard, KuhnCfr, KuhnHistory, KuhnInfoSet, KuhnStrategy};

fn main() {
    let start = std::time::Instant::now();

    header("Kuhn Poker — CFR Convergence Demo");

    // ── 1. Analytical Nash equilibrium ────────────────────────────────────────

    section("1. Analytical Nash Equilibrium (alpha = 1/3)");

    println!(
        "  Kuhn poker's GTO solution is parameterized by a single scalar alpha in [0, 1/3].
  Alpha is P0's bluffing frequency with a Jack. All other probabilities derive
  from it by indifference conditions.\n"
    );

    let nash = KuhnStrategy::default(); // alpha = 1/3
    print_strategy_table(&nash, "Analytical Nash (alpha = 1/3)");

    println!(
        "
  Key indifference conditions at alpha = 1/3:
  • P0(J) bets alpha = 1/3 of the time  → makes P1(Q) indifferent to calling/folding a bet.
  • P0(K) bets 3·alpha = 1  (always)    → balanced value-bet range.
  • P1(J) bluffs 1/3 after a check      → makes P0(Q) indifferent to calling/folding [C,B].
  • P1(Q) calls a bet 1/3               → makes P0(J) indifferent to bet/check bluffing.
  • Game value for P0: −1/18 ≈ −0.0556 chips/hand  (P0 is structurally disadvantaged).
"
    );

    // ── 2. CFR strategy snapshots ─────────────────────────────────────────────

    section("2. CFR Strategy Snapshots at Iteration Milestones");

    println!(
        "  CFR traverses all 6 possible deals every iteration (no Monte Carlo sampling
  is needed because Kuhn poker's tree is tiny). Regrets are accumulated at each
  of the 12 decision nodes and averaged into a converging strategy.\n"
    );

    let milestones: &[(u32, &str)] = &[
        (1, "1 iteration"),
        (9, "10 iterations"),
        (90, "100 iterations"),
        (900, "1 000 iterations"),
        (9_000, "10 000 iterations"),
    ];

    let mut cfr = KuhnCfr::new();
    let mut total_iters: u32 = 0;

    for &(extra, label) in milestones {
        cfr.train(extra).expect("Kuhn training cannot fail on valid deals");
        total_iters += extra;
        let avg = cfr.average_strategy();
        let exploit = cfr.exploitability();
        println!("  ── After {} ──", label);
        println!("     Exploitability: {exploit:.6} chips");
        print_key_info_sets(&avg);
        println!();
    }

    // ── 3. Exploitability decay ───────────────────────────────────────────────

    section("3. Exploitability Decay");

    println!(
        "  Exploitability = the sum of each player's best-response gain against the
  opponent's average strategy. At Nash equilibrium it equals zero.
  CFR guarantees convergence to ε-Nash with ε ∝ 1/√iterations.
"
    );

    let mut cfr2 = KuhnCfr::new();
    println!(
        "  {:>12}   {:>14}   {:>10}",
        "Iterations", "Exploitability", "Δ from Nash"
    );
    println!("  {}", "─".repeat(44));
    let mut prev_exploit = f64::INFINITY;
    for &iters in &[1u32, 10, 100, 1_000, 10_000] {
        let needed = iters - (iters / 10).max(if iters == 1 { 0 } else { iters / 10 });
        // Re-train on the fresh cfr2 to hit exactly `iters` total.
        let run = if iters == 1 { 1 } else { iters - iters / 10 };
        cfr2.train(run).expect("Kuhn training cannot fail on valid deals");
        let exploit = cfr2.exploitability();
        let reduction = if prev_exploit.is_finite() {
            format!("{:.1}%", (1.0 - exploit / prev_exploit) * 100.0)
        } else {
            "  —".to_owned()
        };
        println!("  {:>12}   {exploit:>14.6}   {reduction:>10}", iters);
        prev_exploit = exploit;
        let _ = needed; // suppress warning
    }

    // ── 4. Convergence to analytical Nash ────────────────────────────────────

    section(&format!("4. Convergence to Analytical Nash ({total_iters} iterations)"));

    println!(
        "  After {total_iters} iterations, CFR has visited each of the 6 deals {total_iters} times.
  The average strategy converges toward the analytical alpha = 1/3 solution.
  (Run with --release and more iterations for tighter convergence.)
"
    );

    let final_avg = cfr.average_strategy();
    let nash_ref = KuhnStrategy::default();

    println!("  {:<30}  {:>8}  {:>8}  {:>8}", "Info set", "CFR %", "Nash %", "Δ");
    println!("  {}", "─".repeat(58));

    for_each_info_set(|info, action| {
        let cfr_probs = final_avg.action_probs(&info);
        let nash_probs = nash_ref.action_probs(&info);

        let cfr_p = cfr_probs
            .iter()
            .find(|(a, _)| *a == action)
            .map(|(_, p)| *p)
            .unwrap_or(0.0);
        let nash_p = nash_probs
            .iter()
            .find(|(a, _)| *a == action)
            .map(|(_, p)| *p)
            .unwrap_or(0.0);
        let delta = cfr_p - nash_p;

        let label = format!("{info} → {action}");
        let diff_flag = if delta.abs() < 0.005 { "  ✓" } else { "  !" };
        println!(
            "  {label:<30}  {:>7.1}%  {:>7.1}%  {:>+7.3}{diff_flag}",
            cfr_p * 100.0,
            nash_p * 100.0,
            delta
        );
    });

    // ── Summary ───────────────────────────────────────────────────────────────

    section("Summary");

    println!("  Total elapsed: {:.2?}", start.elapsed());
    println!();
    println!("  Demonstrated:");
    println!("    ✓ Analytical Nash equilibrium (alpha parameterization)");
    println!("    ✓ CFR strategy snapshots at 6 logarithmic milestones");
    println!("    ✓ Exploitability decay toward Nash equilibrium");
    println!("    ✓ Convergence comparison: CFR vs. analytical per info set");
    println!();
    println!("  Run 'cargo run --example kuhn_repl' to play interactively.");
    println!("  Run 'cargo run --example kuhn_tree' to explore the game tree.");
}

// ── Display helpers ───────────────────────────────────────────────────────────

fn header(title: &str) {
    let width = 56;
    let pad = (width - title.len()).saturating_sub(2) / 2;
    println!("╔{}╗", "═".repeat(width));
    println!(
        "║{}{title}{}║",
        " ".repeat(pad + 1),
        " ".repeat(width - pad - title.len() - 1)
    );
    println!("╚{}╝", "═".repeat(width));
    println!();
}

fn section(title: &str) {
    println!();
    println!("── {title} {}", "─".repeat(54usize.saturating_sub(title.len() + 4)));
    println!();
}

/// Prints the full 12-row strategy table for `strategy`.
fn print_strategy_table(strategy: &KuhnStrategy, label: &str) {
    println!("  {label}");
    println!("  {:<30}  {:>8}  {:>8}", "Info set → Action", "Prob", "Pct");
    println!("  {}", "─".repeat(52));
    for_each_info_set(|info, action| {
        let probs = strategy.action_probs(&info);
        let p = probs.iter().find(|(a, _)| *a == action).map(|(_, p)| *p).unwrap_or(0.0);
        let label = format!("{info} → {action}");
        println!("  {label:<30}  {p:>8.4}  {:>7.1}%", p * 100.0);
    });
}

/// Prints strategy for the 6 most illustrative info sets (one per decision point).
fn print_key_info_sets(strategy: &KuhnStrategy) {
    let key_info = [
        // P0 initial action
        (KuhnCard::Jack, KuhnHistory::new(), KuhnAction::Bet),
        (KuhnCard::King, KuhnHistory::new(), KuhnAction::Bet),
        // P1 facing check
        (
            KuhnCard::Jack,
            KuhnHistory::new().push(KuhnAction::Check),
            KuhnAction::Bet,
        ),
        (
            KuhnCard::King,
            KuhnHistory::new().push(KuhnAction::Check),
            KuhnAction::Bet,
        ),
        // P1 facing bet
        (
            KuhnCard::Queen,
            KuhnHistory::new().push(KuhnAction::Bet),
            KuhnAction::Call,
        ),
        // P0 facing check-bet
        (
            KuhnCard::Queen,
            KuhnHistory::new().push(KuhnAction::Check).push(KuhnAction::Bet),
            KuhnAction::Call,
        ),
    ];

    let labels = [
        "P0(J) bet %",
        "P0(K) bet %",
        "P1(J) bet-after-check %",
        "P1(K) bet-after-check %",
        "P1(Q) call-a-bet %",
        "P0(Q) call-check-bet %",
    ];

    for ((card, hist, action), label) in key_info.iter().zip(labels.iter()) {
        let info = KuhnInfoSet::new(*card, hist.clone());
        let probs = strategy.action_probs(&info);
        let p = probs.iter().find(|(a, _)| a == action).map(|(_, p)| *p).unwrap_or(0.0);
        print!("     {label}: {p:.3}");
    }
    println!();
}

/// Calls `f(info_set, action)` for every (info_set, primary action) pair in
/// the 12 Kuhn decision nodes, in a consistent canonical order.
fn for_each_info_set<F>(mut f: F)
where
    F: FnMut(KuhnInfoSet, KuhnAction),
{
    let histories = [
        (KuhnHistory::new(), KuhnAction::Bet),
        (KuhnHistory::new().push(KuhnAction::Check), KuhnAction::Bet),
        (KuhnHistory::new().push(KuhnAction::Bet), KuhnAction::Call),
        (
            KuhnHistory::new().push(KuhnAction::Check).push(KuhnAction::Bet),
            KuhnAction::Call,
        ),
    ];
    for card in [KuhnCard::Jack, KuhnCard::Queen, KuhnCard::King] {
        for (hist, action) in &histories {
            let info = KuhnInfoSet::new(card, hist.clone());
            f(info, *action);
        }
    }
}
