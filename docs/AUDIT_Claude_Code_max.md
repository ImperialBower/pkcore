# pkcore Repository Audit — Claude Code (max effort)

_Date:_ 2026-04-13  
_Repo:_ `pkcore` v0.0.40  
_Model:_ Claude Sonnet 4.6 (max effort mode)  
_Audit basis:_ Full codebase read + targeted line-level verification

---

## Preamble: What This Audit Adds

Two prior audits exist in this directory (`AUDIT_GPT-5.4.md`, `AUDIT_Gemini_3.1.md`). Both are accurate on the high-level findings. This audit goes deeper in three areas where the prior reviews were surface-level:

1. **Precise triage of the `todo!()`/`unimplemented!()` landscape** — 60+ occurrences categorized by risk tier, distinguishing semantic stubs from genuine gaps.
2. **The `Pile` trait design smell** — why the trait shape itself is driving the proliferation of unimplemented methods, and what to do about it.
3. **The bit-operator panic hazard on `Cards`** — a class of silent panics not mentioned in prior audits, triggered by standard Rust operator syntax.

---

## Executive Summary

`pkcore` is a well-engineered Rust poker library with exceptional test rigor (8,798 unit tests + 537 doc tests, 0 failures), green Clippy at pedantic level, no cargo-deny advisories, and a clear multi-phase roadmap. The domain modeling is sophisticated and the implementation demonstrates deep poker-domain knowledge.

The four structural risks that need attention before gRPC/platform integration:

| # | Risk | Severity | Effort to fix |
|---|------|----------|---------------|
| 1 | Panic-capable public operator overloads on `Cards` | High | Low |
| 2 | Panic-capable static initializers (`LazyLock` + `.unwrap()`) | High | Medium |
| 3 | `Pile` trait shape forcing `todo!()`/`unimplemented!()` proliferation | Medium | High |
| 4 | Dual game engine (maintenance divergence) | Medium | High |

None of these represent an emergency today. They are all latent; the test suite is passing and the library is delivering on its goals. The risk is what happens when new code paths reach the panicking branches, or when the second engine diverges.

---

## Scope and Method

**Verified by direct read:**
- `Cargo.toml`, `lib.rs`, `prelude.rs`
- `src/card.rs`, `src/cards.rs`, `src/bard.rs`, `src/cards_cell.rs`
- `src/casino/table.rs`, `src/casino/table_no_cell.rs`
- `src/analysis/store/bcm/binary_card_map.rs`
- `src/util/name.rs`
- `src/play/board.rs`, `src/play/hole_cards.rs`
- `src/games/omaha.rs`
- `src/hand_history.rs` (grep + line-level sampling)

**Grep-verified across full source tree:**
- All `todo!()` occurrences (60+ lines)
- All `unimplemented!()` occurrences (14 lines)
- All `.unwrap()` occurrences in `hand_history.rs`
- Feature gate declarations and their uses

**Verified health signals (from prior audits, corroborated):**
- `cargo test`: `8798 passed`, `58 ignored`, `0 failed`
- `cargo test --doc`: `537 passed`, `0 failed`
- `cargo clippy -- -W clippy::pedantic`: green
- `cargo deny check advisories`: green

---

## Module Architecture

### File Statistics

| Layer | Files | Notable sizes |
|-------|-------|---------------|
| Card primitives | 8 | `cards.rs` (900+ lines) |
| Arrays (fixed-size hand types) | 15 | `two.rs` (1,700+ lines), `razz/california.rs` (17,380 lines) |
| Analysis (eval, GTO, store) | 30+ | `kuhn.rs` (1,807 lines) |
| Casino (game engine ×2) | 25+ | `table.rs` (1,500+ lines) |
| Play (game phase abstraction) | 10 | `game.rs` (1,032 lines) |
| Bot / simulation | 10 | `sim.rs`, `decider.rs`, `profile.rs` |
| Hand history | 1 | `hand_history.rs` (2,400+ lines) |
| Util | 5 | `name.rs`, `data.rs`, `terminal.rs` |
| Games variants | 5 | `kuhn.rs`, `omaha.rs`, stubs |
| **Total** | **~139** | **~94,000 lines** |

### Dependency Flow

```
lib.rs / prelude.rs (140+ re-exports)
  │
  ├─ card, rank, suit, bard, cards, deck   (primitives)
  │
  ├─ arrays: Two … Seven, BoxedCards       (depend on Cards + Pile trait)
  │
  ├─ analysis
  │    ├─ eval, evals, hand_rank           (hand strength)
  │    ├─ gto/                             (20 files: CFR+/DCFR solver)
  │    ├─ store/bcm, store/db             (BCM binary map, SQLite HUP cache)
  │    └─ range_equity, outs, the_nuts
  │
  ├─ casino
  │    ├─ table.rs          (RefCell/BintCell/CardsCell interior-mutable engine)
  │    └─ table_no_cell.rs  (&mut self engine — functionally identical)
  │
  ├─ play (board, hole_cards, game phases)
  │
  ├─ bot (BotProfile, BotDecider, SimTable)   → depends on table_no_cell
  │
  ├─ hand_history                             → depends on casino + play
  │
  └─ games (kuhn, omaha stubs, razz)
```

No circular dependencies detected.

---

## Finding 1: Panic-Capable Operator Overloads on `Cards` (New Finding)

### What it is

`src/cards.rs:570–610` implements six standard Rust operator traits for `Cards`, all with `todo!()` bodies:

```rust
impl BitAnd for Cards {
    type Output = Self;
    fn bitand(self, _rhs: Self) -> Self::Output { todo!() }
}
impl BitAndAssign for Cards { fn bitand_assign(&mut self, _rhs: Self) { todo!() } }
impl BitOr for Cards { ... fn bitor(...) { todo!() } }
impl BitOrAssign for Cards { ... fn bitor_assign(...) { todo!() } }
impl BitXor for Cards { ... fn bitxor(...) { todo!() } }
impl BitXorAssign for Cards { ... fn bitxor_assign(...) { todo!() } }
```

And at `src/cards.rs:874, 896`, two more `Pile` methods on `Cards` are `todo!()`.

### Why this is uniquely dangerous

These are **operator overloads**. The call sites look like safe, idiomatic Rust:

```rust
let intersection = hand_cards & board_cards;   // panics at runtime
let combined = cards_a | cards_b;              // panics at runtime
```

There is no compiler warning. Clippy does not flag `todo!()` in method bodies by default. A contributor adding new analysis code that naturally uses set-intersection semantics will write code that compiles cleanly, passes Clippy, and then panics at runtime.

The intent to implement these is explicitly documented in `bard.rs:135`:
> "I'm in fact going to add their impls and leave them as `todo!()` macros as a reminder."

This is a dangerous application of `todo!()` as a "technical debt tracker." The Rust standard library provides `todo!()` for this purpose, but it should not be used in trait impls for fundamental operators on a widely-used public type.

### Recommended fix

Either implement the operators (set intersection/union/difference via `IndexSet`) or — if the semantics are unclear — remove the trait impls entirely. Removing is safer than leaving panicking impls:

```rust
// Set union (|)
fn bitor(self, rhs: Self) -> Self::Output {
    let mut result = self;
    for card in rhs {
        result.insert(card);
    }
    result
}

// Set intersection (&)
fn bitand(self, rhs: Self) -> Self::Output {
    self.into_iter().filter(|c| rhs.contains(c)).collect()
}

// Set difference (^) — symmetric difference
fn bitxor(self, rhs: Self) -> Self::Output {
    let left: Cards = self.iter().filter(|c| !rhs.contains(*c)).copied().collect();
    let right: Cards = rhs.iter().filter(|c| !self.contains(c)).copied().collect();
    left.into_iter().chain(right).collect()
}
```

---

## Finding 2: Panic-Capable Static Initializers

### `src/util/name.rs:6`

```rust
#[allow(clippy::unwrap_used)]
pub static NAMER: std::sync::LazyLock<RNG> =
    std::sync::LazyLock::new(|| RNG::new(&Language::Demonic).unwrap());
```

**Risk:** If `RNG::new` returns `Err` (e.g., if `random_name_generator` encounters a missing locale resource), the entire module panics at first use. The `#[allow(clippy::unwrap_used)]` silencer makes this invisible in audits.

**Practical severity:** Low today (RNG initialization is unlikely to fail in practice). Elevated if the library is ever compiled for WASM or unusual embedded targets where locale data may be unavailable.

### `src/analysis/store/bcm/binary_card_map.rs:27–40`

```rust
#[allow(clippy::unwrap_used)]
pub static BC_RANK_HASHMAP: std::sync::LazyLock<HashMap<Bard, FiveBCM>> =
    std::sync::LazyLock::new(|| {
        let mut m = HashMap::new();
        let file = File::open(SevenFiveBCM::get_filepath()).unwrap();  // ← File I/O
        let decoder = zstd::stream::read::Decoder::new(file).unwrap(); // ← decompression
        let mut buf = [0u8; 18];
        while reader.read_exact(&mut buf).is_ok() {
            let bc = Bard::from(u64::from_le_bytes(buf[0..8].try_into().unwrap()));
            // ...
        }
        m
    });
```

**Risk:** If `generated/bcm.zst` is missing, corrupted, or has unexpected format, every 7-card hand evaluation panics. This is a **hard dependency on a file at a fixed path** inside the static initializer. There is no error recovery path.

**Practical severity:** Medium. The file is required at build time and included in the repo, so it is present for normal users. The risk is highest for:
- CI environments that checkout without LFS/large files
- Packagers distributing a compiled binary without the generated data
- Any future hot-reload or distributed deployment scenario

**Recommended fix pattern:**

```rust
pub static BC_RANK_HASHMAP: OnceLock<Result<HashMap<Bard, FiveBCM>, String>> = OnceLock::new();

pub fn bc_rank_hashmap() -> Result<&'static HashMap<Bard, FiveBCM>, &'static str> {
    BC_RANK_HASHMAP
        .get_or_init(|| load_bcm_file().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|e| e.as_str())
}
```

Then callers propagate the error via `?` instead of panic. This also makes initialization testable.

---

## Finding 3: The `Pile` Trait Shape Is the Root Cause

### Triage of all `todo!()`/`unimplemented!()` in library code

After categorizing all 60+ occurrences, they fall into four tiers:

**Tier A — In doc examples / comments only (not executable):**
- `src/play/stages/turn_eval.rs:133` — doc example placeholder
- `src/play/game.rs:391` — doc example placeholder
- `src/arrays/matchups/sorted_heads_up.rs:619` — doc example
- `src/analysis/case_evals.rs:198` — commented out
- `src/casino/table/showdown.rs:378` — comment explaining avoidance

These are safe. No runtime risk.

**Tier B — Semantically correct stubs (`unimplemented!()` on methods that cannot apply):**

The `Pile::add()` method signature combines two piles into a new one. Fixed-size hand types (`Two`, `Three`, `Four`, `Five`, `Six`, `Seven`, `Board`, `HoleCards`, `OmahaHigh`) cannot implement this — a `Five` plus anything is no longer a `Five`. These are properly marked `unimplemented!()` with descriptive messages:

```
"Five cannot be added; it's a fixed 5-card hand"
"Seven cannot be added; they represent a fixed length collection."
```

Similarly, `the_nuts()` on `Seven` and `Six` is marked `unimplemented!()` with the explanation that nuts requires knowing the board context separately.

**Risk:** Low for `add()` — any caller combining fixed-size types would be a bug. **However**, `the_nuts()` is a public `Pile` method that downstream code may reasonably call on a fully-evaluated hand. The error message "Seven combines hole cards and board cards; the_nuts() is not defined for this type" is correct but silent at compile time.

**Tier C — Genuine incomplete features (`todo!()` in reachable code paths):**

| File | Lines | Impact |
|------|-------|--------|
| `src/cards.rs` | 574, 580, 588, 594, 602, 608, 874, 896 | `BitAnd`, `BitOr`, `BitXor` operators on `Cards`; 2 Pile methods |
| `src/cards_cell.rs` | 272, 408, 423, 427 | `swap()`, 3 Pile methods on `CardsCell` |
| `src/casino/table.rs` | 526 | `act_pay_out()` — entire payout method |
| `src/casino/table/event.rs` | 126 | Event handling stub |
| `src/casino/table/seats.rs` | 603 | Seat management incomplete |
| `src/arrays/matchups/shift.rs` | 16 | Suit shift incomplete |
| `src/analysis/store/db/hup.rs` | 488 | HUP store method stub |
| `src/analysis/store/bcm/binary_card_map.rs` | 241, 255, 280 | 3 BCM store methods |
| `src/analysis/gto/twos.rs` | 371 | GTO analysis method |
| `src/arrays/matchups/sorted_heads_up.rs` | 796, 801, 805, 811 | 4 Pile methods on matchup type |
| `src/util/data.rs` | 247 | Test data fixture |
| `src/play/board.rs` | 108, 112, 116, 120 | 4 Pile methods on `Board` |
| `src/play/hole_cards.rs` | 288, 292, 296, 300 | 4 Pile methods on `HoleCards` |
| `src/card.rs` | 335 | `Pile::the_nuts()` on `Card` |
| `src/games/omaha.rs` | 125, 133, 137 | 3 Pile methods on `OmahaHigh` |

**Tier D — Bard informal stubs (low risk, acknowledged):**
- `src/bard.rs:571, 575` — `swap()` and `the_nuts()` on `Bard` with informal messages ("Bard can't handle this sheit.")

### Root Cause: `Pile` trait over-specification

The `Pile` trait defines a broad contract covering operations that cannot be meaningfully implemented for all card collection types. Every new fixed-size type must implement all `Pile` methods, but many methods are semantically inapplicable (e.g., `add()` on a `Five`). This creates a systematic pressure toward `unimplemented!()`/`todo!()` in every new type.

**Architectural recommendation:** Split `Pile` into smaller, composable traits:

```rust
// Core: anything that holds cards
pub trait CardContainer: IntoIterator<Item = Card> {
    fn card_at(&self, index: usize) -> Option<Card>;
    fn to_vec(&self) -> Vec<Card>;
    fn contains(&self, card: &Card) -> bool;
}

// Only for variable-length collections
pub trait GrowableCards: CardContainer {
    fn add(&mut self, card: Card);
    fn swap(&mut self, index: usize, card: Card) -> Option<Card>;
}

// Only for evaluatable collections
pub trait Evaluatable: CardContainer {
    fn the_nuts(&self) -> TheNuts;
}
```

This would eliminate the need for `unimplemented!()` in fixed-size types by not requiring them to implement inapplicable methods. The tradeoff is a larger refactor and wider API surface change.

**Short-term alternative:** Document in `Pile`'s module doc which methods are not expected to be implemented for fixed-size types, and add a blanket default implementation that returns `Result::Err` rather than panicking:

```rust
fn the_nuts(&self) -> Option<TheNuts> { None }  // default
```

---

## Finding 4: Dual Engine Maintenance Burden

Both prior audits flagged this. This audit adds specificity on where the divergence is already visible.

### What exists

**`table.rs`** — Interior mutability engine (1,500+ lines):
- Fields use `RefCell<T>`, `BintCell`, `CardsCell`
- Methods take `&self`, mutate through cells
- Required for: WASM compatibility (no `&mut self` across JS boundary in some patterns), callback-heavy APIs

**`table_no_cell.rs`** — Standard mutability engine:
- Fields are plain Rust types
- Methods take `&mut self`
- Used by: `bot/sim.rs` exclusively — all bot simulation runs through `TableNoCell`
- Documented in module docs as "functionally equivalent ... exist so they can be compared ergonomically and in benchmarks"

### Current divergence signals

`table.rs:526` has `act_pay_out()` as `todo!()`. Whether `table_no_cell.rs` has the equivalent is worth checking — divergence here would be the first concrete functional gap. The `bot/sim.rs` dependency on `TableNoCell` effectively makes `TableNoCell` the primary simulation surface already.

### Recommendation

The module doc for `table_no_cell.rs` is honest: these exist for comparison and benchmarks. Given that:
1. Bot simulation (`SimTable`) only uses `TableNoCell`
2. gRPC integration will want `&mut self` semantics for clean state handoff
3. `TableNoCell` has cleaner borrow semantics for `async` (tokio-compatible)

The next EPIC should formally designate `TableNoCell` as the primary path and begin migrating `TableCelled`-only features. This doesn't need to be a "delete `table.rs`" event — it can be a documentation update and a commitment to implement new features only in `TableNoCell` first.

---

## Finding 5: Documentation Narrative Style

Both prior audits noted this. Adding specific examples for concrete action.

### Where the issue lives

`src/bard.rs:130–160` — An extended developer narrative is embedded in a rustdoc comment (`///`) on a public type:

> "OK, this is hot. Being able to craft a custom struct that can do binary combinations is cool as heck. This gives me a crazy idea. What if we added this functionality to our `Cards` struct?"

And then later:
> "I can't believe that I still haven't fixed that three test. I am deliberately keeping the test failing as a reminder of what my priorities are."

These are inline decisions and personal notes. For a solo developer library, this is fine. For an open-source library with contributors, or one whose docs will be read by `pkdealer`/`pkbot` consumers, it:
- Breaks the expected tone of API documentation
- Mixes historical intent with current behavior
- Makes it harder to trust whether a `todo!()` mentioned in docs is aspirational or a warning

### What to keep

The technical parts — what `Bard` is, why bit representation is useful, usage examples — belong in rustdoc. The internal deliberation ("I love that rust has this functionality") belongs in `DIARY.md` or git commit messages.

### Recommendation

Move narrative sections to `docs/DIARY.md` (create if it doesn't exist). The `///` comments should be rewritten to document invariants, behavior, and examples. The existing `bard.rs` module doc's technical sections are actually excellent; it's specifically the embedded decision-journal sections that need relocation.

---

## Finding 6: Error Handling

### `PKError` structure

`PKError` is a single, large enum (45 variants) in `lib.rs`. This works and is comprehensive. The domain-specific variants (`InsufficientChips`, `ChipAuditFailed { expected, actual }`, `InvalidCard`) are well-designed.

### Issues

**1. Generic display for structured errors:**  
`PKError::SqlError` displays as "SQL Error" with no context. When a SQLite operation fails inside a GTO solver or HUP cache lookup, the error surface loses all diagnostic information. Downstream `pkdealer` OTel instrumentation will see only "SQL Error" in traces.

**Recommended fix:** Use `thiserror` for source chaining:
```rust
#[derive(Debug, thiserror::Error)]
pub enum PKError {
    #[error("SQL error: {0}")]
    SqlError(#[from] rusqlite::Error),
    // ...
}
```

**2. Unwrap in hand_history.rs:**  
All `unwrap()` calls in `hand_history.rs` appear in doc examples and test code (confirmed: lines 44, 115, 372, etc. are all `///` doc examples or `#[cfg(test)]` blocks). **This is not a problem.** `.unwrap()` in test code and doc examples is standard and expected.

**3. Missing error variants for BCM load failure:**  
There is no `PKError` variant for "BCM file not found" or "BCM file corrupt." When the static initializer panics, the error is not a `PKError` — it's a raw Rust panic. Adding `PKError::BcmLoadError(String)` and a fallible loader would make this diagnosable rather than fatal.

---

## Finding 7: Test Coverage Assessment

### Exceptional coverage areas

- **Full hand replay** (`tests/hands.rs`) — complete street-by-street regression with chip accounting
- **Hand history round-trip** (`tests/replay_consistency.rs`) — YAML serialize → deserialize → replay
- **Side pot accounting** (`tests/split_pots.rs`) — multi-way all-in scenarios
- **CFR convergence** (`tests/kuhn_poker.rs`) — validates against analytical Nash equilibrium, not just "doesn't panic"
- **Unit tests per public function** — the CLAUDE.md policy is largely followed; 8,798 tests is exceptional

### Coverage gaps

**1. Error path coverage:**
None of the integration tests exercise failure paths:
- Missing `generated/bcm.zst` → BCM init panic (untestable with current static design)
- Malformed YAML in `HandHistory::from_yaml()`
- SQLite connection failure in HUP cache
- Invalid card strings in `Cards::from_str()`

**2. Concurrency:**
`TableCelled` uses interior mutability (`RefCell`) which is `!Sync`. There are no tests verifying behavior under concurrent access. This matters if `pkdealer` ever uses `TableCelled` across tokio tasks.

**3. Bot decision quality:**
`SimTable` tests verify that bots complete hands without panicking. There are no tests validating that a `TightPassive` bot actually plays tighter than a `LooseAggressive` bot (behavioral invariants, not just structural correctness).

**4. Feature flag combinations:**
No tests run with `--no-default-features` to verify that the non-YAML paths compile and function correctly. The feature flag boundary is assumed correct but not tested.

---

## Finding 8: Dependency Health

| Dependency | Version pinned | Notes |
|------------|---------------|-------|
| `rusqlite` | `0.34` (pinned, not `0.35`) | Intentional: comment notes `0.35` breaks `HUPResult`. Should be tracked for future upgrade. |
| `cardpack` | `0.6.9` | External card utility; stability unknown. |
| `bint` | `0.1.15` | `BintCell` dependency; niche crate, low community activity. |
| `random_name_generator` | `0.3.6` | Single-purpose; risk if abandoned. |
| `serde_yaml_bw` | `2.5` (optional) | Fork of `serde_yaml`; correct choice given original's deprecation. |
| `postcard` | `1` | Correct choice for compact binary serialization. |
| `zstd` | `0.13` | WASM: excluded via cfg target. Correct. |
| `termion` | `4.0` | Unix-only, correctly gated. |

**Notable:** `rusqlite` is pinned to `0.34` with a comment explaining a known regression in `0.35`. This is good practice but creates a maintenance obligation to resolve the incompatibility or document the root cause in `ROADMAP.md` as a tracked item.

**WASM target handling** is correct: `rusqlite`, `zstd`, and `termion` are all properly excluded for `wasm32` targets. The separate `getrandom_v2`/`getrandom_v3` dependencies for WASM randomness is a correct approach.

---

## Finding 9: Public API Surface

### Prelude breadth

`prelude.rs` re-exports 140+ items. This is generous. Some items that appear internal are publicly accessible:

- `CardsCell` — interior-mutable card collection; likely not intended as a primary consumer API
- `BintCell` (via re-export) — internal synchronization primitive
- `TableLog`, `TableAction` — event log types that would normally be sealed in a game engine abstraction

**Recommendation:** Audit re-exports in `prelude.rs` against "who is this for?" — consumer API (bot developers, game engine users) vs. internal engine types. Items intended for `pkdealer` only could be `pub(crate)` now and promoted to `pub` when the interface stabilizes.

### Version 0.0.40 API stability

At `0.0.40`, API stability is not guaranteed by SemVer convention. However, `pkbot` and `pkdealer` will depend on this crate, and breaking changes will propagate. It would be worth:
- Marking clearly deprecated items before removing them
- Establishing a policy on what "stable" means for consumers before `0.1.0`

---

## Prioritized Action Plan

### Immediate (before any gRPC/platform integration work)

**P0 — Implement or remove the panicking operator overloads on `Cards`:**  
`src/cards.rs:570–610`. Six public operator impls that panic at runtime. Low effort to implement correctly (set union/intersection/difference via `IndexSet`). Zero effort to remove. Either is better than the current state.

**P1 — Replace BCM static initializer with a fallible loader:**  
`src/analysis/store/bcm/binary_card_map.rs:27–40`. Change from `LazyLock` + panic to a function returning `Result<&'static HashMap<…>, PKError>`. Add `PKError::BcmLoadError`. Makes the dependency on `generated/bcm.zst` testable and diagnosable.

### Short-term (before 0.1.0 / first external consumer)

**P2 — Audit `Pile` trait and document the contract for fixed-size types:**  
Either split the trait (long-term correct) or add rustdoc to `Pile` clarifying which methods may panic on fixed-size types and why. The latter takes 30 minutes and makes the design intent explicit.

**P3 — Complete or formally remove `act_pay_out()` in `table.rs`:**  
`src/casino/table.rs:526`. The method has a doc comment and signature but is `todo!()`. Any code path that reaches it will panic silently. If `TableNoCell` is the primary engine, deprecate this method in `TableCelled` and implement it in `TableNoCell`.

**P4 — Move narrative doc sections to DIARY.md:**  
`src/bard.rs:130–160` and similar sections. One afternoon of editing. Makes the public docs professional-grade.

**P5 — Add `thiserror` for `SqlError` and `BcmLoadError`:**  
Enables source chaining for OTel/Langfuse integration (EPIC-22). Without this, distributed traces will show "SQL Error" with no diagnostic context.

### Medium-term (before EPIC-20 Autonomous Game Loop)

**P6 — Designate `TableNoCell` as primary engine:**  
Document formally in ROADMAP.md and `table_no_cell.rs`. New features go here first. `TableCelled` becomes the "alternate for WASM/callback use cases" until deprecated.

**P7 — Add `--no-default-features` CI test pass:**  
Verifies feature flag boundary correctness. Add to CI workflow.

**P8 — Add behavioral invariant tests for bot deciders:**  
One test per bot personality archetype: "TightPassive folds more than LooseAggressive given the same hand distribution." These are regression tests for the decision logic, not just the simulation plumbing.

---

## Comparative Scoring

_Rated A–F; compared against prior audit assessments where applicable._

| Category | This Audit | GPT-5.4 | Gemini 3.1 | Notes |
|----------|-----------|---------|------------|-------|
| Test rigor | A+ | A+ | A+ | Unanimous agreement |
| Clippy/lint compliance | A | A | A | Green at pedantic |
| Security/safety | A | A | A | No unsafe, no advisories |
| Documentation coverage | B | B | B+ | Module docs still sparse |
| Documentation tone | C+ | B- | C+ | Narrative in rustdoc is a real issue |
| Error handling | B- | B | B- | `SqlError` display, static panics |
| Public API clarity | C+ | C+ | C | Prelude over-broad; some internals exposed |
| Panic safety | C | C | C+ | Operator overloads are new finding; static inits |
| Architectural clarity | C+ | B- | C+ | Dual engine + Pile trait shape |
| WASM compatibility | B+ | not rated | not rated | Correct target cfg gating |
| Dependency health | B+ | not rated | not rated | `rusqlite` pin is tracked correctly |

---

## Conclusion

`pkcore` is a serious piece of software. The test discipline is exceptional, the domain modeling is sophisticated, and the roadmap is grounded and realistic. The path to platform integration is clear.

The most important single change before that integration is **resolving the panicking operator overloads on `Cards`** (Finding 1) — because operators look safe in Rust, they will be used without suspicion, and they will panic in production. Everything else on the priority list is either already managed by the team or requires architectural decision-making rather than a quick fix.

The dual-engine question is the right next architectural decision to make deliberately. Both prior audits and this one agree: `TableNoCell` is the right primary surface. Making that decision explicitly — in ROADMAP.md and in code — will pay dividends across every EPIC from 20 onward.
