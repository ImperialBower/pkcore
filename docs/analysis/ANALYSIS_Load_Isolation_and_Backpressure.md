# ANALYSIS: Load, Isolation, and Backpressure

> **Status:** exploration, not a plan. Written 2026-10-12 against pkcore `main`
> @ `53345346` (`0.15.4`), pkdealer, pkspectator and cardroom as checked out
> beside it. Nothing here has been built. Section 7 names the future work and
> who would own it.
>
> **Companion:** [EPIC-92 Elixir Bindings](../epics/EPIC-92_Elixir_Bindings.md).

## 1. Why this doc exists

Piotr Sarnacki's *"I'm sorry, but you still have to think"*
(<https://itsallaboutthebit.com/i-am-sorry-but-you-still-have-to-think/>,
2026-10-11) looks at AI-written rewrites of the Campfire chat server in Rust
and Elixir. Its point is not "Rust bad" or "Elixir good". Its point is that
every system makes choices about **what to do when it is overloaded**, and a
prompt that does not state those choices leaves them to chance: *"if your
prompt is not specific enough, many decisions are a coin flip."*

The pkcore family is now big enough to have those choices in it. pkdealer runs
a table as a tokio gRPC service. pkspectator fans events out over SSE. EPIC-92
proposes a BEAM binding. ROADMAP wants many tables and persistence one day.
This doc asks three questions:

1. Which of the article's failure modes already exist in our code?
2. How do we test so that a performance problem points at its own cause?
3. What would a multi-table, database-backed lab look like, built on purpose
   to make these failures happen where we can watch them?

---

## 2. What the article found

| # | Failure | Cause | Article's numbers |
|---|---|---|---|
| a | Blocking work in async tasks | Sync DB calls and sync locks inside Tokio tasks. Tokio is cooperative: a task that does not yield blocks its worker thread. | A 100 ms blocking query stalls every task on that thread. |
| b | Bounded vs unbounded queues | Rust used `tokio::sync::broadcast` with capacity **256**; slow receivers got `RecvError::Lagged` and lost events. Elixir's `GenServer` mailbox has no bound. | At 100 POST/s: Rust delivered ~14% of events; Elixir ~60%, with 23% POST timeouts, ~180 s worst latency, 1.8 GB peak memory. Raising Rust's capacity to 16,384 lifted delivery to ~90% but worst latency went from 11 s to 130 s+. |
| c | One process for all SQL | The Elixir rewrite ran every query, reads included, through a single process. | — |
| d | Closed-loop load tests | N clients that each wait for a reply before sending again. They can never send faster than the server answers, so they hide overload. | The fix: **open-loop**, constant arrival rate (k6's `constant-arrival-rate`). |
| e | One metric | Throughput alone missed delivery rate, latency, errors and memory. | Rust 1% of ~6–7k events vs Elixir 100% of ~1.7k: not comparable counts. |
| f | No decision on failure | Nobody chose between "drop and let the client reconnect" and "deliver late". | The author prefers fail-fast: an event 11 s late is worse than no event. |

The deeper lesson is (f). The others are what (f) looks like when it is left
to chance.

---

## 3. Where the pkcore family stands

### 3.1 pkcore (the kernel): safe by construction, with two sharp edges

The kernel cannot block on I/O. Default features carry no tokio, no database
and no filesystem (`Cargo.toml:52-57`). `rusqlite` is reachable only through
the opt-in `store` feature (`Cargo.toml:89`), and `make check-purity`
(`Makefile:312-333`) fails the build if it leaks into the default tree.

The two sharp edges are **long CPU calls**, which are pattern (a) waiting for
a caller:

- `equity::compute` (`src/analysis/equity/engine.rs:91`): 3.3 ms for an exact
  heads-up flop, 136 ms for Monte Carlo three-way, 5.5 s for an exact
  heads-up preflop (`docs/perf/RESULTS.md:40-42`).
- `Solver::solve` (`src/analysis/gto/solver.rs:852`): ~1.6 ms per CFR
  iteration (`docs/perf/PROFILING.md:286`) times `max_iterations`.

Called from a tokio handler without `spawn_blocking`, either one blocks a
worker. With the opt-in `parallel` feature (`Cargo.toml:114`), it also blocks
until a rayon pool finishes. pkcore cannot fix that. It can only **say it**, in
the rustdoc of those two functions. That is a cheap, real item.

One smaller edge: `HUP_CACHE` (`src/analysis/store/embedded/hup_cache.rs:5-8`)
is a `LazyLock` that decodes a 15.8 MB blob (`Cargo.toml:128`) the first time
it is touched. The first request that touches it pays a one-time spike. A
service should warm it at start-up.

### 3.2 pkdealer (the table service): four of six patterns present

pkdealer runs **one table per process** (`pkdealer/crates/pkdealer_service/src/main.rs:2829`),
as ROADMAP decided (`ROADMAP.md:733-736`). Table state is
`Arc<std::sync::Mutex<TableState>>` (`main.rs:52`, `:493-498`, `:560-564`).
Every RPC takes that lock.

One lock per table is the right shape. A table is serial by nature: only one
player acts at a time. The problem is **what runs while the lock is held**:

| Pattern | Evidence |
|---|---|
| (a) blocking I/O in an async handler, under the lock | At the end of every hand, `act` rewrites the **whole** session as YAML with `std::fs::create_dir_all` and `std::fs::write` (`main.rs:1152-1162`, called at `:2131-2143`). The cost grows with session length. The demo stack turns it on (`docker-compose.yml:41`). |
| (a) again | `export_session` serialises the whole recorder under the lock (`main.rs:2454-2480`). The Boss agent calls it after every new hand (`pkdealer_agent_boss/src/app.rs:210-224`, `:249`). |
| (a) the lock kind | The lock is `std::sync::Mutex`, so a waiter blocks its OS worker thread instead of yielding. No `spawn_blocking` exists in the workspace. |
| (b) bounded, not counted | `EVENT_CHANNEL_CAPACITY = 1024` (`main.rs:100`). On `Lagged` the stream logs and continues (`main.rs:2767-2779`). This is a good choice: every event carries a full `current_status` snapshot, so the next event heals the gap. But no metric counts the skipped events, so **delivery rate cannot be seen**. |
| (b) unbounded | The in-memory recorder has no cap by default (`main.rs:159-164`), and compose does not set `PKDEALER_RECORD_MAX_HANDS`. When a cap is set, trimming is `hands.remove(0)` in a loop, O(n) each (`main.rs:2147-2151`). |
| (e) one metric | The OTel instruments (`main.rs:427-487`) are game metrics. `action_duration_ms` is **agent think time**, not server latency (`main.rs:1780-1811`). There is no RPC latency, lock wait, lagged count, error rate or memory gauge. |
| (f) no failure policy | The tonic server has no timeout and no concurrency limit (`main.rs:2842-2847`). |

What pkdealer gets right: the guard is never held across an `.await`, and the
`Lagged` handling is deliberate and documented.

### 3.3 pkspectator: the article's exact number

`SSE_CHANNEL_CAPACITY = 256` (`pkspectator/src/state.rs:18`), the same
capacity as the Rust Campfire rewrite. Each SSE client gets a 64-slot mpsc
(`pkspectator/src/handlers.rs:36-38`). On `Lagged` the handler does nothing at
all: no log, no metric (`handlers.rs:272-274`, comment *"a gap is
acceptable"*). Like pkdealer, events carry a snapshot, so the client heals.
Unlike the Campfire rewrite, a slow client is **never disconnected**. There
are no metrics in pkspectator.

So the *policy* (drop and resync from a snapshot) is sound and was chosen on
purpose. What is missing is the **evidence** that it works under load.

### 3.4 cardroom: the one load harness, and it is closed-loop

`../cardroom` runs N sit-and-go tables across W threads to completion
(`cardroom/src/pool.rs:40-60`). It reports hands/s, CPU µs per action, peak
RSS and per-table p50/p90/p99 together (`cardroom/src/metrics.rs:176-183`,
`cardroom/src/report.rs:20-44`). That is already lesson (e) done right.

It is closed-loop batch: run as fast as possible, then stop. It has no network,
no service and no arrival rate, by design
(`cardroom/docs/superpowers/specs/2026-08-15-cardroom-v1-design.md:28-31`).
Its backlog already wants a live, continuous mode (`cardroom/BACKLOG.md`, items
2 and 3).

### 3.5 Databases

There is no database in any hot path in the family. pkdealer persists only to
the YAML sink. `pkdb` is a MySQL image holding one precomputed heads-up equity
table (`pkdb/scripts/init.sql:5-17`) with no application code. pkcore's
`PokerSession::snapshot`/`restore` (`src/casino/session.rs:693`, `:713`,
[EPIC-88](../epics/EPIC-88_Table_Snapshot.md)) is the natural row format for a
table store, and nothing uses it yet.

### 3.6 Summary

| | pkcore | pkdealer | pkspectator | cardroom |
|---|---|---|---|---|
| (a) blocking in async | n/a (no async), but long calls are a trap for callers | **yes** | no | n/a |
| (b) queue policy | n/a | bounded + resync, **not counted**; recorder **unbounded** | bounded + resync, **silent** | n/a |
| (c) one serial process | n/a | one lock per table — fine; I/O inside it — not fine | one fan-out task | n/a |
| (d) closed-loop only | perf harness | agents wait to be prompted | — | **yes** |
| (e) one metric | perf: good | **game metrics only** | **none** | good |
| (f) failure decided | — | **no** | partly (drop + resync) | — |

---

## 4. How to test so a problem points at its cause

The article's authors could not tell whether a result came from the language,
the queue size, or the prompt. That is a test-design failure: too many things
changed at once. The fix is a **ladder**. Each rung adds exactly one thing to
the rung below. If a problem appears at rung N and not at rung N−1, the thing
rung N added is the cause.

| Rung | What runs | What it adds | Tool today | Tool needed |
|---|---|---|---|---|
| 0 | One kernel call in a loop | nothing — pure CPU | `../../perf` (`make perf-native`) | re-baseline: results are from pkcore 0.3.2 (`docs/perf/RESULTS.md:11`) |
| 1 | Many tables, many threads, no I/O | thread contention, memory | `cardroom` | — |
| 2 | One service, in-process, no network | the async runtime, the lock, the event channel | — | a tokio test that drives `DealerService` methods directly |
| 3 | One service over gRPC, open-loop | the network, tonic, back-pressure on the wire | — | an open-loop driver (§4.2) |
| 4 | Many tables + subscribers | fan-out, slow consumers | — | the driver + deliberate slow subscribers |
| 5 | Many tables + a database | storage latency, pool limits | — | the lab (§6) |

Two rules make the ladder work:

1. **Same workload at every rung.** Use one seeded hand script. cardroom's
   backlog item 1 notes that cards are not reproducible today because pkcore
   shuffles from the thread-local RNG. pkcore already takes a caller's RNG
   (`Table::act_shuffle_deck_with`, `src/casino/table.rs:1621`), so a host
   under test should shuffle from a seeded RNG. Without this, two runs are two
   different experiments.
2. **Same five numbers at every rung** (§4.3), so rungs can be compared.

### 4.1 Probes that find blocking

Pattern (a) is invisible in a throughput number. It shows up as **stalls**.
Three probes catch it, from cheapest to richest:

- **The heartbeat task.** Spawn one tokio task that sleeps 1 ms in a loop and
  records how late it wakes. If any other task blocks a worker, the heartbeat
  wakes late on that worker. Report p99 and max lateness. This is the same
  test EPIC-92 uses on the BEAM (`dirty_solve_does_not_starve_heartbeat`). It
  is runtime-agnostic, so it also lets the Rust and Elixir arms be compared
  honestly.
- **Lock hold and wait histograms.** Time how long each `lock()` waits and how
  long the guard lives. pkdealer already funnels every lock through one
  function (`main.rs:560-564`), so this is one place to change.
- **`tokio-console` / `tokio::runtime::RuntimeMetrics`.** Shows tasks that run
  long between yields, and worker busy time. Heavier. Use it once the
  heartbeat says "something blocks" and you need to know what.

A good probe test also **fails on purpose once**. Add one test that puts a
`std::thread::sleep(10ms)` in a handler and assert the heartbeat sees it. A
probe that has never caught anything has not been shown to work.

### 4.2 Open-loop load, and the latency lie

A closed-loop client sends request N+1 only after reply N. If the server slows
down, the client slows down with it, and the measured latency stays nice.
That is **coordinated omission**: the slow moments are exactly the ones the
test forgot to measure.

An open-loop driver sends on a clock, whatever the server does:

- Fire at a fixed rate (say 50 actions/s per table), from a tokio interval
  with `MissedTickBehavior::Burst`, one spawned task per request.
- Measure latency from the **intended** send time, not the actual send time.
  If the driver itself falls behind, that delay counts against the server.
- Record into an HDR histogram so p99, p99.9 and max are exact.
- Report **offered** rate and **achieved** rate side by side. When they split,
  the system is past saturation, and the experiment has found its knee.

Poker adds one wrinkle: actions at one table are ordered. A player cannot act
out of turn. So "open-loop" here means **open-loop across tables and
subscribers**: tables start and act on a schedule, and each table's agents act
after a fixed think time, not "as soon as prompted". Spectators connect and
disconnect on their own clock. The pressure comes from the number of tables
and subscribers, which is the shape of the real traffic anyway.

k6 can do gRPC, including streams, and has the `constant-arrival-rate`
executor the article names. A small Rust driver can reuse pkdealer's own
client crate. Recommendation: **write the Rust driver**. It speaks the real
proto, it can pass a seed to a test-mode host, and it puts the heartbeat probe
in the same binary.

### 4.3 The five numbers

Every load run reports all of these, or it is not a result:

1. **Throughput** — achieved actions/s and hands/s, next to offered rate.
2. **Delivery rate** — events received ÷ events sent, per subscriber, and the
   count of `Lagged` skips. This needs the counter pkdealer and pkspectator
   lack today.
3. **Latency** — p50, p99, max, from intended send time. For events, time
   from `emit_event` to client receipt.
4. **Errors** — RPC errors and timeouts, by status code.
5. **Memory** — peak RSS, and the size of every queue that can grow.

Plus the run's provenance: commit, machine, seed, rate — the way cardroom's
YAML reports already do.

### 4.4 Write the failure policy down first

Before any load run, each queue and lock gets a row in a table like this. The
load run then checks the code against the table, not the other way round.

| Component | Bound | On overflow | Metric | Owner |
|---|---|---|---|---|
| pkdealer event broadcast | 1024 | skip, resync from snapshot | `pkdealer.events.lagged` (new) | pkdealer |
| pkdealer recorder | **none** → proposed default cap | drop oldest hand | `pkdealer.recorder.hands` gauge (new) | pkdealer |
| pkdealer RPC concurrency | **none** → proposed limit | `RESOURCE_EXHAUSTED` | `pkdealer.rpc.rejected` (new) | pkdealer |
| pkdealer RPC time | **none** → proposed timeout | `DEADLINE_EXCEEDED` | `pkdealer.rpc.duration` histogram (new) | pkdealer |
| pkspectator SSE broadcast | 256 | skip, resync | `pkspectator.sse.lagged` (new) | pkspectator |
| pkspectator per-client buffer | 64 | today: back-pressure forever. Proposed: disconnect after N s behind, client reconnects | `pkspectator.sse.disconnects` (new) | pkspectator |
| `PkCore.TableServer` subscriber (EPIC-92) | queue length 1,000 | drop + `:resync` | telemetry event | pkcore.ex |

The article's author prefers fail-fast. So does this table. The point is not
which answer is right. The point is that the answer is **written down before**
the load test, so the load test can prove or break it.

---

## 5. Quick wins (no lab needed)

These are small, local and testable at rung 2. Each would be its own pkdealer
or pkspectator change, with its own CHANGELOG and version bump in that repo.

1. **Move the YAML write out of the lock and off the worker.** Clone the
   finished hand while holding the lock, then hand it to one writer task over a
   bounded mpsc; that task uses `spawn_blocking` or `tokio::fs`. Write only the
   new hand (append), not the whole session.
2. **Count lagged events** in pkdealer (`main.rs:2767-2779`) and pkspectator
   (`handlers.rs:272-274`). One counter each. This turns "a gap is acceptable"
   from a belief into a number.
3. **Cap the recorder by default** and use a `VecDeque` so trimming is O(1)
   (`main.rs:2147-2151`).
4. **Add server latency, not just think time.** A `tower` layer with an RPC
   duration histogram, plus `tower::limit::ConcurrencyLimitLayer` and a
   timeout on `Server::builder()` (`main.rs:2842`).
5. **pkcore rustdoc:** add a "This is a long CPU call. In async code, run it
   with `spawn_blocking`." note to `equity::compute` and `Solver::solve`, and a
   "warm this at start-up" note on the embedded HUP cache. This is the only
   item in pkcore itself.

---

## 6. Future work: the multi-table lab

The quick wins fix what we know. The lab is for **learning what we don't**:
building a multi-table install with real storage, then breaking it on purpose,
one variable at a time, and watching the five numbers.

### 6.1 Shape

```
                      ┌──────────────── open-loop driver ───────────────┐
                      │ seeded decks · fixed rate · HDR · heartbeat     │
                      └──────┬──────────────────────────────┬───────────┘
                         gRPC│ actions                  SSE │ slow + fast spectators
               ┌─────────────▼─────────────┐      ┌─────────▼─────────┐
               │ table host (one of):      │─────▶│ pkspectator       │
               │  A. pkdealer, 1 table/pod │events└───────────────────┘
               │  B. pkdealer, N tables,   │
               │     one task per table    │
               │  C. Elixir, EPIC-92,      │
               │     one GenServer/table   │
               └─────────────┬─────────────┘
                             │ snapshot per hand (EPIC-88 bytes)
               ┌─────────────▼─────────────┐
               │ store (one of):           │
               │  none · SQLite (WAL) ·    │
               │  Postgres (sqlx pool)     │
               └───────────────────────────┘
          OTel → Prometheus / Grafana / Jaeger (EPIC-22 stack, plus table_id label)
```

Three table hosts and three stores give a grid. Every cell runs the same
seeded workload and reports the same five numbers. Host C is the comparison
the article tried to make, done fairly: **the same written failure policy
(§4.4) in both languages**, so a difference in results is a difference in
runtime, not in unstated choices.

### 6.2 Storage design notes

- **Write snapshots per hand, not per action.** A hand is the natural
  transaction. Writing per action puts storage latency in the betting path.
  The cost: a crash loses at most the current hand. EPIC-88's guarantee that a
  resumed hand gives the same `Winnings` makes "resume from last hand" exact.
- **Never call a synchronous driver from a tokio worker.** `rusqlite` is
  synchronous. Use one dedicated DB thread fed by a **bounded** channel, or
  `spawn_blocking`. With Postgres, use an async pool (`sqlx`), and treat the
  pool size as a queue bound: set an acquire timeout, and count timeouts.
- **SQLite is single-writer, and that is fine for writes.** One writer task
  for all tables is the correct design for SQLite in WAL mode. Reads must not
  queue behind it: give them their own connections. This is pattern (c) turned
  the right way round: serialise only what is serial by nature.
- **Snapshot bytes hold the undealt deck** (EPIC-88). The store is private
  data. No spectator path may read it.

### 6.3 Experiments

Each experiment changes one thing and has a stated expected result. If the
result differs, that is the finding.

| # | Change | Expect | Article link |
|---|---|---|---|
| E1 | Put a 50 ms blocking write in the per-hand path, host B | heartbeat p99 jumps; all tables on that worker slow down, not just the writer | (a) |
| E2 | Same as E1 with `spawn_blocking` | heartbeat flat; only that table's hand-end latency rises | (a) |
| E3 | 1 slow spectator among 100, broadcast capacity 256 vs 16,384 | small: drops, low latency. Large: no drops, high max latency, more memory | (b), the 11 s vs 130 s trade |
| E4 | Same as E3 on host C with an unbounded mailbox vs the §4.4 cap | unbounded: memory grows without limit | (b), the 1.8 GB case |
| E5 | All tables' reads and writes through one SQLite connection vs one writer + read pool | one connection: read latency follows write latency | (c) |
| E6 | Closed-loop vs open-loop driver at the same nominal load | closed-loop p99 looks fine; open-loop finds the knee | (d) |
| E7 | Raise table count until offered ≠ achieved; report all five numbers at the knee | the first number to break names the bottleneck | (e) |
| E8 | Kill a table host mid-hand; restore from the store | every table resumes; chips conserved | EPIC-88 |
| E9 | Run E1–E7 on hosts A, B and C with the same §4.4 policy | differences that remain are runtime differences | the article's comparison, done fairly |

### 6.4 Who owns what

| Work | Repo | Form |
|---|---|---|
| Quick wins 1–4 | pkdealer, pkspectator | defect or small EPIC in that repo's block (pkdealer: next free `EPIC-71`, `ROADMAP.md:417`) |
| Quick win 5 | pkcore | doc-only change, patch bump |
| Re-baseline `../../perf` on 0.15.x | pkcore | `make perf-native` |
| cardroom live / open-loop mode | cardroom | its backlog items 2–3 |
| Open-loop driver + heartbeat probe | pkdealer (new crate) or a new repo | EPIC |
| Multi-table host B, table registry, store | pkdealer | EPIC; built on EPIC-88 |
| Host C | pkcore.ex | [EPIC-92](../epics/EPIC-92_Elixir_Bindings.md) Phase 4, then a server repo |
| The lab itself (compose, dashboards, experiment runbooks) | new repo, or pkdealer `ops/` | EPIC |

No EPIC number is claimed for the lab yet. When one of these starts, it takes
a number from its repo's block per `../../ROADMAP.md` "EPIC Numbering Policy".

---

## 7. Open questions

1. pkspectator's gRPC `Endpoint` has `.timeout(30s)` (`pkspectator/src/grpc.rs:103`).
   Does that also cut the long-lived `StreamEvents` call? Not verified. If it
   does, spectators reconnect every 30 s, which hides lag in a different way.
2. Should a slow spectator be disconnected (the article's choice) or kept and
   resynced (ours)? Ours costs nothing extra because every event carries a
   snapshot. The question is whether a spectator many seconds behind is worse
   than a reconnect. E3 can answer it.
3. Multi-table: many tables in one pkdealer process (host B), or one table per
   pod behind a router (host A)? B shares memory and a runtime; A isolates
   failure. The lab measures both before ROADMAP picks one.
4. pkdealer pins pkcore **0.11.0** (`pkdealer/crates/pkdealer_service/Cargo.toml:22`),
   which predates the 0.15 lean defaults. Any lab work should start by moving
   it to 0.15.x, so the kernel under test is the current one.
