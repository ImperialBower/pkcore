# pkcore + cucumber-rs — experiment findings (2026-10-02)

## Verdict
Works well. cucumber-rs 0.23 drives pkcore's public API with no adapters: 7 features, 101 scenarios, 385 steps, all green on default features in ~15 s (debug). The patch is `pkcore-cucumber.patch` (branch `experiment/cucumber`, version 0.15.4 with changelog entry).

## Layout
- `tests/features/*.feature` — cards, hand_ranking, ranges, equity, betting, showdown, omaha
- `tests/cucumber/main.rs` — `harness = false` test target, `futures::executor::block_on`, `fail_on_skipped()` so an unbound sentence fails the run
- `tests/cucumber/world.rs` — one `PokerWorld`, a slice of state per feature area
- `tests/cucumber/steps/*.rs` — one module per area
- Run: `cargo test --test cucumber`, filter with `-- --tags @equity` or `-- --name "side pot"`

## Does it have teeth?
Three hand-made mutants, each caught:
1. Out-of-turn guard removed from `act_fold` → 1 scenario fails
2. DEFECT_017 regression (Omaha plays any five of nine) → 3 fail
3. `exact_threshold` collapsed to 10 → 2 fail

## Findings (pinned as `@finding` scenarios)
1. **`Card::from_str` ignores trailing characters.** `"AHX"` → A♥, `"Kh Qd"` → K♥. A whole hand passed where one card is expected reads as its first card, silently.
2. **Uncalled refunds look like won pots, heads-up.** Deep shoves 1,000 into a 200 stack and loses: `Winnings` holds an 800 `PotWin` credited to Deep's losing pair. Being the largest award, it is what `Winnings::first()` returns.
3. **Heads-up and multiway report refunds differently.** Multiway, Rich's 8,000 side pot and 1,000 refund arrive as one 9,000 award. The chip totals are right on both paths; only the reporting differs.

## Not pinned, worth a look
4. **`Combo::from_str` prints to stdout** on bad input (`println!("Unable to process {s}")`, `src/analysis/gto/combo.rs:3691`, marked `TODO: Add logging`). That is I/O in the kernel; `log::warn!` is already a dependency.
5. **Ranges are always sampled.** One `PlayerSpec::Range` seat forces Monte Carlo even when exact enumeration is cheap (6 × 990 for KK on a flop). This is documented behaviour; the suite records it as a scenario, not a finding.
6. **`Ax` is valid range syntax** (192 combos). I expected an error; the suite now documents it.

## Notes for next time
- The sandbox has no rustup egress, so `rust-toolchain.toml` (1.94.1) cannot install. Stable 1.97 with `RUSTUP_TOOLCHAIN=stable` builds and runs everything.
- `allow-expect-in-tests` does not reach a harness-less binary, so `main.rs` carries a scoped `#![allow(clippy::expect_used, clippy::needless_pass_by_value)]` with the reason. Clippy pedantic is clean otherwise.
- Gherkin `Rule:` + `Background:` inside a rule both work in cucumber-rs.
- Candidate follow-ups: a `kuhn.feature` over the CFR checks, a TDA-rules feature mirroring `tda_conformance.rs`, and `.feature` files as Bower book content (prose-shaped specs that execute).
