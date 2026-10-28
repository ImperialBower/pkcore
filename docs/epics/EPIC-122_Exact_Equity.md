# EPIC-122: Exact Equity — answer exactly what can be answered exactly (EXACT)

> **Provenance.** Drafted 2026-10-24 against `main` @ `e8fd7712` (pkcore
> `0.17.3`, `Cargo.toml:4`). Third EPIC of the fourth pkcore internal block.
> Companions:
> [EPIC-121 Hand Categories](EPIC-121_Hand_Categories.md) and
> [EPIC-120 Suit-Specific Ranges](EPIC-120_Suit_Specific_Ranges.md).
> The [MATHS](#maths) section (added 2026-10-24) explains the math behind
> each phase, with sources.

## Summary

- **Builds:** two new exact paths in `EquityEngine` — **preflop multi-way by
  suit isomorphism** and **hand-vs-range by cached enumeration** — plus a
  cheaper inner loop (rank-only evaluation, cached hero score, early exit) and
  a measured verdict on range-sampling bias.
- **Why:** today any preflop question and any question with a range falls to
  Monte Carlo, honest only to ~0.7 pp (`src/analysis/equity/spec.rs:66-86`).
  Both questions have exact answers that a laptop, and a browser, can compute
  in milliseconds.
- **Shape:** the engine's one decision at `src/analysis/equity/engine.rs:155`
  becomes a three-way choice: *isomorphic exact*, *enumerated exact* (seats
  may be ranges), or *Monte Carlo*. Every exact path returns `Method::Exact`,
  so no caller changes.
- **Proves it:** an isomorphic-vs-brute-force differential test on fixed
  hands; a cross-check of the new heads-up preflop numbers against the
  embedded `hups.bin` chart; a range-seat exact test that agrees with a
  hand-rolled loop; the `perf/` equity workloads before and after.
- **Status:** Planned. Nothing has landed.

---

## Context

### How the engine decides today

`EquityEngine::compute` (`src/analysis/equity/engine.rs:97`) resolves each
seat to `Resolved::{Exact, Range, Random}` (`:31-35`), sizes the runout space
with `n_choose_k` (`:353`), and makes one decision (`:152-155`):

```rust
let all_exact = resolved.iter().all(|r| matches!(r, Resolved::Exact(_)));
if all_exact && runouts <= req.opts.exact_threshold { /* exact */ } else { /* Monte Carlo */ }
```

`exact_threshold` defaults to 100,000 runouts (`spec.rs:65`, `:103`). That
keeps the flop (990), turn (44) and river (1) exact, and sends every preflop
question (C(48,5) = 1,712,304 runouts heads-up) to sampling. **One range seat
sends any street to sampling**, even the river, where the answer is a short
loop over the range.

### What sampling costs

`spec.rs:66-86` records the measured trade: 25,000 samples (the default) are
honest to the nearest whole percent; a decimal place needs 100,000, at about
800 ms for six seats native, and roughly 3× that on `wasm32`.

### The heads-up preflop chart

With the default `hup-charts` feature, `DealEval::new`
(`src/play/stages/deal_eval.rs:54`) answers two seats preflop from the
embedded chart (`heads_up_report`, `:86`; `Method::Hup`,
`src/analysis/equity/result.rs:14-15`). The chart is **15.8 MB** linked into
every binary (`Cargo.toml:129-141`; `src/analysis/store/embedded/hup_cache.rs:5`).
It is generated from the SQLite store (`examples/export_hups_bin.rs`,
`make generate-hups-bin`, `Makefile:372`). Three or more seats preflop go to
the engine (`multiway_report`, `deal_eval.rs:107`), so to Monte Carlo.

### The inner loop

Both the exact and the sampled paths score a hand with
`Eval::from(Seven::from_case_and_board(two, &board)).hand_rank`
(`engine.rs:200`, `:294`). `Eval::from(Seven)` (`src/analysis/eval.rs:329`)
calls `hand_rank_and_hand`, which also builds, sorts and cleans the winning
`Five`. The engine reads only the rank. `Seven::hand_rank_value`
(`src/arrays/seven.rs:179`) already gives the rank alone, over the 21
five-card subsets (`:28`).

### How a range seat is sampled

`sample_once` (`engine.rs:253`) picks seats in request order. A range seat
takes a uniform holding from those not yet blocked (`pick_range`, `:300`, 64
tries). If any pick fails, the whole sample is dropped (`return None`) and
`samples` comes back smaller than `max_samples`. Picking seat by seat, each
uniform given the seats before, is **not** in general uniform over compatible
*tuples* of holdings. Whether that bias is big enough to matter has never been
measured.

### What this EPIC does not do

- **No exact post-flop isomorphism.** The suit trick here is for an empty
  board. Fixed boards break the suit symmetry and are already cheap.
- **No change to `Method::Hup`** or to `DealEval`'s heads-up routing. Phase 3
  only makes it *possible* to drop the chart later; that decision is a
  follow-up.
- **No weighted ranges.** `WeightedCombos`
  (`src/analysis/gto/weighted_combos.rs:59`) stays out; range seats remain
  uniform.
- **No Omaha, Razz or Stud.**

---

## Status

| Component | Status |
|---|---|
| Phase 0 — perf baseline + oracle fixtures | Planned |
| Phase 1 — rank-only inner loop, hero-score cache, early exit | Planned |
| Phase 2 — enumerated exact with range seats (`exact_work_threshold`) | Planned |
| Phase 3 — isomorphic exact preflop, 2–10 exact seats | Planned |
| Phase 4 — range-sampling bias: measure, then fix or close | Planned |
| Phase 5 — docs, CHANGELOG, perf results | Planned |

---

## Goals

- **Exact whenever exact is cheap.** The method follows the *work*, not the
  street or the seat type.
- **Exact preflop for any number of fixed hands**, without a data file.
- **Exact hand-vs-range** on flop, turn and river.
- **A faster inner loop** that every path shares.
- **A measured answer** about range-sampling bias, with a DEFECT write-up if
  it is real.

## Scope

The rules this EPIC must keep:

1. **Equity is share-of-pot over a uniform distribution.** The space is every
   compatible assignment of (one holding per seat, one runout). "Compatible"
   means no card appears twice. Each compatible assignment has equal weight.
2. **An exact path counts every compatible assignment exactly once.** Its
   `wins`, `ties` and `samples` are integers that sum as the definition says.
3. **Exact paths agree with each other.** Isomorphic, enumerated and brute
   force give identical integer tallies on the same request.
4. **Monte Carlo targets the same distribution as rule 1.** Phase 4 decides
   whether it does.
5. **No new dependency, no I/O, no `unwrap`** in library code. The kernel
   purity gates (`make check-purity`) stay green.
6. **Serial and parallel give identical integer tallies** (the existing
   `exact_enumerate__counts_are_identical_serial_or_parallel` test, extended
   to every new path).

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| Runout | `Board` built by `build_board` (`engine.rs:323`) | ✅ |
| Holding / range | `Two`, `Combos` → `Vec<Two>` (`engine.rs:146`) | ✅ |
| Exact tally | `Tally` (`engine.rs:384`), `Method::Exact` | ✅ |
| Rank pattern (multiset of 5 board ranks) | `RankPattern` | ❌ |
| Suit coloring weight | `RankPattern::weight` | ❌ |
| Flush-suit assignment | `FlushAssignment` | ❌ |
| Rank-only 7-card value | `Seven::rank_value_ignoring_suits` | ❌ |
| Work estimate | `EquityEngine::work_estimate` | ❌ |

---

## Lessons Learned

pkcore has computed equity at least four ways before this EPIC: brute force through `CaseEvals`, the `hups.bin` chart, `Versus`/`RangeEquity`, and `EquityEngine`. Each one left a mark. Preflop multi-way used to be exact. `DealEval` stored every `CaseEval` of a full enumeration, and `CaseEvals::from_holdem_at_deal` spawned one OS thread per runout (`15ca3ece`, 2023-04-03). `1b09d630` (2026-06-05, `0.1.4`) replaced the threads with `rayon` and added the engine. `36e44e95` (2026-06-18, first tagged `v0.1.5`) sent heads-up to the chart (6.38 s → 694 ns) and three or more seats to Monte Carlo. So three-plus seats lost exactness in exchange for speed, and Phase 3 restores it. The Goals line "the method follows the *work*, not the street or the seat type" already follows `docs/LESSONS_LEARNED.md` Lesson 5 (branching on a count). Each lesson below names the Scope rule, Design section or Work Item that already applies it. Where the EPIC does not cover it yet, it names the gap. Line numbers in code are against `4d28ebb7` (pkcore `0.17.4`).

### 1. The caller's knobs are a contract

- **Before:** since `1b09d630`, setting `exact_threshold: 0` is the only way to force Monte Carlo. The engine tests use it (`src/analysis/equity/engine.rs:544`, `:563`, `:586`), and so do the perf fixtures (`perf/src/catalog_equity.rs:70`, `:147`, `:173`). The perf guard asserts that every fixture not named `exact.` has `exact_threshold: 0` (`catalog_equity.rs:332-336`). The bot callers set only `max_samples` and `seed` and take everything else from `..EquityOptions::default()` (`src/bot/decider.rs:653-657`, `src/bot/preflop_equity.rs:242-246`).
- **What it cost:** `0.11.0` cut the `max_samples` default from 100,000 to 25,000. That was "a **silent** change: nothing fails to compile" (`CHANGELOG.md:1124`), and `spec.rs:186-197` now pins the defaults because of it. After that, the preflop solver path ignored the profile's budget. A strong-tier bench asked for 500 samples, ran 25,000 per decision, and was killed after two hours (EPIC-39 corrigendum 18, `docs/epics/EPIC-39_Decider_Range_Model.md:549`).
- **Lesson:** a new path must obey the knobs callers already use to choose a path and to cap its cost.
- **In this EPIC:** Design §1 adds a second knob instead of changing what `exact_threshold` means. That is the right call. **Gap:**
  - Design §1 step 1 says "no threshold applies" to isomorphic exact. Every request with all seats `Exact`, an empty board and `exact_threshold: 0` then goes exact, which breaks four things:
    - `compute__aa_vs_kk_preflop_mc` (`engine.rs:542`) and `compute__seed_is_deterministic` (`:584`) fail their `Method::MonteCarlo` asserts.
    - DEFECT_033's regression `compute__small_seeds_draw_different_samples` (`:574`) gets eight identical exact answers and fails.
    - `equity.mc.three_way` (`catalog_equity.rs:61-75`) times the exact path under an `mc.` name, and its guard still passes, because the guard checks the knob and not the path.
    - The result conflicts with exit criterion 5.

    Fix: make `exact_threshold == 0` turn off step 1 as well.
    **Applied 2026-10-24:** Design §1 step 0 and item 3d.
  - Some existing code relies on the current routing and needs updating:
    - `examples/two_tables.rs:137-149` samples preflop on default options and asserts `Method::MonteCarlo`. It needs `exact_threshold: 0`.
    - `spec.rs:206` still asserts "preflop must fall to sampling". The assert will keep passing, but its message will no longer be true.
    - The `hu_preflop` workload's doc says "this is true enumeration" (`catalog_equity.rs:37-42`). After Phase 3 that is false. Keep a brute-force workload under its own name if you want a brute-force baseline.
  - Design §1 calibrates `exact_work_threshold` against the 25,000-sample default (202 ms). A caller who asked for 500 samples would still get exact work up to that size. Bound enumerated exact by the caller's own `max_samples` (for example, go exact only when `work_estimate` is no more than the evaluations `max_samples` would spend), or set the field explicitly in both bot callers. Add the new field to `default_options_are_pinned` (`spec.rs:192`).

### 2. A dropped case is a biased case

- **Before:** the code has repeatedly skipped what it could not compute.
  - `Versus::hups_at_deal` returned a partial map on a chart miss until `4ad54d40` (2026-03-30).
  - The first engine returned all-zero equities when every sample failed, until `165eb13b` (2026-06-05) made `count == 0` an error (`engine.rs:183-185`).
  - The perf harness turned a failing `compute()` into checksum 0 with `Status::Ok` until `45913a72` (2026-08-14; `catalog_equity.rs:80-83`).
- **What it cost:** the worst case was silent and selective. `equity_vs_field` built its own chart key, missed, and `continue`d. AA resolved all 1,225 matchups and `72o` resolved only 820, so `72o` scored `0.3845` against a true `0.3469` (EPIC-39 corrigendum 17, `EPIC-39_Decider_Range_Model.md:507-524`; fixed in `0ed358b2`, `v0.12.0`). The skipped matchups were mostly the ones the hand loses.
- **Lesson:** "a partial average is worse than no answer, because it is silently biased" (`EPIC-39…:524`). Count what you drop, or fail.
- **In this EPIC:** Scope rule 2 and items 3a–3b apply this to the exact paths (weights sum to `C(remaining, 5)`; flush assignments partition every board). The MATHS Phase 4 "second skew" covers `pick_range` giving up after 64 tries and `sample_once` dropping the sample (`engine.rs:268-273`, `:300-308`, `:234`). **Gap:**
  - Design §5 reports dropped samples only if the bias turns out to be large. Count drops in 4a for every case, whatever the verdict.
  - In 3e, a chart miss must fail the test, not be skipped. `HUP_CACHE` decodes to an empty map on error (`src/analysis/store/embedded/hup_cache.rs:23-26`), and no test pins its 812,175 entries. A sampled cross-check that skips misses can pass having compared nothing. Assert the chart's size.
  - In 2c, assert that `samples` equals the number of compatible (tuple, runout) pairs.

### 3. Integer tallies from an independent path are the oracle

- **Before:** `84d432da` (2026-06-19) moved `DealEval`'s determinism test from `equity` to `wins`/`ties`. The reason was that float addition is not associative across rayon's reduce. The perf harness folds only integer counts for the same reason (`perf/src/catalog_equity.rs:3-6`). `Tally.equity` is still an `f64` sum (`engine.rs:406`; `docs/TECHNICAL_DEBT.md:311`).
- **What it cost:** the chart's orientation has caused trouble twice. `HUPResult::lookup` ignores argument order and reports from the higher hand's side, so "read naively, every result is inverted for half of all callers" (EPIC-39 corrigendum 7, `:335-345`). The `72o` bug was "corrigendum 7's perspective trap in a second location" (`:522`), and "a structural test alone would not have caught this" (`:528-530`). The fix was checked against the equity engine as an independent path.
- **Lesson:** compare integer tallies, and compare them against a path that shares no code with the one under test.
- **In this EPIC:** Scope rules 3 and 6 and MATHS "Why the golden fixtures are integers" apply this. 3e compares against `hups.bin`, which is exact: 812,175 records, each summing to 1,712,304 (EPIC-39 corrigendum 8, `:347-357`). The 2026-07-31 brute-force AA vs KK run has checksum 1,721,612 (`docs/perf/results/aarch64-apple-darwin-all-features-2026-07-31.json`). That is 1,712,304 + 9,308 ties, which matches the 9,308 draws `src/analysis/gto/vs.rs:483` uses for the same matchup. **Gap:**
  - 3e must go through `HUPResult::lookup` and call `flip_mode()` when `result.higher` is not the first seat's hand, as `DealEval::heads_up_report` does (`src/play/stages/deal_eval.rs:86-103`). Test both seat orders.
  - 2c checks against a "hand-rolled loop". The repo already has an exact hand-vs-range path that returns integer `WinLoseDraw` counts: `Versus::combined_odds_at_river` (`vs.rs:197`) and `combined_odds_at_flop` (`:49`), which `RangeEquity::combined_odds` uses (`src/analysis/range_equity.rs:92`). Use it as the oracle. Assert its total first, because `vs.rs:70` also drops failures. `vs.rs:202` already scores the hero once per board, so Design §2's hero cache is a second copy of that idea. Say in Reuse whether `Versus` will delegate to the engine (`docs/LESSONS_LEARNED.md` Lesson 6: twins multiply bugs).
  - 0c should call `exact_enumerate` directly, not `compute`. After Phase 3, a huge `exact_threshold` on an empty board routes to the isomorphic path, and the recompute would then check that path against itself.

### 4. A check that never builds or runs a code path does not check it

- **Before:**
  - `9919b5ca` (2026-06-05, Copilot autofix) removed `engine.rs`'s `Pile` import as unused. `equity` was off by default and `make build`/`test`/`clippy` never enabled it, so the feature did not compile until `bbbfd74c` (2026-06-17).
  - The engine has a serial path and a `parallel` path, and each build compiles only one of them. `exact_enumerate__counts_are_identical_serial_or_parallel` proves the two agree only because `make test-serial` runs it without `parallel` (`engine.rs:489-499`, `Makefile:239-245`).
  - The 2023 suit-shift work left a defect marker with no description. Its only record was two `#[ignore]` tests. It sat from `4db372e4` (2023-09-15) until it was retired on 2026-08-21 (`docs/TECHNICAL_DEBT.md:138`).
- **What it cost:** a feature build stayed broken for twelve days, and an unexplained marker sat for three years. `make heavy` runs ignored tests only in `tests/heavy_tests` (`Makefile:98-100`). CI runs them only for `bot_marathon` and `replay_consistency` (`.github/workflows/ci.yml:63`, `:88`). An `#[ignore]` test in `src/` runs only when someone remembers to run it.
- **Lesson:** for every `cfg` branch and every `#[ignore]` test, name the command that runs it.
- **In this EPIC:** Scope rule 6 extends the serial-vs-parallel test to every new path, and 3f runs `make check-wasm`. **Gap:**
  - `make check-wasm` is `cargo check` only (`Makefile:290-293`). It never runs the serial code. Add `make test-serial` to Verification.
  - The Verification line `cargo test --features full --lib -- analysis::equity -- --ignored` does not run ignored tests. libtest stops reading flags at the second `--` and treats `--ignored` as a filter. Write `-- analysis::equity --ignored`. **Applied 2026-10-24** in Verification.
  - 3e is `#[cfg(feature = "hup-charts")]`, and `make test-serial` leaves that feature off (`Makefile:245`). Say that 3e runs only in the `full` build.

### 5. Copying the parallel shape copies its flaw

- **Before:** `CaseEvals::from_holdem_at_deal` spawned one thread per runout and cloned the hands into each one: 1,712,304 threads heads-up (`15ca3ece`, 2023-04-03). `1b09d630` (`0.1.4`) replaced that with `par_bridge()` over `combinations()`, and the engine copied the same shape (`engine.rs:207-210`; `src/analysis/case_evals.rs:48`, `:71`).
- **What it cost:** the thread-per-runout code lived for three years. Its replacement is flagged too. `par_bridge` feeds items one at a time from a sequential iterator behind a mutex, with no chunking; the audit calls this "false parallelism" (`docs/audits/PARALLELISM_AUDIT.md:121-127`, F2). TODO-PAR-5 is still open (`:281-286`). The measurements disagree:
  - `docs/perf/PROFILING.md:348` shows `equity.exact.hu_preflop` 2.12× faster at 4 threads.
  - The committed sweep (`docs/perf/results/aarch64-apple-darwin-sweep-2026-07-31.json`) shows it flat: 16.4 s on 1 thread and 16.6 s on 8.
- **Lesson:** parallelise over an indexed source, and re-measure thread scaling whenever the cost per item changes.
- **In this EPIC:** **Gap:**
  - Design §3 says "Parallelise over runouts, as `exact_enumerate` does", which copies `par_bridge`. Use indexed `into_par_iter` instead: over runout indices with combination unranking (TODO-PAR-5), and over a `Vec` of the ≤ 6,188 rank patterns in Phase 3. Phase 1 makes each item several times cheaper, so the serial producer becomes a larger share of the time. That follows from the numbers; it has not been measured. Add a `--sweep` run to 0a and 5b.
  - The default features do not include `parallel` (`Cargo.toml:52-57`), and `wasm32` is serial. The 202 ms that 2d calibrates against is "with the `parallel` feature on" (`spec.rs:83-85`). Measure 2d in a serial build too.

### 6. Measure the function the engine actually calls

- **Before:** `Pile::is_dealt` allocated twice per five-card evaluation from `2f7a3985` (2022-07-02) to `dac7f677` (2026-07-31). The fix made `Five::hand_rank_value` 7.9× faster and `Seven::hand_rank_value` 2.7× faster ([DEFECT_005](../defects/DEFECT_005_is_dealt_allocation.md)). An earlier side quest had optimised those same functions without a benchmark and found about 4% (DEFECT_005, "Coverage Gap").
- **What it cost:** the engine calls neither of those functions. It calls `Eval::from(Seven)` (`engine.rs:200`, `:294`; `src/analysis/eval.rs:329-334`), which also runs `sort().clean()` on the winning hand (`src/arrays/seven.rs:205`). That path costs 5.25–5.5× the rank-only one. In PROFILING's words, "DEFECT_005 fixed a function the equity engine never calls" (`docs/perf/PROFILING.md:186-221`).
- **Lesson:** benchmark, and test for allocations, the exact function the hot loop calls, and compare it with a cheap baseline function.
- **In this EPIC:** Design §2 and item 1c move the engine to `HandRankValue`, which is the fix PROFILING points to. MATHS Phase 1 makes no speed promise before 1d measures. **Gap:**
  - Add `*_does_not_allocate` tests with `crate::alloc_probe` (`src/lib.rs:474`), as DEFECT_005 did (`src/arrays/five.rs:463`, `seven.rs:283`), for `hand_rank_value_ignoring_suits`, `rank_value_ignoring_suits` and `hand_rank_value_beating`.
  - Per-runout allocations remain after Phase 1: the `Vec<Card>` from `combinations` (`engine.rs:195`), the `ranks` `Vec` (`:198`), and three `Vec`s in each `Tally::zeroed` (`:335`, `:392-399`). The Phase 3 scoring loop should not repeat them.
  - `Seven::hand_rank_value` ignores a subset value of 0, which means "not dealt" (`seven.rs:185`). `hand_rank_value_beating` must ignore it too, because 0 is below every threshold. Add that case to 1b's tests.

### 7. Check how a number was measured before you rely on it

- **Before:** from `1b09d630` until `0.16.0` (`f39f5d77`), the engine seeded sample `i` with `seed ^ i`. For seeds below 8 that only reorders a single set of samples ([DEFECT_033](../defects/DEFECT_033_small_monte_carlo_seeds_draw_the_same_samples.md)). The `two_tables` example (`53345346`, 2026-10-08) worked around this by spreading its seeds. The defect was filed only after the 2026-10-17 automated review found it. The example's comment still describes `seed ^ index` (`examples/two_tables.rs:135-136`).
- **What it cost:** seeds 0–7 gave identical results, and `DEFAULT_SEED` 0 matched user seeds 1–7. The precision table on `max_samples` was "measured … 40 seeds" (`src/analysis/equity/spec.rs:71`). If those were small consecutive seeds, it came from about five distinct sample sets. The re-measurement is still open (`docs/TECHNICAL_DEBT.md:300`).
- **Lesson:** before a measured number becomes a design input, check how it was measured. A workaround in an example is a defect report nobody filed.
- **In this EPIC:** Phase 4a's 200-seed runs come after the fix, and `compute__small_seeds_draw_different_samples` (`engine.rs:574`) guards that seeds draw different samples. **Gap:** the Summary and MATHS Phase 0 treat the ~0.7 pp / ~0.3 pp table as measured truth. Re-measure it in Phase 0 and close the debt item, or cite only the SE bound. Fix the stale comment in `two_tables.rs` along with the change in lesson 1.

### 8. Look outside before you optimise in place: a better evaluator already exists

Lessons 1–7 come from pkcore's own history. This one comes from outside it.
[`b-inary/holdem-hand-evaluator`](https://github.com/b-inary/holdem-hand-evaluator)
by Wataru Inariba (MIT, last commit `d7b2a5bb`, 2022-06-02) solves the
problem Phase 1 works on, and solves it a better way. Phase 1 makes pkcore's
21-subset loop cheaper. That crate does not have the loop at all.

- **Before (pkcore today).** A seven-card hand is ranked by building all 21
  five-card subsets and ranking each one (`src/arrays/seven.rs:179-190`):

  ```rust
  for perm in Seven::FIVE_CARD_PERMUTATIONS {          // 21 times
      let hand = self.five_from_permutation(perm);     // copy 5 cards
      let hrv = hand.hand_rank_value();                // flush table, unique table,
      if (best_hrv == 0) || hrv != 0 && hrv < best_hrv // or a binary search (≤ 13 steps)
          { best_hrv = hrv; }
  }
  ```

  And the engine rebuilds every hand from scratch for every runout
  (`engine.rs:194-200`): it collects the runout into a `Vec`, builds a
  `Board`, then builds a `Seven` per seat from the two hole cards and the five
  board cards.

- **The better way, part 1: a hand is a sum, and ranking it is two reads.**
  The crate stores a hand as two `u64`s. Each card adds a fixed value to
  both (`src/hand.rs:93-99`). The `key` sums rank values chosen so that every
  multiset of 5–7 ranks has a different sum, plus a 4-bit counter per suit
  that starts at 3 (`0x3333`), so the counter's high bit turns on at the
  fifth card of a suit. The `mask` sets one bit per card. Ranking is then
  about ten lines (`src/hand.rs:115-126`):

  ```rust
  pub fn evaluate(&self) -> u16 {
      let is_flush = self.key & FLUSH_MASK;                 // any suit counter reached 8?
      if is_flush > 0 {
          let flush_key = (self.mask >> (4 * is_flush.leading_zeros())) as u16;
          LOOKUP_FLUSH[flush_key as usize]                  // 8,129 entries
      } else {
          let rank_key = self.key as u32 as usize;          // the rank-multiset sum
          let offset = OFFSETS[rank_key >> OFFSET_SHIFT] as usize;
          LOOKUP[rank_key.wrapping_add(offset)]             // perfect hash, ≥ 73,775 entries
      }
  }
  ```

  No subsets, no sort, no search, no allocation. The tables are about
  212 KB (`Readme.md`). The README claims ~1.2 G evaluations per second on
  one Ryzen 9 5950X thread; that is the author's number and has not been
  measured here.

- **The better way, part 2: share the board, add cards one at a time.**
  Because a hand is a sum, the crate adds each board card to both players'
  hands once per loop level, not once per runout
  (`src/heads_up.rs:129-160`):

  ```rust
  for i in 0..(len - 4) {
      let hand1 = hand1.add_card(alive_cards[i]);           // two additions
      let hand2 = hand2.add_card(alive_cards[i]);
      for j in (i + 1)..(len - 3) {
          // … three more levels …
                      let rank1 = hand1.evaluate();         // then two reads each
                      let rank2 = hand2.evaluate();
  ```

  The outer levels' additions are shared by every runout below them. There
  is no `Vec`, no `Board` and no `Seven` per runout.

- **The better way, part 3: test every hand against a naive evaluator.**
  `src/hand_test.rs:103-147` walks all C(52, 7) = 133,784,560 seven-card
  hands with the same nested loops. For each one it checks that the fast rank
  maps to exactly one rank from a slow, obviously correct evaluator, and
  then that the order of the two agrees. That is lesson 3's independent
  oracle, made exhaustive.

- **What it cost to not look.** Nothing yet: Phase 1 has not started. But
  items 1a and 1b (`hand_rank_value_ignoring_suits`, `rank_value_ignoring_suits`,
  `hand_rank_value_beating`) are new code that tunes the loop this design
  removes. The early exit saves at most part of the 21 subsets; the crate
  never has 21 subsets to save.

- **Two things not to copy.** The crate checks nothing: `evaluate` "may
  crush when `self.len() < 5 || self.len() > 7`" (`src/hand.rs:113`), it reads
  tables with `unsafe { get_unchecked }`, and `heads_up_win_frequency` uses
  `assert!`. pkcore's rules forbid all three in library code. And its order
  is the opposite of pkcore's: a higher value is stronger
  (`get_hand_category`, `src/hand.rs:22`), where pkcore's `HandRankValue` is
  lower-is-stronger. Lesson 6 of EPIC-121 is what happens when that kind of
  direction is left implicit.

- **Lesson:** before you optimise a hot loop, look for published work that
  removes the loop, and use its test pattern (a naive oracle over every
  input) even if you keep your own code.

- **In this EPIC:** **Gap.** Nothing in the EPIC mentions outside prior art.
  - Keep item **1c** (score with `hand_rank_value`, not `Eval::from`). It is
    worth doing first whatever comes next.
  - Before items **1a** and **1b**, decide whether to vendor the crate's
    evaluator instead, behind the existing `HandRanker` trait. Vendoring keeps
    Scope rule 5 ("no new dependency"), and the crate is not on crates.io
    anyway (its README uses a `git` dependency). Carry its MIT notice the way
    `src/lookups/LICENSE` carries the pokereval-rs notice. Wrap it so that
    the card count is checked, there is no `unsafe` on an unchecked index,
    there is no `assert!`, and its higher-is-stronger value is mapped to
    `HandRankValue` with a test that pins the direction.
  - The proof for either choice is the crate's own pattern: an exhaustive
    `#[ignore]` test over all 133,784,560 seven-card hands, comparing the new
    path with `Seven::hand_rank_value`, named in Verification with the
    command that runs it (lesson 4).
  - Phase 3 maps directly onto the crate's two branches. A player's `v_rank`
    on a rank pattern is one non-flush `LOOKUP` read on the rank sum of
    their two hole cards plus the pattern's five ranks. Their `v_flush` on a
    flush assignment (s, S) is one `LOOKUP_FLUSH` read on the 13-bit mask of
    S plus their own suit-s ranks.
  - Phase 2's loops can add cards incrementally, as in part 2, instead of
    building a `Board` per runout.

### Still alive today

| Pattern | Where (file:line) | Touched by this EPIC? |
|---|---|---|
| 21 five-card subsets per seven-card hand; every hand rebuilt per runout | `src/arrays/seven.rs:179-190`; `engine.rs:194-200` | Yes. Phase 1 tunes it; lesson 8 replaces it |
| `exact_threshold: 0` is the only way to force Monte Carlo | `src/analysis/equity/engine.rs:544,563,586`; `perf/src/catalog_equity.rs:70,332-336` | Yes. Kept by Design §1 step 0 (lesson 1) |
| Bot callers take every other option from the defaults | `src/bot/decider.rs:653-657`; `src/bot/preflop_equity.rs:242-246` | Yes. They inherit the new `exact_work_threshold` |
| Range seats picked one after another, 64 tries, failed sample dropped | `engine.rs:268-273`, `:300-308`, `:234` | Yes. Phase 4 |
| Average over the chart lookups that succeed, misses skipped | `src/bot/preflop_equity.rs:92-94` | No |
| Failing cases dropped by `filter_map(.ok())` | `src/analysis/case_evals.rs:45`, `:65-68`; `src/analysis/gto/vs.rs:70` | No |
| Chart decode error gives an empty map | `src/analysis/store/embedded/hup_cache.rs:23-26` | Read by 3e; not fixed |
| `f64` equity summed in reduce order | `engine.rs:406` | No. Tests compare integers (Scope rule 6) |
| `par_bridge()` over `combinations()` | `engine.rs:208`; `case_evals.rs:48`, `:71` | Copied by Design §3 unless changed |
| `Eval::from(Seven)` (with sort and clean) in the hot loop | `engine.rs:200`, `:294`; `src/arrays/seven.rs:205` | Yes. Item 1c |
| New `HashSet<Card>` per sample; several `Vec`s per runout | `engine.rs:262`; `:195-198`, `:335` | No (TODO-PAR-2) |
| Stale `seed ^ index` comment | `examples/two_tables.rs:135-136` | No |
| Unfinished suit-shift code | `src/arrays/matchups/sorted_heads_up.rs:145`; `src/arrays/matchups/shift.rs:40`; `src/analysis/store/db/hup.rs:106` | No. Design §4 takes a different route |
| `#[ignore]` tests in `src/` that no make target or CI job runs | `Makefile:98-100` | Yes. 0c, 3e and 4a add more |
| 40-seed precision table, re-measurement open | `src/analysis/equity/spec.rs:71`; `docs/TECHNICAL_DEBT.md:300` | Cited by the Summary and MATHS |
| Preflop brute force collects 1.7M `CaseEval`s; nothing in the crate calls it | `src/analysis/case_evals.rs:63-76` | No |

---

## Design

### 1. The decision, made on work

`EquityEngine::compute` chooses in this order:

0. **`exact_threshold == 0` → Monte Carlo**, as today. It is the one knob
   callers use to force sampling (tests, perf fixtures; see Lessons Learned
   1), so it turns off both exact paths below.
1. **Isomorphic exact**: every seat `Exact`, board empty → Phase 3 path.
   Cost is fixed and small (≤ 6,188 patterns), so its size is not compared
   with a threshold.
2. **Enumerated exact**: `work_estimate(seats, runouts) ≤ exact_work_threshold`.
   Seats may be `Exact` or ranges.
3. **Monte Carlo** otherwise.

```rust
// src/analysis/equity/spec.rs
pub struct EquityOptions {
    pub exact_threshold: u64,      // unchanged meaning: max runouts, all-exact seats
    pub exact_work_threshold: u64, // new: max (runouts × ∏ range sizes) for range seats
    pub max_samples: u64,
    pub seed: Option<u64>,
}
```

`work_estimate` is `runouts × ∏ |candidates_i|` over range seats, saturating
(reuse the `n_choose_k` saturation style, `engine.rs:353`). `Random` counts as
a range of every remaining holding. The default for `exact_work_threshold` is
**set by measurement in Phase 2**: the largest value at which the slowest
qualifying request costs no more than today's six-seat Monte Carlo default
(202 ms native, `spec.rs:77`).

Why a second knob, not a new meaning for `exact_threshold`: the existing knob
is documented and used downstream as "runouts". Changing what a number means
without changing its name is the worst kind of break.

### 2. The inner loop

```rust
// src/arrays/seven.rs
impl Seven {
    /// Best 5-card value ignoring suits: unsuited lookup on every subset.
    pub(crate) fn rank_value_ignoring_suits(&self) -> HandRankValue;
    /// Stops at the first subset that is strictly better than `bar`.
    /// Returns that subset's value, or the true best if none beats `bar`.
    pub(crate) fn hand_rank_value_beating(&self, bar: HandRankValue) -> HandRankValue;
}

// src/arrays/five.rs — the unsuited half of `hand_rank_value` (`:244`)
impl Five {
    pub(crate) fn hand_rank_value_ignoring_suits(&self) -> HandRankValue;
}
```

- The engine scores with `HandRankValue` (`u16`, lower is better,
  `src/analysis/hand_rank.rs:20`) instead of building an `Eval`. This removes
  the `Five` sort and clean on every call. `tally_from_ranks` (`engine.rs:332`)
  changes from `max` over `HandRank` to `min` over non-zero values. One test
  pins that the two orders agree on every hand class.
- **Hero-score cache.** In enumerated exact with one `Exact` seat and range
  seats, the `Exact` seat's value depends only on the runout. Score it once
  per runout, not once per (runout, villain holding).
- **Early exit** applies only where the exact value of the loser is not
  needed: heads-up, one exact seat versus one range seat. Score the hero, then
  call `hand_rank_value_beating(hero)` for the villain. A value below the bar
  means "villain wins"; anything else is the true value and decides win or
  tie.

### 3. Enumerated exact with range seats

```rust
// src/analysis/equity/engine.rs
fn exact_enumerate_ranges(
    resolved: &[Resolved],
    board_vec: &[Card],
    remaining: &[Card],
    b: usize,
) -> Tally;
```

Walk every runout. For each runout, walk the Cartesian product of range-seat
candidates, skipping any tuple whose cards collide with each other, the board
or the runout. Each surviving (tuple, runout) is one count, which is rule 1
exactly. Parallelise over runouts, as `exact_enumerate` does (`:194`).

The heads-up special case (one exact seat, one range seat) is the hot one: it
uses the cache and the early exit from Design 2.

### 4. Isomorphic exact preflop

`src/analysis/equity/isomorphic.rs` (new, `pub(crate)`).

**The idea.** With an empty board and fixed hands, a board's outcome depends
on (a) its **ranks** and (b) **whether and how it makes a flush**. Suits
matter for nothing else. So: group the 1.7 M boards by rank pattern, score
each pattern once without suits, and correct only the boards where a flush is
possible.

**Rank patterns.** A pattern is a multiset of 5 ranks: counts `k_r ≥ 0` with
`Σ k_r = 5`. There are C(13+5−1, 5) = **6,188**. For pattern P and the deck
left after the hole cards, let `live_r` be the cards of rank r still in the
deck. The number of real boards with pattern P is

```text
W(P) = ∏_r C(live_r, k_r)        // 0 when any k_r > live_r: skip P
```

Σ W(P) over all patterns = C(remaining, 5) — this is a unit test.

**Flush assignments.** Five board cards hold at most one suit three or more
times. A *flush assignment* is (suit s, a set S of 3–5 distinct ranks of P
whose s-card is live). Its weight is

```text
W(P, s, S) = ∏_{r∈S} C(live_r^{¬s}, k_r − 1) · ∏_{r∉S} C(live_r^{¬s}, k_r)
```

where `live_r^{¬s}` counts live cards of rank r in suits other than s. Every
board with three or more of one suit belongs to exactly one (P, s, S); every
other board belongs to its P alone. So per pattern:

```text
W_rainbow(P) = W(P) − Σ_{s,S} W(P, s, S)
```

**Scoring.** For each player, per pattern:

- `v_rank` = `rank_value_ignoring_suits` over hole + pattern ranks. This is the
  player's true value on every board of P where they have no flush, because
  non-flush 5-card values depend only on ranks.
- For each (s, S): if the player holds suit-s cards so that `|S| + held_s ≥ 5`,
  `v_flush` = the best flush (or straight flush) from the s-ranks, via the
  flush table indexed by rank bits (`src/lookups/flushes.rs:5`, as
  `Five::hand_rank_value` does at `five.rs:247-248`). The player's value on
  those boards is `min(v_rank, v_flush)` (lower is better). Otherwise it is
  `v_rank`.

Then add `W_rainbow(P)` boards to the tally decided by the `v_rank` values,
and `W(P, s, S)` boards to the tally decided by each assignment's values.
`Tally` already carries `u64` counts and an `f64` equity sum (`engine.rs:384`);
it gains a weighted `add(winners, weight)`.

**Cost.** At most 6,188 patterns × (n rank-only scores + ≤ 64 assignments ×
n cheap flush lookups). That is roughly 3 orders of magnitude fewer full
evaluations than brute force, and the cost does not grow with the number of
boards.

**Why this shape.** The alternative — enumerate every board and cache by
canonical suit permutation — still visits 1.7 M boards. The pattern walk
visits 6,188 and never builds a board. And it needs no table, so it works the
same on `wasm32` and in the no-`hup-charts` build.

### 5. Range-sampling bias

**Hypothesis.** Sequential picking (`sample_once`, `engine.rs:253`) samples
holding tuples with probability `∏ 1/|compatible_i given earlier seats|`,
which is not uniform over compatible tuples when ranges overlap. Dropped
samples (`return None`) add a second, smaller skew.

**Measure first.** With Phase 2 in place, pick small overlapping cases with
an exact answer (for example `AA,KK` vs `AK` on a fixed flop, and the same with
the seats swapped). Run Monte Carlo over 200 seeds. Compare the mean with the
exact value in units of standard error.

**Then decide.**

- If the bias is under 1 standard error of a default 25,000-sample run on
  every case: record the numbers in this EPIC's corrigendum and close.
- If it is larger: write the DEFECT (next free number in `docs/defects/`) and
  replace sequential picking with **joint rejection** — draw every range seat
  independently, reject the whole tuple on any collision, retry. This is
  uniform over compatible tuples by construction. Report dropped samples in
  the result instead of hiding them.

---

## MATHS

One subsection per Work Items phase. Each one breaks down the math the phase
relies on, then lists where to learn more. Every number below was checked on
2026-10-24 with a throwaway script, using brute force wherever brute force was
possible. The worked matchup is **A♠ K♦ vs 7♣ 7♥**, preflop, unless stated.

Notation: `C(n, k)` is "n choose k". `HandRankValue` is lower-is-stronger,
1 = royal flush, 7462 = worst high card, 0 = not dealt
(`src/analysis/hand_rank.rs:20-22`).

### Foundations — one sum, three ways to compute it

Every path in this EPIC computes the same number. Let Ω be the set of
compatible assignments ω = (h₁, …, hₙ, B): one holding per seat plus one
runout, no card used twice (Scope rule 1). For seat i:

```text
share_i(ω) = 1/k   if seat i is one of the k players with the best hand on ω
           = 0     otherwise

E_i = (1 / |Ω|) · Σ_{ω ∈ Ω} share_i(ω)          and  Σ_i E_i = 1
```

`tally_from_ranks` (`engine.rs:332`) is `share_i` for one ω: one winner gets
1.0, k tied winners get 1/k each. The three methods differ only in how they
evaluate the sum:

| Method | How it evaluates the sum | Exact? |
|---|---|---|
| Enumerated exact | visit every ω once | yes |
| Isomorphic exact | group ω into classes on which `share` is constant; add `weight × share` per class | yes |
| Monte Carlo | draw ω at random, average `share` | only on average, and only if every ω is equally likely to be drawn |

The last row is the whole of Phase 4: a sampler that draws some ω more often
than others estimates a different sum.

**Learn more**

- [Expected value](https://en.wikipedia.org/wiki/Expected_value): E_i is the
  expected pot share under the uniform distribution on Ω.
- Bill Chen and Jerrod Ankenman, *The Mathematics of Poker* (ConJelCo, 2006),
  Part I: equity, and card removal against ranges.

### Phase 0 — Baseline and oracles

**Counting runouts.** With b board cards still to come and m cards left in
the deck, there are C(m, b) runouts. Heads-up, each seat removes 2 cards:

| Request | Cards left | Runouts | Five-card lookups, brute force (runouts × seats × 21) |
|---|---|---|---|
| heads-up preflop | 48 | C(48,5) = 1,712,304 | 71,916,768 |
| 3-way preflop | 46 | C(46,5) = 1,370,754 | 86,357,502 |
| 6-way preflop | 40 | C(40,5) = 658,008 | 82,909,008 |
| heads-up flop | 45 | C(45,2) = 990 | 41,580 |
| heads-up turn | 44 | 44 | 1,848 |

More seats means fewer runouts but more hands to score per runout, so
preflop brute force costs about the same at 2, 3 or 6 seats. The 21 is
C(7,5), the five-card subsets of seven cards (`seven.rs:28`).

**Why the golden fixtures are integers.** Exact paths must agree *exactly*
(Scope rule 3). `wins`, `ties` and `samples` are integers, so two exact paths
either give equal integers or one of them is wrong. A float equity would need
a tolerance, and a tolerance can hide an off-by-one board.

**Why Monte Carlo needs a tolerance.** A sample's share lies in [0, 1], so
its variance is at most 1/4 (Popoviciu's inequality). The mean of n
independent samples therefore has standard error

```text
SE ≤ 0.5 / √n          n = 25,000  → SE ≤ 0.32 pp,  95% interval ≈ ±0.62 pp
                       n = 100,000 → SE ≤ 0.16 pp,  95% interval ≈ ±0.31 pp
```

That matches the worst errors measured in `spec.rs:66-86` (~0.7 pp and
~0.3 pp). Those are the *worst* errors over 40 seeds and several seats, and
the worst of many draws sits 2–3 SE from the truth.

**Why the perf baseline uses medians.** The `perf/` harness reports a median
and a MAD (median absolute deviation) per workload. One slow run moves a mean
a lot and a median hardly at all, so before/after deltas are compared on
medians.

**Learn more**

- [Binomial coefficient](https://en.wikipedia.org/wiki/Binomial_coefficient)
- [Standard error](https://en.wikipedia.org/wiki/Standard_error) and
  [Popoviciu's inequality on variances](https://en.wikipedia.org/wiki/Popoviciu%27s_inequality_on_variances)
- [Binomial proportion confidence interval](https://en.wikipedia.org/wiki/Binomial_proportion_confidence_interval):
  why ±1.96 SE is the 95% interval, and when that approximation is poor
  (equity near 0 or 1).
- [Median absolute deviation](https://en.wikipedia.org/wiki/Median_absolute_deviation)

### Phase 1 — Inner loop

**Lower-is-better gives the same winners.** Today the engine takes the
`max` `HandRank`. `HandRankValue` orders the same hands in reverse. A strictly
order-reversing map turns the set of maximal elements into the set of minimal
elements, so `argmax HandRank = argmin HandRankValue` on every runout, ties
included. The one extra rule is that 0 means "not dealt" and must not win:
take the minimum over **non-zero** values. `tally_from_values__agrees_with_hand_rank_order`
pins this.

**Why suits can be ignored for most hands.** There are 7,462 distinct
five-card values (Cactus Kev's classes):

| Needs one suit | Values | | Rank-only | Values |
|---|---|---|---|---|
| straight flush | 10 | | quads | 156 |
| flush | 1,277 | | full house | 156 |
| | | | straight | 10 |
| | | | trips | 858 |
| | | | two pair | 858 |
| | | | pair | 2,860 |
| | | | high card | 1,277 |
| **total** | **1,287** | | **total** | **6,175** |

The 6,175 rank-only values match, one for one, the multisets of five ranks
with no rank five times: C(17, 5) − 13 = 6,188 − 13 = 6,175. So a non-flush
value is a function of the five ranks alone, and suits only decide whether a
flush is present. This one fact makes `hand_rank_value_ignoring_suits` correct
here and the whole of Phase 3 possible.

**What one lookup costs.** `Five::hand_rank_value` (`five.rs:244`) is one
table read for a flush (`FLUSHES`), one for five distinct ranks
(`UNIQUE_5`), and otherwise a binary search over the 4,888 prime products
(`five.rs:137`): at most ⌈log₂ 4,888⌉ = 13 comparisons. A seven-card score is
21 of those. See [Cactus Kev lookup core](../../.okf/cactus-kev-lookup.md)
for the prime-product trick.

**The hero cache.** Heads-up against a range V, the naive loop scores both
hands for each (runout, villain holding): 2·|V| scores per runout. The hero's
hand does not depend on the villain, so score it once per runout:
1 + |V| scores. As |V| grows this tends to **half** the work. With e exact
seats, the saving is e·|V| − e scores per runout.

**Early exit is exact.** Let h be the hero's value and `hand_rank_value_beating(h)`
scan the villain's 21 subsets, stopping at the first value below h.

- If it stops, the villain's best is at most that value, which is below h.
  The villain wins. The villain's true best is not needed for that.
- If it never stops, it has seen all 21 subsets, so it returns the true
  best v ≥ h. Then v = h is a tie and v > h is a hero win.

Both branches give the right `share`. The saving happens only on boards the
villain wins, and only for the subsets after the first winning one. So the
saving is at most P(villain wins) of the villain's scoring cost. Item 1d
measures the real number; do not promise one in advance.

**Learn more**

- [Cactus Kev's Poker Hand Evaluator](https://suffe.cool/poker/evaluator.html):
  the 7,462 classes, the prime products, and the flush table.
- [Poker probability](https://en.wikipedia.org/wiki/Poker_probability):
  the "distinct hands" column gives the class counts above.
- [Stars and bars](https://en.wikipedia.org/wiki/Stars_and_bars_(combinatorics)):
  where C(17, 5) comes from (also used in Phase 3).
- [Binary search](https://en.wikipedia.org/wiki/Binary_search)

### Phase 2 — Enumerated exact with range seats

**Every compatible tuple has the same number of runouts.** A tuple of n
two-card holdings uses 2n cards, whatever they are. So every compatible
tuple leaves m = 52 − |board| − 2n cards, and C(m, b) runouts. With T the
number of compatible tuples:

```text
|Ω| = T · C(m, b)

E_i = (1 / T) · Σ_{compatible tuples t} E_i(t)
```

So equity against a range is **the plain average of the exact equity against
each compatible holding**, every combo weighted the same once blocked combos
are removed. That is the textbook definition of hand-vs-range equity, and it
is also the target distribution Phase 4 holds Monte Carlo to: uniform over
compatible tuples.

**Loop order does not change the answer.** The design walks runouts on the
outside and tuples on the inside. The other order visits the same finite set
of (tuple, runout) pairs, and a finite sum does not depend on the order of its
terms. Runouts go outside because the hero's value is then constant across
the inner loop, which is what makes the cache work.

**`work_estimate` is an upper bound, and the right cost measure.**

```text
work_estimate = R · ∏_{range seats} |C_i|
```

R counts runouts from the cards left after the board and the exact seats.
The product counts tuples before collisions are removed, so it is at least
T · C(m, b). The exact count of compatible tuples needs inclusion–exclusion
over every pair of colliding seats. That is not needed: the loop *visits*
every element of the product and skips collisions, so its cost is
proportional to the bound, not to |Ω|. Example, heads-up flop, hero vs
`Random`: R = C(47, 2) = 1,081; the villain's candidates are the same 1,081
holdings; the estimate is 1,081² = 1,168,561; the true |Ω| is
1,081 · C(45, 2) = 1,070,190. The bound is 9% high.

**Why the product must saturate.** One unrestricted seat has up to
C(52, 2) = 1,326 holdings. 1,326⁶ ≈ 5.4·10¹⁸ still fits in a `u64`
(max ≈ 1.8·10¹⁹); 1,326⁷ ≈ 7.2·10²¹ does not, and the runout factor gets
there sooner. A wrapping multiply would turn "astronomically large" into a
small number and select the exact path by mistake. A saturating multiply
pins it at `u64::MAX`, which is always over the threshold. `n_choose_k`
already saturates this way (`engine.rs:353`).

**Setting the default.** Model the cost as t ≈ c · work, measure c on the
slowest qualifying shapes (item 2d), and take the largest work with
c · work ≤ 202 ms, today's six-seat Monte Carlo cost (`spec.rs:77`). The
exact path then never costs more than the sampling it replaces.

**Learn more**

- [Inclusion–exclusion principle](https://en.wikipedia.org/wiki/Inclusion%E2%80%93exclusion_principle):
  what an exact count of compatible tuples would take, and why the design
  avoids it.
- Ronald Graham, Donald Knuth and Oren Patashnik, *Concrete Mathematics*
  (2nd ed., Addison-Wesley, 1994), ch. 2: manipulating and reordering sums.
- [`u64::saturating_mul`](https://doc.rust-lang.org/std/primitive.u64.html#method.saturating_mul)
- Chen and Ankenman (above): equity against a distribution of hands.

### Phase 3 — Isomorphic exact preflop

**3a. Counting rank patterns.** A pattern is how many board cards of each
rank: k₂, …, k_A ≥ 0 with Σ k_r = 5. Choosing 5 items from 13 kinds with
repetition is "stars and bars":

```text
C(13 + 5 − 1, 5) = C(17, 5) = 6,188
```

Patterns that need more cards of a rank than are live get weight 0 and are
skipped.

**3a. Pattern weights, and why they sum to the right total.** The boards
with pattern P choose k_r of the live_r cards of each rank, independently:

```text
W(P) = ∏_r C(live_r, k_r)

Σ_P W(P) = C(Σ_r live_r, 5) = C(remaining, 5)
```

The second line is the multivariate Vandermonde identity. Proof by counting:
choose 5 cards from the whole remaining deck, then sort them by rank; the
counts per rank are some pattern P, and each 5-card board arises in exactly
one P. For A♠ K♦ vs 7♣ 7♥ the sum is 1,712,304 = C(48, 5), as
`rank_patterns__weights_sum_to_remaining_choose_5` will assert.

**3b. At most one flush suit, and only with three or more on board.** A
player holds two cards, so a flush in suit s needs at least 3 board cards of
s. Two suits with 3 or more each would need 6 board cards. So each board has
at most one suit that can make a flush.

**3b. The flush-assignment weight.** Take a board in pattern P with three or
more cards of suit s. Let S be the ranks of its s-cards; they are distinct,
because there is one card per rank per suit. Every other board card of rank r
is *not* an s-card (otherwise its rank would be in S). So, rank by rank:

- r ∈ S: the s-card of r is on the board; the other k_r − 1 come from the
  live_r^{¬s} non-s cards.
- r ∉ S: all k_r come from the non-s cards.

```text
W(P, s, S) = ∏_{r ∈ S} C(live_r^{¬s}, k_r − 1) · ∏_{r ∉ S} C(live_r^{¬s}, k_r)
             (over S of size 3–5 whose s-cards are all live)

W_rainbow(P) = W(P) − Σ_{s,S} W(P, s, S)
```

Each board with three or more of one suit has exactly one (s, S), so the
assignments plus the rainbow remainder partition the boards. For
A♠ K♦ vs 7♣ 7♥, Σ W(P, s, S) = 628,848, and a brute-force count of boards
with three or more of one suit gives the same 628,848 (36.7% of all boards).
A pattern has at most 5 distinct ranks, so at most
4 · (C(5,3) + C(5,4) + C(5,5)) = 4 · 16 = **64** assignments; the walk
reaches 64.

**3c. Why `min(v_rank, v_flush)` is the true value.** A player's true value
is the best of their 21 subsets. A subset that is not a flush has its rank-only
value (Phase 1). A subset that is a flush has a flush or straight-flush
value, which is always better than the same five cards' rank-only value.
`v_rank` takes the best rank-only value over *all* 21 subsets, and `v_flush`
is the best flush subset. So:

```text
true = min( best non-flush subset , v_flush )
v_rank = min( best non-flush subset , rank-only values of the flush subsets )
and each flush subset's rank-only value ≥ its flush value ≥ v_flush
⇒ min(v_rank, v_flush) = true
```

A player without a flush on those boards has no flush subset, so `v_rank` is
already the true value. This also covers the board that has both a flush and
a full house for someone: the full house is inside `v_rank` and wins the
`min`.

**3c. The weighted tally.**

```text
E_i = (1 / C(remaining, 5)) · Σ_P [ W_rainbow(P) · share_i(v_rank values)
                                  + Σ_{s,S} W(P, s, S) · share_i(values on that assignment) ]
```

Every W is an integer, so `wins`, `ties` and `samples` stay integers. That is
why the result can equal brute force (item 3d) and the `hups.bin` chart
(item 3e) exactly, not approximately.

**Cost, honestly.** Rank-only scoring runs once per (pattern, seat) instead
of once per (board, seat). The ratio is boards ÷ patterns:

| Request | Boards | Patterns | Ratio |
|---|---|---|---|
| heads-up | 1,712,304 | ≤ 6,188 | ≈ 277× |
| 3-way | 1,370,754 | ≤ 6,188 | ≈ 222× |
| 6-way | 658,008 | ≤ 6,188 | ≈ 106× |

That is about 2–2.4 orders of magnitude, not the "roughly 3" in Design §4,
plus at most 64 cheap flush lookups per pattern and seat. The ratio shrinks as
seats are added, because boards fall and patterns do not.

**Relation to hand isomorphism.** Relabelling suits does not change who
wins, and counting one representative per class with a weight is the idea
behind Burnside's lemma and the hand isomorphism used by poker solvers. Hole
cards break most of that symmetry, so this design does not enumerate suit
classes. It splits on the one thing suits can change, whether a flush exists,
and weights that split directly.

**Learn more**

- [Stars and bars](https://en.wikipedia.org/wiki/Stars_and_bars_(combinatorics))
- [Vandermonde's identity](https://en.wikipedia.org/wiki/Vandermonde%27s_identity),
  including the generalized form used above.
- [Burnside's lemma](https://en.wikipedia.org/wiki/Burnside%27s_lemma):
  counting classes under a symmetry group.
- Kevin Waugh, "A Fast and Optimal Hand Isomorphism Algorithm", AAAI-13
  Workshop on Computer Poker and Imperfect Information (2013): the
  suit-isomorphism approach this design deliberately does not take.
- *Concrete Mathematics* (above), ch. 5: binomial-coefficient identities.

### Phase 4 — Range-sampling bias

**4a. What the sampler actually draws.** `sample_once` (`engine.rs:253`)
fills seats in request order. A range seat picks uniformly among the holdings
compatible with the seats already filled (`pick_range`, `:300`). So a
compatible tuple t is drawn with probability

```text
P_seq(t) = ∏_i 1 / |C_i(t₁, …, t_{i−1})|          target: 1 / T
```

That is uniform only when the product of the compatible counts is the same
for every tuple.

**A worked example.** Seat 1 holds {AA, 22} (6 + 6 = 12 combos); seat 2
holds {AK} (16 combos).

- Compatible tuples: an AA combo leaves 2 aces, so 2 · 4 = 8 AK combos;
  a 22 combo leaves all 16. T = 6·8 + 6·16 = 48 + 96 = **144**. The true
  chance that seat 1 holds AA is 48 / 144 = **1/3**. The AK in the other
  hand blocks aces: that is card removal.
- Seat 1 picked first: AA is picked half the time, and whatever seat 2
  then picks cannot change that. P(seat 1 holds AA) = **1/2**.
- Seat 2 picked first: every AK combo leaves 3 AA combos + 6 22 combos = 9
  holdings, so every tuple has probability (1/16)·(1/9) = 1/144. **Uniform.**

So the same request can give different answers depending on seat order. The
first seat is drawn as if the later seats' cards did not exist. How far the
*equity* moves depends on how different AA and 22 do against AK; that is what
4a measures. `Exact` seats add no bias, and neither do `Random` seats: given
the cards already taken, the number of compatible random holdings is always
C(cards left, 2), the same for every earlier pick.

**The second skew: dropped samples.** `pick_range` tries 64 times. If a
fraction q of the seat's candidates is compatible, it fails with probability
(1 − q)⁶⁴: about 5·10⁻²⁰ at q = 0.5, but about 3.7% at q = 0.05. Failures
cluster where q is small, so dropping them under-samples exactly the tuples
where blocking is heaviest.

**4a. How the measurement decides.** Let b = E[Ê] − E be the bias of one
run's estimate Ê. Average 200 seeded runs of 25,000 samples each:

```text
one run:          SE ≈ 0.32 pp          (Phase 0)
mean of 200 runs: SE ≈ 0.32 / √200 ≈ 0.022 pp
z = (mean − exact) / 0.022 pp
```

That resolves a bias down to about 0.05 pp. The closing rule in Design §5
is |b| < 1 run SE (0.32 pp). The reason: the total error of one run is
√(b² + SE²), so a bias equal to one SE makes the error √2 ≈ 1.41 times
larger, which spends the precision the 25,000-sample default promises
(`spec.rs:66-86`).

**4b. Why joint rejection is uniform.** Draw every range seat independently
and uniformly from its full candidate list; accept only if no card repeats.
Before the check, every tuple has the same probability ∏ 1 / |C_i|. Keeping
only the compatible ones leaves equal probabilities, so the accepted tuple is
uniform over the T compatible tuples. The acceptance rate is
α = T / ∏ |C_i|, and the number of tries is geometric with mean 1 / α. For
the example above: 12 · 16 / 144 ≈ 1.33 tries per sample. Narrow,
heavily overlapping ranges make α small, which is why the design reports
dropped samples instead of hiding them.

**Learn more**

- [Rejection sampling](https://en.wikipedia.org/wiki/Rejection_sampling)
- [Bias of an estimator](https://en.wikipedia.org/wiki/Bias_of_an_estimator)
  and [Mean squared error](https://en.wikipedia.org/wiki/Mean_squared_error):
  the b² + SE² decomposition.
- [Geometric distribution](https://en.wikipedia.org/wiki/Geometric_distribution):
  the retry count.
- Art B. Owen, *Monte Carlo Theory, Methods and Examples* (free online
  draft): acceptance–rejection and estimator error.
- Christian Robert and George Casella, *Monte Carlo Statistical Methods*
  (2nd ed., Springer, 2004), ch. 2: accept–reject methods.

### Phase 5 — Docs and release

No new math. Item 5a should link here rather than repeat it.

---

## Work Items

### Phase 0 — Baseline and oracles

- [ ] **0a.** Run `make perf-native` and keep the equity workloads'
  numbers (`perf/src/catalog_equity.rs:43` `hu_preflop_request`, `:61`
  `three_way_request`) as the "before" figures.
- [ ] **0b.** Add perf workloads for hand-vs-range on the flop and on the
  river, and a six-seat preflop request (`perf/src/catalog_equity.rs`).
  `make perf-check` green.
- [ ] **0c.** Golden fixtures: brute-force exact tallies, computed once with
  `exact_enumerate` and a huge `exact_threshold`, for three preflop requests
  (heads-up, 3-way, 6-way). Store them as integer constants in the engine's
  test module. Mark the test that recomputes them `#[ignore]` (slow), and run
  it once by hand.

### Phase 1 — Inner loop

- [ ] **1a.** `Five::hand_rank_value_ignoring_suits` (`five.rs`, beside `:244`)
  + unit and doc tests.
- [ ] **1b.** `Seven::rank_value_ignoring_suits` and
  `Seven::hand_rank_value_beating` (`seven.rs`, beside `:179`) + tests:
  `hand_rank_value_beating__returns_true_value_when_nothing_beats_bar`,
  `hand_rank_value_beating__stops_below_bar`.
- [ ] **1c.** Switch `exact_enumerate` and `sample_once` to `HandRankValue`;
  change `tally_from_ranks` to lowest-non-zero wins. Test
  `tally_from_values__agrees_with_hand_rank_order`.
- [ ] **1d.** `make perf-native`; record the delta. Every existing
  `analysis::equity` test stays green with unchanged numbers.

### Phase 2 — Enumerated exact with range seats

- [ ] **2a.** `EquityOptions::exact_work_threshold` (`spec.rs`) with a
  provisional default; doc test.
- [ ] **2b.** `EquityEngine::work_estimate` + `n_choose_k`-style saturation
  test.
- [ ] **2c.** `exact_enumerate_ranges` with the hero cache and heads-up early
  exit. Tests: `compute__hand_vs_range_river_is_exact`,
  `compute__hand_vs_range_flop_matches_hand_rolled_loop`,
  `compute__range_exact_counts_identical_serial_or_parallel`.
- [ ] **2d.** Measure the slowest qualifying request; set the default for
  `exact_work_threshold` from Design 1's rule; record the table in the field's
  rustdoc, the way `spec.rs:66-86` does for `max_samples`.

### Phase 3 — Isomorphic exact preflop

- [ ] **3a.** `RankPattern` walker + `weight`. Tests:
  `rank_patterns__count_is_6188`,
  `rank_patterns__weights_sum_to_remaining_choose_5` (for several dead-card
  sets).
- [ ] **3b.** Flush assignments + weights. Test
  `flush_assignments__plus_rainbow_partition_every_board` (sum check, and a
  brute-force check on a reduced deck).
- [ ] **3c.** Weighted `Tally::add`; the scoring loop.
- [ ] **3d.** Route in `compute`. Tests: `compute__preflop_is_exact`
  (`Method::Exact`), `isomorphic__matches_brute_force_golden` (Phase 0c),
  `isomorphic__counts_identical_serial_or_parallel`,
  `compute__exact_threshold_zero_still_samples` (an all-`Exact` preflop
  request and a range-seat request with `exact_threshold: 0` both return
  `Method::MonteCarlo`). The existing `compute__aa_vs_kk_preflop_mc`,
  `compute__small_seeds_draw_different_samples` and
  `compute__seed_is_deterministic` (`engine.rs:542`, `:574`, `:584`) pass
  unchanged.
- [ ] **3e.** Cross-check against the chart, `#[cfg(feature = "hup-charts")]`:
  for a sample of heads-up matchups, the isomorphic tally equals the
  `hups.bin` win/lose/draw tally exactly. A full sweep of every chart entry
  runs as an `#[ignore]` test.
- [ ] **3f.** `make check-wasm` green (the path is serial on `wasm32`).

### Phase 4 — Range-sampling bias

- [ ] **4a.** `#[ignore]` measurement test over 200 seeds on three
  overlapping cases; print bias in standard errors.
- [ ] **4b.** Record the numbers. Either close with the evidence, or open the
  DEFECT and implement joint rejection, with
  `monte_carlo__range_seats_are_unbiased_against_exact` as the regression
  test.

### Phase 5 — Docs and release

- [ ] **5a.** Module docs in `src/analysis/equity/mod.rs`: the three-way
  decision, in a few lines; the depth goes in this EPIC.
- [ ] **5b.** `make perf-native`; write the after-numbers to
  `docs/perf/results/`.
- [ ] **5c.** Version bump and CHANGELOG entry (see Compatibility).

---

## Test Plan

- `rank_patterns__count_is_6188` — the walker visits every multiset once.
- `rank_patterns__weights_sum_to_remaining_choose_5` — patterns cover every
  board exactly once (Scope rule 2).
- `flush_assignments__plus_rainbow_partition_every_board` — no board counted
  twice or missed.
- `isomorphic__matches_brute_force_golden` — Scope rule 3.
- `isomorphic_hu__matches_hup_chart` — the new path reproduces the existing
  exact chart.
- `compute__hand_vs_range_*` — range seats go exact when the work allows.
- `*_identical_serial_or_parallel` — Scope rule 6, for each new path.
- `tally_from_values__agrees_with_hand_rank_order` — the switch from
  `HandRank` to `HandRankValue` changes no answer.
- `monte_carlo__range_seats_are_unbiased_against_exact` — only if Phase 4
  finds a defect.

## Key Files

| File | Role |
|---|---|
| `src/analysis/equity/engine.rs` | decision, enumerated exact with ranges, tally weights |
| `src/analysis/equity/isomorphic.rs` | new: rank patterns, flush assignments |
| `src/analysis/equity/spec.rs` | `exact_work_threshold` |
| `src/arrays/five.rs`, `src/arrays/seven.rs` | rank-only and early-exit evaluation |
| `perf/src/catalog_equity.rs` | new workloads |

## Reuse (do NOT recreate)

- `EquityEngine::n_choose_k` — `src/analysis/equity/engine.rs:353`.
- `Tally::combine` and the rayon reduce pattern — `engine.rs:194-216`.
- `Five::or_rank_bits`, `Five::unique_rank`, `Five::not_unique` —
  `src/arrays/five.rs:183`, `:189`, `:169`.
- `FLUSHES` — `src/lookups/flushes.rs:5`.
- `Seven::FIVE_CARD_PERMUTATIONS` — `src/arrays/seven.rs:28`.
- `Cards::deck_minus` — used at `engine.rs:127`.
- The `perf/` harness — `perf/src/catalog_equity.rs`.

## Compatibility

- **Preserves:** `EquityRequest`, `PlayerSpec`, `EquityReport`, `Method`.
  Requests that were exact stay exact with the same integers.
- **Changes behaviour (backward compatible):** some requests that returned
  `Method::MonteCarlo` now return `Method::Exact`, with `samples` equal to the
  number of compatible assignments. Downstream code that branches on `method`
  should be checked (`pkodds`, `pkarena0-web`, `pkgto-web`).
- **Adds** the `exact_work_threshold` field. A full struct literal of
  `EquityOptions` (without `..Default::default()`) stops compiling. That is a
  break, so this ships as **0.18.0**. Run the release audit before tagging.

## Dependencies

- **Blocks:** dropping `hup-charts` from WASM builds (a follow-up, not
  numbered yet).
- **Built on:** EPIC-41 (the equity engine), the `perf/` harness.
- **Related:** EPIC-120 (suit-specific ranges become `Vec<Two>` candidates,
  so they ride Phase 2 for free); EPIC-89 (bare-metal kernel — the
  isomorphic path needs no `std` beyond `Vec`).

## Verification

```bash
make lint
cargo test --features full --lib -- analysis::equity
cargo test --features full --lib -- arrays::seven arrays::five
cargo test --features full --lib -- analysis::equity --ignored   # golden + chart sweep, slow
cargo test --features full --doc -- equity
make check-purity
make check-wasm
make perf-check && make perf-native
```

Exit criteria:

1. Any request with every seat exact, an empty board and
   `exact_threshold > 0` returns `Method::Exact`, and matches the brute-force
   golden tallies exactly. With `exact_threshold: 0` it still returns
   `Method::MonteCarlo`.
2. Heads-up preflop isomorphic tallies equal the `hups.bin` tallies.
3. Hand-vs-range on flop, turn and river returns `Method::Exact` under the
   default options.
4. Phase 4 has a recorded verdict: closed with numbers, or a DEFECT with a
   regression test.
5. No existing equity test changes its expected numbers.
