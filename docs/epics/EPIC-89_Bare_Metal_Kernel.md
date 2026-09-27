# EPIC-89: Bare-Metal Kernel (LITE)

The same kernel, compiled down to a microcontroller: `cargo add pkcore
--no-default-features` yields a `#![no_std]` evaluator that runs on an
Arduino-class board, and the full crate is unchanged for everyone else.
Not a sibling `pkcore-lite` crate — a `std` feature axis on the one kernel,
the same move `hup-charts` made when a Wasm consumer measured the download.

> **The library work lives in pkcore; the firmware never does.** The board
> binaries live in a separate workspace with their own toolchain file
> (exactly as `perf/` does today), because AVR needs nightly and
> `-Zbuild-std` while `rust-toolchain.toml` pins 1.94.1. pkcore's only
> job is to be `no_std`-clean and to prove it in CI, on every commit,
> without hardware.

The kata: the **Thing** is a **hand of cards on a chip with 2–32 KB of
RAM**. The **Business Requirement** is that the hand is ranked by the same
bits that rank it on a workstation — one evaluator, no fork, no second
truth. The **Business Logic** is the `Card`/`Five`/`Seven` evaluator and
its four lookup tables compiled for `thumbv7em-none-eabihf` (and, as a
stretch, `avr-none`), gated by a cross-target `cargo check` alongside
`check-purity` and `check-wasm`.

---

## Status

*As of 2026-09-27. Nothing has been built; every figure below is a
reading of the source tree, not a measurement. Phase 4 is where numbers
become measurements.*

| Component | Status | Where |
|---|---|---|
| Phase 0 — the spike: `cfg_attr(no_std)` + `cargo check --target thumbv7em-none-eabihf`, read the error list | Planned | pkcore |
| Phase 1 — `core` tier compiles: `Card`, `Rank`, `Suit`, `Five`/`Seven`, `HandRanker`, lookups | Planned | pkcore |
| Phase 2 — `alloc` tier: `Cards`, seeded equity with integer counters, `SmallRng` only | Planned | pkcore |
| Phase 3 — CI: `make check-embedded`, three canonical feature profiles, size budget | Planned | pkcore |
| Phase 4 — firmware spike on Cortex-M (Uno R4 Minima or Nano RP2040 Connect) | Planned | firmware workspace |
| Phase 5 — table-free `HandRanker` + 2,598,960-hand agreement test | Planned | pkcore |
| Phase 6 — AVR stretch: Mega 2560 with progmem tables; Uno R3 table-free only | 🔒 Gated (Phase 5) | firmware workspace |
| `f64` → integer counters in `analysis::equity` (behaviour-visible to `std` users) | **Decision needed** | pkcore |

---

## Context

### What the evaluator actually is

- **A Cactus Kev evaluator over a `u32` card.** `Card(u32)`
  (`src/card.rs:37`) packs rank bit, suit nibble and rank prime.
  `Five::hand_rank_value()` (`src/arrays/five.rs:231`) does flush lookup,
  then unique-rank lookup, then a prime-product binary search
  (`find_in_products`, `five.rs:124`). `Seven` is 21 `Five` evaluations
  over `FIVE_CARD_PERMUTATIONS` (`src/arrays/seven.rs:28`, `:166`).
  `HandRankValue` is a `u16` (`src/analysis/hand_rank.rs:20`). Nothing
  here allocates — `hand_rank_value_does_not_allocate` already asserts it
  (`five.rs:450`, `seven.rs:283`).
- **Four `const`/`static` tables, ≈61 KB total** (`src/lookups/`):

  | Table | Shape | Bytes |
  |---|---|---|
  | `FLUSHES` | `[u16; 7937]` | 15,874 |
  | `UNIQUE_5` | `[u16; 7937]` | 15,874 |
  | `VALUES` | `[u16; 4888]` | 9,776 |
  | `PRODUCTS` | `[u32; 4888]` | 19,552 |
  | | | **61,076** |

  On ARM these sit in `.rodata` in flash at zero RAM cost. On AVR,
  `.rodata` is **copied into SRAM at boot** unless placed in program
  memory explicitly — that single fact is what rules the Uno R3 out and
  makes the Mega 2560 a stretch rather than a target.
- **The prime product needs `u32`.** Max product is 41·37·31·29·23 =
  31,367,009. Fine on Cortex-M; a software multiply chain on 8-bit AVR.
- **`hups.bin` is 15.8 MB** and already behind `hup-charts`
  (`src/analysis/store/embedded/hup_cache.rs:5`). Off, obviously — and the
  precedent for this EPIC's shape (Cargo.toml, `hup-charts` doc comment,
  EPIC-39 corrigendum 16).

### How close the core already is to `no_std`

- **The evaluator files import nothing that lacks a `core::` twin.** Across
  `card.rs`, `card_number.rs`, `rank.rs`, `suit.rs`, `arrays/{two,three,
  five,seven}.rs`, `analysis/{hand_rank,class}.rs`, the only external
  `use`s are `std::fmt`, `std::str::FromStr`, `std::cmp::Ordering`,
  `std::slice::Iter`, one `std::collections::HashSet`, `serde`, and
  `strum`. Every one of those is a rename or a no_std feature flag.
- **`Deck` is already `[Card; 52]`** (`src/deck.rs:20`). `Cards` is
  `IndexSet<Card>` (`src/cards.rs:42`) — that is the `alloc`/`indexmap`
  line, and it is one type.
- **`cardpack` is used in exactly one file**: `src/bard.rs:7`
  (`BasicPile`, `Pile`, `Standard52`; the `Bard → BasicPile` conversion at
  `:348`). The two other hits are doc comments. Gating one module removes
  the dependency from the lite tiers; EPIC-84's bijection bridge is
  unaffected because it is `std`-tier by nature.
- **`bitvec` is used in exactly one file**: `src/arrays/matchups/masks/
  suit_mask.rs:5–7`, `:70`. `matchups` is `std`/`alloc`-tier; if the mask
  is ever wanted below that, a `u64` does the job.
- **`--no-default-features` is already OS-pure** (KERNEL_PURITY_AUDIT.md
  §1, §3; `make check-purity`, `Makefile:325`). Pure ≠ `no_std`, but the
  entire coarse cut — YAML, rayon, JSON, CSV, filesystem, console — is
  done. This EPIC is the fine cut.
- **The `entropy` seam is the RNG design this EPIC needs, already
  shipped.** Core code uses only the seed-taking twins; `--no-default-
  features` reads no OS randomness (Cargo.toml `entropy` comment). On a
  board, the shell reads the hardware RNG and hands a `SmallRng` in.
- **`tests/tda_conformance.rs` deliberately carries no
  `required-features`** and touches only the table/betting API. It is the
  ready-made seed of "tests that must build on the lean tiers".

### What is genuinely `std`-shaped

Counted by file, not by call site (`grep -l`, `src/`):

| Crate / type | Files | Tier it belongs to | Why |
|---|---|---|---|
| `uuid` | 18 | `std` | ids for players/tables/sessions; a board has no use for a UUID and the `entropy` split already made ids caller-supplied |
| `wincounter` | 12 | `std` (verify) | no_std status unknown; used by the equity/analysis surfaces, not the evaluator |
| `indexmap` (`IndexSet`/`IndexMap`) | 6 | `alloc`+`hashbrown` or `std` | `Cards` is the one that matters |
| `thousands` | 2 | `std` | display formatting only |
| `regex` | 1 | `std` | not `no_std` at all |
| `percent_encoding` | 1 | `std` | |
| `bitvec` | 1 | `std` | see above |
| `cardpack` | 1 | `std` | see above |
| `f64` in `analysis::equity` | 2 (`engine.rs:330–375, 583`; `result.rs:31–53`) | **decision** | Cortex-M4F has a single-precision FPU only; AVR has none. The kernel counts `wins`/`ties`/`count` as integers already (`engine.rs:371–375`) and divides at the end — the division is the shell's job |

### What this EPIC explicitly does NOT do

- Create a `pkcore-lite` crate (Decision 1).
- Put a board binary, HAL, or `#[entry]` anywhere in pkcore.
- Port the bot deciders, hand histories, player stats, GTO solver, or
  anything else that is `std`-tier today. They stay `std`-tier.
- Change the public API of the default (`std`) build, with the single
  exception flagged under Decision 6.
- Make the Uno R3 (ATmega328P) a target. It is a *consequence* of Phase 5
  if Phase 5 happens.

---

## Decisions

### 1. A `std` feature axis, not a sibling crate

The separate-crate shape buys a hard compiler wall and a crate readable in
an afternoon. It costs a second version number, a second docs.rs page,
re-export plumbing, and "which one do I `cargo add`?" forever. For a
library whose thesis is *one kernel, many deliveries*, the feature flag is
the honest story, and pkcore has made this exact call before: `hup-charts`
became a feature axis rather than a sibling crate when a Wasm consumer
measured an 8× download. `serde`, `rand`, `hashbrown`, `heapless` and
`embedded-hal` all ship this way.

### 2. Three tiers: (none) → `alloc` → `std`

| Tier | Feature | Contents | Heap | Runs on |
|---|---|---|---|---|
| **core** | *(none)* | `Card`, `CardNumber`, `Rank`, `Suit`, `Deck`, `Two`/`Three`/`Five`/`Seven`, `HandRanker`, `HandRankValue`, `Class`, the four lookup tables | none | 2 KB AVR and up (tables permitting) |
| **alloc** | `alloc` | `Cards`, `Pile`, seeded Monte Carlo + exact-enumeration equity with a `const SEATS` bound, `SmallRng` plumbing; later the `play/` state machine | `alloc::` via a global allocator (`embedded-alloc` on the board) | 32 KB SRAM Cortex-M |
| **std** | `std` (in `default`) | everything else | std | workstation, Wasm, mobile |

`hup-charts`, `player-stats`, `equity`'s current shape and every
format/terminal feature gain `std` (or `alloc`) as a Cargo feature
dependency so `--no-default-features --features player-stats` cannot
produce a tree that fails to compile.

### 3. Design for Cortex-M; AVR is the stretch; the Uno R3 is a Phase 5 consequence

| Board | Core | Flash / SRAM | Rust target | Verdict |
|---|---|---|---|---|
| Uno R3 / Nano (ATmega328P) | 8-bit AVR, 16 MHz | 32 KB / 2 KB | `avr-none` (Tier 3, nightly, `-Zbuild-std`) | Tables (61 KB) exceed *flash*. Table-free evaluator only. |
| Mega 2560 | 8-bit AVR, 16 MHz | 256 KB / 8 KB | same | Feasible with tables in progmem (`avr-progmem`). Slow. |
| **Uno R4 Minima / WiFi (Renesas RA4M1)** | Cortex-M4F, 48 MHz | 256 KB / 32 KB | `thumbv7em-none-eabihf` (Tier 2) | **Primary.** Tables in flash for free; on-chip TRNG for the seed. HAL support is thin — expect PAC-level work. |
| Nano RP2040 Connect (and any RP2040) | 2× Cortex-M0+, 133 MHz | 16 MB / 264 KB | `thumbv6m-none-eabi` (Tier 2) | Path of least resistance: `rp2040-hal`/`embassy-rp`, `probe-rs`, ROSC as entropy. Good first board if the R4's HAL stalls. |
| Nano ESP32 / Nano 33 BLE / Giga | Xtensa-S3 / nRF52840 / M7 | ≥1 MB / ≥256 KB | `esp-rs` / `thumbv7em` | Comfortable; not interesting as a constraint. |

The satisfying story is "the same kernel bits run on an 8-bit chip from
2009". The *engineering* story is: get it green on an M4 first, keep the
core tier compiling for `avr-none` in CI so nothing drifts, then earn the
AVR demo.

### 4. Firmware lives in its own workspace with its own toolchain

Proposed repo `pkmetal` (open question 1), one `[[bin]]` per board, its own
`rust-toolchain.toml` (nightly for AVR), `memory.x`, `.cargo/config.toml`
runners. It depends on `pkcore` with `default-features = false`. Nothing
in it is pkcore's concern beyond staying compilable against it.

### 5. Tables stay `static`; a table-free evaluator is a *second* `HandRanker`, never a replacement

The tabled evaluator is the fast path and the reference. A rank-counting
evaluator (`[u8; 13]` histogram → category → arithmetic kicker ordering)
has zero tables, fits a 2 KB part, and is slower. It lands only with a test
that the two agree on all 2,598,960 five-card hands — which is a
conformance artefact worth having regardless of hardware. Senzee's
perfect-hash replacement for the binary search is *faster*, not smaller
(~8 KB extra), and is out of scope.

### 6. Equity counters are integers in the kernel; ratios are computed at the edge

`EquityResult { win: f64, tie: f64, equity: f64 }` (`result.rs:31–35`) is
the one place the lite build changes something a `std` user can see. The
kernel already accumulates `wins[i]`, `ties[i]`, `count` as integers and
divides at `engine.rs:371–375`. Options, to be decided before Phase 2:

- **(a)** keep `f64` fields, gate them behind `std`, expose the integer
  counters on all tiers. No visible change for `std`; two shapes to keep
  in step.
- **(b)** make the counters the result on every tier and move
  `equity_pct()`-style ratios to methods that return `f64` under `std`
  and `f32` otherwise. One shape; a semver-minor API change.
- **(c)** `f32` everywhere. Simplest; changes the last digits of every
  reported equity for existing consumers and every golden fixture. Not
  recommended.

Default if undecided at Phase 2: **(a)**.

---

## Domain map

| Concern | core | alloc | std | Notes |
|---|---|---|---|---|
| Card identity, rank/suit, deck order | ✅ | | | `Deck` is `[Card; 52]` today |
| 5-/7-card ranking | ✅ | | | tables in `.rodata` |
| Parsing (`FromStr`) / `Display` | ✅ (no `String`) | | | `core::fmt` writes to a `Formatter`; parsing of a fixed-arity array needs no heap |
| `Cards` / `Pile` | | ✅ | | `IndexSet` → `alloc` + `hashbrown`, or `heapless::Vec<Card, 52>` |
| Seeded RNG | | ✅ | | `rand` `small_rng` only; `std_rng`/`os_rng` join `std`/`entropy` |
| Equity (MC, exact) | | ✅ | | integer counters (Decision 6) |
| Betting-structure state machine (`play/`) | | ⏳ later | ✅ | the interesting `alloc` candidate for a dealer gadget; needs `uuid` out of the hot types first |
| Bots, histories, stats, GTO, store, YAML/JSON/CSV, terminal | | | ✅ | unchanged |
| UUIDs, `cardpack` bridge (`bard.rs`), `regex`, `percent-encoding`, `thousands`, `bitvec` | | | ✅ | module-gated |

---

## Design

### `lib.rs`

```rust
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

// core tier — always compiled
pub mod card;
pub mod card_number;
pub mod rank;
pub mod suit;
pub mod deck;
pub mod lookups;
pub mod arrays;      // two/three/five/seven + HandRanker; matchups/ gated inside

// alloc tier
#[cfg(feature = "alloc")] pub mod cards;
#[cfg(feature = "alloc")] pub mod cards_cell;

// std tier — one attribute per module, files untouched
#[cfg(feature = "std")] pub mod bard;
#[cfg(feature = "std")] pub mod bot;
#[cfg(feature = "std")] pub mod casino;
#[cfg(feature = "std")] pub mod games;
#[cfg(feature = "std")] pub mod hand_history;
#[cfg(feature = "std")] pub mod play;
#[cfg(feature = "std")] pub mod pokerbench;
#[cfg(feature = "std")] pub mod util;
```

`analysis/` is mixed and gets the same treatment inside its `mod.rs`
(`hand_rank`, `class` core; `equity` alloc; the rest std).

### `Cargo.toml`

```toml
[features]
default = ["std", "player-stats", "equity", "hup-charts", "entropy"]
std   = ["alloc", "serde?/std", "rand/std", "rand/std_rng", "strum/std",
         "dep:indexmap", "dep:uuid", "dep:regex", "dep:percent-encoding",
         "dep:thousands", "dep:wincounter", "dep:cardpack", "dep:bitvec",
         "postcard/use-std"]
alloc = ["serde?/alloc", "postcard/alloc", "dep:hashbrown"]
serde = ["dep:serde"]                # optional even on core; derive is no_std-clean

# existing features acquire their tier
equity                   = ["alloc"]
player-stats             = ["std"]
hup-charts               = ["std"]
entropy                  = ["std", "rand/os_rng", "rand/thread_rng", "uuid/v4", "dep:getrandom_v3"]
# bot-profiles, hand-histories, json, csv, persistence, terminal, store, … = ["std", …]

[dependencies]
rand     = { version = "0.9", default-features = false, features = ["small_rng"] }
serde    = { version = "1.0", default-features = false, features = ["derive"], optional = true }
strum    = { version = "0.28", default-features = false, features = ["derive"] }
postcard = { version = "1", default-features = false }
itertools = { version = "0.14", default-features = false }
log      = "0.4"
hashbrown = { version = "0.15", optional = true }
indexmap  = { version = "2.12", optional = true }
uuid      = { version = "1.22", features = ["serde", "v5"], optional = true }
cardpack  = { version = "0.11.1", optional = true }
bitvec    = { version = "1.1", optional = true, features = [...] }
regex / percent-encoding / thousands / wincounter = { …, optional = true }
```

Whether `serde` stays unconditional on the core tier is open question 3;
`serde` with `derive` and no default features *is* `no_std`-clean, so the
derives on `Card`/`Five` cost nothing but a dependency.

### Import renames (mechanical)

| today | core tier |
|---|---|
| `std::fmt`, `std::fmt::{Display, Formatter}` | `core::fmt`, … |
| `std::str::FromStr` | `core::str::FromStr` |
| `std::cmp::Ordering` | `core::cmp::Ordering` |
| `std::slice::Iter` | `core::slice::Iter` |
| `std::error::Error` | `core::error::Error` (stable since 1.81) |
| `std::collections::HashSet` (`lib.rs:422`) | gate or remove |
| `Vec`, `String`, `format!`, `Box` | `alloc::vec::Vec`, `alloc::string::String`, `alloc::format!` — `alloc` tier only |

### RNG on the board

Unchanged from the `entropy` design: the kernel exposes only seed-taking
twins. The shell does

```rust
let seed: u64 = board_trng.next_u64();          // RA4M1 TRNG, RP2040 ROSC, …
let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
let req = EquityRequest::with_rng(&mut rng, …);  // existing twin
```

### Size and speed budget (estimates; Phase 4 measures)

| | Cortex-M4F @ 48 MHz | ATmega2560 @ 16 MHz |
|---|---|---|
| Lookup tables | 61 KB flash, 0 RAM | 61 KB progmem, 0 RAM (with `avr-progmem`); **61 KB SRAM without** |
| 7-card eval | ~10–20 µs (21 × binary search ≈ 13 iterations each) | ~1–2 ms (software `u32` multiply) |
| 10k-trial MC equity, heads-up | well under 1 s — interactive | tens of seconds — not interactive |
| Exact preflop HU (1.7 M runouts) | minutes — a "leave it running" mode | hours |

---

## Work Items

### Phase 0 — the spike (no hardware, half a day)

- [ ] `rustup target add thumbv7em-none-eabihf`.
- [ ] Add `#![cfg_attr(not(feature = "std"), no_std)]` and a `std` feature
      that is on by default and does nothing else yet.
- [ ] `cargo check --no-default-features --target thumbv7em-none-eabihf`.
      **The error list is the estimate.** Record its length and its shape
      (imports vs. types vs. dependencies) in this EPIC's Status.
- [ ] Confirm `cardpack` is `bard.rs`-only and `bitvec` is `suit_mask.rs`-
      only by compiler evidence, not grep.

### Phase 1 — core tier compiles

- [ ] Module-gate `lib.rs` and `analysis/mod.rs` per the Design.
- [ ] `std::` → `core::` renames in the core-tier files.
- [ ] Make `regex`, `uuid`, `percent-encoding`, `thousands`, `wincounter`,
      `indexmap`, `cardpack`, `bitvec` optional and `std`-owned; move
      `serde`, `rand`, `strum`, `postcard`, `itertools` to
      `default-features = false`.
- [ ] Remove or gate the `HashSet` in `lib.rs`.
- [ ] Every existing feature acquires its tier (`equity = ["alloc"]`, the
      rest `["std", …]`).
- [ ] Green: `cargo check --no-default-features --target
      thumbv7em-none-eabihf --lib`, and `make ayce` unchanged.

### Phase 2 — alloc tier

- [ ] Decision 6 recorded; `EquityResult` reshaped accordingly.
- [ ] `Cards` on `alloc` (`IndexSet` → `hashbrown`-backed, or a
      `heapless::Vec<Card, 52>` behind the same API).
- [ ] `equity` builds and its serial arm runs on `alloc` with a caller-
      supplied `SmallRng`; `exact_enumerate__counts_are_identical_serial_
      or_parallel` remains the agreement test.
- [ ] Green: `cargo check --no-default-features --features alloc,equity
      --target thumbv7em-none-eabihf --lib`.

### Phase 3 — CI

- [ ] `make check-embedded`:
      ```
      cargo check --no-default-features                       --target thumbv7em-none-eabihf --lib
      cargo check --no-default-features --features alloc,equity --target thumbv7em-none-eabihf --lib
      cargo check --no-default-features                       --target thumbv6m-none-eabi    --lib
      ```
      added to `ayce` beside `check-purity` and `check-wasm`
      (`Makefile:279`, `:325`) and to `.github/workflows/ci.yml`.
- [ ] Three canonical profiles, **not** the powerset: `(none)`,
      `alloc,equity`, `default`. `check-features` (`Makefile:241`) keeps
      its per-feature loop for the `std` world.
- [ ] Size budget: `cargo size --release --target thumbv7em-none-eabihf`
      (`cargo-binutils`) on a trivial `#![no_main]` fixture in the
      firmware workspace; fail if `.text + .rodata` for the core tier
      exceeds a recorded ceiling (initial: tables + 16 KB).
- [ ] `tests/` convention: a test with no `required-features` must build
      on the `alloc` tier (`tda_conformance` is the exemplar); document
      in `docs/superpowers/` testing conventions.

### Phase 4 — firmware spike (Cortex-M)

- [ ] Repo per Decision 4. Board: Uno R4 Minima if a usable RA4M1 HAL
      exists at the time; otherwise Nano RP2040 Connect (or a bare Pico)
      for the spike, with the R4 following.
- [ ] Blink → deal seven cards over UART and print `HandRankValue` and
      class → seeded MC equity for two hole-card pairs, N trials.
- [ ] **Measure**: 7-card evals/s, µs per MC trial, `.text`/`.rodata`/
      `.bss` sizes. Replace every estimate in this EPIC with the number.
- [ ] Demo shell (optional): 128×64 OLED + two buttons = pocket equity
      calculator. The shell is firmware; the kernel is the same bits.

### Phase 5 — table-free evaluator

- [ ] `analysis::hand_rank::histogram` (working name): `[u8; 13]` rank
      counts → category → kicker ordering → `HandRankValue` in the same
      7,462-class numbering.
- [ ] `#[ignore]`d-by-default exhaustive test: all 2,598,960 `Five`s agree
      with the tabled evaluator; runs in `make heavy`.
- [ ] Feature `no-tables` selects it and drops `lookups/` from the build.

### Phase 6 — AVR stretch (gated on Phase 5)

- [ ] Core tier `cargo check --target avr-none -Zbuild-std=core
      -Ctarget-cpu=atmega2560` on nightly, in the firmware workspace's CI
      (not pkcore's, which pins stable).
- [ ] Mega 2560: tables via `avr-progmem`; measure.
- [ ] Uno R3: `no-tables` only; measure; expect seconds per equity run.

### Deferred (follow-on EPICs, not this one)

- `play/` betting state machine on the `alloc` tier (needs `uuid` out of
  `Player`/`Table` hot types — a `PlayerId(u32)` newtype question).
- A `postcard` wire format across UART/SPI so a host drives a board-
  resident kernel — the DK boundary as a serial protocol.
- Senzee perfect hash.

---

## Test Plan

- **Host, every commit**: `make ayce` unchanged; `make check-embedded`
  (three `cargo check`s, no hardware).
- **Host, `alloc` tier tests**: `cargo test --no-default-features
  --features alloc,equity` — tests link `std` via `#[cfg(test)] extern
  crate std;` while the library under test is compiled without it.
- **Agreement tests already present**: `hand_rank_value_does_not_allocate`
  (`five.rs:450`, `seven.rs:283`); `exact_enumerate__counts_are_identical_
  serial_or_parallel`; `tests/kernel_determinism` (`make test-kernel`,
  `Makefile:226`).
- **New agreement test (Phase 5)**: tabled vs. histogram over all
  2,598,960 five-card hands, `#[ignore]`, `make heavy`.
- **On-target (Phase 4+)**: a `defmt`/UART smoke test that ranks the
  wheel, a royal flush, and quads-over-full-house and prints the expected
  `HandRankValue`s; equity of AA vs KK at 10k trials within ±2 pp of the
  host's number for the same seed — **identical**, not within tolerance,
  if `SmallRng` and the integer counters are the only arithmetic.

---

## Key Files

| File | Role in this EPIC |
|---|---|
| `Cargo.toml` | feature tiers, optional deps, `default-features = false` |
| `src/lib.rs` | `cfg_attr(no_std)`, module gates, `HashSet` at `:422` |
| `src/card.rs:37` | `Card(u32)` — unchanged |
| `src/deck.rs:20` | `Deck([Card; 52])` — already core-tier |
| `src/cards.rs:42` | `Cards(IndexSet<Card>)` — the `alloc` line |
| `src/arrays/five.rs:124–250`, `src/arrays/seven.rs:28, 166–200` | the evaluator |
| `src/lookups/{flushes,unique5,values,products}.rs` | the 61 KB |
| `src/analysis/hand_rank.rs:20–22` | `HandRankValue`, `NO_HAND_RANK_VALUE` |
| `src/analysis/equity/engine.rs:330–375, 583`, `result.rs:31–53` | Decision 6 |
| `src/bard.rs:7, 348` | only `cardpack` use |
| `src/arrays/matchups/masks/suit_mask.rs:5–7, 70` | only `bitvec` use |
| `Makefile:226, 241, 279, 325` | `test-kernel`, `check-features`, `check-wasm`, `check-purity` — `check-embedded` joins them |
| `tests/tda_conformance.rs` | exemplar of a no-`required-features` test |
| `docs/KERNEL_PURITY_AUDIT.md` §1, §3 | the coarse cut this EPIC refines |

## Reuse (do NOT recreate)

- **The `entropy` seam.** Seed-taking twins exist for every convenience
  constructor. The board shell uses them; nothing new in the kernel.
- **The `hup-charts` precedent** for a feature axis justified by a
  consumer's measured footprint.
- **`check-purity` / `check-wasm` / `check-features`** as the shape of the
  gate; `check-embedded` is a fourth line in `ayce`, not a new mechanism.
- **The wasm target-gating pattern** in `Cargo.toml`
  (`[target.'cfg(target_arch = "wasm32")'.dependencies]`) if any dependency
  needs a per-target feature.
- **`Deck` as a fixed array** and the `does_not_allocate` tests.
- **EPIC-86's "no threads on this target" discipline** — `parallel` is
  already off in the lean builds for the same reason.

## Compatibility

- `cargo add pkcore` (default features) is unchanged: `std` is in
  `default`, every existing feature implies its tier.
- `--no-default-features` today means "pure"; after this EPIC it also
  means "`no_std`, no `alloc`". Consumers who relied on `Cards` or equity
  under `--no-default-features` add `--features alloc,equity`. Call this
  out in `CHANGELOG.md` and `DOWNSTREAM_MIGRATION`.
- Decision 6 is the only possible API change visible to `std` users.
- MSRV unchanged (`core::error::Error` needs ≥ 1.81; we pin 1.94.1).

## Dependencies

- **EPIC-84** (cardpack bridge): `bard.rs` becomes `std`-tier; the
  bijection bridge is unaffected.
- **EPIC-86** (browser bindings): shares the lean-build discipline; the
  `alloc` tier is also the right tier for a smaller Wasm bundle.
- **KERNEL_PURITY_AUDIT fix 1** (0.15.0): prerequisite, done.

---

## Verification

```sh
# pkcore — every commit, no hardware
rustup target add thumbv7em-none-eabihf thumbv6m-none-eabi
make check-embedded                       # three cargo checks, see Phase 3
cargo test --no-default-features --features alloc,equity
make ayce                                 # unchanged behaviour for std

# firmware workspace — Phase 4
cargo size --release --bin pkmetal-uno-r4 -- -A
probe-rs run --chip R7FA4M1AB --release --bin pkmetal-uno-r4   # or elf2uf2-rs for RP2040

# Phase 5
cargo test --release -- --ignored histogram_agrees_with_tables_all_2598960
```

Done when: `make ayce` is green with `check-embedded` in it; a Cortex-M
board prints the correct `HandRankValue` for a dealt seven and a seeded
equity that matches the host bit-for-bit; every estimate in this document
has been replaced by a measurement.

## Open questions

1. **Firmware repo name.** `pkmetal`? `pkiron`? Must follow the `pk*`
   convention and not collide with EPIC-50's `pkgate`/`pkdealer`.
2. **Uno R4 vs RP2040 for the first board.** The R4 is the "Arduino" in
   the story; the RP2040 has the Rust ecosystem. Decide at Phase 4 on
   the state of the RA4M1 HAL that week.
3. **`serde` on the core tier.** Unconditional (`derive` is no_std-clean,
   costs a dependency and compile time) or behind a `serde` feature
   (cleaner tree, more `#[cfg_attr(feature = "serde", derive(…))]`
   noise)? Leaning unconditional for Phase 1, revisit at Phase 3 when
   `cargo size` has an opinion.
4. **`wincounter` no_std status** — unverified; if it is std-only and
   the equity engine needs it on `alloc`, replace with a local counter.
5. **Should `cardpack` itself go `no_std`?** One-file use here makes it
   moot for pkcore, but the same pattern applied to cardpack would carry
   the Ganjifa decks to a board for free. Separate EPIC in that repo if
   wanted.
6. **`PlayerId`/`TableId` as `u32` newtypes** — the precondition for
   `play/` on `alloc`, and a question EPIC-88 (Table Snapshot) may want to
   answer first.
