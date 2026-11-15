# EPIC-125: Dioxus Spike — pkcore apps in one Rust UI (DXS)

> **Provenance.** Drafted 2026-11-02 against pkcore
> `deps/actions-bump-0.20.7` @ `c07b4f95` (`0.20.7`, `Cargo.toml:4`). A
> **spike**: its product is a set of throwaway example apps and a written
> verdict, not a pkcore API. Dioxus facts are from the upstream release
> notes listed under [References](#references) and are re-checked in
> Phase 0.

## Summary

- **Builds:** six small Dioxus example apps over pkcore — a card lab, a
  range painter, an odds board, a heads-up table against a bot, a Kuhn CFR
  trainer, and a server-backed rail — in a standalone `spikes/dioxus/`
  workspace, plus a findings section with a go / no-go verdict.
- **Why:** every pkcore UI today pairs Rust with a JavaScript layer
  (`wasm-bindgen` + vanilla JS or a JS framework), and EPIC-137 has an open
  question whether a Rust UI could serve instead
  (`docs/epics/EPIC-137_The_App.md:68`). No one has measured it.
- **Shape:** each app is a thin view over one pkcore read model
  (`SessionView`, `EquityReport`, `Eval`, `Combos`, `KuhnCfr`). The app
  never computes poker; pkcore does. The spike sits outside the pkcore
  build, the way `perf/` does, so no Dioxus crate enters pkcore's lock file
  or its purity gate.
- **Proves it:** `make dioxus-check` (fmt, clippy, tests of each app's
  model and its SSR render), a `dx bundle` per target, and a measured
  findings table (payload size, time-to-interactive, main-thread stall).
- **Status:** Planned. Nothing built.

---

## Context

**The kernel is UI-ready.** A UI loop needs four things, and pkcore has
each of them:

- A session facade that steps a hand: `PokerSession`
  (`src/casino/session.rs:126`), `next_step() -> SessionStep`
  (`src/casino/session.rs:623`, enum at `:78`), `apply_action`
  (`src/casino/session.rs:582`).
- A redacted read model for one viewer: `PokerSession::view(Option<Principal>)
  -> SessionView` (`src/casino/session.rs:951`, struct at `:1101`;
  `Principal` at `src/casino/principal.rs:33`).
- The questions a seat asks: `Table::legal_actions`
  (`src/casino/table/transition.rs:63`), `Table::raise_bounds`
  (`src/casino/table/actions.rs:365`) for a bet slider, `Table::next_to_act`
  (`src/casino/table.rs:988`).
- Seeded twins for every random step, so a UI can replay:
  `start_hand_with(rng)` (`src/casino/session.rs:352`),
  `start_hand_with_deck(deck)` (`:401`), `EquityOptions.seed`
  (`src/analysis/equity/spec.rs:61`).

**pkcore already builds for the browser.** `make check-wasm` runs three
`cargo check --target wasm32-unknown-unknown` passes (`Makefile:359-362`),
and CI runs it (`.github/workflows/basic.yaml:198-215`). OS randomness on
wasm comes from `getrandom` 0.3 with `wasm_js` (`Cargo.toml:182-184`).
`parallel` (rayon) does not build for wasm (`Cargo.toml:113-118`).
`hup-charts` builds but embeds a 15.8 MB chart; one wasm consumer measured
an 8x larger download with it (`Cargo.toml:132-141`).

**Every UI today is Rust + JavaScript, or Rust + a terminal.**

| Consumer | Stack | Source |
|---|---|---|
| pkarena0-web | `#[wasm_bindgen]` + HTML/JS table | `docs/epics/EPIC-34_Variant_Web_Selection.md:10-11` |
| pkwasm | `wasm-pack --target web`, plain static HTML examples, no UI shipped | `docs/epics/EPIC-86_Browser_Bindings.md:109-111` |
| pkgto-web | 627-line vanilla `www/index.html`; planned: TypeScript + Svelte 5 | `docs/epics/EPIC-137_The_App.md:34`, `:66-70` |
| pksolver-wasm | `wasm-bindgen` `.d.ts` + Web Worker + TS client | `docs/epics/EPIC-135_Browser_Facade.md:10-23` |
| pktui | ratatui | `docs/epics/EPIC-60_Showcase.md:41` |
| Spectator | Axum + SSE; frontend not chosen | `docs/epics/EPIC-21_Spectator.md:9-13` |
| Mobile engine | UniFFI to Swift/Kotlin | `docs/epics/EPIC-37_Mobile_Engine.md:7-12` |

The only mention of Dioxus in the repo is the EPIC-137 line above, which
names the two risks of a Rust UI: "no grid/chart ecosystem and a second
wasm payload".

**The examples are all terminal.** `examples/interactive_play.rs:1` (reedline,
you vs eight bot profiles), `examples/kuhn_repl.rs:1`,
`examples/dealer_repl.rs:1`, `examples/bcrepl.rs:39`. The full-screen TUI
was retired (`examples/retired/tui.rs:18` is an empty `main`).

**Dioxus, as of this draft.** Stable is the 0.7 line (0.7.9 on docs.rs,
built 2026-05-08); `0.8.0-alpha.0` is out with breaking changes to Signals
and the CLI. 0.7 brought Subsecond hot-patching of Rust code under
`dx serve`, a fullstack rewrite on Axum with server functions, and Dioxus
Native, a GPU renderer built on Blitz. One `rsx!` codebase targets web
(wasm), desktop (system webview), mobile (iOS/Android webview), server
(fullstack/SSR) and native (Blitz).

**Does not do:**

- Change pkcore. No new feature, type or dependency lands in `src/`. Gaps
  the spike finds go to the **Lift list** (below) as follow-on EPICs.
- Replace a shipped UI. pkarena0-web, pkwasm, pktui and bowery are
  untouched. The verdict may *recommend* Dioxus for EPIC-137 or EPIC-21;
  it does not migrate them.
- Ship a product. The apps are examples: no accounts, no persistence
  beyond what a finding needs, no deploy.
- Add a CI job. Like `perf/`, the spike is checked locally only.

---

## Status

| Component | Status |
|---|---|
| Phase 0 — workspace, version pin, component-shape decision | Planned |
| Phase 1 — `card-lab` (web) | Planned |
| Phase 2 — `range-painter` (web) | Planned |
| Phase 3 — `odds-board` (web + desktop) | Planned |
| Phase 4 — `heads-up` (desktop + web, mobile build) | Planned |
| Phase 5 — `kuhn-trainer` (web) | Planned |
| Phase 6 — `rail` (fullstack) | Planned |
| Phase 7 — findings, verdict, Lift list | Planned |

---

## Goals

- Measure, not guess, whether **one Rust UI codebase** can carry pkcore to
  web, desktop and mobile.
- Keep the **kernel/shell split**: pkcore decides, the app displays. No
  poker rule is re-implemented in a component.
- Answer EPIC-137 Decision 1 with a **working range editor prototype**,
  not an opinion.
- Find the **redaction boundary** in a fullstack app: the server holds the
  `PokerSession`, a client only ever sees a `SessionView`.
- Leave a **Lift list**: the pkcore gaps that a UI exposed.

## Scope

The rules every example app obeys:

1. **pkcore computes.** A component may format a number. It may not decide
   a legal action, rank a hand, or compute an equity. It calls pkcore.
2. **One model per app.** Each app has a plain Rust `…Model` struct that
   owns the pkcore state and exposes the read model the view needs. The
   model has no Dioxus import and is unit-tested with `cargo test`.
3. **The view reads a read model.** Table apps render
   `SessionView`, never `Table` fields. A snapshot (`Table::snapshot`,
   `src/casino/table.rs:3174`) never reaches a client: it carries the
   undealt deck in order (`src/casino/table/snapshot.rs:11-14`).
4. **Seeded by default.** Each app takes a seed (URL parameter on web, CLI
   flag on desktop) and uses the seeded twins. A bug report is a seed.
5. **No main-thread stall over 100 ms.** Long work (equity Monte Carlo,
   CFR training) runs in chunks that yield to the event loop. Where that
   is not enough, the finding says so.
6. **Lean features.** Each app names the smallest pkcore feature set it
   needs. `hup-charts` is opt-in per app and its size is measured.
7. **House rules hold.** No `unwrap`/`expect`/`panic!` outside tests; no
   free functions except as decided in Phase 0; test names without
   `test_`; test modules named `<path>__<type>_tests`.

---

## Domain map

The Things a UI touches, and where each one lives.

| Domain concept | pkcore construct | App | Status |
|---|---|---|---|
| Cards typed by a person | `impl FromStr for Cards` (`src/cards.rs:940`) | card-lab | ✅ exists |
| A ranked hand | `HandRanker::eval` → `Eval` (`src/arrays/mod.rs:92`, `src/analysis/eval.rs:210`) | card-lab | ✅ exists |
| A range as text | `impl FromStr for Combos` (`src/analysis/gto/combos.rs:333`) | range-painter | ✅ exists |
| A range as a 13×13 grid | `RANGE_MATRIX` labels only (`src/analysis/gto/twos.rs:34`) | range-painter | 🟡 labels, no `Combos` ↔ grid |
| A range written back as text | `impl Display for Combos` (`src/analysis/gto/combos.rs:294`) | range-painter | 🟡 lists combos in `HashSet` order, not `66+, AJo` |
| An equity question | `EquityRequest` + `dead` (`src/analysis/equity/spec.rs:139`, `:150`) | odds-board | ✅ exists |
| A hand in play | `PokerSession` / `SessionStep` (`src/casino/session.rs:126`, `:78`) | heads-up, rail | ✅ exists |
| What one viewer may see | `SessionView` (`src/casino/session.rs:1101`) | heads-up, rail | ✅ exists |
| A bet slider's range | `Table::raise_bounds` (`src/casino/table/actions.rs:365`) | heads-up | ✅ exists |
| A bot's choice | `BotProfile::decide` (`src/bot/profile.rs:967`) | heads-up, rail | ✅ exists |
| A strategy being learned | `KuhnCfr::train` / `exploitability` (`src/games/kuhn.rs:878`, `:963`) | kuhn-trainer | ✅ exists |

---

## Design

### The workspace

`spikes/dioxus/` (new), its own workspace root, the `perf/` pattern:

```toml
# spikes/dioxus/Cargo.toml
[workspace]
resolver = "3"
members = ["card-lab", "range-painter", "odds-board",
           "heads-up", "kuhn-trainer", "rail"]

[workspace.dependencies]
dioxus = { version = "=0.7.9" }      # exact pin, re-checked in Phase 0
pkcore = { path = "../..", default-features = false }
```

Why outside the pkcore package: pkcore's root `Cargo.toml` has no
`[workspace]` table, so a crate with its own `[workspace]` is its own
root. Dioxus pulls a large tree (webview, Axum, wgpu for native). Kept
outside, none of it enters pkcore's `Cargo.lock`, `make check-purity`
(`Makefile:419`) or `make ayce` (`Makefile:292`). Graduation to its own
repo (`pkdioxus`) is a Phase 7 decision, not a Phase 0 one.

### The component-shape question (decide in Phase 0)

Dioxus components are written as free functions:

```rust
#[component]
fn Board(cards: String) -> Element { rsx! { /* … */ } }
```

The global Rust rule puts every function in an `impl` block. `#[component]`
generates a props struct (`BoardProps`) next to the function, and a struct
cannot be declared inside an `impl`, so the macro probably cannot be put on
an associated function. Phase 0 tests this. Two outcomes:

```rust
// Option A — components without the macro, as associated fns.
pub struct CardLab;
impl CardLab {
    pub fn app() -> Element {
        let mut input = use_signal(String::new);
        rsx! { input { oninput: move |event| input.set(event.value()) }
               { Self::verdict(&input.read()) } }
    }
    fn verdict(text: &str) -> Element { /* calls CardLabModel */ }
}

// Option B — a scoped exception: `#[component]` fns are allowed, like
// `fn main()` and serde `with` helpers, because the framework requires them.
```

Recommendation: try A first. Plain functions that return `Element` cannot
hold their own hooks safely when called conditionally, so A holds only
while each `Self::part()` is called unconditionally. If A forces hook
gymnastics, record that as a finding and ask for exception B.

### The model / view split

Every app follows one shape, so the findings compare like with like:

```rust
// card-lab/src/model.rs — no Dioxus import; unit-tested with cargo test.
pub struct CardLabModel;

impl CardLabModel {
    /// Parses "As Ks Qs Js Ts" (5 or 7 cards) and ranks it.
    pub fn rank(text: &str) -> Result<RankedHand, CardLabError> { /* … */ }
}

pub struct RankedHand {
    pub label: String,     // from HandRank's Display
    pub best_five: String, // from Eval
}
```

The view reads `RankedHand`. The test reads `RankedHand`. A Dioxus upgrade
cannot break the poker; a pkcore change shows up in the model tests first.

### View tests without a browser

`dioxus-ssr` renders a component to an HTML string inside `cargo test`.
Each app gets at least one render test that asserts the text a person
would see:

```rust
#[cfg(test)]
#[allow(non_snake_case)]
mod card_lab__view_tests {
    #[test]
    fn royal_flush_renders_its_name() {
        let html = /* VirtualDom::new_with_props(..) → dioxus_ssr::render */;
        assert!(html.contains("Royal Flush"));
    }
}
```

Browser end-to-end tests are out of scope for a spike.

### The six example apps

Ordered smallest first. Each later app reuses what the earlier ones proved.

#### 1. `card-lab` — web

Type cards, see the hand rank and the best five. Things: `Cards`, `Five`,
`Seven`, `Eval`. Features: `--no-default-features` (pure kernel).

Question it answers: **the floor.** What is the smallest wasm payload and
first-paint time for a Dioxus app that links pkcore at all? This is the
baseline every other app is measured against, and the number to compare
with pkwasm's 64.7 KB gzipped (ROADMAP EPIC-86 row).

#### 2. `range-painter` — web

A 13×13 grid. Click or drag to paint hands in or out. A text box shows the
range and accepts pasted ranges. A preset menu (e.g. "22+, A2s+, KTo+").
Things: `Combos`, `RANGE_MATRIX`. Features: pure kernel.

Questions it answers:

- **EPIC-137 Decision 1.** Is a Rust-only range editor as good as the
  Svelte prototype that EPIC-137 Phase 0 plans? Same feature list, same
  keyboard path.
- **Two kernel gaps.** The grid needs `Combos` ↔ 13×13 cells, and the text
  box needs `Combos` → `"66+, AJo"`. Neither exists (Domain map). The app
  writes both in its model; Phase 7 decides whether they lift into pkcore.

#### 3. `odds-board` — web and desktop

Pick two to six players (exact hands, ranges, or random), a board, and dead
cards. See each player's win/tie share. Things: `EquityRequest`,
`PlayerSpec` (`src/analysis/equity/spec.rs:18`), `EquityOptions`,
`EquityReport`. Features: `equity`.

Questions it answers:

- **The main-thread stall.** `EquityEngine::compute`
  (`src/analysis/equity/engine.rs:117`) switches to Monte Carlo above
  `exact_threshold` (default 100,000, `src/analysis/equity/spec.rs:61`).
  On wasm there is no rayon. How long does one call block, and does
  chunking by `max_samples` keep the page live?
- **The same code on desktop.** The desktop build links pkcore natively,
  so it can turn `parallel` on. How much faster is the same screen?

```rust
pub struct OddsModel { request: EquityRequest }

impl OddsModel {
    /// Runs `samples` more draws and merges them into the running report.
    /// Called from a spawned task in slices, yielding between slices.
    pub fn step(&mut self, samples: usize) -> Result<EquityReport, PKError>;
}
```

Whether `step` can be built on the public API (fresh seed per slice,
merge reports) or needs a pkcore hook is itself a finding. If it needs a
hook, it goes on the Lift list next to EPIC-38's planned
`compute_with_progress`.

#### 4. `heads-up` — desktop and web; mobile build

You against one `BotProfile`, No-Limit Hold'em. Buttons from
`legal_actions`, a bet slider from `raise_bounds`, the bot answers through
`BotProfile::decide`. The screen renders `SessionView` for your
`Principal`, so the bot's cards are hidden by pkcore, not by CSS. Features:
`bot-profiles` (for `BotProfile::default_profiles`,
`src/bot/profile.rs:509`).

This is the Dioxus port of `examples/interactive_play.rs`, cut to heads-up.

```rust
pub struct HeadsUpModel {
    session: PokerSession,
    hero: Principal,
    villain: BotProfile,
    rng: ChaCha8Rng,          // seeded; the seed is shown in the footer
}

impl HeadsUpModel {
    pub fn view(&self) -> SessionView;                       // what you see
    pub fn act(&mut self, action: PlayerAction) -> Result<(), PKError>;
    pub fn advance(&mut self) -> Result<SessionStep, PKError>; // runs the bot
}
```

Questions it answers:

- **One codebase, three shells.** Does the same crate run under
  `dx serve --platform desktop`, `--platform web`, and build with
  `dx bundle` for iOS and Android? What differs?
- **Against EPIC-37.** EPIC-37 plans UniFFI and a native Swift/Kotlin UI.
  Is a Dioxus mobile build a usable alternative for a demo app, and at what
  binary size?
- **Hot-patching.** Does Subsecond patch the view while a hand is in
  progress, or does pkcore's compile time make it moot?

#### 5. `kuhn-trainer` — web

Press Train. `KuhnCfr` (`src/games/kuhn.rs:827`) trains in slices of
iterations; a line chart of `exploitability()` falls toward zero; a table
shows `average_strategy` (`src/games/kuhn.rs:908`) per information set. A
"Play it" tab lets you play against the trained strategy, as
`examples/kuhn_repl.rs` does. Features: pure kernel.

Questions it answers:

- **The chart claim.** EPIC-137 says a Rust UI has no chart ecosystem. This
  app draws its chart as inline SVG from `rsx!`, with no library. Is that
  enough for the charts pkcore apps need?
- **Long compute, second shape.** CFR is many short iterations, not one
  long call. Does the slice-and-yield pattern from `odds-board` carry over
  unchanged? If both apps need a worker, the finding names it.

`Solver::iterate` (`src/analysis/gto/solver.rs:810`) is the same shape for
the full solver; the trainer stays on Kuhn so the spike does not depend on
the pksolver block (EPIC-130–139).

#### 6. `rail` — fullstack

A server runs a six-handed table of bots in a loop. Browsers join as rail
birds and watch. The server holds the `PokerSession`; each client receives
`SessionView` for `None` (a spectator) through a server function or a
streamed endpoint. Features: `bot-profiles`, `entropy` on the server only.

```rust
// Runs on the server only. The client gets a SessionView, never the session.
#[server]
async fn rail_view(table_id: Uuid) -> Result<SessionView, ServerFnError>;
```

Questions it answers:

- **The redaction boundary in a fullstack build.** Dioxus compiles one
  crate twice (server and client). Can a `PokerSession` value or a snapshot
  byte leak into the client bundle by accident? The spike tries to make it
  leak, and records what stops it.
- **Against EPIC-21.** EPIC-21 plans Axum + SSE with no frontend chosen.
  Dioxus fullstack is Axum underneath. Is this the EPIC-21 frontend?
- **Serde shape.** `SessionView` must cross as JSON or another server-fn
  encoding. Does it already derive what is needed, or is that a Lift-list
  item?

### Findings (filled in Phase 7)

One table, one row per app and target:

| App | Target | pkcore features | Payload (gz) | First paint | Worst stall | Notes |
|---|---|---|---|---|---|---|
| card-lab | web | none | — | — | — | |
| … | | | | | | |

Then the verdict, one of:

- **Go** — name the downstream app that should use Dioxus, and open its
  EPIC.
- **Go, narrow** — name the targets where it wins (for example desktop and
  fullstack) and the ones where it loses (for example mobile).
- **No-go** — say why, so the question is not re-asked without new facts.

### Lift list (filled in Phase 7)

Gaps in pkcore that the apps had to work around. Each becomes a candidate
EPIC in the pkcore fourth block, or is dropped with a reason. Known
candidates before any code:

1. `Combos` ↔ 13×13 grid cells.
2. A canonical, compact `Combos` writer (`"66+, AJo"`), so a range
   round-trips through text.
3. Incremental equity (`compute_with_progress`, already planned in
   EPIC-38).

---

## Work Items

### Phase 0 — Workspace, version pin, component shape

- [ ] **0a.** Check crates.io and the Dioxus releases page. Pin the newest
  stable `dioxus` exactly. Record whether 0.8 is stable yet and why the pin
  was chosen.
- [ ] **0b.** Create `spikes/dioxus/Cargo.toml` (own `[workspace]`), a
  `README.md`, and an empty `card-lab` crate. Confirm `cargo metadata` at
  the pkcore root does not list it.
- [ ] **0c.** Add `make dioxus-check` (fmt + clippy `-D warnings` + test,
  run from inside `spikes/dioxus/`), modelled on `perf-check`
  (`Makefile:611`). Not added to `ayce`.
- [ ] **0d.** Settle the component shape: try `#[component]` on an
  associated fn, then Option A. Write the result in the README. If neither
  works, ask for exception B before Phase 1.
- [ ] **0e.** Install `dx` at the pinned version. Record the install
  command in the README.

### Phase 1 — `card-lab`

- [ ] **1a.** `CardLabModel::rank` with unit tests: royal flush, a 7-card
  hand, bad input returns `Err`.
- [ ] **1b.** View + one SSR render test.
- [ ] **1c.** `dx bundle --platform web --release`; record gzip size and
  first paint in the findings table.

### Phase 2 — `range-painter`

- [ ] **2a.** Model: `Combos` ↔ grid cells, compact writer, presets. Unit
  tests: `"22+"` paints the diagonal; `"AKs"` paints one cell; parse →
  write → parse is stable.
- [ ] **2b.** View: click, drag, keyboard (arrows + space). SSR render test
  for a preset.
- [ ] **2c.** Compare against the EPIC-137 Phase 0 feature list. Record the
  gaps.

### Phase 3 — `odds-board`

- [ ] **3a.** `OddsModel::step` on the public API. Unit tests: AA vs KK
  with a fixed seed lands near 0.82; a dead card changes the result.
- [ ] **3b.** View with a spawned, slice-and-yield task. Measure the worst
  stall with the browser's long-task timing.
- [ ] **3c.** Desktop build with `parallel` on. Record the speed-up.

### Phase 4 — `heads-up`

- [ ] **4a.** `HeadsUpModel` with unit tests: a seeded hand replays the same
  actions; the bot's cards are absent from `view()` until showdown;
  `act` with an illegal action returns `Err`.
- [ ] **4b.** View for desktop and web from one crate.
- [ ] **4c.** `dx bundle` for iOS and Android. Record success, size, and
  what broke.
- [ ] **4d.** Try Subsecond hot-patching mid-hand. Record what happens.

### Phase 5 — `kuhn-trainer`

- [ ] **5a.** Model: train in slices; unit test that exploitability falls
  below a bound after N iterations with a fixed seed.
- [ ] **5b.** SVG chart and strategy table in `rsx!`; play tab.

### Phase 6 — `rail`

- [ ] **6a.** Server loop of six bots; `rail_view` returns `SessionView` for
  `None`.
- [ ] **6b.** Leak test: assert the client wasm bundle contains no hole
  cards of a live hand and no snapshot bytes. Record how it was checked.
- [ ] **6c.** Two browsers watching one table stay in step.

### Phase 7 — Findings, verdict, Lift list

- [ ] **7a.** Fill the Findings table and write the verdict in this EPIC.
- [ ] **7b.** Fill the Lift list; open an EPIC (fourth block) for each item
  kept.
- [ ] **7c.** Answer EPIC-137 Decision 1 and EPIC-21's frontend question
  in those docs, citing this EPIC.
- [ ] **7d.** Decide: keep `spikes/dioxus/`, move it to a `pkdioxus` repo,
  or delete it.

---

## Test Plan

- `card_lab__model_tests::royal_flush_ranks_first` — the model calls
  pkcore's ranker and reports its name.
- `card_lab__model_tests::six_cards_is_an_error` — bad input is a typed
  `Err`, not a panic.
- `range_painter__model_tests::pairs_plus_paints_the_diagonal` — `"22+"`
  maps to the 13 pair cells.
- `range_painter__model_tests::compact_writer_round_trips` — parse →
  write → parse gives the same `Combos`.
- `odds_board__model_tests::aces_beat_kings_about_four_in_five` — seeded
  equity within tolerance of the known value.
- `odds_board__model_tests::dead_card_changes_equity` — pins EPIC-123's
  `dead` field through the app.
- `heads_up__model_tests::seeded_hand_replays` — same seed, same actions,
  same result.
- `heads_up__model_tests::villain_cards_hidden_before_showdown` — the
  redaction is pkcore's, through `view()`.
- `heads_up__model_tests::illegal_action_is_rejected` — the app does not
  pre-filter; pkcore says no.
- `kuhn_trainer__model_tests::exploitability_falls` — training moves toward
  equilibrium.
- One `<app>__view_tests` SSR render test per app.

## Key Files

| File | Role |
|---|---|
| `spikes/dioxus/Cargo.toml` | Standalone workspace root; exact Dioxus pin |
| `spikes/dioxus/README.md` | `dx` install, how to run each app, component-shape decision |
| `spikes/dioxus/<app>/src/model.rs` | The pkcore-facing model; no Dioxus import |
| `spikes/dioxus/<app>/src/main.rs` | The view and `fn main` |
| `Makefile` | `dioxus-check` target |
| `docs/epics/EPIC-125_Dioxus_Spike.md` | This doc; Findings and Lift list land here |

## Reuse (do NOT recreate)

- `src/casino/session.rs:126` — `PokerSession`: the hand loop. Do not drive
  `Table` by hand.
- `src/casino/session.rs:951` — `view()`: the redaction. Do not hide cards
  in the view layer.
- `src/casino/table/transition.rs:63` — `legal_actions`: the buttons.
- `src/casino/table/actions.rs:365` — `raise_bounds`: the slider limits.
- `src/bot/profile.rs:967` — `BotProfile::decide`: the bot.
- `src/analysis/equity/spec.rs:210` — `EquityRequest::compute`.
- `src/analysis/gto/combos.rs:333` — the range parser.
- `src/analysis/gto/twos.rs:34` — `RANGE_MATRIX`: the grid labels.
- `src/games/kuhn.rs:827` — `KuhnCfr`.
- `examples/interactive_play.rs` — the flow `heads-up` ports.
- `examples/kuhn_repl.rs` — the flow the trainer's play tab ports.
- `perf/Cargo.toml` — the standalone-workspace pattern.

## Compatibility

- **Preserves** all of pkcore: no change to `src/`, `Cargo.toml` features,
  or `Cargo.lock`. `make check-purity`, `make check-wasm` and `make ayce`
  do not see the spike.
- **Adds** a `spikes/dioxus/` directory and one Makefile target.
- **Breaks** nothing. No downstream repo is touched.

## Dependencies

- **Blocks:** nothing. It informs EPIC-137 (Decision 1) and EPIC-21
  (frontend choice).
- **Built on:** EPIC-88 (snapshot/restore and `SessionView`), EPIC-123
  (`dead` cards), EPIC-19 (bot self-play), EPIC-17 (Kuhn CFR).
- **Related:** EPIC-37 (mobile via UniFFI — the native alternative),
  EPIC-38 (`compute_with_progress`), EPIC-60 (showcase), EPIC-86 (pkwasm,
  the JS-binding alternative and its size baseline).

## Verification

```bash
cd spikes/dioxus
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
dx bundle --package card-lab --platform web --release
dx serve --package heads-up --platform desktop
cd ../..
make dioxus-check
make lint          # pkcore itself is unchanged and still clean
```

Exit criteria:

1. All six apps build for their stated targets, or the findings table says
   which target failed and why.
2. Every app has model unit tests and one SSR render test, all green.
3. No component computes a poker rule; a reviewer can check this by
   searching the view files for pkcore calls other than the model's.
4. The rail leak test (6b) passes, and the method is written down.
5. The Findings table is filled with measured numbers, and the verdict is
   one of Go / Go, narrow / No-go.
6. `git diff main -- src/ Cargo.toml Cargo.lock` is empty.

---

## References

- Dioxus 0.7.0-alpha.0 release notes (Subsecond, Dioxus Native, Blitz):
  <https://github.com/dioxuslabs/dioxus/releases/tag/v0.7.0-alpha.0>
- Dioxus 0.7.0-rc.1 release notes (fullstack on Axum, server functions):
  <https://newreleases.io/project/github/DioxusLabs/dioxus/release/v0.7.0-rc.1>
- dioxus 0.7.9 on docs.rs: <https://docs.rs/crate/dioxus/latest/builds/3248802>
- Dioxus 0.8.0-alpha.0:
  <https://newreleases.io/project/github/DioxusLabs/dioxus/release/v0.8.0-alpha.0>
