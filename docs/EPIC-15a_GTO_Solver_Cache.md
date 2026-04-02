# EPIC-15a: GTO Solver Cache

Completes the serialization story from EPIC-15 by adding a **board+range-hash
keyed persistent cache** for solved spots. The solver itself is complete; this
epic adds the lookup layer on top so that a spot solved once is never re-solved.

---

## Prerequisites

- EPIC-15 complete (`Solver`, `SolverResult`, bincode serialization all shipped)
- `SolverResult::save` / `load` already work for explicit paths — the cache
  builds on top of these primitives

---

## What Already Exists

| Component | Location | Relevance |
|-----------|----------|-----------|
| `SolverResult::to_binary_bytes` | `solver.rs` | Payload serialization |
| `SolverResult::from_binary_bytes` | `solver.rs` | Payload deserialization |
| `SolverResult::save_binary` / `load_binary` | `solver.rs` | File I/O primitives |
| `SolverConfig` | `solver_config.rs` | Source of cache key inputs |
| `bincode` | `Cargo.toml` | Already a dependency |

---

## What Is Missing

| Feature | Gap |
|---------|-----|
| Stable cache key | `Combos` uses `HashSet` (non-deterministic order); `Board` has no `Hash` impl |
| `SolverCache` struct | No lookup/store/evict API |
| Module registration | `solver_cache` not listed in `gto/mod.rs` |

---

## Design

### 1. Cache Key

`SolverConfig` cannot be hashed directly today:

- **`Combos`** is backed by `HashSet<Combo>`, which does not implement `Hash`.
  Iteration order is non-deterministic, so two identical ranges produce
  different byte sequences unless sorted first.
- **`Board`** derives `Eq`/`PartialEq` but not `Hash`.
- **`BetSizings`** must be included — the same board with different sizings is a
  different solve.

Add a `cache_key` free function (not a method on `SolverConfig` to keep
`SolverConfig` free of hashing concerns):

```rust
/// Computes a deterministic u64 cache key for a solver configuration.
///
/// Key inputs (all serialized in sorted, canonical order):
///   hero range, villain range, board cards, bet sizings, effective_stack, pot
///
/// Uses `DefaultHasher` — stable within a build, suitable for local disk
/// cache. Not guaranteed stable across Rust versions or machines; a cache
/// miss simply triggers a re-solve rather than a correctness error.
pub fn cache_key(config: &SolverConfig) -> u64 { ... }
```

Implementation sketch:
1. Sort `config.hero_range` and `config.villain_range` into `Vec<Combo>` and
   `bincode::serialize` each.
2. Collect `Board` cards into a sorted `[u8; 5]` and serialize.
3. Serialize `BetSizings` (already `Ord` on `BetSize`), `effective_stack`, `pot`.
4. Feed all byte slices into `std::hash::DefaultHasher` and return `finish()`.

**Where it lives:** `src/analysis/gto/solver_cache.rs`

---

### 2. `SolverCache` Struct

```rust
/// A directory-backed cache of solved spots, keyed by solver configuration.
///
/// Each entry is stored as `{key:016x}.bin` — a 16-hex filename containing a
/// bincode-serialized [`SolverResult`]. Files are independent so the cache is
/// trivially safe for concurrent reads.
///
/// # Examples
/// ```no_run
/// use pkcore::analysis::gto::solver::Solver;
/// use pkcore::analysis::gto::solver_cache::SolverCache;
/// use pkcore::analysis::gto::solver_config::SolverConfig;
/// // ...build config...
/// # use pkcore::analysis::gto::combos::Combos;
/// # use pkcore::play::board::Board;
/// # use std::str::FromStr;
/// # let config = SolverConfig::new(
/// #     Combos::default(), Combos::default(), Board::default(), 500, 100,
/// # );
/// let cache = SolverCache::new("/tmp/pkcore_cache").unwrap();
/// let result = match cache.get(&config) {
///     Some(r) => r,
///     None => {
///         let r = Solver::new(config.clone()).solve();
///         cache.put(&config, &r).unwrap();
///         r
///     }
/// };
/// ```
pub struct SolverCache {
    dir: PathBuf,
}

impl SolverCache {
    /// Opens (or creates) a cache directory at `dir`.
    ///
    /// # Errors
    /// Returns `Err` if the directory cannot be created.
    pub fn new(dir: impl Into<PathBuf>) -> Result<Self, SolverError>;

    /// Returns a cached result for `config`, or `None` on a cache miss.
    ///
    /// Silently returns `None` on any I/O or deserialization error so the
    /// caller can fall through to a fresh solve.
    pub fn get(&self, config: &SolverConfig) -> Option<SolverResult>;

    /// Stores `result` under the key derived from `config`.
    ///
    /// # Errors
    /// Returns `Err` on I/O or serialization failure.
    pub fn put(&self, config: &SolverConfig, result: &SolverResult) -> Result<(), SolverError>;

    /// Returns `true` if a cached result exists for `config`.
    pub fn contains(&self, config: &SolverConfig) -> bool;

    /// Deletes all `.bin` files in the cache directory.
    ///
    /// # Errors
    /// Returns `Err` if any file cannot be deleted.
    pub fn clear(&self) -> Result<(), SolverError>;
}
```

**Where it lives:** `src/analysis/gto/solver_cache.rs`

---

### 3. Module Registration

Add to `src/analysis/gto/mod.rs`:

```rust
pub mod solver_cache;
```

---

## Implementation Order

1. **`cache_key` function** — the foundation; unit-test that two identical
   `SolverConfig` values produce the same key and two different configs produce
   different keys
2. **`SolverCache::new` + `put` + `get`** — core read/write; test with a
   real `SolverResult` round-trip
3. **`SolverCache::contains` + `clear`** — utility methods
4. **Module registration** — add to `mod.rs`

---

## Testing Requirements

| Test | Assertion |
|------|-----------|
| `test_cache_key_same_config_is_deterministic` | Two identical `SolverConfig` values → same `u64` |
| `test_cache_key_different_range_differs` | Changing either range → different key |
| `test_cache_key_different_board_differs` | Changing the board → different key |
| `test_cache_key_different_sizings_differs` | Changing `BetSizings` → different key |
| `test_solver_cache_miss_returns_none` | Fresh cache → `get` returns `None` |
| `test_solver_cache_put_then_get_round_trips` | `put` then `get` → equal `iterations` and `exploitability` |
| `test_solver_cache_contains_after_put` | `contains` returns `false` before, `true` after `put` |
| `test_solver_cache_clear_removes_entries` | After `clear`, `contains` returns `false` |

---

## Out of Scope

- **Cache eviction policy** — no LRU or size limit; the directory grows
  unbounded. A separate maintenance tool or OS-level tmpdir management is
  sufficient for now.
- **Cross-machine stability** — `DefaultHasher` is not guaranteed stable across
  Rust versions. If portability is needed later, swap in `blake3` or `sha2`.
- **WASM support** — filesystem I/O is native-only; `solver_cache.rs` should be
  gated with `#[cfg(not(target_arch = "wasm32"))]` at the module level.

---

## Relationship to Other Epics

| Epic | Relationship |
|------|-------------|
| EPIC-15 (GTO Solver) | Direct prerequisite — `SolverResult` serialization and `SolverConfig` are both consumed here |
| EPIC-08 (Web) | A web service layer could use `SolverCache` to serve pre-solved spots without re-running CFR on every request |
