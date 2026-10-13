# EPIC-92: Elixir Bindings (PKCORE.EX)

> **Provenance.** Drafted 2026-10-12 against `main` @ `53345346` (pkcore
> `0.15.4`). Prompted by Piotr Sarnacki's *"I'm sorry, but you still have to
> think"* (itsallaboutthebit.com, 2026-10-11), which compares Rust and Elixir
> rewrites of the same chat server and names [Rustler](https://github.com/rusterlium/rustler)
> as the proof that the BEAM/Rust trade-off is real. The companion analysis of
> that article's performance lessons is
> [`ANALYSIS_Load_Isolation_and_Backpressure.md`](../ANALYSIS_Load_Isolation_and_Backpressure.md).
> This is a contract doc. The implementation lands in a new sibling repo,
> `pkcore.ex`, the same split [EPIC-85](EPIC-85_Node_Bindings.md) used for
> `pkcore.js`.

## Summary

- **Builds:** `pkcore.ex`, a Hex package that runs the real `pkcore` engine
  inside an Elixir/OTP app through Rustler NIFs.
- **Why:** the BEAM is the natural home for many concurrent tables, and today
  an Elixir app can only reach pkcore over the network or by porting the rules.
- **Shape:** a thin, mechanical wrapper over the 22-class surface `pkcore.js`
  already proves, with each table held in a `ResourceArc` that one OTP process
  owns, and every long computation on a dirty scheduler.
- **Proves it:** the shared parity fixtures (the 271 full-house hand, chip
  conservation, error shape) pass in ExUnit, plus a scheduler-health test that
  shows a 5 s solve does not stall a 1 ms heartbeat process.
- **Status:** Planned. Nothing is built. The repo does not exist.

---

## Context

`pkcore` has three language bindings today. None is for the BEAM.

| Binding | Repo | Tech | pkcore pin |
|---|---|---|---|
| Python | `../pkcore.py` | PyO3 0.28 + maturin (`pkcore.py/Cargo.toml:13`, `pkcore.py/pyproject.toml:1-3`) | `0.11.0` (`pkcore.py/Cargo.toml:14`) |
| Node | `../pkcore.js` | napi-rs ([EPIC-85](EPIC-85_Node_Bindings.md)) | `0.11.0` |
| Browser | `../pkwasm` | wasm-bindgen ([EPIC-86](EPIC-86_Browser_Bindings.md)) | `0.11.0` |

There is no Elixir prior art in the sibling repos. `card_ex_co`, `exgto` and
`expkcalc` look like Elixir names, but all three are Rust. The `ex` means
"example" (`exgto/Cargo.toml:8`, `expkcalc/Cargo.toml:8`). No `mix.exs` exists
anywhere under `ImperialBower/`.

**What pkcore.py teaches.** pkcore.py binds 67 classes and 4 functions
(`pkcore.py/src/lib.rs:4002-4065`, `pkcore.py/src/session.rs:232-234`,
`pkcore.py/src/table_no_cell.rs:226-229`). Three of its habits are what this
EPIC must *not* copy:

1. Every error becomes a `ValueError` carrying a string
   (`pkcore.py/src/lib.rs:67-69`, `:2829-2831`). Callers cannot branch on the
   error kind.
2. The GIL is never released. There is no `allow_threads` anywhere, so
   `Solver.solve` freezes the interpreter. The BEAM version of that mistake is
   worse: a long NIF on a normal scheduler stalls *every* process on that
   scheduler, not just the caller.
3. 13 registered classes are missing from the Python re-export list
   (`pkcore.py/python/pkcore/__init__.py:23-82`), so `from pkcore import Solver`
   fails. A parity gate would have caught it.

**What pkcore.js teaches.** `pkcore.js` binds 22 classes plus `pkcoreVersion()`
(`pkcore.js/index.d.ts:4-465`). EPIC-85 and EPIC-86 fixed four rules this EPIC
inherits: one wrapper type per Rust type with one-line delegation
(EPIC-85 Goals); state lives in the object, never in a `thread_local!` or
singleton (EPIC-86:152-154); errors carry the `DealerError` variant as data
(EPIC-85:160-166); and the binding version moves in lock-step with pkcore
(EPIC-85:135-138).

**The kernel is ready for the BEAM.** On `main` @ `53345346`, `Table`,
`Dealer`, `PokerSession` and `Solver` are all `Send + Sync` with default
features (checked 2026-10-12 with a scratch crate that calls
`fn send_sync<T: Send + Sync>()` on each; it compiles). Rustler needs that to
put a value in a `ResourceArc`. `Table::snapshot`/`restore`
(`src/casino/table.rs:3129`, `:3154`) and `PokerSession::snapshot`/`restore`
(`src/casino/session.rs:693`, `:713`) from [EPIC-88](EPIC-88_Table_Snapshot.md)
return postcard bytes, which map 1:1 to an Elixir binary.

**This EPIC does not:**

- change pkcore. Every item in pkcore is a doc or skill edit.
- bind the GTO solver or Kuhn in v1, except the one stepping seam named in
  [Design](#long-work-and-the-schedulers). That matches EPIC-85 and EPIC-86.
- ship a table *server*. It ships the engine plus one example `GenServer`.
  A supervised multi-table service is the subject of the companion analysis.
- bind `PokerSession::run_hand` (`src/casino/session.rs:768`). A NIF cannot
  call back into Elixir, and EPIC-85:456-469 already explains why a re-entrant
  callback into a `&mut` method is unsound.

---

## Status

| Component | Status |
|---|---|
| `pkcore.ex` repo scaffold (mix project + `native/pkcore_ex` Rustler crate) | Planned |
| Card & eval primitives (`Card`, `Cards`, `Rank`, `Suit`, `Eval`, `HandRank`, `Board`, `HoleCards`, `Two`) | Planned |
| Table engine (`Table`, `Player`, `Seat`, `Seats`, `ForcedBets`) | Planned |
| `Dealer` + `DealerAction` + `TableAction` event log | Planned |
| `PokerSession` / `PlayerAction` / `SessionStep` | Planned |
| Snapshot / restore as Elixir binaries | Planned |
| Dirty-scheduler equity + `Solver.iterate/2` stepping seam | Planned |
| Example `PkCore.TableServer` GenServer | Planned |
| Precompiled NIFs (`rustler_precompiled`) + Hex publish | Planned |
| Parity gate against `pkcore.js/index.d.ts` | Planned |
| GTO solver full surface, Kuhn | **Deferred** — follow-on EPIC |
| pkcore-side doc edits (ROADMAP row, audit-release consumer row) | Planned |

---

## Goals

- Let an **Elixir/OTP** app seat players, run a hand and read results against
  the real pkcore engine, with no network hop and no rules port.
- Keep the binding a **thin, mechanical wrapper**: one Elixir module per bound
  Rust type, one NIF per method, one line of delegation in the Rust body.
- **Never block a normal scheduler for more than 1 ms.** Any call that can run
  longer goes on a **dirty CPU scheduler** or is cut into bounded steps.
- Return **idiomatic results**: `{:ok, value}` / `{:error, reason}` tuples,
  with `reason` as an atom or tagged tuple. Add `!` variants that raise.
- Ship **precompiled NIFs** so `mix deps.get` works without a Rust toolchain,
  the way pkcore.py ships wheels (`pkcore.py/.github/workflows/publish.yml:12-62`).
- Track pkcore **version lock-step**, the rule in `pkcore.py/CLAUDE.md:3-7`.

## Scope

- Bind pkcore **0.15.x** or later, the plain `Table` API. No celled types.
- One native crate, built with **Rustler** (`rustler` crate + `:rustler` mix
  dep). Not a Port, not a C node. See [Design](#why-nifs-and-not-a-port).
- **Integers.** pkcore chips are `usize` (`src/casino/table/player.rs:28-36`).
  Elixir integers have no fixed size, and Rustler's decoder for `usize` fails
  with `ArgumentError` on a negative or oversize value. That is the correct
  behaviour. It is safer than the JS `ToUint32` trap EPIC-86:264-277 had to
  work around. No `f64` or `i64` detour is needed.
- **Cards** cross as strings (`"A♠"`, `"AS"`), the same as `Card.parse` in
  pkcore.py (`pkcore.py/src/lib.rs:310`).
- **Seats** cross as `0..=255` integers decoded to `u8`.
- **Errors** become `{:error, {:dealer, :not_your_turn}}`-style tuples. The
  atom is the snake-case name of the `DealerError` variant
  (`src/casino/dealer.rs:97`). `PKError` (`src/lib.rs:571`) maps the same way
  under `{:pk, atom}`.
- **Async** is not needed. Every bound call is a plain synchronous NIF. The
  BEAM gives concurrency by running many processes, each owning its own table.
- **Features.** Depend on pkcore with `default-features = false` and turn on
  only what v1 binds. The default `hup-charts` feature embeds a 15.8 MB blob
  (`Cargo.toml:128`). That would ship in every precompiled NIF for no v1 use.
  `parallel` (rayon, `Cargo.toml:114`) stays off: a rayon pool inside a dirty
  NIF competes with the BEAM's own scheduler threads.

---

## Domain map

| pkcore type | pkcore.js binding (reference) | Proposed `pkcore.ex` module |
|---|---|---|
| `card::Card` (`src/card.rs:37`) | `Card` (`pkcore.js/index.d.ts:17`) | `PkCore.Card` (struct holding the CKC `u32`) |
| `cards::Cards` (`src/cards.rs:42`) | `Cards` (`:31`) | `PkCore.Cards` |
| `Rank`, `Suit` | `Rank` (`:274`), `Suit` (`:364`) | `PkCore.Rank`, `PkCore.Suit` (atoms) |
| `analysis::eval::Eval` (`src/analysis/eval.rs:210`) | `Eval` (`:95`) | `PkCore.Eval` |
| `analysis::hand_rank::HandRank` (`src/analysis/hand_rank.rs:39`) | `HandRank` (`:130`) | `PkCore.HandRank` (plain struct) |
| `Board`, `HoleCards`, `Two` | `:4`, `:142`, `:431` | `PkCore.Board`, `PkCore.HoleCards`, `PkCore.Two` |
| `casino::table::Table` (`src/casino/table.rs:102`) | `Table` (`:378`) | `PkCore.Table` (resource) |
| `Player`, `Seat`, `Seats` | `:158`, `:301`, `:325` | `PkCore.Player`, `PkCore.Seat`, `PkCore.Seats` (plain structs, read out of the table) |
| `casino::dealer::Dealer` (`src/casino/dealer.rs:182`) | `Dealer` (`:51`) | `PkCore.Dealer` (resource) |
| `ForcedBets` | `:115` | `PkCore.ForcedBets` (plain struct) |
| `TableAction` + event log | `TableAction` (`:415`) | `PkCore.TableAction` (tagged tuples in a list) |
| `Winnings`, `PotWin`, `SeatEquity` | `:447`, `:257`, `:315` | plain structs |
| `casino::session::PokerSession` (`src/casino/session.rs:125`) | `PokerSession` (`:217`) | `PkCore.Session` (resource) |
| `PlayerAction`, `SessionStep` | `:183`, `:350` | tagged tuples |
| `Table::snapshot` / `restore` | not bound | `PkCore.Table.snapshot/1 :: binary()` |
| `analysis::equity` (`src/analysis/equity/engine.rs:91`) | not bound | `PkCore.Equity.compute/1` (dirty CPU) |
| `Solver::iterate` (`src/analysis/gto/solver.rs:752`) | not bound | `PkCore.Solver.iterate/2` (bounded step) |
| `analysis::gto::*` rest, `games::kuhn::*` | ❌ deferred | ❌ deferred |

Value types (cards, hand ranks, forced bets, winnings) cross as **plain Elixir
structs**, encoded with `#[derive(NifStruct)]`. Only the three stateful types
(`Table`, `Dealer`, `Session`) and `Solver` are **resources**. That keeps
pattern matching natural in Elixir and keeps the resource count small.

---

## Design

### Why NIFs and not a Port

A Port runs pkcore in a separate OS process and talks over stdin/stdout. It
cannot crash the VM, but every call pays a serialise/copy/context-switch cost,
and a hand has dozens of calls. A NIF runs in the VM, so a call costs about as
much as a function call. The risk is that a NIF which panics or runs long can
hurt the VM. Rustler turns a Rust panic into an Elixir exception
(`catch_unwind` at the NIF boundary), and pkcore library code already forbids
`unwrap`/`panic!` (project `CLAUDE.md`). Long runs are handled by the next
section. So the NIF's two risks have answers, and the Port's cost does not.

### Long work and the schedulers

A normal BEAM scheduler expects a NIF to return in about 1 ms. Measured kernel
cost decides which calls are safe. The numbers are medians on an Apple M1 from
`docs/perf/RESULTS.md`, taken against pkcore **0.3.2** (`RESULTS.md:11`). They
are old. Phase 5 re-measures before it sets any cap.

| Call | Measured cost | Scheduler |
|---|---|---|
| 7-card `eval` | 4.1 µs (`docs/perf/RESULTS.md:36`) | normal |
| self-play, one 6-max hand | ~95 µs (`docs/perf/PROFILING.md:251`) | normal |
| `Equity.compute`, exact heads-up flop | 3.3 ms (`RESULTS.md:40`) | `DirtyCpu` |
| `Equity.compute`, Monte Carlo three-way | 136 ms (`RESULTS.md:42`) | `DirtyCpu` |
| `Equity.compute`, exact heads-up preflop | 5.5 s (`RESULTS.md:41`) | `DirtyCpu` |
| one CFR `iterate()` | ~1.6 ms (`docs/perf/PROFILING.md:286`) | `DirtyCpu`, via `Solver.iterate/2` |
| `Solver.solve` (`src/analysis/gto/solver.rs:852-864`) | `max_iterations` × the above | **not bound** |

The engine switches from exact enumeration to 25,000 Monte Carlo samples above
100,000 runouts (`src/analysis/equity/spec.rs:65`, `:86`, `:103-104`), so even
the dirty path has a ceiling. A 5.5 s exact preflop call still holds one dirty
scheduler for 5.5 s. With the default of one dirty CPU scheduler per core,
a few of those at once can use them all. The `TableServer` example therefore
never calls equity itself.

pkcore has no progress or cancel seam today. `SolveJob` ([EPIC-37](EPIC-37_Mobile_Engine.md))
and `solve_with_progress` ([EPIC-38](EPIC-38_Observability.md)) are still
Planned, and no cancel token exists in `src/`. `Solver::iterate`
(`src/analysis/gto/solver.rs:752`) is the one real stepping seam, and it is
enough: a `GenServer` holds the `Solver` resource and calls
`Solver.iterate(solver, 100)` in a loop. Between steps it can report progress,
take a `:cancel` message, or stop. Cancel is then plain OTP, with no new pkcore
API.

```rust
// native/pkcore_ex/src/solver.rs
#[rustler::nif(schedule = "DirtyCpu")]
fn solver_iterate(solver: ResourceArc<SolverRes>, steps: u32) -> Result<f64, Error> {
    let mut guard = solver.0.lock().map_err(|_| Error::Term(Box::new(atoms::poisoned())))?;
    let mut last = 0.0;
    for _ in 0..steps.min(MAX_STEPS_PER_CALL) {
        last = guard.iterate();
    }
    Ok(last)
}
```

`MAX_STEPS_PER_CALL` caps one call so that even a dirty scheduler is not held
for minutes. At ~1.6 ms per iteration, a cap near 50 keeps one call under
100 ms. The cap is a constant in the crate, set by measurement in Phase 5.

### Resources and ownership

```rust
// native/pkcore_ex/src/dealer.rs
pub struct DealerRes(pub std::sync::Mutex<pkcore::casino::dealer::Dealer>);

#[rustler::resource_impl]
impl rustler::Resource for DealerRes {}

#[rustler::nif]
fn dealer_act(dealer: ResourceArc<DealerRes>, seat: u8, action: Action) -> Result<Vec<Event>, Error> {
    let mut guard = dealer.0.lock().map_err(|_| poisoned())?;
    guard.act(seat, action.into()).map(Event::from_log).map_err(dealer_err)
}
```

A `ResourceArc` can be shared by any number of Elixir processes, so the Rust
side needs a lock. A `std::sync::Mutex` is correct here: each hold lasts
microseconds and never crosses an `.await` (there is none). That is the case
the article calls safe — "synchronous locks acceptable only for very short
hold times."

The lock is a safety net, not the concurrency model. The intended model is
**one process per table**: a `GenServer` owns the resource and nobody else
holds a reference. All actions on one table go through that process's mailbox,
in order. Different tables run in different processes, on different
schedulers, in parallel.

That is the opposite of the Elixir Campfire mistake the article found — one
process that ran *every* SQL query in sequence. One process per table
serialises only what is serial by nature (the betting order at one table).

### The example `PkCore.TableServer`

The package ships one `GenServer` as an example and as the target for the
scheduler-health test. It is small on purpose. Two choices in it are explicit,
because the article shows what happens when they are left to chance:

- **Backpressure.** `GenServer.call/3` with a finite timeout (default 5 s), so a
  caller learns about overload instead of waiting forever. A BEAM mailbox has no
  bound. Elixir's version of the bug is a mailbox that grows to 1.8 GB.
- **Event fan-out.** Subscribers get events with `send/2`. The server checks
  each subscriber's `Process.info(pid, :message_queue_len)` before each send and
  drops a subscriber whose queue is over a set limit. The dropped subscriber
  gets a `:resync` message and re-reads state from a snapshot. That is the
  "fail fast and let the client reconnect" choice the article prefers to
  three-minute-late delivery.

```elixir
defmodule PkCore.TableServer do
  use GenServer
  @max_subscriber_queue 1_000

  def act(server, seat, action), do: GenServer.call(server, {:act, seat, action}, 5_000)

  @impl true
  def handle_call({:act, seat, action}, _from, %{dealer: dealer} = state) do
    case PkCore.Dealer.act(dealer, seat, action) do
      {:ok, events} -> {:reply, :ok, fan_out(state, events)}
      {:error, _} = err -> {:reply, err, state}
    end
  end
end
```

### Persistence

`PkCore.Table.snapshot/1` returns a binary and `restore/1` takes one. A caller
can keep a snapshot in ETS, a file, or a database column without pkcore knowing.
The `TableServer` example snapshots after each completed hand, not after each
action, so a DB write is never in the per-action path. The trade-off (losing up
to one hand on crash) is stated in the module doc, not left as a coin flip.

### Repository layout

New repo `pkcore.ex`, sibling to `pkcore.py` and `pkcore.js`:

```
pkcore.ex/
├── mix.exs                     # app :pkcore, deps :rustler, :rustler_precompiled
├── lib/pk_core/                # one .ex module per bound type
├── native/pkcore_ex/
│   ├── Cargo.toml              # crate pkcore_ex, cdylib, pkcore = "=0.15.x"
│   └── src/                    # one .rs file per bound type
├── test/                       # ExUnit, parity fixtures
├── checksum-Elixir.PkCore.Native.exs   # rustler_precompiled checksums
└── .github/workflows/{ci,release}.yml
```

The crate is `pkcore_ex` (snake case is Rustler's convention under `native/`).
The Hex package name is `pkcore` if it is free on hex.pm; check before
Phase 0, since npm already refused the unscoped `pkcore` (EPIC-86:16-17).

---

## Work Items

### Phase 0 — Repo, toolchain, and pkcore-side claims

- [ ] **0a.** Check `pkcore` on hex.pm. Record the chosen package name here.
- [ ] **0b.** `mix new pkcore_ex --module PkCore`, add `{:rustler, "~> 0.36"}`,
  `mix rustler.new` → `native/pkcore_ex`. Pin `pkcore` with
  `default-features = false, features = ["equity"]`.
- [ ] **0c.** `PkCore.Native.pkcore_version/0`, plus a test that it equals the
  version in `mix.exs` (the lock-step gate, as EPIC-85:372-374).
- [ ] **0d.** In pkcore: add `pkcore.ex` to the Tier 2 table in
  `.claude/skills/audit-release/SKILL.md:28-41` and add
  `$BASE/*/native/*/Cargo.toml` to its discovery grep (`:83`), which would
  otherwise miss a Rustler crate.
- [ ] **0e.** Green: `mix test`, `cargo clippy --manifest-path native/pkcore_ex/Cargo.toml -- -D warnings`.

### Phase 1 — Card & eval primitives

- [ ] **1.** `Card`, `Cards`, `Rank`, `Suit`, `Board`, `HoleCards`, `Two` as
  `NifStruct`s with `parse/1` and `parse!/1`.
- [ ] **2.** `Eval.from_cards/1`, `HandRank`. Test the shared fixture:
  `6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠` → rank 271, `:full_house`, `:sixes_over_fives`
  (EPIC-85:389-395).

### Phase 2 — Table engine

- [ ] **3.** `Table`, `Player`, `Seat`, `Seats`, `ForcedBets`. `Table` is a
  resource; the rest are structs read out of it.
- [ ] **4.** Test that `-1` chips raises `ArgumentError` at the boundary.

### Phase 3 — Dealer, event log, session, snapshot

- [ ] **5.** `Dealer` resource, `act/3`, event list as tagged tuples.
- [ ] **6.** `dealer_err` → `{:error, {:dealer, atom}}`. Test every
  `DealerError` variant maps to a distinct atom.
- [ ] **7.** `Session` resource, `PlayerAction`, `SessionStep`. Not
  `run_hand`.
- [ ] **8.** `snapshot/1`, `restore/1`. Test a mid-hand round trip, and that a
  corrupt binary gives `{:error, {:pk, :snapshot_corrupt}}`.
- [ ] **9.** Test `two_dealers_are_independent` (EPIC-86:456-460).

### Phase 4 — Example `TableServer`

- [ ] **10.** `PkCore.TableServer` as in [Design](#the-example-pkcoretableserver).
- [ ] **11.** Test that a subscriber that never reads is dropped and gets
  `:resync`, and that the server's memory stays flat while it happens.
- [ ] **12.** Test 1,000 `TableServer`s running full hands at once under one
  supervisor. Assert chip conservation on every table.

### Phase 5 — Long work on dirty schedulers

- [ ] **13.** `Equity.compute/1` with `schedule = "DirtyCpu"`.
- [ ] **14.** `Solver.new/1`, `Solver.iterate/2` with `MAX_STEPS_PER_CALL`.
  Measure one `iterate()` on the default config and set the cap so one call
  stays under 100 ms.
- [ ] **15.** The scheduler-health test (see [Test Plan](#test-plan)).

### Phase 6 — Packaging, CI, parity

- [ ] **16.** `rustler_precompiled` release workflow: linux x86_64/aarch64
  (gnu + musl), macOS x86_64/aarch64, windows x86_64, NIF versions 2.16 and
  2.17. Matches the pkcore.py wheel matrix
  (`pkcore.py/.github/workflows/publish.yml:12-62`) plus musl for Alpine
  containers, which are common for Elixir releases.
- [ ] **17.** Parity gate: a script reads the 22 `export declare class` names
  from `pkcore.js/index.d.ts` and fails CI if a bound module is missing.
- [ ] **18.** `mix docs` (ExDoc) with a doctest on every public function.
- [ ] **19.** Add the EPIC-92 row to `ROADMAP.md`'s pkcore Epics table (done
  with this draft) and flip it when Phase 6 ships.

### Deferred (follow-on EPIC, not this one)

- Full GTO surface (`SolverConfig`, `SolverResult`, `ActionFrequencies`), and
  Kuhn. Same deferral as EPIC-85 and EPIC-86.
- A `SolveJob`-shaped API, once EPIC-37 lands one in pkcore.
- A Phoenix LiveView spectator example.

---

## Test Plan

- `card_parse_round_trip` — every one of 52 cards parses and prints back.
- `full_house_fixture` — the 271 hand (EPIC-85:389-395).
- `negative_chips_raise` — `-1` chips is `ArgumentError`, not a huge number.
- `dealer_error_shape` — each `DealerError` variant gives a distinct
  `{:dealer, atom}`.
- `full_hand_chip_conservation` — chips in = chips out after a hand
  (EPIC-85:525-545).
- `snapshot_round_trip_mid_hand` — restore gives an equal table and the hand
  plays on.
- `two_dealers_are_independent` — EPIC-86:456-460.
- `slow_subscriber_is_dropped` — a subscriber that never reads gets `:resync`
  and the server's `:message_queue_len` and memory stay bounded.
- `thousand_tables_conserve_chips` — 1,000 `TableServer`s, full hands, chips
  conserved everywhere.
- `dirty_solve_does_not_starve_heartbeat` — start a process that wakes every
  1 ms and records the gap. Run `Solver.iterate/2` in a loop for 5 s on all
  dirty schedulers. Assert the heartbeat's p99 gap stays under 5 ms. Run the
  same test once with the NIF deliberately on a normal scheduler and assert it
  *fails*, so the test proves it can see the problem.
- `version_lock_step` — native version = `mix.exs` version.

## Key Files (proposed, in the new `pkcore.ex` repo)

| File | Role |
|---|---|
| `mix.exs` | Hex package, `rustler_precompiled` config, version |
| `native/pkcore_ex/Cargo.toml` | `cdylib`, pinned `pkcore` with lean features |
| `native/pkcore_ex/src/lib.rs` | `rustler::init!`, atoms, `dealer_err` |
| `native/pkcore_ex/src/{card,table,dealer,session,solver}.rs` | one NIF module per bound type |
| `lib/pk_core/*.ex` | Elixir modules, `!` variants, typespecs |
| `lib/pk_core/table_server.ex` | example `GenServer` with explicit backpressure |
| `test/parity_test.exs` | 22-class parity gate vs `pkcore.js/index.d.ts` |

## Reuse (do NOT recreate)

- `pkcore.js/index.d.ts:4-465` — the 22-class surface to mirror.
- `pkcore.js` `dealer_err` (EPIC-85:303-331) — the variant-as-data pattern.
- `pkcore.py/.github/workflows/publish.yml:12-62` — the platform matrix.
- `pkcore.py/Makefile:87-94` — the "dirty tree / existing tag / missing
  CHANGELOG" release guard.
- `src/casino/table.rs:3129,3154` and `src/casino/session.rs:693,713` — snapshot
  and restore. Do not invent an Elixir-side state format.
- `src/analysis/gto/solver.rs:752` — `iterate()`, the only stepping seam.

## Compatibility

- **Preserves** everything in pkcore. No pkcore code changes.
- **Adds** a new downstream consumer that `audit-release` must check.
- **Breaks** nothing.

## Dependencies

- **Built on:** [EPIC-85](EPIC-85_Node_Bindings.md) (surface and rules),
  [EPIC-86](EPIC-86_Browser_Bindings.md) (object-scoped state rule),
  [EPIC-88](EPIC-88_Table_Snapshot.md) (snapshot/restore).
- **Related:** [EPIC-37](EPIC-37_Mobile_Engine.md) (`SolveJob`),
  [EPIC-38](EPIC-38_Observability.md) (progress seams),
  [`ANALYSIS_Load_Isolation_and_Backpressure.md`](../ANALYSIS_Load_Isolation_and_Backpressure.md)
  (the multi-table load work this binding makes possible).
- **Blocks:** nothing.

## Verification

```bash
# In the new pkcore.ex repo
mix deps.get
mix compile                                   # builds the NIF from source
mix test                                      # all ExUnit tests incl. scheduler health
mix test --only scheduler_health
cargo clippy --manifest-path native/pkcore_ex/Cargo.toml -- -D warnings
cargo test   --manifest-path native/pkcore_ex/Cargo.toml
mix docs
# Precompiled path, from a clean machine with no Rust toolchain:
MIX_ENV=prod mix deps.get && mix compile
```

Exit criteria:

1. All 22 classes from `pkcore.js/index.d.ts` have a matching `PkCore.*` module.
2. `dirty_solve_does_not_starve_heartbeat` passes, and its normal-scheduler
   twin fails.
3. `thousand_tables_conserve_chips` passes.
4. A clean machine with no Rust installs and runs the package from Hex.
5. pkcore's `audit-release` finds `pkcore.ex` with no manual path.
