# Defect: `SolverCache::cache_key` omits every convergence-control field

**File:** `docs/defects/DEFECT_016_solver_cache_key_omissions.md`  
**Date:** 2026-08-18  
**Severity:** High  
**Status:** Fixed  
**Introduced in:** `03815cab` (2026-04-02) — the commit that added
`solver_cache.rs`. Both omitted fields already existed on `SolverConfig`
(`max_iterations` since `e9b85ef6`, 2026-03-31; `cfr_variant` since `4b4443fc`,
2026-04-01), so the key was incomplete from birth rather than drifting out of
date.  
**Fixed in:** working tree on top of `a3cf7d7f` (pending commit), pkcore `0.5.3`

---

## Summary

`cache_key` hashed the fields that describe *which spot* is being solved — hero
range, villain range, board, bet sizings, effective stack, pot — and none of the
three that describe *how it is solved*: `max_iterations`,
`target_exploitability`, and `cfr_variant`. Two configs differing only in
iteration count or CFR update rule collided on the same `u64`, so
`SolverCache::get` returned a stored `SolverResult` computed under settings the
caller never asked for. Nothing errored: the caller received a well-formed
result carrying its own (wrong) exploitability figure.

---

## Symptom

There is no panic, no error return, and no failing test — which is what makes
this a wrong-result defect rather than a crash. The failure is only visible by
comparing keys:

```
cache_key(river_config().with_max_iterations(3))       == 17001302369990563409
cache_key(river_config().with_max_iterations(100_000)) == 17001302369990563409
cache_key(river_config().with_cfr_variant(Vanilla))    == 17001302369990563409
cache_key(river_config().with_cfr_variant(CfrPlus))    == 17001302369990563409
```

Every config above hashes identically, so they share one `{key:016x}.bin` file.
End to end, a warm cache serves the wrong solve:

```rust
cache.put(&short, &Solver::new(short).solve());   // 3 iterations, stored
cache.get(&long)                                  // asks for 100_000 iterations
// → Some(the 3-iteration result), reported as valid
```

The blast radius is any caller that reuses one `SolverCache` directory across
runs while tuning solver settings — the exact workflow the cache exists to
support. A tuning sweep over `max_iterations` or `cfr_variant` reads its first
solve back for every subsequent setting, and the resulting "the variant makes no
difference" conclusion is an artifact of the cache, not of the solver.

---

## Root Cause

The key was assembled field by field, and the tail of the function stopped at
`pot`:

```rust
    config.effective_stack.hash(&mut hasher);
    config.pot.hash(&mut hasher);

    hasher.finish()
}
```

`SolverConfig` has nine public fields. Six were hashed. The invariant a cache key
must hold is that **any field that can change the cached value must change the
key**, and all three unhashed fields do exactly that:

- `max_iterations` sets how far CFR runs, which decides the returned strategy,
  its `exploitability`, and the reported iteration count.
- `target_exploitability` stops the solve early, so it can cut a solve short
  independently of the iteration cap.
- `cfr_variant` selects the update rule (`Vanilla`, `CfrPlus`, `Discounted`),
  which changes the equilibrium the run converges toward.

The likely reason all three were skipped is mechanical rather than conceptual:
the six hashed fields are all directly `Hash`-able, and the three omitted ones
are not convenient. `max_iterations` is (`usize` derives `Hash`), but the other
two involve `f64`, which has no `Hash` impl because floating-point equality is
not reflexive — `CfrVariant` therefore cannot derive `Hash` at all. A field that
needs a manual encoding is a field that gets left for later, and the function
signature gave no signal that anything was missing.

---

## Fix

All three fields now feed the hasher, and the float-carrying ones are hashed by
IEEE-754 bit pattern:

```rust
    config.effective_stack.hash(&mut hasher);
    config.pot.hash(&mut hasher);

    // Convergence controls. These do not describe the *spot*, but they do
    // decide the `SolverResult` that comes back — iteration count, early-stop
    // threshold, and update rule all move the equilibrium and the reported
    // exploitability. Omitting them collides a long DCFR solve with a short
    // vanilla one.
    config.max_iterations.hash(&mut hasher);
    config.target_exploitability.to_bits().hash(&mut hasher);
    hash_cfr_variant(&config.cfr_variant, &mut hasher);

    hasher.finish()
}

fn hash_cfr_variant(variant: &CfrVariant, hasher: &mut impl Hasher) {
    match variant {
        CfrVariant::Vanilla => 0_u8.hash(hasher),
        CfrVariant::CfrPlus => 1_u8.hash(hasher),
        CfrVariant::Discounted { alpha, beta } => {
            2_u8.hash(hasher);
            alpha.to_bits().hash(hasher);
            beta.to_bits().hash(hasher);
        }
    }
}
```

Three details make this correct rather than merely more thorough:

- **The discriminant tag is written first.** `Vanilla` and `CfrPlus` carry no
  payload, so without a tag they would contribute nothing and stay collided —
  the exact bug, one layer down.
- **`f64::to_bits()` rather than a cast or a formatted string.** It is total,
  allocation-free, and distinguishes every distinct value including the
  subnormals. Its one asymmetry is that `-0.0` and `0.0` compare equal but hash
  differently; for a disk cache that direction of error is a miss and a
  re-solve, never a wrong answer, so the safe failure mode is the one taken.
- **The helper is private and takes `&mut impl Hasher`.** No `Hash` impl is added
  to `CfrVariant`, so the public API is unchanged and this is a patch bump. A
  derived-`PartialEq`-plus-manual-`Hash` pair on a float-carrying type would
  also have invited the standard `Hash`/`Eq` consistency question for no gain
  here.

The tradeoff is that every cache entry written by `0.5.2` or earlier now has a
stale key. Those files are orphaned, not misread — they simply never match
again, so the first solve after upgrading recomputes and rewrites. Stale entries
cost disk, not correctness, and `SolverCache::clear` removes them.

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/analysis/gto/solver_cache.rs` | `cache_key_different_max_iterations_differs` | 3 vs 100 000 iterations produce different keys |
| `src/analysis/gto/solver_cache.rs` | `cache_key_different_cfr_variant_differs` | `Vanilla` and `CfrPlus` differ — the payload-free pair the discriminant tag exists for |
| `src/analysis/gto/solver_cache.rs` | `cache_key_different_discount_exponents_differ` | `Discounted { alpha: 1.5 }` and `Discounted { alpha: 3.0 }` differ — same variant, different floats |
| `src/analysis/gto/solver_cache.rs` | `cache_key_different_target_exploitability_differs` | 0.1 and 0.0001 produce different keys |
| `src/analysis/gto/solver_cache.rs` | `cache_key_same_cfr_variant_is_deterministic` | Bit-pattern hashing stays stable across calls, so the new inputs did not trade a collision for a cache that never hits |
| `src/analysis/gto/solver_cache.rs` | `solver_cache_does_not_serve_a_short_solve_for_a_long_one` | End to end: a stored 3-iteration result is a **miss** for a 100 000-iteration request, and still a hit for its own config |
| `src/analysis/gto/solver_cache.rs` | `solver_cache_does_not_serve_one_cfr_variant_for_another` | End to end: a stored `Vanilla` result is a miss for a `CfrPlus` request |

The last two assert the user-facing symptom through `put`/`get`, not just the
key. A key test alone would pass against a fix that changed the hash without
changing which file is read.

Verified by removing the three new hash lines and re-running: 6 of the 22 tests
in the module fail, including both end-to-end tests. With the fix, all 22 pass.

---

## Coverage Gap

The module already had six `cache_key` difference tests —
`test_cache_key_different_hero_range_differs`, `..._villain_range_...`,
`..._board_...`, `..._sizings_...`, `..._stack_...`, `..._pot_...`. That is one
test per hashed field, and the suite is therefore a mirror of the implementation
rather than a check on it: it was written by reading the function body and
asserting what it does, so it could only ever cover the fields already present.
Six of nine fields tested looked like thorough coverage precisely because the
three missing tests corresponded to the three missing lines.

The other half of the gap is that no test went through `put` then `get` with two
*different* configs. Every cache test used a single `river_config()`, so
`test_solver_cache_put_then_get_round_trips` proved a key matches itself, and
nothing proved a key fails to match something else.

Catching this needed a test written from `SolverConfig`'s field list rather than
from `cache_key`'s body — the question "does every field change the key?" rather
than "does each hashed field change the key?".

---

## Prevention

- The five key tests plus two end-to-end cache tests above pin all three
  previously-unhashed fields.
- `cache_key`'s doc comment now lists every hashed input and states the
  invariant explicitly, so the next field added to `SolverConfig` has a written
  contract to violate rather than an implicit one.
- **The transferable lesson is about how the tests were derived.** Tests written
  by reading the implementation inherit the implementation's blind spots — they
  can confirm what the code does and never notice what it omits. For any
  function whose contract is "consider all of X", derive the tests from X, not
  from the function body. This is the same failure shape as
  [`DEFECT_015`](DEFECT_015_act_raise_all_in_underflow.md)'s crossing-point gap,
  reached from the opposite direction: there, two test groups each covered half
  the ingredients; here, one test group covered two-thirds of the fields and
  looked complete.
- A future `SolverConfig` field is still not forced into the key by the
  compiler. Destructuring the config in `cache_key` — `let SolverConfig { hero_range, .. }`
  without the `..` — would make an added field a compile error. That is a real
  hardening step, deliberately not taken here to keep the defect fix to one
  change; it is recorded in `docs/TECHNICAL_DEBT.md` instead.

---

## Affected Code

| File | Change |
|------|--------|
| `src/analysis/gto/solver_cache.rs` | `cache_key` now hashes `max_iterations`, `target_exploitability` (by bit pattern), and `cfr_variant` |
| `src/analysis/gto/solver_cache.rs` | New private `hash_cfr_variant` helper: discriminant tag plus `alpha`/`beta` bit patterns |
| `src/analysis/gto/solver_cache.rs` | Module doc and `cache_key` doc list every hashed input and state the key invariant |
| `src/analysis/gto/solver_cache.rs` | Seven tests added to the `tests` module |

---

## Related

- [`DEFECT_015`](DEFECT_015_act_raise_all_in_underflow.md) — the previous defect
  from the same 2026-08-18 automated review pass; same lesson about tests that
  mirror the code they test.
- [`docs/TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md) — found by the 2026-08-18
  automated review pass, listed under *Correctness*.
