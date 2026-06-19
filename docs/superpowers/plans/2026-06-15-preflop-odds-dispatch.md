# Preflop Odds Dispatch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `DealEval::new` compute every seat's preflop equity by dispatching on player count — an O(1) embedded heads-up lookup for 2 seats, the existing equity engine for 3–10 seats.

**Architecture:** `DealEval` stops brute-forcing ~1.7M runouts. `new` becomes fallible, validates seat count, and routes: 2 seats → `SortedHeadsUp::hup_result()` (embedded, wasm-safe), 3+ seats → `EquityRequest::compute()` (auto exact-or-Monte-Carlo). Both branches normalize into the existing `EquityReport` type (with a new `Method::Hup` variant); `DealEval` stores that report and renders it via `Display`.

**Tech Stack:** Rust, rayon (already in the equity engine), criterion (added for the benchmark).

**Spec:** `docs/superpowers/specs/2026-06-15-preflop-odds-dispatch-design.md`

---

## File map

- **Modify** `src/analysis/equity/engine.rs` — add missing `use crate::Pile;` (Task 0 bugfix).
- **Modify** `Cargo.toml` — add `"equity"` to default features (Task 0); plus `criterion` dev-dep, `[[bench]]`, exclude `benches/*` (Task 2).
- **Modify** `src/analysis/equity/result.rs` — add `Method::Hup`.
- **Modify** `src/play/stages/deal_eval.rs` — reshape `DealEval`, dispatch in `new`, two private report builders, new `Display`, tests.
- **Modify** `examples/bcrepl.rs` — handle the now-`Result`-returning `new`.
- **Create** `benches/preflop_odds.rs` — criterion benchmark.

> **Feature note:** the `analysis::equity` module is behind `#[cfg(feature = "equity")]`. Task 0 makes `equity` a default feature (decision recorded in the spec), so the rest of the plan's plain `cargo test` / `clippy` commands compile the engine. Until Task 0 lands, append `--features equity` to any cargo command that touches the engine. wasm guard: `cargo build --lib --features equity --target wasm32-unknown-unknown` must stay green.

---

## Task 0: Make the equity feature compile and turn it on by default

The `analysis::equity` module did not compile (missing `Pile` import) and is off by default. `DealEval`'s dispatch and `Method::Hup` both live behind that gate, so fix and enable it first. See the spec's "Feature gating & wasm" section.

**Files:**
- Modify: `src/analysis/equity/engine.rs` (imports, ~line 17)
- Modify: `Cargo.toml` (`[features]` default list, ~line 40)

- [ ] **Step 1: Add the missing trait import**

In `src/analysis/equity/engine.rs`, alongside `use crate::{Cards, PKError};`, add:

```rust
use crate::Pile;
```

(`cards()` and `to_vec()` are `Pile` trait methods; without this the engine fails with `E0599: no method named cards/to_vec`.)

- [ ] **Step 2: Verify the feature now compiles, native and wasm**

Run: `cargo build --lib --features equity`
Expected: Finished, no errors.

Run: `cargo build --lib --features equity --target wasm32-unknown-unknown`
Expected: Finished, no errors (proves Option A is wasm-safe at compile time).

- [ ] **Step 3: Add `"equity"` to the default features**

In `Cargo.toml`, add `"equity"` to the `default = [...]` array:

```toml
default = [
    "bot-profiles",
    "hand-histories",
    "player-stats",
    "player-stats-persistence",
    "equity",
]
```

- [ ] **Step 4: Verify the engine now compiles under a plain build**

Run: `cargo build --lib`
Expected: Finished — the engine is now in the default build.

- [ ] **Step 5: Commit (two logical commits)**

```bash
git add src/analysis/equity/engine.rs && git commit -m "fix: add missing Pile import so the equity feature compiles"
git add Cargo.toml && git commit -m "build: enable the equity feature by default"
```

---

## Task 1: Add `Method::Hup` provenance variant

**Files:**
- Modify: `src/analysis/equity/result.rs:4-10`
- Test: same file (inline `#[cfg(test)]` module, or add an assertion test)

- [ ] **Step 1: Write the failing test**

Add to the test module at the bottom of `src/analysis/equity/result.rs` (create the module if none exists):

```rust
#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__equity__result_tests {
    use super::*;

    #[test]
    fn method__hup_is_distinct() {
        assert_ne!(Method::Hup, Method::Exact);
        assert_ne!(Method::Hup, Method::MonteCarlo);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib method__hup_is_distinct`
Expected: FAIL to compile — `no variant named Hup found for enum Method`.

- [ ] **Step 3: Add the variant**

In `src/analysis/equity/result.rs`, extend the enum:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
    /// Every board runout was enumerated; the result is exact.
    Exact,
    /// The result was estimated by Monte Carlo sampling.
    MonteCarlo,
    /// An exact, precomputed heads-up preflop table lookup (`HUPResult`).
    Hup,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib method__hup_is_distinct`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/analysis/equity/result.rs
git commit -m "feat: add Method::Hup provenance variant"
```

---

## Task 2: Benchmark scaffolding + capture the pre-change baseline

This benchmarks the **current** brute-force `DealEval::new` (still infallible here) so we get a real before/after. The bench is updated to `.unwrap()` in Task 3 once `new` returns `Result`.

**Files:**
- Modify: `Cargo.toml:11` (exclude) and `Cargo.toml:162` (`[dev-dependencies]`); add a `[[bench]]` block.
- Create: `benches/preflop_odds.rs`

- [ ] **Step 1: Add `benches/*` to the publish exclude**

In `Cargo.toml` line 11, add `"benches/*"` to the `exclude` array:

```toml
exclude = [".github/workflows/*", "data/*", "docs/*", "examples/*", "benches/*", "generated/hups.db", "generated/old/*", "proto/*", "scripts/*", ".gitignore", "Cargo.lock", ".claude", "Claude.md"]
```

- [ ] **Step 2: Add the criterion dev-dependency**

Under `[dev-dependencies]` (after line 162), add:

```toml
criterion = "0.5"
```

- [ ] **Step 3: Register the bench target**

Add a new top-level block (place it just after the `[dev-dependencies]` block):

```toml
[[bench]]
name = "preflop_odds"
harness = false
```

- [ ] **Step 4: Write the benchmark (against the current infallible `new`)**

Create `benches/preflop_odds.rs`:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use pkcore::arrays::two::Two;
use pkcore::play::hole_cards::HoleCards;
use pkcore::play::stages::deal_eval::DealEval;
use std::hint::black_box;

fn heads_up(c: &mut Criterion) {
    let hands = HoleCards::from(vec![Two::HAND_AS_AH, Two::HAND_KS_KH]);
    c.bench_function("deal_eval_heads_up", |b| {
        b.iter(|| DealEval::new(black_box(hands.clone())));
    });
}

fn three_way(c: &mut Criterion) {
    let hands = HoleCards::from(vec![Two::HAND_AS_AH, Two::HAND_KS_KH, Two::HAND_QS_QH]);
    c.bench_function("deal_eval_three_way", |b| {
        b.iter(|| DealEval::new(black_box(hands.clone())));
    });
}

criterion_group!(benches, heads_up, three_way);
criterion_main!(benches);
```

- [ ] **Step 5: Run the baseline and record it**

Run: `cargo bench --bench preflop_odds -- deal_eval_heads_up`
Expected: it compiles and runs; the brute-force heads-up path is slow (hundreds of ms to seconds). **Record the reported time** in the commit message — this is the "before" number.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml benches/preflop_odds.rs
git commit -m "bench: add preflop_odds criterion baseline (pre-dispatch DealEval)"
```

---

## Task 3: Dispatch implementation (reshape `DealEval`, both branches, Display, consumers)

This is one atomic refactor: `new` changes signature, so the struct, helpers, `Display`, the `bcrepl` example, and the benchmark must all change together to keep the tree compiling. TDD here is type-driven — the new tests fail to **compile** against the old shape, then pass after the refactor.

**Files:**
- Modify: `src/play/stages/deal_eval.rs` (whole file)
- Modify: `examples/bcrepl.rs:49-57`
- Modify: `benches/preflop_odds.rs` (add `.unwrap()`)

- [ ] **Step 1: Write the failing tests**

Replace the existing test module in `src/play/stages/deal_eval.rs` (currently misnamed `play__stages__flop_eval_tests`) with:

```rust
#[cfg(test)]
#[allow(non_snake_case)]
mod play__stages__deal_eval_tests {
    use super::*;
    use crate::analysis::equity::Method;
    use crate::arrays::two::Two;
    use std::str::FromStr;

    fn hands(twos: Vec<Two>) -> HoleCards {
        HoleCards::from(twos)
    }

    #[test]
    fn new__heads_up_uses_hup() {
        let eval = DealEval::new(hands(vec![Two::HAND_AS_AH, Two::HAND_KS_KH])).unwrap();
        assert_eq!(eval.report.method, Method::Hup);
        assert_eq!(eval.report.players.len(), 2);
    }

    #[test]
    fn new__heads_up_favorite_equity() {
        // AA vs KK preflop is ~82% for the aces.
        let eval = DealEval::new(hands(vec![Two::HAND_AS_AH, Two::HAND_KS_KH])).unwrap();
        let aa = eval.report.players[0].equity;
        assert!(aa > 0.80 && aa < 0.84, "AA equity was {aa}");
    }

    #[test]
    fn new__heads_up_orientation_follows_seat_order() {
        // Seat 0 = KK, seat 1 = AA: the ~82% must land on seat 1, not seat 0.
        let eval = DealEval::new(hands(vec![Two::HAND_KS_KH, Two::HAND_AS_AH])).unwrap();
        assert!(eval.report.players[1].equity > 0.80, "AA (seat 1) should be the favorite");
        assert!(eval.report.players[0].equity < 0.20, "KK (seat 0) should be the underdog");
    }

    #[test]
    fn new__multiway_sums_to_one() {
        let eval =
            DealEval::new(hands(vec![Two::HAND_AS_AH, Two::HAND_KS_KH, Two::HAND_QS_QH])).unwrap();
        assert_eq!(eval.report.players.len(), 3);
        let sum: f64 = eval.report.players.iter().map(|p| p.equity).sum();
        assert!((sum - 1.0).abs() < 0.01, "equities summed to {sum}");
        // Aces are the clear favorite three-way.
        assert!(eval.report.players[0].equity > eval.report.players[1].equity);
        assert!(eval.report.players[0].equity > eval.report.players[2].equity);
    }

    #[test]
    fn new__multiway_is_deterministic() {
        let h = hands(vec![Two::HAND_AS_AH, Two::HAND_KS_KH, Two::HAND_QS_QH]);
        let a = DealEval::new(h.clone()).unwrap();
        let b = DealEval::new(h).unwrap();
        assert_eq!(a.report.players[0].equity, b.report.players[0].equity);
        assert_eq!(a.report.players[1].equity, b.report.players[1].equity);
    }

    #[test]
    fn new__too_few_hands_errors() {
        assert!(DealEval::new(hands(vec![Two::HAND_AS_AH])).is_err());
    }

    #[test]
    fn new__too_many_hands_errors() {
        // 11 hands (22 cards) exceeds the engine's 10-seat cap.
        let eleven = HoleCards::from_str(
            "As Ah Ks Kh Qs Qh Js Jh Ts Th 9s 9h 8s 8h 7s 7h 6s 6h 5s 5h 4s 4h",
        )
        .unwrap();
        assert_eq!(eleven.len(), 11);
        assert!(DealEval::new(eleven).is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib play__stages__deal_eval_tests`
Expected: FAIL to compile — `DealEval::new` returns `DealEval` not `Result`, and there is no field `report`.

- [ ] **Step 3: Rewrite `deal_eval.rs` head (imports, struct, dispatch, helpers)**

Replace everything in `src/play/stages/deal_eval.rs` from the top of the file through the end of the `impl DealEval` block (i.e. the imports, the `struct DealEval`, and `impl DealEval`) with:

```rust
use crate::PKError;
use crate::analysis::equity::{EquityReport, EquityRequest, Method, PlayerEquity, PlayerSpec};
use crate::analysis::gto::odds::WinLoseDraw;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::play::hole_cards::HoleCards;
use std::fmt::Formatter;

/// Fixed RNG seed for the multi-way Monte Carlo path so preflop odds are
/// reproducible across runs and test threads.
const PREFLOP_SAMPLING_SEED: u64 = 0xDEA1_5EED;

/// Per-seat preflop odds for a whole table.
///
/// `DealEval::new` dispatches on seat count: a precomputed O(1) heads-up table
/// lookup for two seats, the multi-way equity engine for three to ten. Both
/// produce an [`EquityReport`]; `hands[i]` corresponds to `report.players[i]`.
#[derive(Clone, Debug)]
pub struct DealEval {
    pub hands: HoleCards,
    pub report: EquityReport,
}

impl DealEval {
    pub const HEADSUP_PREFLOP_COMBO_COUNT: usize = 1_712_304;

    /// Computes preflop odds for every seat.
    ///
    /// # Errors
    ///
    /// - [`PKError::NotEnoughHands`] for fewer than two seats.
    /// - [`PKError::TooManyHands`] for more than ten seats.
    /// - Propagates lookup / equity-engine errors (e.g. duplicate cards).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::play::stages::deal_eval::DealEval;
    /// use pkcore::play::hole_cards::HoleCards;
    /// use pkcore::arrays::two::Two;
    ///
    /// let hands = HoleCards::from(vec![Two::HAND_AS_AH, Two::HAND_KS_KH]);
    /// let eval = DealEval::new(hands).unwrap();
    /// assert_eq!(eval.report.players.len(), 2);
    /// ```
    pub fn new(hands: HoleCards) -> Result<DealEval, PKError> {
        let report = match hands.len() {
            0 | 1 => return Err(PKError::NotEnoughHands),
            2 => heads_up_report(&hands)?,
            _ => multiway_report(&hands)?,
        };
        Ok(DealEval { hands, report })
    }
}

/// Converts a heads-up win/lose/draw tally into a single seat's equity.
#[allow(clippy::cast_precision_loss)]
fn player_equity_from_wld(wld: WinLoseDraw) -> PlayerEquity {
    let total = wld.total().max(1) as f64;
    PlayerEquity {
        win: wld.wins as f64 / total,
        tie: wld.draws as f64 / total,
        equity: (wld.wins as f64 + wld.draws as f64 / 2.0) / total,
        wins: wld.wins,
        ties: wld.draws,
    }
}

/// Heads-up branch: O(1) embedded HUP lookup, mapped back to seat order.
fn heads_up_report(hands: &HoleCards) -> Result<EquityReport, PKError> {
    let a = *hands.get(0).ok_or(PKError::NotEnoughHands)?;
    let b = *hands.get(1).ok_or(PKError::NotEnoughHands)?;
    let shu = SortedHeadsUp::new(a, b);
    let hup = shu.hup_result()?;
    let higher = player_equity_from_wld(hup.odds);
    let lower = player_equity_from_wld(hup.flip_mode().odds);
    let players = if shu.is_higher(&a) {
        vec![higher, lower]
    } else {
        vec![lower, higher]
    };
    Ok(EquityReport {
        players,
        method: Method::Hup,
        samples: hup.odds.total(),
    })
}

/// Multi-way branch (3–10 seats): the equity engine, seeded for reproducibility.
fn multiway_report(hands: &HoleCards) -> Result<EquityReport, PKError> {
    let players: Vec<PlayerSpec> = hands.iter().map(|two| PlayerSpec::Exact(*two)).collect();
    let mut req = EquityRequest::new(players);
    req.opts.seed = Some(PREFLOP_SAMPLING_SEED);
    req.compute()
}
```

- [ ] **Step 4: Rewrite the `Display` impl**

Replace the existing `impl std::fmt::Display for DealEval` block (and delete the commented-out earlier `Display` block above it) with:

```rust
impl std::fmt::Display for DealEval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut v = Vec::new();
        v.push(format!(
            "Method: {:?} ({} samples)",
            self.report.method, self.report.samples
        ));
        for (i, hand) in self.hands.iter().enumerate() {
            if let Some(pe) = self.report.players.get(i) {
                v.push(format!(
                    "Player #{i}: {hand}  win {:.2}% / tie {:.2}% / equity {:.2}%",
                    pe.win * 100.0,
                    pe.tie * 100.0,
                    pe.equity * 100.0
                ));
            }
        }
        write!(f, "{}", v.join("\n"))
    }
}
```

- [ ] **Step 5: Fix the `bcrepl` example consumer**

In `examples/bcrepl.rs`, replace the `work` function body (lines 49-57) so it handles the `Result` from `new`:

```rust
fn work(hands: HoleCards, cache: &mut HashMap<HoleCards, DealEval>) -> Result<(), PKError> {
    let now = std::time::Instant::now();

    if !cache.contains_key(&hands) {
        let eval = DealEval::new(hands.clone())?;
        cache.insert(hands.clone(), eval);
    }
    let results = &cache[&hands];

    println!("{results}");
    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}
```

- [ ] **Step 6: Update the benchmark to unwrap the `Result`**

In `benches/preflop_odds.rs`, change both `b.iter(...)` closures to unwrap:

```rust
        b.iter(|| DealEval::new(black_box(hands.clone())).unwrap());
```

(Apply to both `heads_up` and `three_way`.)

- [ ] **Step 7: Run the new tests to verify they pass**

Run: `cargo test --lib play__stages__deal_eval_tests`
Expected: PASS (7 tests).

- [ ] **Step 8: Run the doc test**

Run: `cargo test --doc deal_eval`
Expected: PASS.

- [ ] **Step 9: Confirm the whole tree builds (lib, examples, benches)**

Run: `cargo build --all-targets`
Expected: builds clean — `bcrepl` and the bench compile against the new `Result` signature.

- [ ] **Step 10: Re-run the benchmark and record the "after"**

Run: `cargo bench --bench preflop_odds -- deal_eval_heads_up`
Expected: heads-up now runs in microseconds (O(1) lookup). **Record the time**; compare to the Task 2 baseline in the commit message.

- [ ] **Step 11: Commit**

```bash
git add src/play/stages/deal_eval.rs examples/bcrepl.rs benches/preflop_odds.rs
git commit -m "feat: dispatch preflop odds (HUP heads-up, equity engine multi-way)"
```

---

## Task 4: Full verification sweep

**Files:** none (verification only).

- [ ] **Step 1: Full test suite**

Run: `cargo test`
Expected: all pass.

- [ ] **Step 2: Doc tests**

Run: `cargo test --doc`
Expected: all pass.

- [ ] **Step 3: Clippy (matches CI)**

Run: `cargo clippy --all-targets -- -Dclippy::all -Dclippy::pedantic`
Expected: no **new** findings in `deal_eval.rs`, `result.rs`, or `benches/preflop_odds.rs`. (Pre-existing findings elsewhere on the branch are out of scope — see the turn_eval sidequest notes.) If pedantic flags `cast_precision_loss` in `benches/preflop_odds.rs`, it won't — the bench has no casts; the only casts are in `player_equity_from_wld`, already `#[allow]`-ed.

- [ ] **Step 4: Format**

Run: `cargo fmt`
Expected: no churn beyond what you wrote; if it reformats, include it in a follow-up `fmt` commit.

- [ ] **Step 5: Commit any fmt fixes (if needed)**

```bash
git add -A
git commit -m "fmt"
```

---

## Notes for the implementer

- **Why `new` is fallible now:** both engines return `Result` (HUP lookup miss, duplicate cards, seat-count bounds). The only live caller is `examples/bcrepl.rs`; `examples/retired/deal.rs` is retired and not built by default.
- **Orientation is the subtle bit:** `HUPResult` odds are oriented by *higher/lower hand*, never seat order. `SortedHeadsUp::is_higher(&a)` tells you whether seat 0 is the higher hand; `flip_mode()` swaps wins↔losses for the other seat. The `new__heads_up_orientation_follows_seat_order` test is the guard.
- **Determinism:** the multi-way path exceeds `EquityOptions::exact_threshold` (100k) preflop, so it Monte-Carlo samples; the fixed `PREFLOP_SAMPLING_SEED` makes that reproducible (the `new__multiway_is_deterministic` test guards it).
- **`EquityReport` vs `CaseEvals`:** this report is the equity-summary view and does not carry per-runout detail (outs/the nuts). That is intentional and unchanged for the later streets — see the spec's cross-stage note.
```
