# EPIC-130: The Reference Spot — measure, oracle, go/no-go (SPOT)

> **Provenance.** Drafted 2026-10-28 against pkcore `main` @ `44b484ee`
> (`0.20.2`, `Cargo.toml:4`) and `pkgto-web` `origin/main` @ `064015e`
> (`0.1.6`, pinned to `pkcore = "0.11.0"`, `pkgto-web/Cargo.toml:14`). First
> EPIC of the **pksolver block (EPIC-130 through EPIC-139)**, claimed in
> `ROADMAP.md` § EPIC Numbering Policy. The block's docs live here in pkcore
> until the `pksolver` repo exists; they move there (keeping their numbers)
> in Work Item 0a. This EPIC also carries the block-level summary, decisions,
> status and dependencies — see [The pksolver block](#the-pksolver-block).

## Summary

- **Builds:** one fixed benchmark spot, pinned fixtures for its correct
  answer, a timing of the existing pkcore solver on it, and the written
  decision that opens the block.
- **Why:** every later EPIC is justified by a ratio nobody has measured.
  The pkcore solver's documented example is `AA,KK` vs `QQ,JJ` on a river
  (`src/analysis/gto/solver.rs:44-49`). Nobody has timed it on two real
  ranges with a turn to come.
- **Shape:** a `perf/` entry and a `tests/fixtures/spot-130/` directory in
  `pksolver`; nothing in pkcore changes unless the measurement finds a
  defect.
- **Proves it:** the EPIC's output *is* the proof — a table of (engine,
  spot, iterations, seconds, exploitability, peak RSS).
- **Status:** Planned. Nothing has landed. This EPIC is the gate for the
  block: nothing else starts until Block Decision 1 is written down with
  the ratio behind it.

---

## Context

`pkcore::analysis::gto` already has river and turn trees, Vanilla / CFR+ /
DCFR (`src/analysis/gto/solver_config.rs:53-77`), best-response
exploitability and serialisable results (`SolverResult::to_binary_bytes`,
`src/analysis/gto/solver.rs:255`). It cannot build a flop tree, has no
raises (`build_facing_bet_node`, `src/analysis/gto/game_tree.rs:795`; the
turn builder at `:684` is the deepest), and `Solver::iterate`
(`src/analysis/gto/solver.rs:810`) traverses the whole tree once per
`(oop_hand, ip_hand)` pair over `RegretAccumulator`'s nested-`HashMap`
storage (`src/analysis/gto/regret.rs:94`). That is correct and teachable.
Nobody knows how far it is from a product, because nobody has timed it on
a spot a player would ask about.

`pkgto-web` at `064015e` is one exported function over the preflop chart
(`analyze_gto`, `pkgto-web/src/lib.rs:50`), a 627-line vanilla page
(`www/index.html`), licensed `GPL-3` (`pkgto-web/Cargo.toml:6`) and pinned
to pkcore `0.11.0`.

### What this EPIC does not do

- **No engine code.** If the measurement tempts a quick fix to
  `Solver::iterate`, it goes in a note, not a commit.
- **No pkcore API change.** A defect found here is filed as a pkcore
  DEFECT with a hand file, as DEFECT_040 was.

---

## Status

| Component | Status |
|---|---|
| 0a — `pksolver` repo scaffold | Planned |
| 0b — `pkgto-web` relicense, pkcore bump, two wasm sizes | Planned |
| 1a — `spot-130.toml` fixture | Planned |
| 1b — oracle run, `spot-130-oracle.json` | Planned |
| 1c — pkcore solver timings on river and turn sub-spots | Planned |
| 1d — pkcore solver vs oracle on the river sub-spot | Planned |
| 2 — Block Decision 1 written, EPIC-131 opened | Planned |

---

## Goals

- One **reference spot**, fixed for the life of the block, that every later
  EPIC measures against.
- **Oracle fixtures** for its correct answer, loaded by tests.
- The **ratio**: how far the existing solver is from a product, in seconds
  and bytes.
- A **go/no-go**: Block Decision 1 confirmed or revised in writing.

## Scope

1. **The spot is fixed for the life of the block.** A 3-bet pot: OOP
   `66+,A8s+,A5s-A4s,AJo+,K9s+,KQo,QTs+,JTs,96s+,85s+,75s+,65s,54s`
   vs IP `QQ-22,AQs-A2s,ATo+,K5s+,KJo+,Q8s+,J8s+,T7s+,96s+,86s+,75s+,64s+,53s+`,
   flop `T♦ 9♦ 6♥`, pot 200, effective stack 900, sizes 60% pot /
   geometric / all-in, raises 2.5×, target exploitability 0.5% of pot.
   Two sub-spots: the same with turn `Q♣` dealt (turn-rooted), and with
   river `7♠` dealt (river-rooted), so the existing solver has something
   it can finish.
2. **Oracle numbers are pinned as fixtures**, not re-derived: root strategy
   per hand, OOP EV and equity at the root, exploitability after N
   iterations, from an independent solver run locally (Block Decision 3)
   and, for the river sub-spot, from `pkcore::analysis::gto::Solver`.
3. **The measurement is of the function a product would call**
   (`Solver::solve` to a target, not `iterate` alone) — EPIC-122 lesson 6,
   "Measure the function the engine actually calls"
   (`docs/epics/EPIC-122_Exact_Equity.md:233`).
4. **`pkgto-web` is unblocked in the same EPIC**: bump to pkcore `0.20.2`,
   build the existing preflop page with `--no-default-features --features
   equity,hup-charts` (`Cargo.toml:108`, `:141`) and record the wasm size;
   build once more without `hup-charts` and record that size. The second
   number is the solver build's budget.
5. **No engine code** (see Context).

---

## Work Items

| # | Item | Done when |
|---|---|---|
| 0a | `pksolver` repo scaffold: workspace with empty `pksolver-core`, `pksolver-testkit`, `pksolver-wasm`; `MIT OR Apache-2.0`; `rust-version = "1.99.0"` (matching `Cargo.toml:5`); `make check-purity` copied from pkcore; this block's EPIC docs moved to `pksolver/docs/epics/` with a pointer left here | `cargo build` green |
| 0b | `pkgto-web` relicense (`GPL-3` → `MIT OR Apache-2.0`, its first commit in the block) + pkcore bump; two measured wasm sizes in `docs/SIZES.md` | sizes recorded |
| 1a | Fixture: the spot as `spot-130.toml` (ranges, board, pot, stack, sizes) | file |
| 1b | Oracle run; root strategy / EV / equity / exploitability pinned as `spot-130-oracle.json` | fixture |
| 1c | `pkcore::analysis::gto::Solver` on the river sub-spot: iterations to 0.5%, seconds, RSS; on the turn sub-spot: seconds per iteration (it will not reach target) | table in `docs/EPIC-130_RESULTS.md` |
| 1d | River sub-spot: pkcore solver vs oracle, max abs strategy difference, EV difference | numbers in the same table |
| 2 | Block Decision 1 written: ratio, placement confirmed or revised, EPIC-131 opened | this doc's Status rows flipped |

## Key Files

| File | Role |
|---|---|
| `pksolver/tests/fixtures/spot-130/spot-130.toml` | the spot |
| `pksolver/tests/fixtures/spot-130/spot-130-oracle.json` | oracle answer |
| `pksolver/perf/` | timing harness |
| `pksolver/docs/EPIC-130_RESULTS.md` | the results table |
| `pkgto-web/docs/SIZES.md` | the two wasm sizes |

## Reuse (do NOT recreate)

- `src/analysis/gto/solver.rs:810` — `Solver::iterate`, the engine being timed.
- `src/analysis/gto/solver_config.rs` — `SolverConfig`, `BetSize`, `CfrVariant`.
- `src/analysis/gto/combos.rs`, `weighted_combos.rs:59` — range parsing.

## Verification

```bash
# in pksolver/
cargo build
cargo test -- spot_130
# in pkgto-web/
wasm-pack build --release -- --no-default-features --features equity,hup-charts
wasm-pack build --release -- --no-default-features --features equity
```

Exit criteria:

1. The three sub-spots parse into pkcore types (`Combos`, `Board`) with no
   manual card translation.
2. The oracle fixtures are loaded by a test, not pasted into one.
3. The results table has a number in every cell or the words *timed out*
   with the cutoff.

---

## The pksolver block

### Block summary

- **Builds:** a heads-up no-limit postflop solver that runs at the scale
  people expect from a solver — two full ranges, flop to river, a dozen
  bet lines per street — natively and in the browser, and the `pkgto-web`
  app that drives it.
- **Why:** see this EPIC's Context. The existing solver is correct and
  teachable; it is expected to be three to four orders of magnitude from a
  product. This EPIC measures that.
- **Shape:** a sibling crate, `pksolver`, with a pure kernel
  (`pksolver-core`), a testkit (`pksolver-testkit`) and a browser facade
  (`pksolver-wasm`); `pkgto-web` becomes the app. pkcore is unchanged
  except where EPIC-130 finds a pkcore defect; the existing
  `analysis::gto` solver stays as the reference implementation the new
  engine is checked against.
- **Proves it:** every EPIC ends with a measured number or a byte-exact
  comparison — iteration time, exploitability per iteration, strategy
  agreement with the reference solver and with an independent oracle,
  byte-identical results serial vs. threaded.
- **License:** everything in the block is `MIT OR Apache-2.0`, the same as
  pkcore.

### Block decisions

1. **Sibling crate, not pkcore.** A vectorised `f32` engine with SIMD paths
   and a browser thread pool is 10–15k lines that every pkcore consumer
   would compile and every wasm consumer would download. pkdealer and
   pkmine already set the pattern: the kernel stays the kernel. `pksolver`
   depends on pkcore for ranges (`Combos`, `WeightedCombos`, and EPIC-120's
   `HandRange` once it lands — it is Planned at `0.20.2`,
   `docs/epics/EPIC-120_Suit_Specific_Ranges.md:25`), boards, cards and the
   ckc-rs rank; nothing flows back.
2. **The existing solver is not deleted.**
   `pkcore::analysis::gto::solver` is the slow, obviously-correct second
   implementation: EPIC-131 proves the new engine against it on every spot
   small enough for it to finish. It is also Bower material — the
   pairwise → vectorised change is a measured before/after chapter.
3. **Oracle, never source.** An independent third-party solver is run
   locally to check numbers on a fixed spot. Its code is never read for
   implementation and never copied; the algorithms (DCFR, Brown &
   Sandholm 2019; range-vs-range showdown sweeps; suit isomorphism) come
   from the literature and from pkcore's own equity work — isomorphic
   exact preflop is designed in EPIC-122 Design § 4
   (`docs/epics/EPIC-122_Exact_Equity.md:481`), Planned as its Phase 3.
4. **One license.** `pksolver`, `pksolver-wasm` and `pkgto-web` are
   `MIT OR Apache-2.0`. Anything in the pkcore ecosystem — pkdealer bots,
   the arena, pkmine — may embed solver output without a license seam.
5. **Numbers in pkcore's chip units.** `TreeConfig` takes integer chips
   (`u32`) for pot and effective stack and pkcore's rational `BetSize`
   for sizes; no `f64` chips anywhere a chip is counted. Rake is the one
   `f64` rate, applied once at terminals.
6. **Engine before app.** The browser build (EPIC-135/136) and the UI
   (EPIC-137) consume a public Rust API that EPIC-134 freezes. No UI work
   starts against an engine that cannot solve the EPIC-130 reference spot
   in under a minute natively.

### Block status

| EPIC | Title | Depends on | Status |
|---|---|---|---|
| [EPIC-130](EPIC-130_Reference_Spot.md) | The Reference Spot — measure, oracle, go/no-go | — | Planned |
| [EPIC-131](EPIC-131_Vectorised_Engine.md) | The Vectorised Engine — `pksolver-core` kernel | 130 | Planned |
| [EPIC-132](EPIC-132_Trees.md) | Trees — the full line grammar, flop to river | 131 | Planned |
| [EPIC-133](EPIC-133_Isomorphism_and_Memory.md) | Isomorphism and Memory — fewer runouts, half the bytes | 131, 132 | Planned |
| [EPIC-134](EPIC-134_Solve_API.md) | The Solve API — step, inspect, lock, save | 131–133 | Planned |
| [EPIC-135](EPIC-135_Browser_Facade.md) | The Browser Facade — `pksolver-wasm`, single thread | 134, EPIC-86 | Planned |
| [EPIC-136](EPIC-136_Threads_in_the_Browser.md) | Threads in the Browser — atomics build and hosting | 135 | Planned |
| [EPIC-137](EPIC-137_The_App.md) | The App — `pkgto-web` becomes a solver | 135 (136 optional) | Planned |
| [EPIC-138](EPIC-138_Solve_from_a_Hand.md) | Solve from a Hand — hand histories in, EV loss out | 134, pkcore EPIC-93 | Planned |
| [EPIC-139](EPIC-139_Bunching_from_Data.md) | Bunching from Data — folded ranges from pkmine | 133, pkmine Phase 5 | Planned, last |

### Block dependencies

| Depends on | For |
|---|---|
| pkcore `0.20.2` | ranges, boards, `BetSize`, `CfrVariant`, `WeightedCombos`, the reference solver, `HandHistory::cursor` (`src/hand_history/replay.rs:275`) |
| pkcore EPIC-120 (Planned) | `HandRange`: suit-specific ranges in the range editor and the isomorphism rules |
| pkcore EPIC-86 | the typed-class browser-bindings shape; `pksolver-wasm` is its first full instance |
| pkcore EPIC-93 | faithful replay for EPIC-138 |
| pkmine Phase 5 | the corpus for EPIC-139 |
| ckc-rs | the rank in every showdown sweep |

### Compatibility

- pkcore's public API does not change. If EPIC-130 or EPIC-131 finds a
  defect (a `Combos` parse, a `BetSize` rounding), it is filed as a pkcore
  defect with a hand file, as DEFECT_040 was.
- `pkgto-web`'s `analyze_gto` keeps working until EPIC-137 Phase 1 moves
  the preflop page; the URL-parameter format is preserved.

### What the block does not do

- **No preflop solving.** Preflop stays a chart (`hups.bin`) and a range
  editor; a preflop CFR is a different tree and a different block.
- **No multiway solving.** Bunching models folded players; it does not
  make them act.
- **No ICM, no tournaments.** Chip EV only.
- **No accounts, no server.** Everything runs in the browser or in a
  process the user owns; solves are files.
- **No new evaluator.** ckc-rs is the rank. If EPIC-130's profile shows
  the showdown sort dominating, an evaluator comparison is the next EPIC,
  in ckc-rs, not here (EPIC-122 lesson 8,
  `docs/epics/EPIC-122_Exact_Equity.md:250`).
