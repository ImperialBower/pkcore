//! EPIC-15: GTO Solver — feature demonstration.
//!
//! Walks through every component built in EPIC-15:
//!
//! 1. `SolverConfig` + `BetSizings`
//! 2. `GameTree` — river-only and turn+river trees
//! 3. `StrategyProfile` — uniform initialization
//! 4. `RegretAccumulator` — regret-matching
//! 5. `Solver::iterate` / `Solver::solve` — CFR iteration loop
//! 6. `WeightedCombos::after_action` — range propagation
//! 7. `Solver::compute_exploitability` — Nash gap metric
//! 8. `SolverResult::save_binary` / `save_json` / `load_binary` — serialization
//!
//! ```
//! cargo run --example gto_solver
//! ```

use pkcore::analysis::gto::combo::Combo;
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::game_tree::{GameTree, Node};
use pkcore::analysis::gto::solver::{Solver, SolverResult};
use pkcore::analysis::gto::solver_config::{BetSize, BetSizings, SolverConfig};
use pkcore::analysis::gto::strategy_profile::StrategyProfile;
use pkcore::analysis::gto::weighted_combos::WeightedCombos;
use pkcore::play::board::Board;
use std::str::FromStr;
use std::time::Instant;

fn separator(title: &str) {
    println!();
    println!("─── {title} {}", "─".repeat(55usize.saturating_sub(title.len() + 5)));
}

fn main() {
    let start = Instant::now();
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║           EPIC-15: GTO Solver Demo                  ║");
    println!("╚══════════════════════════════════════════════════════╝");

    // ── 1. SolverConfig + BetSizings ─────────────────────────────────────────

    separator("1. SolverConfig + BetSizings");

    // BetSizings lets you configure which bet sizes are available on each
    // street as fractions of the pot.
    let sizings = BetSizings::new(
        vec![BetSize::half_pot()],                 // flop
        vec![BetSize::half_pot()],                 // turn
        vec![BetSize::half_pot(), BetSize::pot()], // river: two sizes
    );

    let oop_range = Combos::from_str("AA,KK").unwrap_or_default();
    let ip_range = Combos::from_str("QQ,JJ").unwrap_or_default();
    let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();

    let config = SolverConfig::new(
        oop_range.clone(),
        ip_range.clone(),
        board.clone(),
        1_000, // effective stack (chips)
        200,   // pot (chips)
    )
    .with_bet_sizings(sizings)
    .with_max_iterations(100);

    println!("OOP range : {oop_range}");
    println!("IP range  : {ip_range}");
    println!("Board     : {board}");
    println!("Pot       : {} chips", config.pot);
    println!("Max iters : {}", config.max_iterations);
    println!(
        "River sizes: {}",
        config
            .bet_sizings
            .river
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // ── 2. GameTree — river-only ──────────────────────────────────────────────

    separator("2. GameTree (river-only)");

    // build_river constructs an arena of nodes: Action, Terminal.
    // No Chance nodes on the river — the board is already fully run out.
    let river_tree = GameTree::build_river(&config);
    println!("River tree nodes    : {}", river_tree.len());
    println!(
        "  Action nodes      : {}",
        (0..river_tree.len())
            .filter_map(|i| river_tree.get(pkcore::analysis::gto::game_tree::NodeId::new(i)))
            .filter(|n| n.is_action())
            .count()
    );
    println!("  Terminal nodes    : {}", river_tree.count_terminals());
    println!("  Chance nodes      : {}", river_tree.count_chance_nodes());
    println!(
        "Root is OOP action  : {}",
        matches!(river_tree.root(), Node::Action(n) if
        n.player == pkcore::analysis::gto::game_tree::Player::Oop)
    );

    // ── 3. StrategyProfile — uniform init ────────────────────────────────────

    separator("3. StrategyProfile (uniform)");

    // A freshly-built profile assigns equal probability to every action at
    // every node — the standard CFR starting point.
    let uniform = StrategyProfile::from_uniform(&river_tree, &oop_range, &ip_range);
    println!("Profile nodes (action nodes × hands): {}", uniform.len());
    println!("is_empty: {}", uniform.is_empty());

    // ── 4. GameTree — turn + river ────────────────────────────────────────────

    separator("4. GameTree (turn + river, with chance nodes)");

    // build_turn fans out at every showdown continuation on the turn into a
    // ChanceNode with 48 possible river runout cards.
    let turn_config = SolverConfig::new(oop_range.clone(), ip_range.clone(), board.clone(), 1_000, 200);
    let turn_tree = GameTree::build_turn(&turn_config);
    println!("Turn+river tree nodes : {}", turn_tree.len());
    println!("  Chance nodes        : {}", turn_tree.count_chance_nodes());
    println!("  Terminal nodes      : {}", turn_tree.count_terminals());
    println!("  (each chance node has 48 river runout branches)");

    // ── 5. Solver — CFR iteration ─────────────────────────────────────────────

    separator("5. Solver — CFR iteration (river solve)");

    let mut solver = Solver::new(config.clone());
    println!("Hand pairs (OOP × IP, board-filtered): {}", {
        // Run one iteration to get a sense of convergence speed
        let ev0 = solver.iterate();
        format!("iter 1 avg EV to OOP = {ev0:.2} chips")
    });

    // Run to convergence
    println!("Running to {} iterations...", config.max_iterations);
    let t_solve = Instant::now();
    let result = solver.solve();
    println!("Solve time          : {:.2?}", t_solve.elapsed());
    println!("Iterations run      : {}", result.iterations);
    println!("Exploitability      : {:.4} chips", result.exploitability);

    // ── 6. Equilibrium strategy inspection ───────────────────────────────────

    separator("6. Equilibrium Strategy (OOP at root)");

    let eq = &result.equilibrium;
    let root = solver.root_id();

    // Print OOP's frequencies at the root for each specific hand (Two)
    // that CFR visited. Actions: [Check, Bet(1/2), Bet(pot)]
    let action_names = ["Check", "Bet(½)", "Bet(1×)"];
    for &(oop_hand, _) in solver.hand_pairs().iter().take(6) {
        if let Some(freq) = eq.get(root, &oop_hand) {
            let parts: Vec<String> = action_names
                .iter()
                .zip(freq.as_slice())
                .map(|(name, p)| format!("{name} {:.0}%", p * 100.0))
                .collect();
            println!("  {oop_hand}: {}", parts.join("  "));
        }
    }

    // ── 7. WeightedCombos — range propagation ────────────────────────────────

    separator("7. WeightedCombos::after_action (range propagation)");

    // WeightedCombos models a mixed strategy range with per-combo frequencies.
    let mut oop_wc = WeightedCombos::default();
    oop_wc.insert(Combo::COMBO_AA, 1.0);
    oop_wc.insert(Combo::COMBO_KK, 1.0);
    println!(
        "Before action — AA: {:.2}, KK: {:.2}",
        oop_wc.frequency(&Combo::COMBO_AA).unwrap_or(0.0),
        oop_wc.frequency(&Combo::COMBO_KK).unwrap_or(0.0)
    );

    // after_action returns a new WeightedCombos scaled by the action frequency
    // from the equilibrium profile — so the range reflects how often each hand
    // actually takes this action.
    let after_check = oop_wc.after_action(eq, root, 0); // action 0 = Check
    let after_bet = oop_wc.after_action(eq, root, 1); // action 1 = Bet(½)

    println!(
        "After Check  — AA: {:.4}, KK: {:.4}",
        after_check.frequency(&Combo::COMBO_AA).unwrap_or(0.0),
        after_check.frequency(&Combo::COMBO_KK).unwrap_or(0.0)
    );
    println!(
        "After Bet(½) — AA: {:.4}, KK: {:.4}",
        after_bet.frequency(&Combo::COMBO_AA).unwrap_or(0.0),
        after_bet.frequency(&Combo::COMBO_KK).unwrap_or(0.0)
    );

    // ── 8. compute_exploitability (manual call) ───────────────────────────────

    separator("8. compute_exploitability");

    // compute_exploitability runs a best-response pass: OOP plays greedily
    // against the fixed IP equilibrium, and vice versa. The gap is the Nash
    // distance.  0.0 = perfect Nash equilibrium.
    let expl = solver.compute_exploitability(eq);
    println!("Nash gap (exploitability): {expl:.6} chips");
    println!("(Same as result.exploitability: {:.6})", result.exploitability);

    // ── 9. Turn solver ────────────────────────────────────────────────────────

    separator("9. Turn Solver (new_turn)");

    let turn_solve_config = SolverConfig::new(
        Combos::from_str("AA").unwrap_or_default(),
        Combos::from_str("KK").unwrap_or_default(),
        board.clone(),
        1_000,
        200,
    )
    .with_max_iterations(5);

    println!("Running turn solve (AA vs KK, 5 iterations)...");
    let t_turn = Instant::now();
    let turn_result = Solver::new_turn(turn_solve_config).solve();
    println!("Turn solve time    : {:.2?}", t_turn.elapsed());
    println!("Iterations         : {}", turn_result.iterations);
    println!("Exploitability     : {:.4} chips", turn_result.exploitability);

    // ── 10. Serialization ─────────────────────────────────────────────────────

    separator("10. Serialization");

    let bin_path = std::env::temp_dir().join("gto_demo.bin");
    let json_path = std::env::temp_dir().join("gto_demo.json");

    // Binary (bincode) — compact, fast, default format
    result.save_binary(&bin_path).expect("save_binary failed");
    let bin_size = std::fs::metadata(&bin_path).map(|m| m.len()).unwrap_or(0);
    let loaded = SolverResult::load_binary(&bin_path).expect("load_binary failed");
    assert_eq!(loaded.iterations, result.iterations);
    println!("Binary save  : {} ({} bytes)", bin_path.display(), bin_size);
    println!(
        "Binary load  : OK — iterations={}, exploitability={:.4}",
        loaded.iterations, loaded.exploitability
    );

    // JSON — human-readable, useful for debugging
    result.save_json(&json_path).expect("save_json failed");
    let json_size = std::fs::metadata(&json_path).map(|m| m.len()).unwrap_or(0);
    let loaded_j = SolverResult::load_json(&json_path).expect("load_json failed");
    assert_eq!(loaded_j.iterations, result.iterations);
    println!("JSON save    : {} ({} bytes)", json_path.display(), json_size);
    println!(
        "JSON load    : OK — iterations={}, exploitability={:.4}",
        loaded_j.iterations, loaded_j.exploitability
    );
    println!(
        "Size ratio   : binary is {:.1}× smaller than JSON",
        json_size as f64 / bin_size as f64
    );

    // default save/load dispatches to binary (or JSON with --features debug-json)
    result.save(&bin_path).expect("save (default) failed");
    let _roundtrip = SolverResult::load(&bin_path).expect("load (default) failed");
    println!("Default save/load: OK (binary unless debug-json feature enabled)");

    // Cleanup
    let _ = std::fs::remove_file(&bin_path);
    let _ = std::fs::remove_file(&json_path);

    // ── Summary ───────────────────────────────────────────────────────────────

    separator("Summary");
    println!("Total elapsed: {:.2?}", start.elapsed());
    println!();
    println!("Components demonstrated:");
    println!("  ✓ SolverConfig + BetSizings");
    println!("  ✓ GameTree::build_river (no chance nodes)");
    println!("  ✓ GameTree::build_turn  (48-branch chance nodes)");
    println!("  ✓ StrategyProfile::from_uniform");
    println!("  ✓ Solver::new + iterate (CFR loop)");
    println!("  ✓ Solver::solve → SolverResult");
    println!("  ✓ WeightedCombos::after_action (range propagation)");
    println!("  ✓ Solver::compute_exploitability (best-response pass)");
    println!("  ✓ Solver::new_turn (turn+river multi-street solve)");
    println!("  ✓ SolverResult::save_binary / load_binary");
    println!("  ✓ SolverResult::save_json   / load_json");
    println!("  ✓ SolverResult::save / load (feature-flag dispatch)");
}
