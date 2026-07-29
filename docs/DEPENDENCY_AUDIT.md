# Dependency Audit

**Audited:** 2026-07-28 at `1fe55b8` · rustc 1.97.1 / cargo 1.97.1 (crate MSRV 1.94.1)
**Scope:** 26 direct dependencies (21 third-party, 5 first-party), resolved
normal (non-dev) tree of **172 crates** across all targets — **140 on the host
target** (`aarch64-apple-darwin`), default features. Dev-dependencies add a
further 85 crates on host that never ship.
**Method:** /untangle — evidence commands in the appendix; scores use the 1–5
anchors, verdicts use the controlled vocabulary. Host license is
`MIT OR Apache-2.0`, so every permissive upstream is copy-compatible.

> **Headline:** `random_name_generator` drags **31 crates that nothing else in
> the graph needs** — including all of `clap` — into pkcore's *shipping*
> dependency graph to serve **one** production call site. `bitvec` costs 5
> unique crates for an algorithm that is `u8::reverse_bits() >> 4` in std.
> Those two changes remove **36 crates — 26% of the host graph** — and break
> nothing. A third, purely manifest-level change (`cardpack` with
> `default-features = false` in both pkcore and `pkstate`) removes **25 more**,
> taking the host graph from **140 → ~78 crates**.

---

## Summary

Unique baggage = crates that leave the resolved graph if this dependency's node
vanishes entirely, measured on the host target. `0 via <holder>` means another
dependency still holds the same subtree, so removal buys ownership, not size.

| Dependency | Version | License | Score | Unique baggage | Effort | Verdict |
|---|---|---|---|---|---|---|
| **random_name_generator** ¹ | 0.3.6 | BSD-3-Clause | 1 | **31 crates** | M | `absorb` |
| **bitvec** | 1.1.1 | MIT | 1 | **5 crates** | S | `replace-std` |
| **percent-encoding** | 2.3.2 | MIT OR Apache-2.0 | 1 | 0 via wincounter | S | `drop` |
| **thousands** | 0.2.0 | MIT/Apache-2.0 | 1 | 1 crate | S | `rewrite` |
| **regex** | 1.12.3 | MIT OR Apache-2.0 | 1 | 0 via cardpack | S | `replace-std` |
| serde | 1.0.228 | MIT OR Apache-2.0 | 5 | 0 via cardpack | XL | `keep` |
| wincounter ¹ | 0.1.6 | MIT | 5 | 1 crate | L | `keep` |
| indexmap | 2.14.0 | Apache-2.0 OR MIT | 4 | 0 via serde_yaml_bw | L | `keep` |
| serde_yaml_bw | 2.5.6 | MIT OR Apache-2.0 | 4 | 0 via cardpack | L | `keep` |
| uuid | 1.23.2 | Apache-2.0 OR MIT | 4 | 3 crates | L | `keep` |
| itertools | 0.14.0 | MIT OR Apache-2.0 | 3 | 0 via cardpack | L | `keep` |
| pkstate ¹ | 0.1.2 | MIT OR Apache-2.0 | 3 | 4 crates | L | `keep` |
| postcard | 1.1.3 | MIT OR Apache-2.0 | 3 | 2 crates | L | `keep` |
| rand | 0.9.4 | MIT OR Apache-2.0 | 3 | 0 via cardpack | L | `keep` |
| rayon | 1.12.0 | MIT OR Apache-2.0 | 3 | 0 via indexmap | L | `keep` |
| rusqlite | 0.34.0 | MIT | 3 | 7 crates | L | `keep` |
| bint ¹ | 0.1.16 | MIT | 3 | 1 crate | M | `keep` |
| csv | 1.4.0 | Unlicense/MIT | 2 | 2 crates | M | `keep` |
| log | 0.4.30 | MIT OR Apache-2.0 | 2 | 1 crate | M | `keep` |
| serde_json | 1.0.150 | MIT OR Apache-2.0 | 2 | 1 crate | S | `keep` |
| strum | 0.28.0 | MIT | 2 | 1 crate | M | `keep` |
| strum_macros | 0.28.0 | MIT | 2 | 0 via strum | S | `keep` |
| cardpack ¹ | 0.6.12 | Apache-2.0 | 2 | 0 via pkstate | M | `keep` |
| zstd | 0.13.3 | MIT | 1 | 3 crates | M | `keep` |
| termion | 4.0.6 | MIT | 1 | 2 crates | S | `keep` |
| getrandom (v2/v3 shims) | 0.2.17 / 0.3.4 | MIT OR Apache-2.0 | 1 | 0 crates | S | `keep` ² |

¹ First-party — judged with the absorption rubric (see First-party section).
² `getrandom_v2` becomes `drop`-able the moment `random_name_generator` goes;
see the dossier.

---

## Cross-cutting findings

### 1. `clap` ships in pkcore's normal dependency graph (highest-value fix)

`clap v4.6.1` enters the **non-dev** graph through exactly one edge:

```
clap v4.6.1
└── random_name_generator v0.3.6
    └── pkcore v0.3.2
```

Upstream declares `clap`, `anyhow`, `rust-embed`, `titlecase`, `regex`,
`rand`, and `lazy_static` as **non-optional normal dependencies** because its
CLI binary is not feature-gated. pkcore uses the crate for a single
`RNG::new(&Language::Demonic)` call. **Upgrading does not help** — 0.4.0 still
lists `clap ^4.6.2` as a normal dep and moves to `rand ^0.10`, which would add
a *third* `rand` major to the graph.

### 2. Every duplicate version in the normal graph traces to two edges

`cargo tree -d` reports many duplicates, but most are dev-only (`reedline`,
`clap-repl`, `criterion`). In the **normal** graph there are exactly seven:

| Crate | Versions | Cause |
|---|---|---|
| `rand` / `rand_core` / `rand_chacha` | 0.8 + 0.9 | `random_name_generator → rand 0.8` |
| `getrandom` | 0.2 + 0.3 + 0.4 | 0.2 ← `rand 0.8` (via rng); 0.4 ← `uuid 1.23` |
| `hashbrown` | 0.15 + 0.17 | `rusqlite → hashlink` vs `indexmap` |
| `r-efi`, `wit-bindgen` | 2 each | `getrandom 0.4`'s wasip3 backend (non-host targets) |

Removing `random_name_generator` collapses the `rand` 0.8 chain **and** the
`getrandom 0.2` chain in one move.

### 3. wasm32 `getrandom` shims: checked, correct

`Cargo.toml:97–100` pins `getrandom` 0.2 (`js`) and 0.3 (`wasm_js`) shims for
`wasm32`. `getrandom 0.4.2` has since entered via `uuid 1.23.2` with no
matching shim, which looked like a latent wasm break. **It is not:** `uuid`
target-gates its `getrandom` dep, `cargo tree --target wasm32-unknown-unknown`
shows only 0.2 and 0.3 present, and `cargo check --target
wasm32-unknown-unknown --lib` succeeds (57s, clean). Re-check this if `uuid` is
upgraded past 1.23.

### 4. Unconditional dependencies used only by optional code

Two crates are declared unconditionally but their production usage lives
entirely behind non-default features:

| Crate | Production sites outside `#[cfg(test)]` | Actually reachable via |
|---|---|---|
| `serde_json` | 11 lines, 2 modules | `debug-json` (solver save/load) + `pokerbench` (both non-default) |
| `csv` | 4 sites | `store` (BCM/HUP export) + `pokerbench` |

`serde_json` also keeps a permanently-compiled `From<serde_json::Error> for
SolverError` impl, so making it optional needs that impl feature-gated too.
Small tree win, real feature-graph-hygiene win.

### 5. `bitvec` ships its `testing` feature

`Cargo.toml:66` enables `["alloc", "atomic", "std", "serde", "testing"]`.
`testing` is upstream's own test-support feature and has no business in a
published dependency. Moot if the `replace-std` verdict is executed.

### 6. `strum_macros` is declared redundantly

`strum = { features = ["derive"] }` already re-exports every macro used
(`EnumIter`, `EnumCount`, `AsRefStr`, `Display`). Five files import from
`strum_macros::` directly, keeping the second direct edge alive. Rewriting
those five imports to `strum::` lets the direct `strum_macros` dep be deleted —
no crates leave the tree, but the manifest stops lying about the boundary.

### 7. Version drift and pins

| Crate | Pinned | Latest | Note |
|---|---|---|---|
| `rusqlite` | 0.34.0 | 0.40.1 | 6 minors back; pin is **documented** (`Cargo.toml:91` — 0.35 breaks `HUPResult`). Real debt, not an oversight. |
| `cardpack` ¹ | 0.6.12 | 0.9.0 | 3 minors back; upgrade blocked in practice by `pkstate 0.1.2` also depending on 0.6.x |
| `random_name_generator` ¹ | 0.3.6 | 0.4.0 | Upgrade does not fix the `clap` problem (see finding 1) |
| `rand` | 0.9.4 | 0.10.2 | Major; would need `cardpack`/`pkstate` lockstep |
| `itertools` | 0.14.0 | 0.15.0 | Leaks into public API (`Combinations<…>`) — bumping is a breaking release |
| `log`, `serde`, `serde_json`, `uuid` | — | +1 patch/minor | Routine |

### 8. `cardpack`'s i18n subtree — 25 crates, removable **without any upstream change**

`cardpack 0.6.12` pulls a 41-crate `fluent-templates` → `ignore` → `globset` →
`regex-automata`/`aho-corasick`/`memchr` localization stack that pkcore never
exercises (no call site touches localized or colored card rendering).

**The features are already gated upstream** — no cardpack release is needed:

```toml
# cardpack 0.6.12
default = ["i18n", "colored-display", "yaml", "serde"]
i18n            = ["dep:fluent-templates"]
colored-display = ["dep:colored"]
```

pkcore needs only `yaml` + `serde`. The catch is **Cargo feature unification**:
`pkstate 0.1.2` declares a bare `cardpack = "0.6.9"` with defaults on, so a
pkcore-only change accomplishes **nothing**. Both manifests must set
`default-features = false, features = ["yaml", "serde"]`, which makes this a
two-repo change gated on a `pkstate` release.

Measured effect if both do it — **25 crates leave the host graph (140 → 115)**:

```
bstr  displaydoc  fluent-bundle  fluent-langneg  fluent-syntax
fluent-template-macros  fluent-templates  flume  globset  ignore
intl-memoizer  intl_pluralrules  lock_api  proc-macro-hack  rustc-hash
scopeguard  self_cell  spin  tinystr  type-map  unic-langid
unic-langid-impl  unic-langid-macros  unic-langid-macros-impl  (+ colored)
```

Verify `Pile::<Standard52>::from_str` still works without `i18n` before
committing — that is the only cardpack API pkcore calls (`bard.rs:348`,
`table_celled.rs:1619`).

This subtree is also why `regex`, `serde`, `itertools` and `rand` all show
`0 unique` baggage; trimming it turns `regex`'s number from 0 into 5.

### 9. Tooling and licensing status

- `cargo deny check licenses advisories` → **`advisories ok, licenses ok`**.
  `deny.toml` allows only `MIT`, `Apache-2.0`, `BSD-3-Clause`, `MPL-2.0`,
  `Unicode-3.0`, `Zlib`, with an empty `ignore` list and empty `exceptions`.
- `cargo audit` could **not** run: the network fetch fails on an unsigned
  advisory-db commit, and the cached DB fails to parse (`unsupported CVSS
  version: '4.0'` in `RUSTSEC-2026-0073`). The installed `cargo-audit` is too
  old for CVSS 4.0 entries — **upgrade it**. Advisory coverage this run comes
  from `cargo deny` only.
- `cargo machete` and `cargo license` are not installed; `cargo udeps` is
  installed but needs a nightly toolchain (not run).
- Every license in the tree is permissive and compatible with the
  `MIT OR Apache-2.0` host. No GPL/LGPL/AGPL, no unlicensed crates, so no
  `rewrite`-only constraints apply anywhere.

### 10. Packaging note for any future vendoring

`Cargo.toml:13` excludes `docs/*` from the published `.crate`. **This audit
file does not ship.** If any verdict here becomes a `vendor-partial` /
`vendor-full`, the attribution and full license text must go in a root-level
`LICENSE-THIRD-PARTY.md`, verified with `cargo package --list`. A ledger entry
in `docs/VENDORED.md` alone would not reach consumers.

---

## Third-party dossiers

### serde 1.0.228

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked (1.0.229 available) · **Advisories:** none (`cargo deny`)
- **Features used:** `derive`
- **Usage census:** 53 files, 168 references, **172 derive sites** (83 `Serialize`, 89 `Deserialize`), 96 `#[serde(...)]` attributes, 55 imports
- **Public API leakage:** pervasive — the `Serialize`/`Deserialize` impls on ~86 public types *are* part of pkcore's contract; e.g. `src/card.rs:30` uses `#[serde(deserialize_with = "deserialize_card_index")]` to fix the on-disk representation of the crate's most fundamental type
- **Contract exposure:** total. Hand-history YAML, bot-profile YAML, `StatsRegistry` persistence, postcard solver caches, pokerbench JSON — all downstream repos (pkpy, pknotebook, pkdealer, pkarena0-web) read formats defined by these derives
- **Unique baggage:** 0 (`serde_core`, `serde_derive` also held via `cardpack`)
- **Replaceability:** hard — no meaningful alternative at this saturation
- **Score:** **5** — ecosystem hub: derive machinery with downstream consumers pinned to its output
- **Effort:** XL — run `/epic` before attempting
- **Verdict:** `keep` — argued, not assumed: the cost is 172 attribute sites and a permanent public-contract commitment, but serde is the *reason* pkcore's formats are stable and cross-language readable. Any replacement re-derives the same entanglement with a smaller ecosystem. The ownership fact to hold onto is that **pkcore's persisted formats are serde's output**, so a serde major bump is a pkcore breaking release.

### indexmap 2.14.0

- **License:** Apache-2.0 OR MIT · **Last release:** unchecked · **Advisories:** none
- **Features used:** `rayon`
- **Usage census:** 5 files, 7 references, 4 imports, 0 derive sites
- **Public API leakage:** **yes, load-bearing** — `src/cards.rs:35` `pub struct Cards(pub IndexSet<Card>)` exposes the type as a **public tuple field** on the library's central collection; `src/cards.rs:390` `pub fn index_set(&self) -> &IndexSet<Card>`; `src/analysis/outs.rs:10` `pub struct Outs(IndexMap<usize, Cards>)`; `src/lib.rs:356` imports `indexmap::set::IntoIter` for the public `combinations` return types
- **Contract exposure:** indirect — insertion order determines serialized card order in every persisted format
- **Unique baggage:** 0 on host (also held by `serde_yaml_bw`)
- **Replaceability:** hard — std has no insertion-ordered set; `Vec` + `HashSet` would change `Cards`'s complexity guarantees
- **Score:** **4** — leaks into public API *and* the leaked type is the crate's core collection
- **Effort:** L — run `/epic` before attempting
- **Verdict:** `keep` — deterministic card ordering is a correctness property here, not a convenience, and `pub IndexSet<Card>` means removal is a breaking release for every downstream. The actionable note is narrower: **making that tuple field private** would downgrade this from 4 to 3 and is worth doing independently of any removal.

### uuid 1.23.2

- **License:** Apache-2.0 OR MIT · **Last release:** unchecked (1.24.0 available) · **Advisories:** none
- **Features used:** `serde`, `v4`, `v5`; plus `js` on `wasm32`
- **Usage census:** 18 files, 42 references, 16 imports, 61 constructor/parse call sites
- **Public API leakage:** `src/casino/principal.rs:29` `pub struct Principal(pub Uuid)` (public field), `:33` `pub fn new(id: Uuid)`, `:38` `pub fn id(&self) -> Uuid`; `src/casino/manager.rs:37,120,124` `create_table → Uuid`, `get_table(id: Uuid)`, `remove_table(id: Uuid)`; `src/casino/dealer.rs:631` `pub fn table_id(&self) -> Uuid`; `src/analysis/player_stats.rs:288` `pub fn get(&self, id: Uuid)`
- **Contract exposure:** yes — player and table IDs are serialized into hand-history YAML and the stats registry; `v5` means some IDs are *derived* and must stay reproducible
- **Unique baggage:** 3 (`getrandom 0.4`, `sha1_smol`)
- **Replaceability:** hard — v5 namespace hashing plus a stable string form is a spec, not a utility
- **Score:** **4** — public API leakage across four modules plus persisted-format dependence
- **Effort:** L — run `/epic` before attempting
- **Verdict:** `keep` — the identity scheme is part of the wire contract with pkdealer and the web repos. Note that `uuid` is the sole source of `getrandom 0.4` (cross-cutting finding 2); a `uuid` upgrade should re-verify the wasm32 build (finding 3).

### serde_yaml_bw 2.5.6 *(optional)*

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked · **Advisories:** none
- **Features used:** default. **Optional**, activated by `bot-profiles`, `hand-histories`, `player-stats-persistence`, `bot-training` — all but `bot-training` are in `default`
- **Usage census:** 4 files, 48 references, 0 bare imports (fully-qualified paths)
- **Public API leakage:** `src/hand_history.rs:825` `impl From<serde_yaml_bw::Error> for HandHistoryError` — a public trait impl downstream error handling can pattern-match through
- **Contract exposure:** **highest in the crate.** Every `.yaml` hand history, bot profile, and stats-registry file on disk is this crate's output, and those files are read by pkpy, pknotebook, pkarena0-web and the replay tests. `docs/AUDIT_Fable_5.md` III.2 already records a prior migration cost here
- **Unique baggage:** 0 (its `serde_norway` subtree is shared with `cardpack`)
- **Replaceability:** hard — a YAML emitter swap is a format-compatibility exercise, not a code exercise
- **Score:** **4** — persisted-format contract plus public error-conversion leakage
- **Effort:** L — run `/epic` before attempting
- **Verdict:** `keep` — this crate exists precisely because a prior YAML backend was replaced once; doing it again risks silent format drift in files that are already committed to the repo and to downstream repos. The ownership fact: **pkcore's YAML dialect is serde_yaml_bw's dialect.**

### postcard 1.1.3

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked · **Advisories:** none
- **Features used:** `alloc`, `use-std`, `default-features = false` — the manifest comment at `Cargo.toml:77–79` documents that dropping `heapless-cas` is what removed `atomic-polyfill` / RUSTSEC-2023-0089. That decision is still correct and should not be reverted
- **Usage census:** 4 files, 14 references, 0 bare imports
- **Public API leakage:** `src/analysis/gto/solver.rs:160` `impl From<postcard::Error> for SolverError`
- **Contract exposure:** yes — `SolverResult::save`/`load` (`solver.rs:250,283`) is the default binary format for solver caches, and `analysis/store/embedded/hup_cache.rs:9` decodes the **compiled-in** `HUPS_BIN` blob. Changing serializer breaks both on-disk caches and the embedded artifact
- **Unique baggage:** 2 (`cobs`)
- **Replaceability:** hard — bincode/rkyv would be a format migration with a regeneration step for `generated/hups.db`
- **Score:** **3** — structural: leaks into public API in one bounded place, and persisted artifacts depend on it
- **Effort:** L — run `/epic` before attempting
- **Verdict:** `keep` — two crates of baggage for a format that ships inside the binary. The `default-features = false` line is doing real supply-chain work; keep the comment.

### itertools 0.14.0

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked (0.15.0 available) · **Advisories:** none
- **Usage census:** 6 files, 6 references, 5 imports
- **Public API leakage:** **yes** — `src/cards.rs:242` `pub fn combinations(&self, k: usize) -> Combinations<IntoIter<Card>>` and `src/deck.rs:95` `pub fn combinations(&self, k: usize) -> Combinations<IntoIter<Card, 52>>` return the upstream iterator type directly
- **Contract exposure:** none (iterators are not persisted)
- **Unique baggage:** 0 (`either` also held via `cardpack`/`rayon`)
- **Replaceability:** rewrite-blind ~60 LOC for a lazy k-combinations iterator — genuinely doable, but the *leaked return type* is the cost, not the algorithm
- **Score:** **3** — structural: leaks into public API in one bounded place (two `combinations` methods)
- **Effort:** L — run `/epic` before attempting
- **Verdict:** `keep` — but note the consequence: **`itertools` majors are pkcore breaking releases**, which is why 0.14 → 0.15 is not a routine bump. Boxing these returns behind `impl Iterator<Item = Vec<Card>>` would sever the leak in one small change and is worth considering on its own merits.

### rayon 1.12.0

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked · **Advisories:** none
- **Usage census:** 12 files, 28 references, 10 imports
- **Public API leakage:** `src/cards.rs:247` `pub fn par_combinations(&self, k) -> IterBridge<Combinations<IntoIter<Card>>>` — leaks `rayon` *and* `itertools` types in one signature
- **Contract exposure:** none directly; the `equity` feature's Monte Carlo determinism depends on seeding, not on rayon
- **Unique baggage:** 0 (also held via `indexmap`'s `rayon` feature)
- **Replaceability:** hard — a work-stealing pool is not a weekend rewrite, and the equity engine and `Twos`/`turn_eval` paths depend on it for throughput
- **Score:** **3** — structural: one leaked signature, plus saturation of the analysis subsystem
- **Effort:** L — run `/epic` before attempting
- **Verdict:** `keep` — this is the crate whose value most clearly exceeds its cost. Zero unique baggage, and it is the performance foundation of `analysis::equity`.

### rand 0.9.4

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked (0.10.2 available) · **Advisories:** none
- **Usage census:** 10 files, 84 references, 46 imports
- **Public API leakage:** none found in signatures — `SmallRng`/`SeedableRng` are used behind `SimTable::seeded` and decider internals rather than exposed
- **Contract exposure:** **behavioural** — `bot::sim` and `training::trainer` promise *reproducible* seeded runs (`src/bot/sim.rs:187,393,434`), and the replay/marathon tests assert on them. A `rand` major that changes generator output invalidates recorded fixtures even though no type signature changes
- **Unique baggage:** 0 (also held via `cardpack`)
- **Replaceability:** hard — reproducibility guarantees, not just an API
- **Score:** **3** — structural: saturates the bot subsystem, with a reproducibility contract
- **Effort:** L — run `/epic` before attempting
- **Verdict:** `keep`. The upgrade note matters more than the removal note: **0.9 → 0.10 must be treated as a fixture-invalidating change**, and it also needs `cardpack`/`pkstate` in lockstep.

### rusqlite 0.34.0 *(optional, `store`, non-wasm)*

- **License:** MIT · **Last release:** unchecked (0.40.1 available) · **Advisories:** none
- **Features used:** `bundled` (compiles SQLite from source — the heaviest build-time item in the tree)
- **Usage census:** 6 files, 30 references, 5 imports, plus 26 files in `examples/`+`tests/`
- **Public API leakage:** `Connection` in eight public signatures — `src/analysis/store/db/hup.rs:39,50,65,253,259`, `src/arrays/matchups/sorted_heads_up.rs:117,134`, and `hup.rs:228` returns `rusqlite::Result<Connection>`. All are behind `#[cfg(feature = "store")]`
- **Contract exposure:** yes — the `generated/hups.db` SQLite schema
- **Unique baggage:** 7 (`libsqlite3-sys`, `hashlink`, `fallible-iterator`, `fallible-streaming-iterator`, `foldhash`, `hashbrown 0.17`, `vcpkg`)
- **Replaceability:** hard — the alternative is a different embedded database, i.e. a schema migration
- **Score:** **3** — structural: leaks into public API in one bounded, feature-gated subsystem
- **Effort:** L — run `/epic` before attempting
- **Verdict:** `keep` — feature-gating means non-`store` consumers pay nothing, and the leak is confined to `analysis::store`. **The live issue is the pin, not the dependency**: 0.34 is six minors behind with a documented `HUPResult` breakage at 0.35. That is accruing debt and deserves its own scheduled work item rather than indefinite deferral.

### csv 1.4.0

- **License:** Unlicense/MIT · **Last release:** unchecked · **Advisories:** none
- **Usage census:** 10 files, 36 references, 4 imports; **4 production call sites** (`analysis/store/bcm/index_card_map.rs:55`, `binary_card_map.rs:272`, `analysis/store/db/hup.rs:211,307`, `arrays/matchups/sorted_heads_up.rs:582,597`, `pokerbench/loader.rs:142`) — the remainder are doc examples and tests
- **Public API leakage:** `src/pokerbench/error.rs:61` `impl From<csv::Error> for PokerBenchError` (gated behind the non-default `pokerbench` feature)
- **Contract exposure:** the CSV export format for BCM/HUP generator artifacts in `generated/`
- **Unique baggage:** 2 (`csv-core`)
- **Replaceability:** vendorable, but the used slice (`WriterBuilder` + serde row serialization + `Reader`) is broad enough that hand-rolling would re-import the quoting-and-escaping bugs csv exists to prevent
- **Score:** **2** — spread-shallow: several call sites, one narrow API surface, leakage only under an optional feature
- **Effort:** M
- **Verdict:** `keep` — 2 crates for correct CSV quoting is a good trade. **But make it optional** (cross-cutting finding 4): every production use sits behind `store` or `pokerbench`, so `--no-default-features` consumers currently compile `csv` for nothing.

### serde_json 1.0.150

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked (1.0.151 available) · **Advisories:** none
- **Usage census:** 18 files, 71 references — but only **11 lines outside `#[cfg(test)]`**, in two modules. The other 60 are round-trip assertions in unit tests
- **Public API leakage:** `src/analysis/gto/solver.rs:153` `impl From<serde_json::Error> for SolverError` and `src/pokerbench/error.rs:68` `impl From<serde_json::Error> for PokerBenchError`. The `SolverError` impl is **not** feature-gated, so it compiles unconditionally
- **Contract exposure:** minimal — JSON is the *debug* solver format (`debug-json`, non-default; `solver.rs:390,422` switch on it) and the pokerbench input format
- **Unique baggage:** 1
- **Replaceability:** hard to replace *well*, but nearly free to keep
- **Score:** **2** — spread-shallow: wide test usage, one narrow production surface, no persisted default format
- **Effort:** S
- **Verdict:** `keep` — the honest verdict for a 1-crate dependency behind the most-used serialization crate in Rust. The finding is not "remove it" but "**it should be `optional = true`**", gated on `debug-json` + `pokerbench`, with the `SolverError` impl gated to match. That is a manifest correctness fix, roughly an hour.

### log 0.4.30

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked (0.4.33 available) · **Advisories:** none
- **Usage census:** 32 files, 320 references, 8 imports, **148 macro call sites** (39 `trace!`, 44 `debug!`, 17 `info!`, 21 `warn!`, 27 `error!`)
- **Public API leakage:** none — pkcore emits records, it does not expose `log` types
- **Contract exposure:** none. `testing_logger` and `test-log` (dev-deps) assert on emitted records, which is an internal test contract only
- **Unique baggage:** 1 (itself; it is a pure facade with no dependencies)
- **Replaceability:** rewrite-blind ~30 LOC for a facade — but that would fork pkcore off the ecosystem-standard logging interface every downstream already configures
- **Score:** **2** — spread-shallow: 148 call sites across 32 files, one narrow macro surface, zero leakage; removal would be mechanical but pointless
- **Effort:** M
- **Verdict:** `keep` — the clearest cost/benefit in the audit: **one crate, zero transitive deps, zero public leakage**, in exchange for interoperating with every logging backend a consumer might use (and with the OTel work in `ROADMAP.md`/EPIC-22).

### strum 0.28.0 + strum_macros 0.28.0

- **License:** MIT (both) · **Last release:** unchecked · **Advisories:** none
- **Features used:** `strum` with `derive`
- **Usage census:** `strum` 7 files / 10 references / 9 imports; `strum_macros` 5 files / 6 references / 5 imports. **12 derive sites**: 8 `EnumIter`, 3 `EnumCount`, 1 `AsRefStr`, 1 `strum_macros::Display` — on `Rank`, `Suit`, `CardNumber`, `Position`, `Phases`, `Actions`, and the Razz `California` ranks
- **Public API leakage:** the generated `IntoEnumIterator` / `EnumCount` impls are visible on public enums (`src/rank.rs:11`, `src/suit.rs:7`, `src/casino/position.rs:6`); consumers can and do call `Rank::iter()`
- **Contract exposure:** `strum_macros::Display` on `Phases` (`src/play/phases.rs:7`) and `AsRefStr` on `California` (`src/games/razz/california.rs:66`) produce **user-visible and serialized strings**
- **Unique baggage:** 1 for `strum`; 0 for `strum_macros` (held by `strum`'s `derive` feature)
- **Replaceability:** rewrite-blind ~40 LOC — a `impl_enum_iter!` macro in `src/macros.rs` would cover all 12 sites; pkcore already has a macro module
- **Score:** **2** — spread-shallow: 12 derive sites, one narrow surface, trait leakage but no structural coupling
- **Effort:** M
- **Verdict:** `keep` — the rewrite is feasible but trades a maintained crate for hand-rolled macro maintenance at no tree saving. The concrete fix is finding 6: **rewrite the five `strum_macros::` imports to `strum::` and drop the redundant direct dependency.**

### bitvec 1.1.1 — `replace-std`

- **License:** MIT · **Last release:** unchecked · **Advisories:** none
- **Features used:** `alloc`, `atomic`, `std`, `serde`, **`testing`** (see finding 5)
- **Usage census:** 2 files, 4 references, 4 imports, 0 derive sites
- **Public API leakage:** **none** — no `BitVec`/`BitArray` appears in any public signature
- **Contract exposure:** none
- **Unique baggage:** **5** (`funty`, `radium`, `tap`, `wyz`)
- **Replaceability:** **std.** There are exactly two usages, and both have direct std equivalents:
  1. `src/arrays/matchups/masks/suit_mask.rs:62–68`, `SuitMask::invert` — `view_bits_mut::<Msb0>()` + `reverse()` + `to_bitvec()` + `shift_end(4)` + `load_be::<u8>()` computes a 4-bit nibble reversal. That is `mask.reverse_bits() >> 4` (`u8::reverse_bits`, stable since Rust 1.37). Verified against the crate's own `TYPE_1122` table at `suit_mask.rs:31–44`: 1↔8, 2↔4, 4↔2, 8↔1 — `1u8.reverse_bits() >> 4 == 8`, `2 → 4`, `4 → 2`, `8 → 1`.
  2. `src/casino/table_celled.rs:21,910`, `use bitvec::macros::internal::funty::Fundamental` for a single `idx.as_u8()`. Note the import path is explicitly `macros::internal` — pkcore is reaching into a module upstream marks as private-by-convention. `u8::try_from(idx)` is the std form.
- **Score:** **1** — contained: leaf usage, two call sites, no leakage
- **Effort:** **S** — under an hour, two files
- **Verdict:** **`replace-std`** — 5 crates and a shipped `testing` feature for one nibble reversal and one integer cast. No license text or provenance note is needed: `u8::reverse_bits` is std, so nothing is being re-implemented *from* bitvec. Highest ratio of win to risk in this audit.

### regex 1.12.3 — `replace-std`

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked (1.13.1 available) · **Advisories:** none
- **Features used:** `features = []` — already trimmed to no defaults
- **Usage census:** **1 production file**, 1 `Regex::new` call, 1 import (`src/analysis/nubibus.rs:8,436`). One further use in `examples/retired/pluribus.rs`, which is retired
- **Public API leakage:** none
- **Contract exposure:** none — it parses the Pluribus research dataset's card strings
- **Unique baggage:** **0 via `cardpack`** — `regex-syntax`, `regex-automata`, `aho-corasick` and `memchr` are all held anyway by `cardpack → fluent-templates → ignore → globset`. Direct-edge removal drops **zero** crates from the graph. *(If cross-cutting finding 8 ever lands and cardpack's i18n subtree goes, this number becomes 5.)*
- **Replaceability:** **std.** The pattern is `^(?<dealt>[0-9a-zA-Z|]+)/(?<board>.+)$`, invoked only inside a `if s.contains('/')` guard (`nubibus.rs:435`). That is `s.split_once('/')`. The one behavioural difference: the regex rejects a `dealt` segment containing characters outside `[0-9a-zA-Z|]`, whereas `split_once` would pass it through — but both paths end in `HoleCards::from_pluribus(...).unwrap_or_default()`, so a malformed segment yields `HoleCards::default()` either way. Also removes an `unwrap()` on `Regex::new` from a hot parse path, which the project's no-`unwrap`-in-library rule wants gone
- **Score:** **1** — contained: single leaf call site, no leakage
- **Effort:** **S** — under an hour, one file
- **Verdict:** **`replace-std`** — **be clear that this is an ownership win, not a tree win.** Today it removes 0 crates. What it buys is one fewer direct dependency to track, one fewer `unwrap()`, and a version pin (1.12 → 1.13) that stops mattering. Do it because it is nearly free, not because it shrinks the build.

### percent-encoding 2.3.2 — `drop`

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked · **Advisories:** none
- **Usage census:** **1 file, 1 reference, 0 imports** — `src/util/mod.rs:65–67`, the entire body of `Util::percent_decode`
- **Public API leakage:** `pub fn percent_decode(s: &str) -> Result<String, Utf8Error>` is public, but returns std's `Utf8Error`, so **no upstream type escapes**
- **Contract exposure:** none
- **Unique baggage:** **0 via `wincounter`** — the first-party `wincounter 0.1.6` depends on `percent-encoding` too, so cutting pkcore's direct edge removes nothing from the graph. This is an ownership and dead-code win only
- **Replaceability:** moot — **`Util::percent_decode` has zero callers.** A repo-wide sweep across `src/`, `examples/`, `tests/` and `benches/` finds the definition and nothing else. And the reason `wincounter` also holds `percent-encoding` is that **`src/util/mod.rs` is a fork of `wincounter-0.1.6/src/util.rs`** — `Percentage`, `Util`, `calculate_percentage`, `percent_decode`, `read_lines` and `replace_plus` are byte-for-byte identical down to the doc comment *"Blank struct that is home to misfit utility functions. There is a whole school that argues against util functions and modules like this. Obviously, I am not one of them."* pkcore is maintaining a private copy of a module it already depends on
- **Score:** **1** — contained
- **Effort:** **S** — under an hour, one file
- **Verdict:** **`drop`** — delete `Util::percent_decode` along with the manifest line. Two things to handle deliberately rather than silently:
  1. It is a `pub fn`, so removal is a **breaking change to pkcore's public surface** even though nothing in-repo uses it. Confirm against the downstream repos (the `audit-release` skill covers pkpy / pknotebook / pkdealer / pkgto-web / pkkuhn-web / pkarena0-web) and land it in a minor bump. If a downstream does use it, the verdict softens to `rewrite` — percent-decoding is ~20 lines and the crate is MIT/Apache, so either path is legally clear.
  2. The **duplication is the larger finding.** Three options, in preference order: delegate `pkcore::util::Util` to `wincounter::util::Util` (already a dependency, already public — deletes the fork outright); or keep pkcore's copy and drop the dead function only; or push the shared utilities down into `wincounter` properly. The first is the only one that stops the two copies drifting.

### thousands 0.2.0 — `rewrite`

- **License:** MIT/Apache-2.0 · **Last release:** unchecked (no version newer than 0.2.0 exists) · **Advisories:** none
- **Usage census:** 1 production file, 1 import — `src/casino/cashier/chips.rs:5` `use thousands::Separable`, used to render chip stacks with comma grouping
- **Public API leakage:** none — `Separable` is consumed inside `Display for Chips`, not re-exported
- **Contract exposure:** **user-visible output only** — the formatted string appears in terminal play and commentary. A rewrite must match `separate_with_commas` exactly for the existing display tests to pass
- **Unique baggage:** 1 (itself; no transitive deps)
- **Replaceability:** rewrite-blind ~20 LOC — insert `,` every three digits from the right, preserving sign and any fractional part. This is fully specified by observed output; no upstream source needs to be read
- **Score:** **1** — contained: single leaf call site
- **Effort:** **S** — under an hour, one file
- **Verdict:** `rewrite` — **lowest priority in this report, and worth saying so.** It removes one dependency and zero transitive crates. The argument for doing it is that a 0.2.0 crate with no releases since is a dormant single-point-of-failure for a 20-line formatting concern; the argument against is that the trade is one crate for one more piece of hand-maintained code. If it is done, it needs only a module-doc provenance note (crate name, 0.2.0, MIT/Apache-2.0, `github.com/tov/thousands-rs`) — **no license text and no `VENDORED.md` entry**, since nothing is copied.

### zstd 0.13.3 *(optional, `store`, non-wasm)*

- **License:** MIT · **Last release:** unchecked · **Advisories:** none
- **Usage census:** 1 file, 8 references, 0 bare imports — `src/analysis/store/bcm/binary_card_map.rs:50` (`stream::read::Decoder`) and `:240` (`stream::write::Encoder`)
- **Public API leakage:** none — errors are mapped to `PKError::BcmUnavailable` at the boundary (`binary_card_map.rs:31,51`)
- **Contract exposure:** yes — the compressed BCM artifact format (~300–600 MB uncompressed, per `binary_card_map.rs:208`)
- **Unique baggage:** 3 (`zstd-safe`, `zstd-sys`)
- **Replaceability:** hard in practice — a codec swap invalidates generated artifacts, and no std equivalent exists
- **Score:** **1** — contained: optional/feature-gated, two leaf call sites, no leakage
- **Effort:** M
- **Verdict:** `keep` — textbook well-contained dependency: optional, target-gated, errors mapped at the boundary, two call sites. Nothing to fix.

### termion 4.0.6 *(optional, unix)*

- **License:** MIT · **Last release:** unchecked · **Advisories:** none
- **Usage census:** 3 files, 4 references, 4 imports — `color` in `table_celled.rs:28` and `nubibus.rs:16`, `TermRead`/`IntoRawMode` in `util/terminal.rs:17,19`
- **Public API leakage:** none
- **Contract exposure:** none
- **Unique baggage:** 2 (`numtoa`)
- **Replaceability:** rewrite-blind for the `color` half (ANSI escapes are ~20 LOC); the raw-mode/`TermRead` half is genuine termios work worth delegating
- **Score:** **1** — contained: optional, feature- and target-gated, leaf usage
- **Effort:** S
- **Verdict:** `keep` — correctly gated behind both `terminal` and `cfg(unix)`, so it never reaches wasm or non-Unix consumers. Note it is Unix-only: any Windows support story needs a different crate here, not a removal.

### getrandom 0.2.17 / 0.3.4 *(wasm32-only feature shims)*

- **License:** MIT OR Apache-2.0 · **Last release:** unchecked · **Advisories:** none
- **Features used:** `js` (0.2 alias `getrandom_v2`), `wasm_js` (0.3 alias `getrandom_v3`) — `Cargo.toml:98–99`
- **Usage census:** **0 files, 0 references** — these are pure feature-activation shims, not code dependencies. They exist so transitive `getrandom` copies pick a browser entropy backend on `wasm32`
- **Public API leakage:** none
- **Contract exposure:** none
- **Unique baggage:** 0
- **Replaceability:** n/a — the mechanism (a same-crate alias dependency enabling a feature) is the standard idiom
- **Score:** **1** — contained
- **Effort:** S
- **Verdict:** `keep`, with one conditional: **`getrandom_v3` is required** (`rand 0.9` and `cardpack` both pull `getrandom 0.3`), but **`getrandom_v2` exists only because `random_name_generator → rand 0.8 → rand_core 0.6 → getrandom 0.2`.** Executing the `random_name_generator` verdict makes `getrandom_v2` dead and it should be deleted in the same change. Verified live: `cargo tree --target wasm32-unknown-unknown -i getrandom@0.2.17` shows `rand 0.8` as the only non-shim holder.

---

## First-party dependencies

Five direct dependencies share this project's authorship (`electronicpanopticon`
/ `folkengine`, orgs `ImperialBower` and `ContractBridge`). License risk is moot
for all of them; the questions are two-place maintenance cost, other consumers,
and whether the boundary itself is drawn in the right place.

### random_name_generator 0.3.6 (first-party) — `absorb`

- **Relationship:** author `folkengine` (this repo's git user) —
  `https://github.com/folkengine/random_name_generator_rs`. BSD-3-Clause.
  Library name is `rnglib`, which is why a naive `rg random_name_generator`
  census returns zero source hits
- **Usage census:** **1 file, 1 import, 1 production call site.**
  `src/util/name.rs:1` `use rnglib::{Language, RNG}`; `:6` a `LazyLock<RNG>`
  built with `Language::Demonic`; `:11` `Name::generate()` joins two generated
  names. The single production caller is `src/casino/player.rs:499`
  (`handle: Name::generate()`), which fills in a default player handle
- **Public API leakage:** **none of `rnglib`'s types** — `Name` is re-exported
  from `src/prelude.rs:72`, but its surface is `fn generate() -> String`.
  Removal changes no signature
- **Unique baggage:** **31 crates** — `clap`, `clap_builder`, `clap_derive`,
  `clap_lex`, `anstream`, `anstyle`, `anstyle-parse`, `anstyle-query`,
  `colorchoice`, `is_terminal_polyfill`, `utf8parse`, `strsim`, `anyhow`,
  `lazy_static`, `titlecase`, `joinery`, `rust-embed`, `rust-embed-impl`,
  `rust-embed-utils`, `sha2`, `digest`, `block-buffer`, `crypto-common`,
  `generic-array`, `typenum`, `cpufeatures`, `rand 0.8`, `rand_chacha 0.3`,
  `rand_core 0.6`, `getrandom 0.2`. **18% of the host graph, for one line.**
- **Absorption analysis:**
  - **(a) Two-place maintenance:** pkcore uses one constructor and one method
    of this crate. Folding in a name generator costs ~60 LOC plus a syllable
    table; maintaining it in a separate published crate costs a release cycle
    per change. For a surface this small the standalone crate is pure overhead
    *to pkcore*.
  - **(b) Other consumers:** `random_name_generator` is published on crates.io
    and stands alone as a general-purpose fantasy-name library — absorbing it
    into pkcore would not remove it from the world, and other users are
    unaffected. Nothing breaks downstream.
  - **(c) Is the boundary wrong?** **Yes — and this is the real finding.** The
    crate is not heavy; its *manifest* is. `clap`, `anyhow`, `rust-embed`,
    `titlecase`, `regex` and `rand` are declared as **non-optional normal
    dependencies** because the crate's CLI binary is not feature-gated. Every
    library consumer pays for a command-line parser it can never call. Checked
    against the registry index: **0.4.0 has the same problem** (`clap ^4.6.2`
    normal, plus `rand ^0.10` which would add a third `rand` major here).
- **Score:** **1** — contained: leaf usage, one file, no leakage
- **Effort:** **M** — one focused session either way; cross-repo if fixed
  upstream, but the upstream change is small
- **Verdict:** **`absorb`**, with a strong preference for fixing the boundary
  first. In priority order:
  1. **Preferred — fix upstream.** In `random_name_generator`, move `clap` and
     `anyhow` behind a `cli` feature with `required-features` on the `[[bin]]`.
     That is a small change to a repo this author controls, it fixes the
     problem for every consumer, and pkcore then keeps the dependency at a cost
     of ~6 crates instead of 31.
  2. **Otherwise — absorb.** Move `Name::generate` to an in-repo generator
     (BSD-3-Clause, same author, so the syllable data may be copied outright
     with attribution). `src/util/name.rs` is the only file that changes;
     `player.rs:499` and `prelude.rs:72` are untouched.

  Either path **also removes `rand 0.8`, `rand_chacha 0.3`, `rand_core 0.6`,
  `getrandom 0.2`, and the now-dead `getrandom_v2` wasm shim** (cross-cutting
  finding 2). Because code would be copied under path 2, that route needs a
  `docs/VENDORED.md` entry **and** a root-level `LICENSE-THIRD-PARTY.md`
  carrying the BSD-3-Clause text — `docs/*` is excluded from the published
  crate (finding 10).

### wincounter 0.1.6 (first-party)

- **Relationship:** `ImperialBower/wincounter` — same org. MIT
- **Usage census:** 13 files, 35 references, 28 imports
- **Public API leakage:** **maximal, and deliberate** — `src/prelude.rs:9–12`
  re-exports `PlayerFlag`, `WinResults`, `Win`, and `Wins` wholesale. They then
  appear in public signatures: `src/play/game.rs:347` `turn_calculations(&self)
  -> (CaseEvals, Wins, WinResults, Outs)`, `src/analysis/case_evals.rs:100`
  `wins(&self) -> Wins`, `src/arrays/matchups/sorted_heads_up.rs:733`
  `wins(&self) -> Result<Wins, PKError>`, `src/analysis/store/db/hup.rs:181`
- **Contract exposure:** yes — `Wins` values are serialized into HUP records
  and CSV exports, and every downstream that calls `turn_calculations` handles
  these types by name
- **Unique baggage:** 1 (itself). It also holds `percent-encoding`, which is why pkcore's direct edge to that crate is worth 0 (see the `percent-encoding` dossier)
- **Code duplication:** `src/util/mod.rs` is a **verbatim fork of `wincounter-0.1.6/src/util.rs`** — `Percentage`, `Util`, `calculate_percentage`, `percent_decode`, `read_lines`, `replace_plus`, identical to the doc comments. Two copies of the same module, one of which is already a dependency of the other. Worth resolving independently of any verdict here
- **Absorption analysis:**
  - **(a) Two-place maintenance:** moderate, and **already visibly costing** — the `util` fork above is exactly the drift this boundary invites. Four types are re-exported through
    pkcore's prelude, so downstream consumers already experience them as pkcore
    API while the source of truth lives in another repo. Any change to `Wins`
    is a two-repo, two-release operation.
  - **(b) Other consumers:** published on crates.io. Win-counting over player
    bitflags is not poker-specific, so a standalone crate is defensible and
    absorption would strand any external user.
  - **(c) Is the boundary wrong?** Arguably yes in *presentation* — a prelude
    re-export erases the boundary for consumers without any of the isolation
    benefit. But the domain split is clean and the crate carries 1 crate of
    baggage, so the boundary is cheap even if the seam is invisible.
- **Score:** **5** — ecosystem hub: types are re-exported into pkcore's prelude
  and downstream consumers are pinned to their shape
- **Effort:** L — run `/epic` before attempting
- **Verdict:** `keep` — argued explicitly: absorption would save exactly one
  crate and would force pkcore to own a general-purpose abstraction that is
  usefully separate. What the score-5 rating buys is the knowledge that
  **`wincounter` is effectively part of pkcore's public API** — a semver bump
  there is a semver bump here, and it should be released in lockstep.

### pkstate 0.1.2 (first-party)

- **Relationship:** `ImperialBower/pkstate` — same org. MIT OR Apache-2.0
- **Usage census:** 3 files, 19 references, 1 import
- **Public API leakage:** two public trait impls — `src/casino/table_celled.rs:1586`
  `impl From<&TableCelled> for pkstate::PKState` and `:1776`
  `impl From<TableCelled> for pkstate::PKState`. The conversion body reaches
  into `pkstate::seat::Seat`, `pkstate::act::{Action, Round}`,
  `pkstate::game::{ForcedBets, GameType}`
- **Contract exposure:** **this is the point of the crate** — `PKState` is the
  interchange snapshot format shared with the gRPC/dealer side of the platform
  (`ROADMAP.md`)
- **Unique baggage:** 4 (`chrono`, `iana-time-zone`, `iana-time-zone-haiku`,
  `core-foundation-sys`). Its 76-crate subtree is otherwise `cardpack`, which
  pkcore holds directly anyway
- **Absorption analysis:**
  - **(a) Two-place maintenance:** low. pkcore consumes `PKState` at exactly
    one seam and does not extend it.
  - **(b) Other consumers:** the whole reason it exists — the sibling gRPC
    service and any other repo speaking the same snapshot format. Absorbing it
    into pkcore would **destroy the shared-schema property** that justifies it.
  - **(c) Is the boundary wrong?** No. A shared wire-format crate consumed via
    `From` impls at a single module boundary is the boundary working correctly.
- **Score:** **3** — structural: leaks into public API in one bounded place
  (`table_celled.rs`), with wire-contract exposure
- **Effort:** L — run `/epic` before attempting
- **Verdict:** `keep` — the clearest `keep` among the first-party crates.
  One note: **`pkstate 0.1.2` pins `cardpack 0.6.x`**, which is what blocks
  pkcore's own `cardpack` upgrade (finding 7) and keeps the i18n subtree alive
  (finding 8). Any cardpack work starts here.

### cardpack 0.6.12 (first-party)

- **Relationship:** `ImperialBower/cardpack.rs` (docs also reference
  `ContractBridge/cardpack.rs`) — same org. Apache-2.0
- **Usage census:** **2 production sites.** `src/bard.rs:7,343,348` —
  `Bard::to_pile()` parses via `CPile::<Standard52>::from_str` and returns
  `Option<BasicPile>`; `src/casino/table_celled.rs:1615–1619` — parses a board
  string as `Pile<Standard52>` inside the `PKState` conversion. The remaining
  hits are doc-comment references (`cards.rs:345`, `analysis/eval.rs:65`)
- **Public API leakage:** `src/bard.rs:343` `pub fn to_pile(&self) ->
  Option<BasicPile>` returns an upstream type
- **Contract exposure:** none directly — pkcore has its own `Card`/`Cards`
  representation; cardpack is used for string parsing and interop
- **Unique baggage:** **0 via `pkstate`.** Its own subtree is 65 crates —
  `fluent-templates` → `ignore` → `globset` → `regex-automata`/`aho-corasick`/
  `memchr`, an i18n stack pkcore never exercises — but `pkstate 0.1.2` depends
  on `cardpack` too, so cutting pkcore's direct edge removes nothing
- **Absorption analysis:**
  - **(a) Two-place maintenance:** near zero for pkcore — two call sites, both
    string-parsing conveniences.
  - **(b) Other consumers:** a general-purpose card-deck library with a real
    audience beyond poker. Absorbing it would be plainly wrong.
  - **(c) Is the boundary wrong?** The *dependency* boundary is fine and the
    **feature** boundary is already correct upstream — `i18n` and
    `colored-display` are optional in cardpack 0.6.12. What is wrong is the
    **consumer** side: both pkcore and `pkstate 0.1.2` take cardpack with
    default features, so every downstream pays for a 41-crate localization
    stack neither of them calls.
- **Score:** **2** — spread-shallow: two call sites, one narrow API surface,
  a single leaked return type
- **Effort:** M for pkcore's own usage; **M–L** for the tree win — no upstream
  cardpack work is needed, but it requires a `pkstate` release, so the two
  manifests land in lockstep
- **Verdict:** `keep` — with the real work in the **manifests, not the code**:

  ```toml
  # in BOTH pkcore and pkstate
  cardpack = { version = "0.6.9", default-features = false, features = ["yaml", "serde"] }
  ```

  Measured at **25 crates** (cross-cutting finding 8) — the largest remaining
  reduction after the `random_name_generator` and `bitvec` verdicts, and
  cheaper than either since no pkcore source changes at all. A pkcore-only
  change does nothing: Cargo unifies features across the graph, and `pkstate`
  holds cardpack with defaults on. Fold the 0.6.12 → 0.9.0 upgrade into the
  same lockstep release.

### bint 0.1.16 (first-party)

- **Relationship:** `electronicpanopticon/bint-rs` — this crate's own author.
  MIT
- **Usage census:** **1 file, 1 import** — `src/casino/table_celled.rs:20`
  `use bint::{BintCell, DrainableBintCell}`
- **Public API leakage:** `src/casino/table_celled.rs:146` `pub button:
  BintCell` — a **public field** on `TableCelled`. Also constructed at `:229`,
  `:774`, `:1346`, `:1505`
- **Contract exposure:** none — `button` serializes as its underlying value
- **Unique baggage:** 1 (itself; no transitive dependencies)
- **Absorption analysis:**
  - **(a) Two-place maintenance:** negligible — a bounded-integer cell type
    that is essentially finished.
  - **(b) Other consumers:** published standalone; a wrapping bounded counter
    is domain-neutral.
  - **(c) Is the boundary wrong?** No — and here the *usage* is the argument.
    `src/casino/table.rs:1–7` documents `TableCelled` as the
    "teaching/benchmark twin" of the primary `Table`, existing specifically to
    demonstrate interior mutability (`Cell`, `RefCell`, `BintCell`,
    `CardsCell`). `BintCell` is not an implementation detail of that type; it
    is part of what the type exists to show.
- **Score:** **3** — structural: leaks into public API in one bounded place
  (a public field on one struct)
- **Effort:** M
- **Verdict:** `keep` — removing it would defeat the documented purpose of the
  only type that uses it, at a saving of one dependency-free crate. Worth
  recording as a **conditional**, though: if `TableCelled` is ever retired in
  favour of `Table` (see `docs/ANALYSIS_TableCelled_vs_Table.md`), `bint`'s
  last consumer goes with it and the verdict becomes `drop` for free.

---

## Dev-dependencies

Never shipped to consumers; they add 85 crates to the host build (112 across
all targets) for local builds and CI only. All licenses are permissive and pass
`cargo deny`.

| Dependency | Version | License | Role |
|---|---|---|---|
| clap | 4.6.1 | MIT OR Apache-2.0 | Example CLI argument parsing |
| clap-repl | 0.3.2 | MIT OR Apache-2.0 | Interactive REPL examples (`bcrepl`, Kuhn) |
| criterion | 0.5.1 | Apache-2.0 OR MIT | `benches/preflop_odds` harness |
| elr_primes | 0.1.2 | MIT OR Apache-2.0 | Prime generation for lookup-table tests |
| env_logger | 0.11.10 | MIT OR Apache-2.0 | Log output in examples |
| reedline | 0.38.0 | MIT | Line editing under `clap-repl` |
| rstest | 0.26.1 | MIT OR Apache-2.0 | Parameterized test cases |
| serde_test | 1.0.177 | MIT OR Apache-2.0 | Serde impl round-trip assertions |
| serde_yaml_bw | 2.5.6 | MIT OR Apache-2.0 | YAML in tests when the feature is off |
| test-log | 0.2.21 | Apache-2.0 OR MIT | Auto-init logging per test |
| testing_logger | 0.1.1 | BSD-3-Clause | Assertions on emitted log records |

Note: `clap-repl 0.3.2` and `criterion 0.5.1` are the source of most entries in
`cargo tree -d` (`clap_lex` 0.7/1.1, `strum` 0.26/0.28, `thiserror` 1.0/2.0,
`unicode-width` 0.1/0.2, `crossbeam-utils`). **None of these reach consumers**
— do not chase them as if they were shipping bloat.

---

## Evidence appendix

Raw outputs for diffing against the next audit run.

### Tool availability

| Tool | Status |
|---|---|
| `cargo metadata --format-version 1` | ran — 308 packages (unfiltered: all targets, all optional edges) |
| `cargo tree -e normal` | ran — 172 normal-graph crates all-targets, **140 host**, default features |
| `cargo tree -d` | ran — see below |
| `cargo tree -i <dep>` / per-dep subtree diff | ran for all 25 host direct deps |
| `cargo check --target wasm32-unknown-unknown --lib` | ran — **clean**, 57.03s |
| `cargo deny check licenses advisories` | ran — **`advisories ok, licenses ok`** |
| `cargo audit` | **failed both ways** — network fetch rejects an unsigned advisory-db commit; cached DB fails to parse (`unsupported CVSS version: '4.0'`, `RUSTSEC-2026-0073`). Installed `cargo-audit` predates CVSS 4.0 support. **Action: upgrade `cargo-audit`.** |
| `cargo machete` | not installed |
| `cargo udeps` | installed, requires nightly — not run |
| `cargo license` | not installed (licenses taken from `cargo metadata`) |

### Duplicate versions

Full `cargo tree -d` includes many dev-only duplicates. Filtered to the
**normal (non-dev) graph, all targets**:

```
getrandom:    v0.2.17  v0.3.4  v0.4.2
hashbrown:    v0.15.5  v0.17.1
r-efi:        v5.3.0   v6.0.0
rand:         v0.8.6   v0.9.4
rand_chacha:  v0.3.1   v0.9.0
rand_core:    v0.6.4   v0.9.5
wit-bindgen:  v0.51.0  v0.57.1
```

Dev-only duplicates (ignore for shipping purposes): `clap_lex` 0.7.7/1.1.0,
`crossbeam-utils` ×2, `itertools` 0.10.5/0.12.1/0.14.0, `strum` 0.26.3/0.28.0,
`strum_macros` 0.26.4/0.28.0, `thiserror` 1.0.69/2.0.18, `unicode-width`
0.1.14/0.2.2.

Attribution of the normal-graph duplicates:

```
clap v4.6.1
└── random_name_generator v0.3.6 → pkcore          # sole non-dev holder of clap

getrandom v0.2.17
├── pkcore (wasm32 shim `getrandom_v2`)
└── rand_core v0.6.4 → rand v0.8.6 → random_name_generator v0.3.6 → pkcore

getrandom v0.3.4
├── cardpack v0.6.12 → pkcore / pkstate
├── pkcore (wasm32 shim `getrandom_v3`)
└── rand_core v0.9.5 → rand v0.9.4 → pkcore

getrandom v0.4.2
└── uuid v1.23.2 → pkcore                          # absent on wasm32 (target-gated)
```

### Per-dependency census

`hits` = all textual references in `src/`; `prod` distinguishes call sites
outside `#[cfg(test)]` where the two differ materially.

| Dependency | Files | References | Imports | Derive sites | Notes |
|---|---|---|---|---|---|
| serde | 53 | 168 | 55 | **172** (+96 `#[serde(…)]` attrs) | 83 `Serialize`, 89 `Deserialize` |
| log | 32 | 320 | 8 | 0 | 148 macro calls: 39/44/17/21/27 t/d/i/w/e |
| uuid | 18 | 42 | 16 | 0 | 61 constructor/parse sites |
| serde_json | 18 | 71 | 0 | 0 | **11 prod lines**; rest are tests |
| wincounter | 13 | 35 | 28 | 0 | 4 types re-exported from prelude |
| rayon | 12 | 28 | 10 | 0 | |
| csv | 10 | 36 | 4 | 0 | **4 prod sites**; +21 files in examples/tests |
| rand | 10 | 84 | 46 | 0 | |
| strum | 7 | 10 | 9 | 12 | shared derive count with strum_macros |
| itertools | 6 | 6 | 5 | 0 | |
| rusqlite | 6 | 30 | 5 | 0 | +26 files in examples/tests |
| indexmap | 5 | 7 | 4 | 0 | |
| strum_macros | 5 | 6 | 5 | — | |
| cardpack | 4 | 6 | 1 | 0 | **2 prod sites**; rest doc comments |
| postcard | 4 | 14 | 0 | 0 | |
| serde_yaml_bw | 4 | 48 | 0 | 0 | |
| pkstate | 3 | 19 | 1 | 0 | |
| termion | 3 | 4 | 4 | 0 | |
| bitvec | 2 | 4 | 4 | 0 | **2 real usages** |
| thousands | 2 | 2 | 1 | 0 | **1 real usage** |
| bint | 1 | 1 | 1 | 0 | |
| percent-encoding | 1 | 1 | 0 | 0 | **0 callers of its only consumer** |
| regex | 1 | 1 | 1 | 0 | +1 retired example |
| zstd | 1 | 8 | 0 | 0 | |
| random_name_generator | 1 | 2 | 1 | 0 | imported as `rnglib`; **1 prod caller** |
| getrandom_v2 / getrandom_v3 | 0 | 0 | 0 | 0 | feature shims only |

### Unique baggage (host target, `aarch64-apple-darwin`)

Method: for each direct dependency *D*, `subtree(D) \ ⋃ subtree(other directs)`.
"Unique" therefore means *these crates leave the graph entirely if D vanishes*,
which is stricter (and more useful) than cutting the direct edge.

| Dependency | Subtree | Unique | Unique members |
|---|---|---|---|
| random_name_generator | 49 | **31** | anstream, anstyle, anstyle-parse, anstyle-query, anyhow, block-buffer, clap, clap_builder, clap_derive, clap_lex, colorchoice, cpufeatures, crypto-common, digest, generic-array, getrandom, is_terminal_polyfill, joinery, lazy_static, rand, rand_chacha, rand_core, rust-embed, rust-embed-impl, rust-embed-utils, sha2, strsim, titlecase, typenum, utf8parse |
| rusqlite | 9 | 7 | fallible-iterator, fallible-streaming-iterator, foldhash, hashbrown, hashlink, libsqlite3-sys, vcpkg |
| bitvec | 12 | **5** | funty, radium, tap, wyz |
| pkstate | 76 | 4 | chrono, core-foundation-sys, iana-time-zone (+ haiku) |
| uuid | 6 | 3 | getrandom 0.4, sha1_smol |
| zstd | 3 | 3 | zstd-safe, zstd-sys |
| termion | 3 | 2 | numtoa |
| postcard | 11 | 2 | cobs |
| csv | 6 | 2 | csv-core |
| bint, thousands, wincounter, strum, serde_json | 1–9 | 1 | (self only) |
| cardpack | 65 | **0** | entire subtree also held by `pkstate` |
| percent-encoding | 1 | **0** | also held by `wincounter` |
| serde, serde_yaml_bw, indexmap, itertools, rayon, rand, regex, log, strum_macros | — | 0 | held by another direct dep (mostly `cardpack`) |

Direct-edge-removal counts (what `cargo tree -i` alone would suggest) differ
from the above for `cardpack` (0 unique but a 65-crate subtree) and for
`regex`/`serde`/`itertools`/`rand` (0 unique because `cardpack`'s
`fluent-templates`/`serde_norway` stack holds their subtrees). The
node-vanishes number is the one quoted in the Summary table.

### Graph size baselines

All counts exclude `pkcore` itself, default features, `cargo tree` (not
`cargo metadata`, whose 308-package figure is unfiltered by target and by
optional-edge activation and is **not** comparable):

```
normal (non-dev) graph, all targets : 172 crates
normal graph, host target           : 140 crates
with dev-dependencies, host         : 225 crates  (dev-only adds  85)
with dev-dependencies, all targets  : 284 crates  (dev-only adds 112)
src/ Rust files                     : 170
src/ lines                          : 113,484
```

Projected after executing the `S`-effort verdicts plus the
`random_name_generator` fix — 31 (`random_name_generator`) + 5 (`bitvec`) + 1
(`thousands`), with `percent-encoding` and `regex` contributing 0:

```
host graph after : ~103 crates  (−37, −26%)
```

`clap`, `rand 0.8`, `rand_chacha 0.3`, `rand_core 0.6`, `getrandom 0.2`, the
`rust-embed`/`sha2`/`digest` block and the `funty`/`radium`/`tap`/`wyz` block
all leave the graph.

Adding cross-cutting finding 8 (`cardpack` `default-features = false` in both
pkcore and pkstate, measured at 25 crates):

```
host graph after both : ~78 crates  (−62, −44%)
```

---

## Notes (human)

<!-- Never regenerated. Add anything here that should survive future
     /untangle runs. -->
