//! Full game-tree visualization for Kuhn poker.
//!
//! Renders the complete Kuhn poker game tree — all 6 possible deals, all
//! possible action sequences, terminal payoffs, and GTO reach probabilities.
//!
//! For each of the 6 deals the tree shows:
//! - Every path through the game tree with its terminal payoff `[P0, P1]`
//! - The GTO reach probability (the chance both players play GTO and arrive here)
//! - The contribution to P0's expected value (reach × payoff)
//!
//! At the bottom, the marginal expected values per card for each player are
//! shown alongside the aggregate game value (≈ −0.0556 for P0 at any Nash alpha).
//!
//! # Usage
//!
//! ```bash
//! cargo run --example kuhn_tree
//! ```

use pkcore::games::kuhn::{KuhnAction, KuhnCard, KuhnState, KuhnStrategy};

// ── All possible Kuhn deals ───────────────────────────────────────────────────

const DEALS: [(KuhnCard, KuhnCard); 6] = [
    (KuhnCard::Jack, KuhnCard::Queen),
    (KuhnCard::Jack, KuhnCard::King),
    (KuhnCard::Queen, KuhnCard::Jack),
    (KuhnCard::Queen, KuhnCard::King),
    (KuhnCard::King, KuhnCard::Jack),
    (KuhnCard::King, KuhnCard::Queen),
];

// ── Terminal node record ──────────────────────────────────────────────────────

/// A terminal path through the game tree for one specific deal.
struct TerminalPath {
    actions: Vec<KuhnAction>,
    payoff: [i32; 2],
    /// P0's reach probability under the given strategy.
    p0_reach: f64,
    /// P1's reach probability under the given strategy.
    p1_reach: f64,
}

impl TerminalPath {
    /// Joint reach probability (both players play this path).
    fn reach(&self) -> f64 {
        self.p0_reach * self.p1_reach
    }

    /// P0's EV contribution for this path (reach × payoff[0]).
    fn ev_p0(&self) -> f64 {
        self.reach() * f64::from(self.payoff[0])
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    let strategy = KuhnStrategy::default(); // alpha = 1/3

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          Kuhn Poker — Complete Game Tree                 ║");
    println!("║  GTO strategy: alpha = 1/3  (max bluff frequency)        ║");
    println!("║  Each deal has probability 1/6.                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("  Notation: reach = P(both players follow GTO to this node)");
    println!("            EV(P0) = reach × payoff[P0]  per deal");
    println!("            All reaches and EVs are conditional on the deal.");
    println!();

    let mut total_ev_p0 = 0.0_f64;
    let mut total_ev_p1 = 0.0_f64;

    for &(c0, c1) in &DEALS {
        let state = KuhnState::new(c0, c1).expect("valid deal");
        let paths = collect_paths(&state, &strategy, 1.0, 1.0);

        println!("┌─ Deal: P0=[{c0}]  P1=[{c1}] ─────────────────────────────────────────");

        let mut deal_ev_p0 = 0.0_f64;
        let mut deal_ev_p1 = 0.0_f64;

        for path in &paths {
            let actions_str = path
                .actions
                .iter()
                .map(|a| format!("{a}"))
                .collect::<Vec<_>>()
                .join(" → ");

            let reach = path.reach();
            let ev0 = path.ev_p0();
            let ev1 = reach * f64::from(path.payoff[1]);
            deal_ev_p0 += ev0;
            deal_ev_p1 += ev1;

            println!(
                "│  {actions_str:<30}  payoff [{:+},{:+}]  reach {:.4}  EV(P0) {:+.4}",
                path.payoff[0], path.payoff[1], reach, ev0,
            );
        }

        println!("│  ─────────────────────────────────────────────────────────────────");
        println!(
            "│  Deal EV (conditional on this deal):  P0 {:+.4}  P1 {:+.4}",
            deal_ev_p0, deal_ev_p1
        );
        println!("└───────────────────────────────────────────────────────────────────");
        println!();

        // Each deal is equally likely (prob = 1/6).
        total_ev_p0 += deal_ev_p0 / 6.0;
        total_ev_p1 += deal_ev_p1 / 6.0;
    }

    // ── Aggregate expected values ─────────────────────────────────────────────

    println!("═══ Aggregate Expected Values (averaged over all 6 deals) ═══");
    println!();
    println!("  E[chips, P0] = {total_ev_p0:+.6}");
    println!("  E[chips, P1] = {total_ev_p1:+.6}");
    println!(
        "  Sum          = {:+.6}  (should be 0 — zero-sum game)",
        total_ev_p0 + total_ev_p1
    );
    println!();
    println!("  Analytical game value for P0: −1/18 = {:.6}", -1.0_f64 / 18.0);
    println!(
        "  Difference from analytical  : {:.2e}",
        (total_ev_p0 - (-1.0_f64 / 18.0)).abs()
    );
    println!();

    // ── Info set summary ──────────────────────────────────────────────────────

    println!("═══ GTO Strategy Table (all 12 decision nodes) ═══");
    println!();
    println!("  {:<28}  {:<6}  {:<6}  {:<6}", "Info set", "Act1", "p1", "Act2");
    println!("  {}", "─".repeat(56));

    let histories = [
        KuhnHistory::new(),
        KuhnHistory::new().push(KuhnAction::Check),
        KuhnHistory::new().push(KuhnAction::Bet),
        KuhnHistory::new().push(KuhnAction::Check).push(KuhnAction::Bet),
    ];

    for card in [KuhnCard::Jack, KuhnCard::Queen, KuhnCard::King] {
        for hist in &histories {
            use pkcore::games::kuhn::KuhnInfoSet;
            let info = KuhnInfoSet::new(card, hist.clone());
            let probs = strategy.action_probs(&info);
            if probs.is_empty() {
                continue;
            }
            let (a0, p0) = probs[0];
            let (a1, p1) = probs[1];
            println!("  {info:<28}  {a0:<6}  {p0:.3}  {a1:<6}  {p1:.3}", p0 = p0, p1 = p1);
        }
    }
    println!();
    println!("  Run 'cargo run --example kuhn_repl' to play interactively.");
    println!("  Run 'cargo run --example kuhn_cfr'  to watch CFR converge.");
}

// ── Game-tree traversal ───────────────────────────────────────────────────────

/// Recursively collects all terminal paths from `state`, tracking reach probs.
///
/// `p0` and `p1` are the reach probabilities for the current node from each
/// player playing according to `strategy`. At terminal nodes a `TerminalPath`
/// record is emitted. At decision nodes each legal action is explored with the
/// reach probability multiplied by the acting player's strategy probability.
fn collect_paths(state: &KuhnState, strategy: &KuhnStrategy, p0: f64, p1: f64) -> Vec<TerminalPath> {
    if state.is_terminal() {
        let payoff = state.payoff().expect("terminal state has payoff");
        return vec![TerminalPath {
            actions: state.history().as_slice().to_vec(),
            payoff,
            p0_reach: p0,
            p1_reach: p1,
        }];
    }

    let player = state.current_player().expect("non-terminal has a player");
    let info = state.info_set(player);
    let probs = strategy.action_probs(&info);
    let actions = state.legal_actions();

    let mut results = Vec::new();
    for (action, strategy_prob) in actions.iter().zip(probs.iter().map(|(_, p)| p)) {
        let next = state.apply(*action).expect("action from legal_actions");
        let (next_p0, next_p1) = if player == 0 {
            (p0 * strategy_prob, p1)
        } else {
            (p0, p1 * strategy_prob)
        };
        results.extend(collect_paths(&next, strategy, next_p0, next_p1));
    }
    results
}

// ── Re-export KuhnHistory so the strategy table loop can use it ───────────────

use pkcore::games::kuhn::KuhnHistory;
