# pkcore Repository Audit — Claude Fable 5

_Date:_ 2026-07-03
_Repo:_ `pkcore` v0.1.8, branch `fabio` (HEAD `0b063c8`, identical to `origin/main`)
_Model:_ Claude Fable 5 (xhigh effort)
_Audit basis:_ Full health-check run, five parallel deep-read passes, a mechanical
domain-kernel purity checker, and — for the variant engine — **empirical
verification via a probe binary compiled against the crate**. Behavioral claims
in Part II were confirmed by execution, not just by reading.

---

## Preamble: What This Audit Adds

Three prior audits exist in this directory (`AUDIT_GPT-5.4.md`,
`AUDIT_Gemini_3.1.md`, `AUDIT_Claude_Code_max.md`), all conducted at v0.0.40 on
2026-04-13. Since then: **193 commits**, five playable variants, the equity
engine, player stats, the exploitative decider, profile training, pokerbench,
and publication to crates.io (~2,193 downloads). This audit adds five things the
prior three could not:

1. **A prior-findings scorecard** — every recommendation from the v0.0.40 audits
   re-verified at current HEAD: fixed, partially fixed, or unchanged.
2. **Confirmed behavioral bugs in the variant engine** (EPIC-29–33), verified by
   compiling and running probe code: the PLO pot cap is wrong in both
   directions, stud completion is impossible, and Razz bring-in treats the ace
   as high. No prior audit saw this code; none of it runs in CI.
3. **A domain-kernel assessment** — pkcore is positioned as the foundational
   library of a domain kernel. This audit evaluates it against the kernel
   invariants (purity, delivery-agnosticism, format-crate leakage,
   pure-by-default, hidden-information projection, narrow boundary) with a
   hard-vs-cosmetic classification for every leak.
4. **A crates.io published-artifact trace** — what actually works when a
   stranger runs `cargo add pkcore`, traced end to end (answer: core evaluation
   yes, HUP odds yes, BCM analysis panics, one static silently degrades to
   empty).
5. **An enforcement-mechanism pairing for every recommendation.** The clearest
   meta-finding of this audit: since v0.0.40, *findings that could become a
   lint got fixed* (the `unwrap` cleanup, driven by
   `#![warn(clippy::unwrap_used)]`) while *findings that required a scheduled
   decision stayed open* (the `Cards` operators, the `Pile` split, `PKError`) —
   despite three audits agreeing on them. Recommendations below therefore come
   with the gate that keeps them fixed.

**A note on tone.** The prior audits spent findings on narrative and profanity
in rustdoc. Per the author, the banter is intentional — this project began as a
book, and the voice is part of the artifact. This audit does not re-litigate
tone. It flags only the *mechanical* consequences of docs: statements that
contradict behavior (e.g. a `# Errors` section naming an error variant that
does not exist, over a body that panics), and packaging effects (internal docs
shipping in the published crate).

---

## Executive Summary

`pkcore` at 0.1.8 is a serious, unusually well-tested poker platform core whose
**NLHE spine is genuinely hardened**: a runtime chip-conservation audit after
every hand, stratified side pots with real regression suites, short-blind rules
fixed and locked by tests, turn-order enforcement in the engine itself, and
~9,800 tests passing with zero failures and zero lib-level pedantic warnings.
The bot/analysis layer added since 0.0.40 is the best code in the repo —
**zero panics in library-code bodies across all sixteen new files audited.**

The four structural risks, in order:

| # | Risk | Severity | Verified how |
|---|------|----------|--------------|
| 1 | Variant engine rule bugs: PLO pot cap wrong both ways, no stud completion, Razz ace-high bring-in, stud action-order code dead — and none of it CI-exercised | **High** | Probe binary, executed |
| 2 | Published-crate panic boundary: prelude-exported BCM APIs hard-panic on every crates.io install; `UNIQUE_HANDS` silently degrades to empty | **High** | `cargo package --list` + trace |
| 3 | Kernel purity currently *unreachable*: `rusqlite`/`zstd`/`termion`/`dotenvy` non-optional, and `pkstate` transitively pins `serde_yaml_bw` into even a `--no-default-features` build | **Medium** (strategic) | `cargo tree -e no-dev` |
| 4 | Semver exposure: zero `#[non_exhaustive]` on 50-variant `PKError` and 57-variant serialized `TableAction`, six downstream consumers, `cargo-semver-checks` commented out in CI | **Medium** | grep + CI read |

None of these is an emergency for the sim/demo path that runs today. Risk 1 is
an emergency for EPIC-34 (variant web selection) — shipping PLO to the web app
in its current state ships a game that rejects the most common legal action in
pot-limit poker.

### Verified health signals

| Check | Result |
|-------|--------|
| `cargo test` (default features) | **9,117 lib + integration, 0 failed** (58 ignored) |
| `cargo test --doc` | **665 passed, 0 failed** (10 ignored) — up from 537 |
| `cargo test --features bot-training,pokerbench --lib` | 9,207 passed, 0 failed |
| `cargo clippy -- -W clippy::pedantic` (lib) | **0 warnings** |
| `cargo clippy --all-targets -- -W clippy::pedantic` | 2,177 warnings, all in tests/examples/benches (permitted by house rules; the `f64` strict-comparison and cast-truncation clusters are worth a skim) |
| `cargo deny check advisories` | ok |
| `cargo check --no-default-features` | ok |
| `cargo test --no-default-features` | **fails** — `examples/calc.rs` uses feature-gated `DealEval` with no `required-features` entry (see II.11) |
| `cargo check --target wasm32-unknown-unknown --lib` | ok (3 unused-import warnings) — not gated in CI |
| Domain-kernel purity checker | 6 hard, 30 warn (triaged in Part III) |

---

## Part I — Prior-Findings Scorecard

Every substantive finding from the three v0.0.40 audits, re-verified at HEAD.

| Prior finding | Status at 0.1.8 | Evidence |
|---|---|---|
| 1. Panicking `BitAnd`/`BitOr`/`BitXor` (+`Assign`) on `Cards` — the unanimous P0 | **UNCHANGED** | All six still bare `todo!()`: `src/cards.rs:579-617`; plus `Pile::clean`/`the_nuts` at `:883`, `:905`. `Bard`'s equivalents are properly implemented (`src/bard.rs:361-401`), so the fix is a transcription job, not a design problem |
| 2. Panic-capable statics `NAMER`, `BC_RANK_HASHMAP` | **UNCHANGED** — now with `#[allow(clippy::unwrap_used)]` suppressions, which hides them from the lint gate that fixed everything else | `src/util/name.rs:5-6`; `src/analysis/store/bcm/binary_card_map.rs:26-41`. One improvement: `PKCORE_75BCM_PATH` env override (`:152-154`) |
| 3. `Pile` trait over-specification | **UNCHANGED in substance** | 72 in-code `todo!`/`unimplemented!` (57 bare `todo!()` + 1 with message + 14 messaged `unimplemented!`) vs ~74 before. Trait not split (`src/lib.rs:676-847`); messages were added to the `unimplemented!` stubs |
| 4. `act_pay_out()` `todo!()` | **UNCHANGED + new contradiction** | `src/casino/table.rs:551-556`: doc now claims it returns `PKError::NotImplemented` — a variant with zero occurrences in `lib.rs` — over a body that panics |
| 5. Dual engine | **PARTIALLY FIXED** | Renamed `Table` → `TableCelled`; its rustdoc (`table.rs:124-139`) designates `TableNoCell` the successor and names the two remaining dependents (Pluribus `TryFrom`, `interactive_play`). No `#[deprecated]`; both fully public; `casino/mod.rs` still steers nobody. Decisively: **only `TableNoCell` got the variant engine** (Part II) — `TableCelled` is now two major feature generations behind |
| 6. `PKError` lossiness | **GREW; substance unchanged** | 50 variants (was 45), `src/lib.rs:425-493`. Both lossy `From`s intact: `io::Error` → `DBConnectionError` (`lib.rs:567-572`), `rusqlite::Error` → `DBConnectionError` (`:560-565`). `SqlError` still displays as bare "SQL Error" (`:539`). No thiserror, no `BcmLoadError` |
| 7. Narrative rustdoc / missing module docs | **UNCHANGED** (tone out of scope per preamble; the missing `//!` docs are in scope) | `src/casino/mod.rs`, `src/analysis/mod.rs`, `src/analysis/store/mod.rs` still have no module docs — full list in Part VI.3 |
| 8. README staleness, `Claude.md` exclude typo | **UNCHANGED** | README.md:15 still "Currently only supports hold'em" with five variants shipped; `carg test` typo at :38; `Cargo.toml:11` still excludes `Claude.md` (case-sensitive miss — see Part VI for what that ships) |
| 9. `unwrap()` in library code | **SUBSTANTIALLY IMPROVED** | `#![warn(clippy::unwrap_used, clippy::expect_used)]` at `src/lib.rs:1` (the comment credits the Cloudflare 2025 outage). Of 1,689 raw grep hits, only **12 genuine library-code unwraps** remain across 7 files, all behind exactly 6 annotated `#[allow]`s. Worst offenders: the two statics above; `impl From<&HUPResult> for Masked` panicking on malformed data (`src/arrays/matchups/masked.rs:466-468`); `Sqlable::select_all` (`src/analysis/store/db/hup.rs:576-577`); `bcm_rayon_case_evals` ("Fingers crossed", `src/arrays/hole_cards/twos.rs:137-147`) |

The pattern: the one finding that came with a *mechanism* (the lint gate) got
fixed. The findings that needed a scheduled decision did not, three audits of
consensus notwithstanding. Recommendations in this audit are paired with
mechanisms accordingly.

---

## Part II — Confirmed Behavioral Bugs

These were verified by compiling and running probe code against the crate
(items 1–7), or by line-level logic tracing (items 8–11). Items 1–5 are the
EPIC-29–33 variant layer; they matter now because EPIC-34 plans to put these
variants in front of users.

### II.1 — PLO: pot-limit max raise is undersized (rejects legal pot-raises) — HIGH

`BettingStructure::max_raise` (`src/games/betting_structure.rs:154-161`)
implements `current_bet + pot + call`, which is correct **only if `pot`
includes all live wagers**. The call site passes `self.pot`:

```rust
// src/casino/table_no_cell.rs:2648
let max = self.betting.max_raise(self.pot, self.bet, seat.player.bet, stack, tier);
```

but `self.pot` excludes current-street bets — they live in `player.bet` until
`bring_it_in()` sweeps them (documented at `effective_pot`,
`table_no_cell.rs:1921-1948`). Preflop, the blinds are not yet in `self.pot`.

**Probe result** (3-handed PLO, 50/100): the standard pot-open to 350 —
the most common single action in pot-limit poker — is rejected:

```
PLO utg pot-raise to 350 (legal in real PLO): Err(ExceedsBettingCap)
PLO utg raise to 200: Ok  |  raise to 201: Err(ExceedsBettingCap)
```

Postflop the cap is undersized by the sum of outstanding live bets. Fix
direction: pass pot-plus-live-wagers (`effective_pot()`-style accounting)
rather than `self.pot`.

### II.2 — PLO: all-in bypasses the pot cap entirely — HIGH

In `act_raise` (`table_no_cell.rs:2632-2652`), `would_be_all_in` skips both the
`cap_reached` and `max_raise` checks; `act_all_in` (`:2689`) performs no
structure check at all.

**Probe result:** `PLO utg open-shove 5000 into a 150 pot: Ok(5000)` — illegal
in real pot-limit. Combined with II.1, PLO as implemented is "call, min-ish
raise, or unlimited shove" — not pot-limit.

### II.3 — Stud: completion is impossible; the betting ladder is shifted by the bring-in — MEDIUM

There is no completion concept. With bring-in 5 / small bet 20, `min_raise()`
returns the tier increment (20), so completing to 20 fails
(`InsufficientIncrement`) while raise-to-25 succeeds. The engine's ladder is
5→25→45→65 where real stud plays 5→20→40→60. FLHE is unaffected only because
its BB equals the small bet.

### II.4 — Razz: bring-in selection treats the ace as high — MEDIUM

`act_bring_in` (`table_no_cell.rs:2327-2343`) correctly dispatches Razz to
*highest* upcard, but `third_street_extreme_upcard_seat` compares
`card.get_rank()` where `Rank::ACE = 14` (`src/rank.rs:14`). In Razz aces are
low; a King must bring in over an Ace.

**Probe result** (rigged upcards K♥ vs A♠): bring-in assigned to the Ace seat.
The same ace-high assumption sits in `visible_strength`
(`table_no_cell.rs:34-58`), which feeds the `LowRazz` inversion (`:1840`) — an
A-2 board would act last instead of first on later streets. (Suit tiebreak is
correct: clubs<diamonds<hearts<spades, `src/suit.rs:9-13`.)

### II.5 — Stud/Razz: the action-order machinery is dead code — MEDIUM

`first_to_act_this_street` (`table_no_cell.rs:1746`) and
`best_visible_hand_seat` (`:1808`) — the EPIC-32/33 "action by best visible
hand" implementation — have **zero callers** in src/, tests/, or examples/. The
live path is `PokerSession::next_actor` → `next_to_act` (`:1718`) →
`determine_utg()`, i.e. position-based. Every actually-driven stud/razz hand
starts action left of the button on every street — self-consistent, but wrong
per the rules, and it means the per-street upcard logic has never executed.

### II.6 — Stud: antes are absorbed into the street bet — LOW

`act_antes` posts via `act_forced_bet`, leaving the ante in `player.bet`; the
bring-in then only adds the difference and every caller gets ante credit toward
calls. Standard rules treat antes as dead money. Chip-conserving, but
systematically undersizes stud pots. (Probe: bring-in seat's street bet was 5
total = ante 2 + 3, rather than ante 2 + bring-in 5.)

### II.7 — Dealing methods have no phase guards — MEDIUM

Player *actions* are well-guarded (`TableActionOutOfOrder` on out-of-turn; a
rejected raise provably leaves state intact — regression-tested at
`table_no_cell.rs:~4192`). Dealing is not:

- `deal_flop` before blinds: Ok. `deal_flop` twice: 6-card board, no error;
  `deal_river` after that: 7-card board. `determine_betting_phase`'s
  `_ => Showdown` arm silently absorbs the corruption.
- `act_forced_bets` called twice fails only *accidentally*, with a misleading
  `InsufficientChips`.
- `TableNoCell::act()` (`:2159`) still drives streets from `board.len()`
  (`:2141-2149`) — on a stud table (board always empty) it would post antes and
  then deal a flop. Not used by `PokerSession` (which dispatches on family,
  `session.rs:334-342`), but it is a live public footgun.

Street sequencing today rests entirely on `PokerSession` caller discipline —
the `phase` field is written by dealing methods but almost never checked.

### II.8 — ExploitTrainer: the documented early-termination is unreachable — LOW (wastes compute)

Sigma decays via `(sigma * 0.90).max(self.config.sigma_tol)`
(`src/bot/training/trainer.rs:244`) while the exit check is strict:
`if sigma < self.config.sigma_tol { break; }` (`:207`). Sigma clamps *at* the
tolerance and can never pass below it, so the documented convergence exit
(`:41-42`) never fires. A fully converged run still burns
`max_generations × λ × replicates × hands_per_eval` (default ≈ 3M simulated
hands). One-character fix on either comparison.

### II.9 — ExploitTrainer: training is irreproducible despite its fixed seed — MEDIUM

Mutations use a hardcoded `SmallRng::seed_from_u64(42)` (`trainer.rs:203`, not
configurable), but the fitness evaluator builds
`SimTable::new_with_registry(...)` **without** `.with_seed(...)`
(`src/bot/training/evaluator.rs:92`), so candidate *scores* ride the
thread-local RNG (`sim.rs:502-503`). Identical `train()` calls produce
different `best_config`s. Compounding: the (1+λ)-ES never re-evaluates the
retained parent (`trainer.rs:196, 234-237`), so a noise-lucky parent score can
stall progress for many generations; and `run_session` maps any sim error to
`0.0` fitness (`evaluator.rs:94`), silently converting engine failures into
"neutral" candidates. The seeded infrastructure exists — the evaluator just
doesn't use it.

### II.10 — Player-stats store: one truncated file bricks the whole directory — MEDIUM

`YamlPlayerStatsStore::save` uses bare, non-atomic `fs::write`
(`src/analysis/player_stats_store.rs:179`); `load_all` fails wholesale on the
first corrupt file (`:170`), mapping the error to the payload-free
`PKError::InvalidIO`. Failure scenario: process killed during flush-on-`Drop`
mid-write → truncated YAML → next session's `with_store()` fails **for every
player**, with no hint which of N files is bad. Fix: temp-file + `fs::rename`,
and skip-and-log (or collect) semantics in `load_all`. No test covers the
corrupt-file path.

### II.11 — `examples/calc.rs` breaks `cargo test --no-default-features` — LOW

`src/play/stages/mod.rs:1` gates `deal_eval` behind the `equity` feature;
`examples/calc.rs:124` uses `DealEval` with no
`required-features = ["equity"]` entry (the other 14 examples all have theirs).
One manifest line — but it proves no CI job runs the test suite in
no-default-features mode, which is exactly the mode the kernel work (Part III)
will make load-bearing.

### CI-visibility footnote to all of the above

The only gameplay-level variant tests are the four
`tests/replay_consistency.rs` round-trips (FLHE/PLO/stud/razz) — **all
`#[ignore]`d**, and CI passes `--include-ignored` only to `bot_marathon`,
which is NLHE-only. `RaiseCapReached` and `ExceedsBettingCap` appear in zero
tests. The `TableNoCell` unit-test module (66 tests) contains no
FLHE/PLO/stud/razz tests. The variant layer's correctness currently rests on
constructors and doctests — which is how II.1–II.6 survived to release.

---

## Part III — Domain-Kernel Assessment (Mode A)

pkcore is positioned as the foundational library of a **domain kernel**: pure,
delivery-agnostic, single-domain, narrow-boundaried. Assessed against each
invariant, with every mechanical-checker finding verified by hand.

### Verdict summary

| Invariant | Verdict |
|---|---|
| Pure (no I/O of its own) | **FAIL** — but concentrated in identifiable adapters |
| No format crate in public API | **FAIL** — 4 error surfaces leak; 1 in-repo counter-example to copy |
| Pure by default | **FAIL twice** — defaults are maximal, *and opting out doesn't produce a kernel* |
| Delivery-agnostic | **FAIL** — terminal UI inside the library; gRPC/HTTP/CLI clean |
| Clock/randomness injectable | **MOSTLY PASS** — seeded paths exist everywhere that matters |
| Hidden-information projection | **PASS at the bot seam**, partial at the engine |
| Narrow boundary / transition surface | **FAIL for the flagship engine; PASS for Kuhn** |
| Single-domain | **PASS** |

### III.1 — The purity ceiling: opting out is not currently possible

`default = ["bot-profiles", "hand-histories", "player-stats",
"player-stats-persistence", "equity"]` (`Cargo.toml:22-28`) means a bare
`cargo add pkcore` compiles the full YAML/persistence stack. That is the common
finding. The deeper one, verified with `cargo tree -e no-dev
--no-default-features`: **even with every feature off**, the build still
contains:

- `rusqlite` (bundled C SQLite) + `zstd` — non-optional for all non-wasm
  targets (`Cargo.toml:88-90`)
- `termion` — non-optional on unix (`:92-93`)
- `dotenvy` — non-optional everywhere (`:68`); single call site
  (`src/analysis/store/db/hup.rs:58`)
- `serde_yaml_bw` — **arrives transitively through `pkstate`**, a non-optional
  dependency. pkcore's purity is capped upstream by its own sibling crate;
  either pkstate makes its YAML optional or the purity gate needs a documented
  allowlist entry.

Correspondingly ungated modules: `analysis::store` (SQLite/zstd),
`analysis::gto` (filesystem I/O), `util::terminal`, `analysis::nubibus`
(`src/analysis/mod.rs:14,27`, `src/lib.rs:378`).

### III.2 — Format-crate leaks in public signatures

| Site | Classification |
|---|---|
| `HandHistory::from_yaml/to_yaml` and `HandCollection` equivalents return `Result<_, serde_yaml_bw::Error>` (`src/hand_history.rs:817, 847, 1171, 1196`) | **HARD** — naked format-crate error in public return types; no owned error type at all |
| `BotError::Yaml(serde_yaml_bw::Error)` (`src/bot/profile.rs:124`) | **HARD** (variant payload) — the `From` impl at `:154-159` is the acceptable seam; the payload should be boxed |
| `SolverError { Io(std::io::Error), Json(serde_json::Error), Binary(postcard::Error) }` (`src/analysis/gto/solver.rs:117-123`) | **HARD** — two format crates in a public enum in an ungated, default-on module. *Not caught by the mechanical checker* |
| `Sqlable` trait: every method takes `rusqlite::Connection`, returns `rusqlite::Result` (`src/analysis/store/db/sqlite.rs:76-90`) | **HARD** — a public trait defined *in terms of* the storage crate. *Not caught by the checker* |
| `PokerBenchError` — `Csv(String)`, `Json(String)` (`src/pokerbench/error.rs:12-27`) | **The counter-example.** Stringified payloads; nothing escapes. This is the in-repo template for fixing the other four |

### III.3 — I/O and hidden state inside the kernel

Verified checker findings, worst first:

- **`BC_RANK_HASHMAP`** (`src/analysis/store/bcm/binary_card_map.rs:26-41`) —
  a `LazyLock` doing env-var read + `File::open(...).unwrap()` + zstd decode on
  first deref, imported into the core arrays layer
  (`arrays/matchups/sorted_heads_up.rs:2`). Hidden I/O, panic-on-touch,
  ungated. The single worst site in the crate (see also Part VI: it panics for
  every crates.io consumer).
- **`UNIQUE_HANDS`** (`src/arrays/five/hands.rs:19-35`) — CWD-relative read of
  `generated/5card_distinct_hands.txt` with `unwrap_or_default()`: **silently
  empty** when the file is missing. The silent flavor is arguably worse than
  the panicking one.
- **`HandCollection::save`** (`src/hand_history.rs:1221-1228`) — all three
  kernel violations in one function: `SystemTime::now()`, a hardcoded
  `generated/{run}_{ts}.yaml` path opinion, and `fs::create_dir_all`.
- **GTO solver persistence** (`src/analysis/gto/solver.rs:386-523`,
  `solver_cache.rs:169,199`) — six path-taking public fns plus an on-disk cache,
  ungated. Pure siblings (`to_binary_bytes`, `to_json_string`) already exist;
  the fs wrappers belong behind a feature or in an adapter crate.
- **`hup.rs`** (`:19,58`) — SQLite plus `.env`-file loading via dotenvy inside
  the kernel.
- **Feature-gated but default-on**: `YamlPlayerStatsStore` (a properly
  trait-separated adapter — right architecture, wrong default),
  `BotProfile::to_file/from_file`, hand-history persistence.
- **Acceptable as-is**: `pokerbench` loaders (opt-in feature, off by default);
  `util::read_lines` (utility-grade, easy to gate).
- **False positives**: everything the checker flagged in `#[cfg(test)]`
  modules.

### III.4 — Delivery-agnosticism

`src/util/terminal.rs` is an interactive TUI component inside the library:
`Terminal::pause` puts stdout into **raw mode** and reads keypresses
(`:44-53`); `receive_cards` reads stdin (`:94-100`). `analysis/nubibus.rs:16`
and `casino/table.rs:27` use `termion::color` for ANSI output — each with a
hand-rolled `#[cfg(not(unix))] mod color` shim, which is the tell that the
misplacement is already known. Remaining `println!` in library paths:
`play/game.rs:384, 654` (+ a stray `println!("boop!")` at `:270`),
`casino/table.rs:668-679`, `util/mod.rs:52-57`; `bot/sim.rs:748-829` uses
`eprintln!` for stall diagnostics where `log::warn!` (already a dependency,
used correctly elsewhere) belongs.

No tonic/axum/clap in the library. gRPC/HTTP/CLI agnosticism: clean.

### III.5 — What is already kernel-grade (the good news, and it is real)

- **Hidden-information projection exists and is honest.**
  `TableSnapshot<'a>` (`src/bot/table_snapshot.rs:105`) is a genuine per-seat
  view: own hole cards, board, pot, `to_call`, `min_raise`, per-seat chip
  info — and the construction code provably never copies an opponent's hole
  cards (`:194-199`). `BotDecider::decide` takes only the snapshot
  (`decider.rs:87`): **shipped deciders cannot cheat.** Caveats: the projection
  lives in `bot/`, not on the engine — `TableNoCell`'s fields are fully `pub`,
  so any non-bot caller reads all hole cards; and a custom *harness* (as
  opposed to a decider) holds the full table. Promoting `view_for(seat)` onto
  the engine is the seam a mental-poker/privacy layer (EPIC-79) would plug
  into.
- **Randomness is injectable everywhere that matters.**
  `Cards::shuffle_in_place_with<R: Rng>` (`src/cards.rs:476`);
  `SimTable::with_seed/with_rng` (`sim.rs:184-191`);
  `BotDecider::decide_seeded` (`decider.rs:98,151`); the equity engine's
  per-sample-index `SmallRng::seed_from_u64(seed ^ i)` (`engine.rs:190,205`)
  makes Monte Carlo deterministic *under rayon* — with a test proving it.
  Gaps: `Uuid::new_v4()` in player/table constructors
  (`table_no_cell.rs:158,181`; `player.rs:29,42`; `table.rs:194`),
  `SystemTime::now()` in `sim.rs:905` and `hand_history.rs:1223`, and the
  trainer evaluator (II.9).
- **The transition surface already exists — in miniature.** `games::kuhn` has
  the exact target shape: `legal_actions() -> Vec<KuhnAction>` (`kuhn.rs:439`),
  `apply(action) -> Result<KuhnState, PKError>` (`:465`, validating against
  `legal_actions`), `payoff()` (`:510`). `TableNoCell` is the opposite: ~125
  public fns, with mutation spread across a dozen `act_*`/`deal_*` methods and
  legality checked *by trying* — visible in `sim.rs:786-829`, where the sim
  dispatches actions and handles rejections post-hoc. The engine already has
  `next_to_act` (≈ `to_act`), `TableSnapshot` (≈ `view_for`), and
  `Winnings`/showdown (≈ `outcome`). **Missing: `legal_actions(seat)` and a
  single `apply(action)`.** Those two methods would complete a WIT-mappable
  kernel boundary — and would have made the Part II bugs testable as
  table-driven rule checks instead of probe archaeology.

### III.6 — The three highest-leverage fixes, ranked

1. **Make purity reachable, then make it the default.** Feature-gate
   `rusqlite`+`zstd` behind `store` (or `hup-db`), `termion` behind `terminal`
   (the `#[cfg(not(unix))]` shims already exist — extend them); **delete
   `dotenvy`** (one call site → `std::env::var`, or better, take the value as a
   parameter); then flip `default = []` (or `["equity"]`) with a `full`
   umbrella so the 14 `required-features` examples and 8 test targets keep
   resolving. Enforcement: a CI job asserting `cargo tree
   --no-default-features -e no-dev` contains no
   rusqlite/zstd/termion/serde_yaml_bw — with a documented allowlist entry for
   the `pkstate` ceiling until pkstate makes YAML optional upstream.
2. **De-leak the four public error surfaces** (mechanical, near-zero risk):
   own-typed error for `HandHistory/HandCollection::{from,to}_yaml`; box the
   payloads of `BotError::Yaml` and `SolverError::Json/Binary`; re-shape
   `Sqlable` (or gate it into the `store` feature). Keep the existing `From`
   impls as the seam. `PokerBenchError` is the template. Enforcement:
   `clippy.toml` `disallowed-types` for the format crates in public modules.
3. **Kill the hidden-I/O statics and the path-opinionated save.**
   `BC_RANK_HASHMAP` and `UNIQUE_HANDS` become explicit fallible
   `from_reader/from_bytes` constructors owned by an adapter (this also fixes
   the crates.io panic, Part VI); `HandCollection::save` takes a path and a
   timestamp — or returns the YAML string and lets the caller write.

Strategic follow-up, after the above: promote Kuhn's `legal_actions/apply`
shape onto `TableNoCell` (Mode C of the kernel pattern — the WIT boundary).
That is the point at which pkcore stops being "a very good Rust crate" and
becomes a kernel drivable from any stack — which is the stated point of the
whole program.

---

## Part IV — The Post-0.0.40 Surface

Modules the prior audits never saw, each read end-to-end. House-rule headline:
**zero `unwrap()`/`expect()`/`panic!()` in library-code bodies across all
sixteen audited files** — the CLAUDE.md standard is actually being met in all
code written since the lint gate landed.

| Module | Verdict | Notes |
|---|---|---|
| `analysis::equity` | **SOLID** | Exact-vs-MC split with saturating `n_choose_k`; per-sample-index seeding is rayon-deterministic (tested); duplicate-card rejection tested; known-value tests incl. AA-vs-KK ≈ 0.82 and exact-enumeration sum-to-1. Nits: doc at `engine.rs:46` attributes >10-seat rejection to `NotEnoughHands` (code returns `TooManyHands`); the f64 `equity` field is last-ulp scheduling-dependent when ties occur (integer counts are exact — relevant if pkodds ever byte-compares responses); no dedicated empty-range test |
| `analysis::player_stats` | **SOLID** | All 11 derived ratios divide-by-zero-safe via `Option` (tested); `Confidence` boundaries tested at 0/49/50/199/200; sparse-seat position remapping regression-tested. Semantics quirks documented in code (fold-to-3bet slightly over-inclusive; check-raise opportunities granted optimistically). Weakest doc-test ratio of the new modules (~30% of accessors) |
| `analysis::player_stats_store` | **MINOR ISSUES** | II.10 (durability). Path traversal impossible (UUID `Display` filenames); non-UUID files skipped, tested |
| `bot::exploit` + `ExploitativeDecider` | **SOLID** | All 8 deviation rules clamp-bounded through `Percentage` (`exploit.rs:117-123`, extreme-input test present); `adjust_profile` is provably clone-and-mutate pure; sample-size gates (30/50 hands). Nit: `largest_active_opponent`'s doc example never calls the function |
| `bot::training` | **MINOR ISSUES** | II.8, II.9. Correct pieces: `decode` clamps all 16 dims and enforces `min_hands_heavy >= min_hands_light` (tested); Box-Muller guards `ln(0)`; NaN-safe candidate selection; fitness is BB/100 with outlier-capping stack sizes |
| `pokerbench` | **SOLID** | No unwraps on parse; per-variant 7-variant error type, tested per-variant, fail-fast on malformed fixtures (tested against `malformed.csv/json`); scoring known-value tests; CSV/JSON cross-format consistency test. The error type is the best in the crate (III.2) |
| `bot::profile` YAML | **MINOR ISSUES** | `Percentage` has a validating `Deserialize` — `aggression_factor: 500` fails loudly; `playbook`/`betting_structure`/`street_aggression` properly `#[serde(default)]` for backward compat; all 8 `data/bots` YAMLs load-tested plus per-variant deserialize tests. Gaps: no `deny_unknown_fields` — a typo'd optional key silently vanishes (defensible for forward-compat, but undocumented); `value_threshold: 5.0` loads silently despite a documented `[0.0, 1.0]` domain and would disable value-betting |

---

## Part V — Engine Strengths Worth Naming

The audit pattern of leading with problems undersells what is unusually good
here; these are verified, not vibes:

- **Chip conservation is a runtime invariant, not a test assertion.**
  `act_forced_bets` snapshots `hand_chip_total` before chips move
  (`table_no_cell.rs:2212`); `end_hand` re-audits after **every hand**
  (`:3680-3687`), returning `ChipAuditFailed { expected, actual }` and logging
  the event. The multiway showdown even sweeps orphaned dead money explicitly
  to preserve the invariant (`:3560-3583`). A 1,000-hand marathon exercises it
  in CI (NLHE only — the variant caveat of Part II stands).
- **The defect docs map to living regression tests.**
  `DEFECT_heads-up-side-pot.md` → four tests in `tests/split_pots.rs`
  (`:219, :285, :339, :403`) plus the `heads_up_is_symmetric` guard;
  `BUGFIX_short_blind_call_target.md` → six `short_bb`/`short_blind` tests
  across both engines. (`EPIC-DEFECT-Minraise.md` is a title-only stub, but its
  behavior is covered: `act_raise` pre-validates before mutating, with a
  no-state-corruption regression test.) This defect→doc→test discipline is
  rarer than the test count itself.
- **All-in-below-min-raise accounting is right where it counts:**
  `set_raise_increment` correctly does not treat a sub-min all-in as a full
  raise (`:2706-2716`), and action correctly returns to players facing the
  extra chips. The one deviation is permissive: a player who already acted may
  re-raise after a sub-min all-in (TDA forbids it) — no chip loss, worth a
  documented decision.
- **Heads-up blind logic** (button = SB, acts first preflop) is correct with
  dedicated tests, including the button-swap-on-elimination case.
- **`games/` is complementary, not duplicative:** the casino engine consumes
  `OmahaHigh::permutations` and `Eval::from_seven_razz` →
  `CaliforniaHandRank` for showdown. EPIC-33's claim of finishing EPIC-10
  holds for showdown evaluation (bring-in and action order are Part II).
  `games/stud.rs` is an empty placeholder file.

---

## Part VI — API Surface, Packaging, Process

### VI.1 — The crates.io functionality trace

Package verified with `cargo package --list` and a real `--no-verify` build:
210 files, 19.8 MiB / 4.9 MiB compressed.

| Capability from `cargo add pkcore` | Status |
|---|---|
| 5/7-card evaluation, equity engine | **Works** — pure Cactus-Kev lookup; the tables ship in `src/lookups/`. pkodds is safe |
| Heads-up preflop odds | **Works** — `generated/hups.bin` (15.8 MB) is git-tracked, not excluded, and `include_bytes!`-embedded (`src/analysis/store/embedded/hup_cache.rs:5`) |
| `SortedHeadsUp::wins()`, `StartingHands` case evals (BCM path) | **Panics** — `generated/bcm.zst` (422 MB) is gitignored and absent from the package; `BC_RANK_HASHMAP` does `File::open(...).unwrap()` on first deref. These types are prelude-exported with no warning; recovery (set `PKCORE_75BCM_PATH`, self-generate ~400 MB via `SevenFiveBCM::generate_bin` over C(52,7)=133M combos) is undocumented in the README |
| `Five` distinct-hands enumeration | **Silently empty** — `UNIQUE_HANDS` `unwrap_or_default()` on the missing text file |
| `cargo test` on the published tarball | Fails — `data/*` is excluded but unit tests read `data/bots/**.yaml` and `data/sample_hups.db` (affects distro/vendored builds) |

### VI.2 — Packaging and manifest bugs

- `Cargo.toml:11` excludes `"Claude.md"`; the file is `CLAUDE.md`
  (case-sensitive). Confirmed shipping in the published crate: **CLAUDE.md,
  DIARY.md, `marathon_failure.yaml`** (a bug-dump artifact),
  `generated/kuhn-repl-history`, `.env_EXAMPLE`.
- `docs/*` is excluded while the README links into `docs/` — dead links on the
  crates.io render. Same for `examples/*.rs` links.
- `description = "Prototype core poker library."` undersells a crate with six
  named downstream consumers; no `keywords`/`categories` — crates.io
  discoverability is zero.
- No `[package.metadata.docs.rs]` and zero `doc_cfg` annotations: docs.rs
  builds with default features, so `pokerbench`/`bot-training`/`debug-json`
  items are **invisible** there, and nothing renders "available on feature X"
  banners for a crate whose own manifest comment tells consumers to use
  `default-features = false`.

### VI.3 — API surface: growing, not curated

| Metric | v0.0.40 | v0.1.8 |
|---|---|---|
| prelude re-exports | ~140 | **203** + 2 glob re-exports (`prelude.rs:37,49`) |
| `pub fn` | ~1,250 | **1,401** |
| `pub struct` / `pub enum` / `pub trait` | — | 184 / 53 / 16 |
| `#[non_exhaustive]` | 0 | **0** |
| `#[doc(hidden)]` | 0 | **0** |

- Internal types flagged at 0.0.40 remain exported: `CardsCell`
  (`prelude.rs:45`), `TableLog` (`:54`), `SeatCell` (`:57`), `TableCelled`
  (`:52`); `bint::BintCell` still leaks as a public field
  (`table.rs:147`).
- `docs/public_structs_prelude_report.md` was an inventory that asked "which
  should we add?" — and ~15 of its listed items were added. Its direction was
  expansion; expansion happened. No curation pass has ever run.
- Deprecation hygiene: `cards_cell.rs:41` says `#[deprecated(since =
  "0.8.0")]` — a version that doesn't exist; `arrays/four.rs:67` is a bare
  `#[deprecated]` with no note.
- Semver exposure with six downstream consumers: `PKError` (50 variants),
  `TableAction` (**57 variants, `Serialize`/`Deserialize` — a wire format**;
  EPIC-32 already added a variant mid-0.1.x), `GameType`, `ActionType` — none
  `#[non_exhaustive]`. `cargo-semver-checks` is present in
  `.github/workflows/basic.yaml` **commented out**. The 81 `pub` fields on
  `hand_history.rs` schema structs are mitigated by `format_version` +
  `pkcore_version` + the legacy-YAML test — good; the Display impls are
  load-bearing wire format (`"6♠ 6♥"` ↔ `Two::from_str`), which deserves a
  written stability promise since pkpy notebooks parse them.
- Module docs (`//!`) missing from the entire primitives layer — the first
  thing every consumer reads: `card.rs`, `cards.rs`, `deck.rs`, `rank.rs`,
  `suit.rs`, `bard.rs`, plus `analysis/mod.rs`, `arrays/mod.rs`,
  `casino/mod.rs`, `games/mod.rs`, `play/mod.rs`, `util/mod.rs` and all
  `store/*` mod files. New code (`bot/` with its included module guide,
  `hand_history.rs`, `equity`) is well-documented — the gap is the old core.

### VI.4 — Feature-gate and target hygiene

- Combination discipline is **good**: 144 `#[cfg(feature)]` sites;
  `hand_history.rs` correctly gates all bot references; CI's
  `no-default-features` job checks six configurations including
  equity-in-isolation (with a comment noting it "silently broke once" —
  institutional memory encoded in CI, excellent). Gaps: `bot-training` and
  `debug-json` compile today but are unguarded in CI; `examples/calc.rs`
  (II.11).
- WASM: 106 `cfg(target_arch = "wasm32")` sites; `rusqlite`/`zstd`/`termion`
  correctly target-gated; prelude gates native-only re-exports; HUP lookups on
  wasm use the embedded store. `cargo check --target wasm32-unknown-unknown
  --lib` passes — but **no CI job builds wasm32**, and two production WASM
  apps depend on it. `make check-wasm` exists; CI doesn't run it.
- CI vs Makefile gap list: wasm build, `--all-features`, semver-checks,
  cargo-udeps, mutants, coverage, `debug-json`, full `cargo deny check`
  (CI runs advisories only, weekly).

### VI.5 — Process-docs decay across 0.1.x

- `CHANGELOG.md` covers 4 of the twelve 0.1.x releases (0.1.3, 0.1.6, 0.1.7,
  0.1.8). `docs/RELEASE_*.md` stops at 0.0.55; `RELEASE_AUDIT_*` covers one
  0.1.x release (0.1.4) — despite the repo having `release-notes` and
  `audit-release` skills purpose-built for both. The entries that exist are
  high quality and reason explicitly about wire compatibility.
- Stale tags `v0.1.13` / `v0.1.15` (dated 2025, from a pre-reset versioning
  lineage) still exist and sort *above* the real current version. They were
  never published to crates.io (registry lineage starts 2026-03-22 and tops out
  at 0.1.8), so this is confusion, not hazard — but deleting or renaming them
  (`legacy-v0.1.15`) would prevent a future `cargo release`-style tool from
  mis-inferring the latest version.

---

## Prioritized Action Plan

Each item paired with the mechanism that keeps it fixed — per the Part I
lesson that unenforced consensus findings survive three audits untouched.

### P0 — Before EPIC-34 ships variants to users

**P0a. Fix the PLO pot cap and the all-in bypass** (II.1, II.2).
Pass live-wager-inclusive pot to `max_raise`; route `would_be_all_in` raises
through the cap check. *Mechanism:* a table-driven betting-rules test file
(`tests/betting_rules.rs`) with the canonical scenarios: 50/100 PLO pot-open =
350; over-pot shove rejected; FLHE cap ladder; stud completion; Razz A-vs-K
bring-in. These are exactly the probe cases from this audit — they should live
in the repo, not in an audit doc.

**P0b. Fix Razz ace-low bring-in and visible-strength** (II.4) and **wire in
or delete the stud action-order machinery** (II.5). *Mechanism:* same test
file; plus un-`#[ignore]` the four variant replay round-trips in CI (a
dedicated job, like the pokerbench one).

**P0c. Decide the stud completion and ante-as-dead-money semantics** (II.3,
II.6) — these are rules decisions, then one-line-ish fixes.

### P1 — Published-crate honesty (one afternoon, real consumer impact)

Make `BC_RANK_HASHMAP` fallible (`PKError::BcmUnavailable(String)` or an
explicit `try_bc_rank_hashmap()`), fix `UNIQUE_HANDS`' silent-empty fallback,
fix the `Claude.md` exclude casing, add `keywords`/`categories` and
`[package.metadata.docs.rs] all-features = true`, and README-document what
requires self-generated data. *Mechanism:* a CI smoke job that installs the
packaged crate in a temp project and calls the headline APIs.

### P2 — Kernel step 1: make purity reachable, then default (III.6.1)

Feature-gate rusqlite/zstd/termion, delete dotenvy, flip defaults with a
`full` umbrella, fix `examples/calc.rs` required-features. Coordinate the
`pkstate` YAML-optionality fix upstream. *Mechanism:* CI purity gate on
`cargo tree --no-default-features`.

**Status (0.1.9): "reachable" half done; default-flip deferred.** Landed:
`store = ["dep:rusqlite", "dep:zstd"]` and `terminal = ["dep:termion"]`, both
**added to `default`** (so no consumer sees a change); `dotenvy` deleted
(`std::env::var`); store code re-gated to
`all(feature = "store", not(target_arch = "wasm32"))`, terminal code to
`all(unix, feature = "terminal")`; the CI purity gate + `make check-purity`
enforce that `cargo tree --no-default-features -e no-dev` is free of
rusqlite/zstd/termion/dotenvy (`serde_yaml_bw` allowlisted — the pkstate
ceiling, III.1). `--no-default-features` now yields a pure build.

**The default-flip is the breaking half — deferred to 0.2.0.** Adding the
features to `default` is *non-breaking*: every current consumer takes the
default set, so the compiled API is byte-for-byte identical. Verified against
every in-tree dependant — **none uses `default-features = false`**, so none is
affected by 0.1.9:

| Consumer | pkcore declaration | Needs `["store"]`/`["terminal"]` when defaults flip? |
|---|---|---|
| `pkarena0-web` | `features = ["bot-profiles", "hand-histories"]` | Yes if it uses `Terminal::pause`/BCM — audit at flip time |
| `pkdealer_client` | `"0.1.3"` (plain default) | Yes if it uses store/terminal APIs |
| `pkdealer_service` | `"0.1.3"` (plain default) | Yes if it uses store/terminal APIs |
| `pkdealer_agent_rules` | `features = ["bot-profiles"]` | Likely no (profile-only) — confirm at flip |
| `pkgto-web` | `"0.0.28"` (wasm) | No — wasm never had these deps |
| `pkkuhn-web` | `"0.0.39"` (wasm) | No — wasm never had these deps |
| `pkpy` | `"0.0.35"` | Yes if it exposes store/terminal APIs |
| `exgto` | `"0.0.25"` | Yes if it uses store/terminal APIs |

When flipping `default = ["equity"]` (+ a `full` umbrella): bump to 0.2.0, and
land companion PRs adding explicit `features = [...]` to any consumer above that
actually calls a store (`wins()`, `Sqlable`/`Connect`, `FiveBCM`/`SevenFiveBCM`,
`HUPResult` SQLite methods) or terminal (`Terminal::pause`, ANSI colour) API.
The wasm consumers need no change.

### P3 — Kernel step 2: de-leak the four public error surfaces (III.6.2)

`hand_history`, `BotError`, `SolverError`, `Sqlable`. Copy the
`PokerBenchError` pattern. *Mechanism:* `clippy.toml` `disallowed-types`.

**Status (0.1.9): done.** All three format-crate leaks stringified onto owned
error types, following the `PokerBenchError` template:
- `HandHistory`/`HandCollection::{from,to}_yaml` now return a new owned
  `HandHistoryError` (was `serde_yaml_bw::Error`) — the surface that previously
  had *no* owned error type at all.
- `BotError::Yaml(serde_yaml_bw::Error)` → `Yaml(String)`.
- `SolverError::{Json(serde_json::Error), Binary(postcard::Error)}` →
  `Json(String)` / `Binary(String)` (its `Io(std::io::Error)` stays — std is not
  a format-crate leak and keeps the `source()` chain).
- `Sqlable`'s `rusqlite` surface is resolved by the P2 route: the trait is now
  gated behind the `store` feature (III.6.1's "or gate it into the `store`
  feature"), so the storage crate is opt-in rather than always-public.

The existing `From` impls remain the blessed seams (each carries a local
`#[allow(clippy::disallowed_types)]`). *Mechanism landed:* `clippy.toml`
`disallowed-types` for `serde_yaml_bw::Error`, `serde_json::Error`, and
`postcard::Error` — under `-Dclippy::all` any new use in a non-`#[allow]`ed
(i.e. public) position fails the build. **Source-breaking only** for a consumer
that named a format-crate error type in a `match`/signature; none of the in-tree
consumers do (they use `?`/`unwrap`).

### P4 — Close the P0s of the last three audits, this time with gates

Implement the six `Cards` bit-operators (transcribe from `Bard`) and delete
the remaining reachable `todo!()`s or convert them to `Err`; `#[deprecated]`
on `TableCelled` (its variant-lessness has settled the dual-engine question —
see Part II) and remove it plus `CardsCell`/`SeatCell`/`TableLog` from the
prelude; fix `act_pay_out`'s contradictory doc by deleting or implementing the
method. *Mechanism:* `clippy.toml` `disallowed-macros` for
`todo!`/`unimplemented!` in non-test code — the same trick that fixed
`unwrap`.

**Status (0.1.9): non-breaking half done; the two breaking pieces deferred to
0.2.0** (same split as P2's default-flip). What landed:

- **The six `Cards` bit-operators are implemented** — not a literal transcribe
  from `Bard` (which is a `u64` bitmask), but the *set* operations its bitwise
  ops correspond to, since `Cards` is an `IndexSet<Card>`: `&` intersection,
  `|` union, `^` symmetric difference (plus the three `*Assign` forms). Doc
  examples + 10 colocated unit tests; the old six `#[should_panic]` `*__panics`
  stubs (which asserted the `todo!()`) are gone.
- **`Cards::clean` is implemented** (element-wise `Card::clean`, mirroring
  `Two::clean`); `Cards::the_nuts` is now a messaged `unimplemented!` (it needs
  board context — even `Bard::the_nuts` punts).
- **`act_pay_out`'s doc-contradicts-body defect (Part I #4) is fixed.** It named
  a `PKError::NotImplemented` that did not exist, over a panicking body. That
  variant now exists and the method returns it (`Err(PKError::NotImplemented)`)
  — recoverable, not a panic — with a doctest asserting exactly that.
  `SortedHeadsUp::hup_result_from_shift` got the same treatment.
- **Every remaining reachable `todo!()` in `src/` is eliminated** (64 bare +
  1 messaged `todo!("Doesn't apply")` the gate caught on first run). The
  structurally-undefined `Pile` stubs (`card_at`/`clean`/`swap`/`the_nuts`/`add`
  on fixed-size hands — the deferred finding #3 population) became messaged
  `unimplemented!("…")` that point at the `.cards()` workaround; the genuinely
  unfinished non-`Result` methods (`percentage`, `generate_player_loses`,
  `Shifter::shifts`, `is_seat_all_in`, `deck_the_hand_dealable`, the `Sqlable`
  bulk stubs) became messaged `unimplemented!("… not yet implemented")`. Zero
  `#[allow(clippy::disallowed_macros)]` were needed — the gate is unqualified.

*Mechanism landed:* `clippy.toml` now carries
`disallowed-macros = [{ path = "std::todo" }]`. Under CI's existing
`-Dclippy::all` (basic.yaml) and `-D warnings` (ci.yml) this makes any `todo!()`
in lib/bin code a hard error — verified: the gate fired on the one messaged
`todo!` a plain-`grep` had missed. **`unimplemented!` is deliberately *not*
gated**: the enforced convention is that no unfinished spot may be a *silent*
`todo!()` — it must be a *messaged* `unimplemented!("why + workaround")` or a
returned `PKError::NotImplemented`. This is the pragmatic substitute for the
literal proposal (one gate over both macros), which is unreachable while the
`Pile` over-specification (#3) is deferred — ~50 of the stubs are `Pile`
methods that are structurally undefined for their types, and gating
`unimplemented!` too would force the trait split.

**The two breaking pieces are deferred to 0.2.0** (bundled with the P6 semver
work): `#[deprecated]` on `TableCelled` (104 in-crate references would each need
`#[allow(deprecated)]` — noisy, and deprecation is a public-API signal that
belongs with the 0.2.0 legacy-engine sunset) and removing
`CardsCell`/`SeatCell`/`TableLog`/`TableCelled` from the prelude (a
source-breaking change). Both are the "legacy `TableCelled` sunset" and are
naturally a single 0.2.0 change; nothing about them blocks the gate that P4
came for.

*Verification:* `cargo build --lib` ok; `cargo test --lib` **9,132 passed, 0
failed**; `cargo test --doc` 0 failed; `cargo clippy -- -Dclippy::all
-Dclippy::pedantic` and `cargo clippy --features pokerbench -- -D warnings` both
clean.

### P5 — Trainer determinism and stats-store durability (II.8, II.9, II.10)

Thread `TrainingConfig.seed` into both the mutation RNG and
`SimTable::with_seed`; fix the sigma-tol comparison; temp-file + rename in
`save`; skip-and-log in `load_all`. *Mechanism:* a train-twice-compare test
and a corrupt-file test.

**Status (0.1.9): done.** All three fixes landed with their mechanism tests:

- **Trainer reproducibility (II.9).** `TrainingConfig` gained a `seed: u64`
  field (default `42`). It now seeds *both* the Gaussian mutation stream
  (`SmallRng::seed_from_u64(self.config.seed)`, was a hardcoded `42`) *and*
  every fitness session: `evaluator::evaluate` takes a `seed` and derives a
  distinct per-`(opponent, replicate)` session seed from it, which
  `run_session` feeds to `SimTable::with_seed`. The derivation is independent
  of the candidate config, so every candidate is scored on the **same hands**
  (common random numbers) — this both removes the thread-local-RNG noise that
  made `train()` irreproducible and cuts the between-candidate variance the
  optimiser sees. Test: `train_twice_with_same_seed_is_reproducible` asserts a
  byte-identical `best_config` (via `encoding::encode`) across two runs, plus
  `evaluate_is_deterministic_for_fixed_seed`.
- **Sigma early-exit (II.8).** The convergence check is now
  `if sigma <= self.config.sigma_tol` (was `<`). Because `sigma` clamps *at*
  `sigma_tol` via the `.max(sigma_tol)` floor, the strict `<` could never fire,
  so a fully-converged run burned every generation (~3M simulated hands at the
  defaults). Test: `converged_run_terminates_before_max_generations` sets
  `initial_sigma_fraction == sigma_tol` and asserts `generations_run == 0` —
  which fails under the old `<`.
- **Stats-store durability (II.10).** `YamlPlayerStatsStore::save` is now atomic
  (serialise to a `.yaml.tmp` sibling, then `fs::rename` over the target — the
  `.tmp` extension keeps it out of `load_all`'s `.yaml`-only scan). `load_all`
  now **skips-and-logs** an unreadable or malformed file (`log::warn!` with the
  path) instead of mapping the first bad file to `PKError::InvalidIO` for the
  whole directory. Tests: `load_all_skips_corrupt_yaml_file` and
  `save_leaves_no_temp_file_behind`.

Not addressed here (out of the II.8–II.10 scope, noted for later): the
(1+λ)-ES still never re-evaluates the retained parent, and `run_session` still
maps any sim error to `0.0` fitness — both called out in II.9 as *compounding*
factors. Common-random-numbers scoring makes the parent's score stable, which
defuses the "noise-lucky parent stalls progress" case the re-evaluation gap
caused; the error→`0.0` mapping is a smaller latent issue left for a follow-up.

*Verification:* `cargo test --features bot-training --lib bot__training` **16
passed**; stats-store suite green; `cargo clippy` clean (below).

### P6 — Semver posture for 0.2.0

`#[non_exhaustive]` on `PKError`, `TableAction`, `GameType`, `ActionType`
(breaking once, protective forever — do it at 0.2.0); re-enable
`cargo-semver-checks` in CI; split the lossy `From<io::Error> for PKError`
mapping; write the one-paragraph stability promise for Display-based card
encodings. *Mechanism:* semver-checks in CI is the mechanism.

### P7 — CI completes its own Makefile

Add wasm32 build job, `bot-training`/`debug-json` checks,
`cargo test --no-default-features`. *Mechanism:* it is CI.

### P8 — Kernel step 3 (strategic): the transition surface

`legal_actions(seat)` + `apply(action)` on `TableNoCell`, promoting
`TableSnapshot` toward an engine-level `view_for`. Kuhn is the in-repo proof
of shape; this is what makes the WIT/component boundary (and EPIC-79's
privacy layer) possible — and it converts betting-rules correctness from
probe archaeology into table-driven tests, which is how Part II stays fixed.

---

## Comparative Scoring

_A–F, alongside the prior audits' self-assessments._

| Category | Fable 5 | Claude max (0.0.40) | GPT-5.4 | Gemini 3.1 |
|----------|---------|---------------------|---------|------------|
| Test rigor | **A+** | A+ | A+ | A+ |
| Lint/advisory compliance | **A** | A | A | A |
| New-module code quality (post-0.0.40) | **A** | — | — | — |
| Chip/pot accounting (NLHE) | **A−** | — | — | — |
| Variant rule correctness | **D** | — | — | — |
| Panic safety | **C** | C | C | C+ |
| Published-crate integrity | **C−** | — | — | — |
| Error handling | **C+** | B− | B | B− |
| Public API curation | **C** | C+ | C+ | C |
| Domain-kernel purity | **D+** | — | — | — |
| Kernel seams in place (projection, seeding, Kuhn surface) | **B+** | — | — | — |
| Process docs (changelog/releases) | **C** | — | — | — |
| WASM handling | **B+** (unC-I'd) | B+ | — | — |
| Documentation coverage (module docs) | **B−** | B | B | B+ |

The grade spread is the story: the newest code (equity, pokerbench, exploit)
earns A-grades against the house rules, the NLHE engine is hardened, and the
variant layer — structurally elegant, behaviorally unfinished, CI-invisible —
earns the only D. Quality here tracks *enforcement*, not effort.

---

## Conclusion

pkcore has crossed a threshold since v0.0.40 that none of the prior audits
could see coming: the code written under the lint gate is house-rule-clean,
the defect→doc→regression-test loop is real discipline, and the three seams a
domain kernel needs most — hidden-information projection, injectable
randomness, a transition-function surface — all already exist in the codebase.
They're just not yet *load-bearing*: the projection lives beside the engine
rather than on it, the seeds aren't threaded through the trainer, and the
transition surface only wraps a three-card toy game.

The immediate work is not architectural: it is fixing four confirmed variant
rule bugs before EPIC-34 puts them in front of users, and drawing an honest
panic boundary around the published crate. The strategic work is the kernel
sequence (P2 → P3 → P8), which this repo is closer to than the purity
checker's failure count suggests — most of the leakage is adapters that grew
inside the crate and want a feature gate, not a rewrite. The one external
dependency is `pkstate`, which caps pkcore's purity from upstream until its
YAML becomes optional.

And the meta-lesson this audit inherits from watching three predecessors age:
**consensus without a gate changes nothing; a lint changed everything.** Every
recommendation above names its gate. The unwrap cleanup proved the pattern
works here — the rest of the crate is waiting for the same treatment.
