# Kernel Purity Audit — pkcore

| | |
|---|---|
| Subject | `pkcore` 0.14.0 — the poker engine (`casino::table::Table`), its drivers (`PokerSession`, `Dealer`), the bot stack, and `analysis::*` |
| Commit | `cda90569` (2026-09-13), branch `main`, tree clean |
| Date | 2026-09-15 |
| Method | `/domain-kernel` Mode A (Assess). Invariants 1–8 from `references/invariants.md`; every mechanical finding re-read against the source |
| What was run | `python3 ~/.claude/skills/domain-kernel/scripts/check_purity.py .` — **94 hard, 0 warn, exit 1**. It is a **grep**, not a compiler: it does not see `#[cfg]`, does not see `pub(crate)`, and does not follow an aliased import. No build, no test and no `cargo tree` was run for this audit |
| Re-run | 2026-09-15 at `d6df2d00`: checker **105 hard, 0 warn, exit 1** — the 11 extra are the checker's new alias detection (`SmallRng`, `WriterBuilder`, `Reader`, `Connection`, `rng`), not source changes; no `src/` or `Cargo.toml` change since `cda90569`. `make check-purity` run (passes, warns) and `cargo tree -i` traced every survivor. Findings in §1a (random IDs) and §3 (dependency tree) |
| Supersedes | the domain-kernel sections of [`docs/audits/AUDIT_Fable_5.md`](audits/AUDIT_Fable_5.md) (2026-07-03, v0.1.8, graded **D+**). Parts I, II and IV of that audit still stand |

---

## Verdict

**pkcore is a working poker library that grew a kernel rather than being built as
one, and it is now about two-thirds of the way there.** Five of the eight
invariants either pass or fail only cosmetically. Since v0.1.8 the crate has
acquired everything a kernel is hard to retrofit with: a feature-free transition
surface (`legal_actions` / `apply_action`), a redacting projection keyed on
identity (`PokerSession::view`), a versioned wire form separate from the engine
type (`TableState`), injected randomness on every path that matters, and a CI
purity gate. Those are the expensive parts and they are done.

What is left is bookkeeping and one genuine design debt. The bookkeeping: the
default feature set still turns on a YAML format crate and a thread pool, four
banned crates are non-optional so they survive `--no-default-features`, and
roughly a dozen filesystem calls sit in always-compiled modules that are honestly
adapters and want a feature gate, not a rewrite. The design debt: **pkcore has
never said who owns its state**, and a downstream service has quietly answered
for it — `pkdealer_service` writes chip stacks directly onto seats and
re-implements hole-card redaction rather than calling the projection pkcore
ships. That is invariant 8 failing in the field, not in theory.

**The single highest-leverage change is to flip `default` to a pure set and widen
`make check-purity` to the skill's banned list.** It is near-zero risk — CI
already runs `--no-default-features`, and every example and test that needs a
feature already declares `required-features` — and it is what makes `cargo add
pkcore` yield a kernel instead of a kernel plus a YAML parser plus rayon.

The honest cost of becoming a kernel end-to-end, including the boundary work in
§7 and §8 and one breaking release, is on the order of **three to four weeks**.
The cost of fixing everything mechanical in this report is about **three days**.

### The eight invariants

| # | Invariant | Verdict | Why |
|---|---|---|---|
| 1 | Pure — no I/O of its own | ~~**Fail**~~ **Pass for the kernel build, 0.15.0** (default build: by choice, see fix 8) | ~12 real filesystem / clock / ambient-RNG sites in always-compiled code, **plus OS-random IDs minted by `Table::from_seats` and `Player::new`** (re-run); the rest is behind `store`, `pokerbench`, `generators`. **0.15.0:** IDs injectable; the file wrappers moved behind `persistence`, `csv`, `json`; every ambient site has a seeded or id-taking twin, core code uses only the twins, the ambient forms and `SimTable`'s clock are behind `entropy` (on by default), and `make check-purity` fails on `getrandom` in `--no-default-features`. Still open: `HandCollection::save`'s clock (behind `persistence`, an adapter anyway) and `util::csv`'s CWD-relative read (behind `csv`; fix 7) |
| 2 | No format/transport crate in the public API | **Pass** | Every error type stringifies its format cause (`BotError::Yaml(String)`, `SolverError::Json(String)`, `PokerBenchError::Csv(String)`). Only `rusqlite` remains, behind the non-default `store` |
| 3 | Pure by default | ~~**Fail**~~ **Pass, 0.15.0** | Was: `default` pulled `serde_yaml_bw` and `rayon`; `csv` and `serde_json` were non-optional. Now `default = ["equity", "player-stats", "hup-charts"]`, `csv`/`serde_json` are optional, and `make check-purity` gates the default tree too. Only `getrandom` remains (warned, fix 8) |
| 4 | Delivery-agnostic | ~~**Fail** (narrow)~~ **Partial, 0.15.0** | Was: `util::terminal` (stdin/stdout, termion) and `Util::commentary_action_to` (`println!`) ungated. Now the console functions need `terminal` and `commentary_action_to` is deleted. Still open: `TableAction` is the kernel's event type *and* a declared wire enum (fix 10). No gRPC/HTTP/CLI in the lib |
| 5 | Hidden-information projection | **Pass in the kernel** | `PokerSession::view(Option<Principal>)` and `TableSnapshot::from_table` both redact by construction. The consumer ignores them — see §8 |
| 6 | Narrow, stable boundary | **Partial** | The `legal_actions`/`apply_action` pair exists and is feature-free (up from **Fail** in v0.1.8). But `Table` has 21 public mutable fields and `apply_action` returns `()`, not events |
| 7 | Things that change together live in one kernel | **Pass, unanswered** | Everything is in one `Table`, so nothing all-or-nothing crosses a kernel line. The intra-kernel question has never been asked; `PokerSession::advance_street` composes two mutating calls with no in-between state |
| 8 | State belongs to the kernel that changes it | **Fail** | `pkdealer_service` writes `seat.player.chips` directly and re-implements card visibility; 21 public mutable fields make every consumer a potential writer |

---

## 1 — Pure: no I/O of its own · FAIL, concentrated

The checker's 94 findings sort into three piles. Only the first is a kernel
problem.

### 1a. Real leaks in always-compiled code — HARD

These compile into a bare `cargo add pkcore` **and** into
`--no-default-features`, because their modules carry no `#[cfg]`.

- **`src/casino/table.rs:445-447` — `Table::from_seats`, and
  `src/casino/table/player.rs:69,92` — `Player::new` / `new_with_chips`.**
  *(Found on re-run, 2026-09-15.)* Each calls `Uuid::new_v4()`, which reads OS
  entropy through `getrandom`. `from_seats` writes that ID into the event log as
  its first entry, `TableAction::TableOpen(id)`. So two tables built from the same
  seats, the same seed and the same actions have **different event logs**, and
  their players have different IDs. This is ambient randomness in the most central
  constructor the kernel has — worse placed than every `rand::rng()` site below,
  because none of those has a seeded twin missing. **No constructor takes an ID**:
  there is no `with_id` form on `Table`, `Player` or `PokerSession`. The checker
  cannot see it, because `uuid` is not on its banned list and `new_v4` is not in
  its grep. *Minimal fix:* add `Table::from_seats_with_id(…, id: Uuid)` and
  `Player::with_id(id, handle, chips)`; make the current three shims over them;
  later move the shims and uuid's `v4` feature behind `entropy` (§1, "rand
  question"). ~half a day, non-breaking. Every other `new_v4` in `src/` is in a
  test module or a doc example.
  **Done in 0.15.0:** `Table::from_seats_with_id` (the id lands in both
  `Table::id` and `TableOpen`) and `Player::with_id`; the old three are shims.
  The variant constructors (`nlh_`, `limit_holdem_`, `plo_`, `stud_family_from_seats`,
  `table.rs:209,247,288,361`) still call `from_seats`, so they still mint random
  ids; they have no `_with_id` twin yet. `PokerSession::new` takes a built
  `Table`, so it is as deterministic as the table its caller hands it.
- **`src/hand_history.rs:1343-1350` — `HandCollection::save`.** Three violations
  in seven lines: `SystemTime::now()`, a hardcoded `generated/` directory
  (`format!("generated/{run_name}_{ts}.yaml")`), and `create_dir_all` +
  `fs::write`. The kernel is asserting a working-directory layout. It already has
  the pure half — `to_yaml()` — so this is a wrapper, not a capability.
  *Minimal fix:* move `save` behind `hand-histories`+`persistence`, or delete it
  and let the caller write the string. ~30 min.
- **`src/util/mod.rs:2,89` — `Util::read_lines<P: AsRef<Path>>`.** A path in a
  public signature plus `File::open`. Two callers in-repo.
  *Minimal fix:* delete, or take `&str` and let the caller read the file. ~30 min.
- **`src/util/csv.rs:4,10` — `distinct_shus_from_csv_as_masked_vec()`.** A
  pure-looking, argument-free `pub fn` that reads
  `data/csv/shus/distinct_masked_shus.csv` relative to the CWD and logs an error
  into an empty `Vec` when it is not there. This is the shape the previous audit
  called hidden I/O (III.3) and it is unchanged. *Minimal fix:* take the parsed
  rows as a parameter. ~1 h.
- **`src/arrays/matchups/sorted_heads_up.rs:12,21,578,594` — `generate_csv` /
  `read_csv`.** `use csv::{Reader, WriterBuilder}` and `use std::fs::File` with
  no gate; these are the reason `csv` cannot be made optional today.
  *Minimal fix:* gate the pair behind a `csv` feature. ~1 h.
- **`src/cards.rs:472-474` — `Cards::shuffle_in_place`.** Calls ambient
  `rng()` (OS entropy). The seeded twin `shuffle_in_place_with` is right beside
  it at `:482`. **The checker missed this one**: `rand::rng` is imported as `rng`
  at `src/cards.rs:15-16`, so the `rand::` grep never fired on line 473.
- **`src/bot/decider.rs:150,391,441` — `RuleBasedDecider::decide`,
  `JokerDecider::new`, `on_new_hand`.** Each calls `rand::rng()`; each has a
  seeded twin (`decide_seeded`, `new_with_rng`, `on_new_hand_with_rng`) that the
  three shipped deciders all override. The ambient call is the *default*, which
  is backwards for a kernel.
- **`src/analysis/equity/engine.rs:140` —
  `req.opts.seed.unwrap_or_else(rand::random)`.** The seed is injectable and the
  fallback is not.
- **`src/bot/sim.rs:1131` — `SystemTime::now()`** stamping a recorded hand.
  Cheap to inject; it is metadata, not logic.

*Minimal fix for the whole randomness pile:* invert the defaults — make the
seeded form the real method and the ambient form a shim behind an `entropy`
feature. ~1 day, one breaking rename.

### 1b. Honest adapters — the fix is a boundary, not a rewrite

These are real I/O, correctly identified, in code that is unambiguously an
adapter. They are already `#[cfg]`-gated behind non-default features, so they do
not reach a pure build. The right long-term move is a `pkcore-store` sibling
crate; the right short-term move is nothing.

- `src/analysis/store/db/sqlite.rs`, `db/hup.rs`, `store/bcm/binary_card_map.rs`
  — 4 + 4 + 3 findings, all behind `#[cfg(all(feature = "store", not(target_arch
  = "wasm32")))]`, including the `rusqlite` public-signature findings at
  `sqlite.rs:56,76,84,90` and `hup.rs:233,245`. `store` left the default set in
  0.11.0.
- `src/pokerbench/loader.rs:141,179,180` — behind `#[cfg(feature =
  "pokerbench")]` at `src/lib.rs:454`.
- `src/analysis/gto/solver.rs:389-526` — six path-taking `save_*`/`load_*`
  methods and four `std::fs` calls. Every one is a thin wrapper over an existing
  pure pair (`to_binary_bytes`/`from_binary_bytes`,
  `to_json_string`/`from_json_str`). *Minimal fix:* one `#[cfg(feature =
  "persistence")]` on the six wrappers; the pure core is already exported. ~1 h.
- `src/bot/profile.rs:883,885,909,910` — `to_file`/`from_file`, already gated on
  `bot-profiles`, already wrapping `to_yaml_string`/`from_yaml_str`. Same
  one-line fix.
- `src/analysis/player_stats_store.rs:128` — `YamlPlayerStatsStore::new<P:
  AsRef<Path>>`. This is a *named* store adapter behind
  `player-stats-persistence`; a path is its whole job. Cosmetic — but the feature
  is on by default, which is a §3 problem, not a §1 one.
- `src/analysis/store/bcm/binary_card_map.rs:219,225` and
  `src/analysis/store/db/hup.rs:58` — three `std::env::var` calls reading
  `PKCORE_75BCM_PATH` / `PKCORE_75BCM_CSV_PATH` / the HUP DB key. Environment
  lookups are configuration policy and belong to the shell, but all three are
  `store`-gated.

### 1c. False positives — 6 of 94

Worth recording so the next run does not re-litigate them.

- `src/arrays/five/hands.rs:5` — `use std::fs::read_to_string` carries
  `#[cfg(feature = "generators")]` on line 4. The grep reads line 5 alone.
- `src/bot/decider.rs:165` — `decide_with_rng` is `pub(crate)`, not public.
- `src/bot/sim.rs:205` — `seed_rng` is a **private** field of `SimTable`; every
  field of that struct is private.
- `src/analysis/store/embedded/hup_cache.rs:7` — `postcard` appears in the
  initializer body, not in `HUP_CACHE`'s type.
- `src/analysis/store/db/hup.rs:19` — the `use std::fs::File` is `store`-gated on
  line 18.
- `src/bot/decider.rs:84,100` — flagged as "public field or variant payload";
  they are trait method parameters (`&mut dyn rand::RngCore`). Real API surface,
  wrong category — see §2's note on `rand`.

### The `rand` question, answered

> **Status (0.15.0): point 1 fixed, point 2 open.** The defaults are inverted:
> `BotDecider::decide_seeded` is the required method, `decide`/`on_new_hand`
> are `entropy`-gated conveniences, and the same holds for ids, shuffles,
> `PokerSession::start_hand`/`run_hand`, `SimTable` and seedless equity.
> `entropy` is **on by default** — a deliberate choice, so no default user or
> sibling repo breaks; the kernel build (`--no-default-features`) is the one
> with no OS entropy, and `tests/kernel_determinism.rs` runs it. `rand` still
> appears in public signatures (fix 9).

**The `rand::` findings in the bot and equity code are not a kernel violation of
the kind the checker implies.** Nine of them are the *prescribed fix* already in
place: `decide_seeded(&self, …, rng: &mut dyn rand::RngCore)` (`decider.rs:100`),
`BotProfile::decide<R: rand::Rng>` (`profile.rs:957`), `preflop_equity<R:
rand::Rng + ?Sized>` (`preflop_equity.rs:126`), `Cards::shuffle_in_place_with<R>`
(`cards.rs:482`), `SimTable::with_seed`/`with_rng` (`sim.rs:441,452`). That is
randomness injected at the seam, which is exactly what invariant 1 asks for.

Two things are nonetheless true and worth fixing:

1. **The ambient default sits in front of the injected path** (§1a) — `decide`
   calls `decide_with_rng(…, &mut rand::rng())`. A kernel should make the caller
   supply entropy and let a *shell* default it.
2. **`rand` is named in six public signatures**, so pkcore's API is
   semver-coupled to a third-party crate: the 0.8 → 0.9 `rand` break was a
   breaking change for every pkcore caller, whether or not they use bots. That is
   an invariant-2-shaped cost from a non-format crate. *Minimal fix:* pkcore's
   own one-method seam — `pub trait Entropy { fn next_u64(&mut self) -> u64; }`
   with a blanket impl for `rand::RngCore` behind the `entropy` feature. ~2 days,
   breaking.

Bot decision-making itself is a legitimately separate concern from the table
engine — a *supporting* domain over the core one, in the skill's terms. It reads
a projection (`TableSnapshot`) and returns a `PlayerAction`; it never touches
`Table`. If pkcore is ever split, `bot` is the clean cut, and its randomness
travels with it.

---

## 2 — No format/transport crate in the public API · PASS

The v0.1.8 audit found four leaking error surfaces. All four are closed and the
fix held through seven minor versions:

- `src/bot/profile.rs:120-132` — `BotError::Yaml(String)`, with the
  `From<serde_yaml_bw::Error>` impl as the blessed boxing seam, and a comment
  citing the audit that demanded it.
- `src/analysis/gto/solver.rs:117-124` — `SolverError::Json(String)` /
  `Binary(String)`.
- `src/pokerbench/error.rs:12-27` — every variant is a `String`.
- `src/lib.rs:571` — `PKError` carries no third-party payload at all; its one
  non-trivial variant is `TableActionOutOfOrder(TableAction)`, pkcore's own type.

Variant *names* (`Yaml`, `Json`, `Csv`) are cosmetic by the skill's own grading
and need no work. `Table::snapshot() -> Result<Vec<u8>, PKError>`
(`table.rs:2877`) is the model: postcard is the implementation and never appears
in the signature.

The one survivor is `analysis::store::db::sqlite` — `pub struct Connect { pub
connection: rusqlite::Connection }` and the whole `Sqlable` trait
(`sqlite.rs:56-95`) in `rusqlite::Result`. It is a hard leak by the letter of the
invariant, and it is entirely behind the non-default `store` feature, so it never
reaches a kernel build. Treat it as an in-crate adapter awaiting extraction, not
a defect.

---

## 3 — Pure by default · FAIL → PASS in 0.15.0

> **Status (0.15.0, 2026-09-15): fixed.** Default is `equity`, `player-stats`,
> `hup-charts`; `full` restores the old set. `csv` and `serde_json` became
> optional behind new `csv` and `json` features. `make check-purity` now fails
> on either crate — or on any other HARD name — in the default tree as well as
> in `--no-default-features`. `rand` stays required, per the re-run below.
> `postcard` stays required, per the table. Everything below is the finding as
> written.

`Cargo.toml:47-55`:

```toml
default = ["bot-profiles", "hand-histories", "player-stats",
           "player-stats-persistence", "equity", "parallel", "hup-charts"]
```

Three of those seven pull `serde_yaml_bw` (a concrete format crate) and one
pulls `rayon` (a thread pool). So a bare `cargo add pkcore` still compiles a YAML
parser and a work-stealing runtime it may never call. 0.11.0 already did the hard
part by evicting `store` and `terminal`; this is the same move, one notch
further.

Worse, and new to this audit: **`--no-default-features` does not produce a pure
build either.** Four crates on the skill's banned list are non-optional
dependencies (`Cargo.toml:106-127`):

| Crate | Why it is there | Move to |
|---|---|---|
| `csv` 1.4 | `sorted_heads_up.rs:12`, `util/csv.rs`, `pokerbench/loader.rs` | a `csv` feature (or fold into `store`) |
| `serde_json` 1.0 | `solver.rs` JSON dump, `SessionView` tests | the existing `debug-json` feature |
| `rand` 0.9 | shuffling and every bot decision | a new `entropy` feature |
| `postcard` 1 | `Table::snapshot`/`restore` bytes | **keep** — it never appears in a signature (§2), so it is an implementation detail, not a leak. Downgrade this finding to cosmetic |

### Confirmed by `cargo tree` (re-run, 2026-09-15)

`cargo tree --no-default-features -e no-dev`, each survivor traced with `-i`.
The source reading above was right about the four crates, and missed three more
paths into the pure build:

| Survivor | Pulled by | Brings | Fix |
|---|---|---|---|
| `clap` 4.6, `rand` 0.8, `getrandom` 0.2 | `random_name_generator` 0.3.6 (non-optional, `Cargo.toml:123`) | a **CLI argument parser** and a third `rand` major into the kernel | Its only use was `util::name` (`NAMER` and `Name::generate`), which had **no caller** in `src/`, `examples/`, `tests/`, `benches/` or any sibling repo. **Done in 0.15.0:** module and dependency deleted; `clap` moved to the gate's HARD list |
| `getrandom` 0.4 | `uuid`'s `v4` feature (`Cargo.toml:136`) | OS entropy | Only needed by the random-ID constructors in §1a. Fix those, then move `v4` behind `entropy` and into `[dev-dependencies]` for the tests |
| `rand` 0.10 | `cardpack` 0.11.1 (`src/bard.rs:7`) | a PRNG library, **no `getrandom`** — cardpack already builds with `default = []` and `rand` at `default-features = false` | Nothing. This is what a pure `rand` looks like |
| `getrandom` 0.3 | pkcore's own `rand` 0.9 (`thread_rng`/`os_rng` via its default features) | OS entropy | §1a randomness inversion, then `rand = { default-features = false, features = ["std", "std_rng", "small_rng"] }` with `thread_rng`/`os_rng` only under `entropy` (feature list not build-tested) |

**`rand` is not the leak; `getrandom` is.** A PRNG that takes a seed is pure
arithmetic. The I/O is the OS entropy source behind `rand::rng()` and
`Uuid::new_v4()`, and every road to it goes through `getrandom`. That changes the
ratchet in `make check-purity`: its comment plans to move `rand` from WARN to
HARD once pkcore's `rand` is optional, but `cardpack` needs `rand` non-optionally,
so **`rand` can never leave the tree and the ratchet could never shut on it**.
Ratchet on **`getrandom`** instead. It is removable — three paths, all listed
above — and it is the thing that actually breaks replay. `postcard` should leave
the WARN list entirely (§3 table: implementation detail, never in a signature).

 `make check-purity` (Makefile) greps
`cargo tree --no-default-features -e no-dev` for exactly
`rusqlite|zstd|termion|dotenvy|serde_yaml_bw|rayon` — six names, hand-maintained,
and it passes today. The skill's `BANNED_CRATES` list is wider. The gate is
sound; its vocabulary is stale.

### Exactly what must move for `cargo add pkcore` to yield a pure core

**Out of `default`:** `bot-profiles`, `hand-histories`,
`player-stats-persistence` (all three pull `serde_yaml_bw`), and `parallel`
(pulls `rayon`).

**Stays in `default`, legitimately:** `equity` (feature = `[]`, pure compute),
`player-stats` (feature = `[]`, a pure aggregator), `hup-charts` (feature =
`[]`, links data, not code — but see the caveat below).

**Made optional in `[dependencies]`:** `csv`, `serde_json`, `rand`, per the table
above.

**Added:** a `full = [...]` umbrella so `cargo test --features full`, the
examples and docs.rs keep resolving.

**Widened:** the `check-purity` grep, to the skill's banned list plus `csv`,
`serde_json` and `rand`.

The ripple is unusually small for this kind of change, because the repo did the
prep work already: `Cargo.toml` declares `required-features` on 24 examples and
11 tests, and `make ayce` already runs `test-kernel` (`cargo test
--no-default-features`) alongside CI's `no-default-features` job
(`.github/workflows/basic.yaml:98-135`). Estimate **1 day**, plus a release note
— downstream consumers who relied on default YAML must add
`features = ["bot-profiles", "hand-histories"]`.

> **Status (0.15.0): fixed.** `hup_cache` is `#[cfg(feature = "hup-charts")]`,
> and so is everything that reads it — which the source reading below missed:
> `HUPResult::lookup`, `SortedHeadsUp::hup_result`, `Versus::hups_at_deal`,
> `derive_hand_ordering`, and the heads-up branch of `DealEval::new` (which now
> falls back to the equity engine without the chart).

**`hup-charts` caveat.** `src/analysis/store/embedded/hup_cache.rs` carries no
`#[cfg]` at all — neither the module declaration
(`store/embedded/mod.rs:1`) nor the `include_bytes!` of the 15.8 MB
`generated/hups.bin` on line 5. Only its *callers* in
`src/bot/preflop_equity.rs` are gated. So the feature does not gate the blob by
`#[cfg]`; it gates it by leaving a `pub static` unreferenced and trusting the
linker to drop it. That works for a final binary and not for an rlib, and it
makes the Cargo.toml comment's claim ("turning it off removes the chart")
true only by accident. *Minimal fix:* `#[cfg(feature = "hup-charts")] pub mod
hup_cache;`. ~15 min. **Not verified by a build** — this is a source reading.

---

## 4 — Delivery-agnostic · FAIL, narrowly

The good news first: there is no `tonic`, no `axum`, no `clap` and no web type in
the library. gRPC lives entirely in the `pkdealer` repo, which is where it
belongs. `clap` is a dev-dependency.

Two things are still delivery-shaped:

- **Status (0.15.0): fixed differently than proposed.** Gating the whole
  module would break the kernel: `Cards::from_str` and `arrays::sliced` call the
  pure `Terminal::index_cleaner`, and `nubibus` calls `random_happy`/`random_sad`.
  So only the six stdin/stdout functions went behind `terminal`.
  `Util::commentary_action_to` is deleted.
- **`src/util/terminal.rs` is an ungated module** (`src/util/mod.rs:13`). It
  imports `std::io::{BufRead, Write, stdin, stdout}` at line 14 and reads keys in
  raw mode when `terminal` is on. `termion` is properly optional; the stdin/stdout
  path is not. It also holds three `rand::rng()` calls (`:13,69,86`) picking
  emoticons. *Minimal fix:* `#[cfg(feature = "terminal")] pub mod terminal;`.
  ~15 min. **Hard by the letter, cosmetic in effect** — nothing else depends on it.
- **`Util::commentary_action_to` (`src/util/mod.rs:67-73`) `println!`s** the last
  action and whose turn it is. Its own doc comment says it has no callers in this
  repo. *Minimal fix:* delete it; `Table::commentary_*` already returns `String`.

### Are the kernel's own events separable from what goes on the wire? — No.

`src/casino/action.rs:86-88`:

```rust
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Ord, …)]
#[non_exhaustive] // 0.2.0: this is a serialized wire enum; adding a variant must stay non-breaking.
pub enum TableAction {
```

`TableAction` is simultaneously the engine's internal event log
(`Table::event_log: Vec<TableAction>`, `table.rs:105`), the payload of a public
error variant (`PKError::TableActionOutOfOrder`), and — by its own comment — a
serialized wire enum whose shape is a compatibility obligation. That is a fact
the domain records (DDD: *domain event*) and a message sent to another system
(DDD: *integration event*) collapsed into one type. The consequence is concrete:
pkcore cannot add an event without a wire-compatibility argument, and cannot
change the wire without touching the engine.

`TableState` (`snapshot.rs:143-168`) is the counter-example and shows the team
already knows the pattern — its module doc says outright that deriving
`Serialize` on `Table` would freeze 21 public fields into the wire format, so a
DTO carries the obligation instead. `HandHistory` (`src/hand_history.rs`) is the
same shape at the session level.

*Minimal fix:* mirror `TableState`'s move — keep `TableAction` as the internal
event and add `TableActionDto` in the snapshot module with a `From` pair, so the
event log's wire form is versioned alongside `SNAPSHOT_VERSION` rather than
alongside the enum. ~2 days, non-breaking if the DTO is added and the derive is
left in place for one release.

---

## 5 — Hidden-information projection · PASS in the kernel

Two projections exist, both redacting by construction:

- **`PokerSession::view(&self, viewer: Option<Principal>) -> SessionView`**
  (`src/casino/session.rs:810`). Hole cards survive only on the seat whose player
  the viewer owns; `None` is a spectator; there is no reveal-all, even at
  showdown. Keyed on `Principal`, not seat index, so a network client's identity
  maps to whichever seat it holds. `SessionView` carries **no deck field**, so no
  viewer can ever be handed the runout — the type makes the leak
  unrepresentable rather than merely unlikely.
- **`TableSnapshot::from_table(&table, hero_seat)`**
  (`src/bot/table_snapshot.rs`) — the bot's read model. Opponents' hole cards are
  never included.

Stud up-card visibility rides through both (`play::visibility::Visibility`,
round-tripped by `snapshot.rs:584`).

This is the strongest kernel property pkcore has, and it is why the invariant-8
finding below is frustrating rather than fatal: the seam is built, shipped, and
unused.

---

## 6 — Narrow, stable boundary · PARTIAL

**Up from FAIL in v0.1.8.** `src/casino/table/transition.rs` now holds exactly
the shape the skill asks for, and its module doc calls it "the feature-free
transition surface":

- `Table::legal_actions(&self, seat_id: u8) -> Vec<PlayerAction>` (`:65`)
- `Table::apply_action(&mut self, seat, action) -> Result<(), PKError>` (`:148`)

with the fidelity property — every action `legal_actions` advertises is accepted
by `apply_action` — asserted as a test for both no-limit and fixed-limit
(`transition.rs:239,352`), and raise legality shared with `act_raise` through one
`validate_raise` so the advisory and mutating halves cannot drift. `act_bet` and
`act_raise` both pre-validate *before* any mutation (`actions.rs:479-486`,
`:644-652`), each with a comment naming the corruption that taught them to.

Three gaps keep it from a pass:

1. **`Table` exposes 21 public mutable fields** (`table.rs:89-152`: `pub pot`,
   `pub bet`, `pub seats`, `pub deck`, `pub event_log`, …). A narrow boundary
   means a change to internals does not ripple to callers; here every internal
   *is* the boundary. This is also the mechanism behind §8 and behind
   `TESTKIT_AUDIT.md`'s T1 finding.
2. **`apply_action` returns `()`.** The kernel shape is `state × action → state ×
   events`; pkcore's returns nothing and appends to `self.event_log` as a side
   effect, so consumers slice the log by index to find what just happened —
   `pkdealer_service` keeps a `hand_event_log_start: usize` for exactly this.
3. **No `outcome` on the surface.** `end_hand` returns `Winnings` and also
   resets, which `MURATORI_AUDIT.md` covers in more depth.

**`src/games/kuhn.rs` is the in-repo reference implementation** and should be
cited in any ADR: `KuhnState::apply(&self, action) -> Result<KuhnState, PKError>`
(`:465`) takes a value and returns a value, alongside `legal_actions` (`:439`),
`current_player` (`:412`), `is_terminal`/`payoff` (`:383`, `:510`) and
`info_set(player)` (`:555`) — a complete transition surface with no features, no
I/O and no mutation. It is what `Table` would look like if it were built as a
kernel.

---

## 7 — Things that must change together · design question, never answered

*pkcore has never recorded an answer here. What follows is the question, the
evidence, and a recommendation — not a finding of fault.*

### What is pkcore's all-or-nothing step?

It is **`Table::apply_action`**, and it is smaller than the table.

One `apply_action` changes: one seat's chips and state, `table.bet`,
`raise_increment`, `raises_this_street`, `actions_this_street`,
`chip_actions_this_street`, and two entries in `event_log`. It does **not** move
chips into `pot` — that is `bring_it_in()`. It does not deal — that is
`deal_flop` / `deal_turn` / `deal_river` / `deal_stud_street`. It does not change
`phase`.

**So the intra-kernel answer is: one `apply` changes one group, not the whole
state.** The betting group moves per action; the pot, board and phase catch up at
a street boundary, driven from outside. Both answers are legitimate — this one is
in force and undocumented.

### Is there anything that must never half-happen whose writes cross a boundary?

Yes, and the good news is that it does not cross a *kernel* boundary — because
there is only one kernel. Every field involved lives on one `Table`.

But it does cross **two calls**, and the composition is in the shell:

```rust
// src/casino/session.rs:744-771 — PokerSession::advance_street
3 => {
    self.table.bring_it_in()?;   // sweeps every seat's bet into the pot
    self.table.deal_turn()?;     // can fail: NotEnoughCards, AlreadyDealt
}
```

"End the street and deal the next card" must never half-happen. If `deal_turn`
fails after `bring_it_in` succeeded, the bets are already in the pot and the
board is one card short, with nothing in `GamePhase` to name that state and no
rollback — `Table` is mutated in place, so `?` is not a transaction. The stud arm
(`:748-752`) has the same shape.

The same pattern appears inside `apply_action`'s callees, where it was *already*
found and fixed twice: `act_bet` and `act_raise` both carry comments describing
exactly this failure ("`seats.act_bet` mutated the seat and then
`set_raise_increment` rejected, stranding a live Bet") and both now pre-validate.
The street boundary is the same bug one level up, not yet found.

**Recommendation.** Do not split anything. Instead:

1. Add `Table::advance_street(&mut self) -> Result<(), PKError>` that validates
   the deal is possible *before* sweeping — the `act_raise` pattern, applied at
   the street boundary. ~half a day, no API break.
2. Record in `docs/KERNEL_ADR.md`: *pkcore's cluster of data that changes
   together (DDD: aggregate) is one hand at one table. One `apply_action` changes
   the betting group only; the pot, board and phase advance at a street boundary.
   The wider boundary rejected is the multi-table session; the narrower one
   rejected is the seat.* ~2 h.

---

## 8 — State belongs to the kernel that changes it · FAIL

*Also never answered — but here a consumer has answered it by default, and the
answer is wrong.*

### Who owns which state?

`Table::apply_action` and its siblings own: seat chips and state, pot, bet, deck,
board, muck, phase, button, event log. `PokerSession` owns hand numbering, the
shuffled-deck record and pending blind changes (`SessionState`,
`session.rs:915-925`). Nothing else in pkcore writes table state.

### Does a consumer read full state where a projection belongs? — Yes, twice.

`pkdealer` (pkcore 0.11.0), `crates/pkdealer_service/src/main.rs`:

1. **`:1178-1193` — `cap_stacks_to` does `seat.player.chips = cap;`** inside a
   loop over `table.seats.get_seat_mut(i)`. The service is writing chip stacks —
   state whose only rightful writer is the table's own transition path. It is
   deliberate (a blind-cycle stack cap) and it works, but it means two
   independent writers now determine a stack, and the chip-conservation audit
   `Table::end_hand` runs cannot account for chips the service confiscated.
   Compensating state (`banked_profit`, `:390`) exists in the service to paper
   over exactly this.
2. **`:1052-1071` — `card_visibility_from_metadata`** builds a `CardVisibility`
   enum from a gRPC token, and `hole_cards_string(seat)` (`:1076`) reads
   `seat.cards` straight off the engine. **`.view(` appears nowhere in the
   pkdealer workspace.** The entitlement rule — who may see which hole cards —
   has been re-implemented outside the kernel that owns it, which is the failure
   mode invariant 5 exists to prevent. Two implementations of one rule will drift;
   only one of them has pkcore's redaction tests behind it.

A third, milder instance: the service keeps its own struct also named
`TableState` (`:350`) holding the `PokerSession` plus token maps and spans. That
one is fine — it is shell state about the session, not a copy of it — but the
name collision with `pkcore::casino::table::snapshot::TableState` is an accident
waiting to confuse.

### Recommended answer

Using *The Hard Parts*' four options as the skill maps them: this is **one owner,
others ask** (Hard Parts: *delegate*). The table owns chips and card visibility;
the service asks.

1. **Adopt the projection.** Replace `card_visibility_from_metadata` +
   `hole_cards_string` with `session.view(Some(Principal::from(uuid)))` and
   `session.view(None)` for spectators. Deletes code in pkdealer, deletes a
   duplicated rule. ~1 day, crosses repos, requires bumping pkdealer to 0.14.
2. **Give the cap a kernel operation.** Add `Table::cap_stacks(&mut self, cap:
   usize) -> Vec<(u8, usize)>` returning what it confiscated, so the write
   happens inside the owner, logs a `TableAction`, and stays inside the chip
   audit. ~half a day in pkcore, a small edit in pkdealer.
3. **Then seal the fields.** Once no consumer writes them, `pub pot` / `pub bet`
   / `pub seats` can become getters. Breaking, and worth a major version — but
   it is the change that makes invariants 6 and 8 true rather than merely
   observed.

Explicitly **resist** the other two answers: merging the service into the kernel
(Hard Parts: *service consolidation*) grows a god-kernel, and a schema both sides
write (Hard Parts: *data domain*) is DDD's Shared Kernel in database form.

---

## Where this loses — honest limits

- **Little was compiled or executed.** No `cargo build`, no `cargo test`, no
  `make ayce`. The re-run did resolve the tree (`cargo tree`, `make
  check-purity`), which confirmed the §3 survivors and found three more. The
  `hup_cache` linking claim in §3 is still a source reading and could be wrong
  about what the linker actually keeps.
- **A non-banned crate hid the worst leak.** The random IDs in §1a come from
  `uuid`, which no banned list names. Any crate with a `new_v4`/`random`/`now`
  style constructor deserves the same look; a `clippy.toml`
  `disallowed-methods` entry for `uuid::Uuid::new_v4` would catch the next one.
- **The checker's list is not exhaustive.** `BANNED_CRATES` covers the common
  ecosystem, not every crate. A clean report means "none of the known shapes",
  not "pure". pkcore's `random_name_generator`, `percent-encoding`, `regex`,
  `thousands` and `wincounter` were not assessed against a kernel standard at all.
- **The grep misses what it cannot see.** `src/cards.rs:473` is genuine ambient
  randomness the checker never reported, because the import was aliased. There
  may be more of these; a compiler-level check (`clippy.toml`
  `disallowed-methods`, per Mode B) is the only way to know.
- **Only one consumer was read.** `pkdealer` was checked because the brief named
  it. `pkcore.js`, `pkcore.py`, `pkwasm`, `pkspectator`, `pksrv`, `pkarena0-web`
  and a dozen other siblings were not. §8's findings are a floor, not a census.
- **`perf/` is a separate workspace** and was not scanned.
- **Effort estimates are for the mechanical work only** and exclude review,
  release notes and downstream migration.
- **No judgement is offered on whether pkcore *should* be a kernel.** It is a
  working library with real consumers; this audit prices the transformation, it
  does not argue for it.

---

## Ranked fixes

Highest leverage first. Effort is engineering time, excluding review.

| # | Fix | Invariants | Effort | Risk |
|---|---|---|---|---|
| 1 | ~~**Flip `default` to `["equity", "player-stats", "hup-charts"]`, add `full`, make `csv`/`serde_json`/`rand` optional, widen `make check-purity` to the skill's banned list**~~ **Done, 0.15.0** — `rand` deliberately left required (§3 re-run) | 3 | 1 day | Low — consumers relying on default YAML add `features = ["full"]` |
| 1a | ~~**Inject IDs**: `Table::from_seats_with_id`, `Player::with_id`; current constructors become shims (§1a, re-run)~~ **Done, 0.15.0** | 1 | half a day | None — additive |
| 1b | ~~**Delete `util::name`** and drop `random_name_generator`: removes `clap`, `rand` 0.8, `getrandom` 0.2 from the pure build (§3, re-run)~~ **Done, 0.15.0** | 3, 4 | 15 min | Low — no caller anywhere; breaking in letter |
| 1c | ~~**Re-aim the purity ratchet at `getrandom`**, not `rand`; drop `postcard` from WARN (§3, re-run)~~ **Done, 0.15.0** — `clap`/`structopt` now HARD | 1, 3 | 15 min | None |
| 2 | ~~**Gate the six adapter wrappers**: `HandCollection::save`, `SolverResult::save_*`/`load_*`, `BotProfile::to_file`/`from_file` behind a `persistence` feature; `util::terminal` behind `terminal`; `hup_cache` behind `hup-charts`; delete `Util::read_lines` and `Util::commentary_action_to`~~ **Done, 0.15.0** — plus `Pluribus::read_in_log` (new pure twin `parse_log`); `terminal` gates functions, not the module (§4) | 1, 4 | 1 day | Low |
| 3 | **Adopt `PokerSession::view` in pkdealer** and delete `card_visibility_from_metadata` / `hole_cards_string` | 5, 8 | **1 day** | Medium — crosses repos, needs a pkdealer bump to 0.14 |
| 4 | **Write `docs/KERNEL_ADR.md`**: name the cluster of data that changes together, record the intra-kernel answer from §7, record delegate-ownership from §8, name the rejected wider (multi-table) and narrower (seat) boundaries | 7, 8 | **half a day** | None |
| 5 | **Make the street boundary all-or-nothing**: `Table::advance_street` that validates the deal before sweeping bets | 7 | **half a day** | Low — same pattern as `act_raise`'s pre-validation |
| 6 | **Move the cap into the kernel**: `Table::cap_stacks`, logging a `TableAction` and staying inside the chip audit | 8 | **half a day** | Low |
| 7 | **Fix `src/util/csv.rs` and `sorted_heads_up.rs`**: take rows as parameters instead of reading CWD-relative files. *No longer a prerequisite for #1:* 0.15.0 gated both behind the `csv` feature instead, so they are out of the pure build but still read CWD-relative paths when on | 1 | **1 day** | Low |
| 8 | ~~**Invert the randomness defaults**: seeded form becomes the method, ambient becomes a shim behind `entropy`; inject the two `SystemTime::now` sites~~ **Done, 0.15.0** — `entropy` on by default; `getrandom` now HARD for the kernel build; replays and fixtures got derived or fixed ids; `HandCollection::save`'s clock left to `persistence` | 1 | 1–2 days | Medium — `BotDecider` trait inverted |
| 9 | **Own the entropy seam**: `pkcore::Entropy` replacing `rand::Rng` in six public signatures | 1, 6 | **2 days** | Breaking |
| 10 | **Split `TableAction`** into the internal event and a `TableActionDto` versioned with `SNAPSHOT_VERSION` | 4 | **2 days** | Low if added alongside for one release |
| 11 | **Return events from `apply_action`** (`Result<Vec<TableAction>, PKError>`) so consumers stop slicing `event_log` by index | 6 | **3 days** | Breaking |
| 12 | **Seal `Table`'s 21 public fields** behind getters, once #3 and #6 land | 6, 8 | **1 week** | Breaking, major version |
| 13 | **Mode B enforcement**: `clippy.toml` `disallowed-methods` for `std::fs`/`std::env`/`rand::rng`, a `cargo-deny` `[bans]` fragment, and `check_purity.py` wired into `make ayce` | all | **1 day** | None — but do it *after* #1 and #2, or it fails red on day one |

Items 1, 2 and 4 together are about 2.5 days and close or answer five of the
eight invariants. Items 9–12 are one breaking release and should be batched.

---

## How to re-run

```bash
cd /path/to/pkcore
python3 ~/.claude/skills/domain-kernel/scripts/check_purity.py .
```

Exit 1 with a per-file list of hard findings. It needs only Python — no Rust
toolchain. Then the two build-level gates this audit did **not** run:

```bash
make check-purity     # cargo tree --no-default-features -e no-dev, grepped
make test-kernel      # cargo test --no-default-features
```

Then re-read the sections above against the current tree. The verdict table in
§Verdict is the diffable part: an invariant moving from **Fail** to **Pass** is
the unit of progress. Update the version, commit and date in the header table
each time, and note in §1c any finding that has become a false positive because
a `#[cfg]` moved.
