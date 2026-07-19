# EPIC-50: Transport Unification & Gateway Foundation (GATE)

One wire-protocol family, one port, one auth header
(`Authorization: Bearer <token>`) reachable from native clients, browsers,
and mobile — plus the single pure seam pkcore contributes so the gateway
can distinguish players without the kernel ever learning what a token is.

> **The implementation of this EPIC lives in a new sibling repo,
> [`pkgate`](https://github.com/ImperialBower/pkgate) (to be created).**
> This document is the **pkcore-side contract**: the `Principal` identity
> seam and per-viewer redaction that pkgate consumes, plus the transport
> decisions the whole EPIC-50–53 suite is built on. It is the first EPIC
> in the **EPIC-50–59 ten-block** claimed by `pkgate` under the ROADMAP
> Numbering Policy (`ROADMAP.md:374-382`). Design of record:
> `docs/superpowers/specs/2026-07-19-networking-security-epics-design.md`.

The kata: the **Thing** is a **`Principal`** — the settled identity of
whoever is acting. The **Business Requirement** is that pkcore must be
able to *distinguish* actors (so stats accumulate, so a seat resumes, so
a view is redacted) while *never authenticating* them — no token, claim,
or secret may enter the kernel. The **Business Logic** is a `Uuid`
newtype plus one pure redaction function, driven out test-first; every
other layer (verification, scopes, TLS) is pgate's, downstream.

---

## Status

*As of 2026-07-19, crate `0.3.1` (`Cargo.toml:4`). No pkgate repo exists
yet; no code has landed. pkcore-side rows are the only ones that touch
this crate.*

| Component | Status | Repo |
|---|---|---|
| Transport decision (Connect RPC vs. Tonic + `tonic-web` fallback) | Planned — Phase 0 spike | pkgate |
| `pkgate_tower` — Tower auth/observability `Layer` stack | Planned | pkgate |
| pkdealer / pkspectator adopt the layer (behavior-preserving) | Planned | pkdealer, pkspectator |
| **`Principal` newtype** (pure identity seam) | 🔒 Gated (design only) | **pkcore** |
| **`uuid` `v5` feature** (deterministic IdP-sub mapping, EPIC-51) | 🔒 Gated (design only) | **pkcore** |
| **Per-viewer redaction** `for_principal` on the EPIC-37 `SessionView` | 🔒 Gated — blocked on EPIC-37 | **pkcore** |

---

## Context

Where transport and identity stand today:

- **Four platforms, four transport stories, no shared security layer.**
  Native agents speak gRPC to `pkdealer_service`; browsers cannot (gRPC
  needs HTTP/2 trailers the browser fetch API will not expose), so
  `pkspectator` bridges via Axum + SSE (`ROADMAP.md:66`, EPIC-21); the
  WASM web apps (pkgto-web, pkarena0-web) avoid networking entirely and
  embed their data. There is no single ingress a browser, a phone, and a
  bot can all authenticate against the same way.
- **Auth is a POC placeholder, not a layer.** The only decision on record
  is a *shared secret token* — gRPC metadata for players, header/query
  param for the spectator SSE endpoint — explicitly marked replaceable
  by "JWT + OAuth2 without restructuring" (`ROADMAP.md:710-715`). Seat
  resume uses an ad-hoc `client_secret` (`ROADMAP.md:367`).
- **Authorization is one rule, implemented downstream.** "The server
  knows all cards; `GetStatus` returns hole cards only for the requesting
  player's seat; a separate admin/spectator token reveals all"
  (`ROADMAP.md:401-403`). This lives in pkdealer's RPC handler, not in
  pkcore, so every new transport must re-implement it.
- **pkcore already has the identity atom, unshaped.** A player *is* a
  `Uuid`: `Player.id: Uuid` (`src/casino/player.rs:10-11`), carried by
  `Seat.player: Player` (`src/casino/table/seat.rs:26-27`), announced as
  `TableAction::PlayerSeated(u8, Uuid)` (`src/casino/table.rs:352`), and
  used as the `StatsRegistry` key —
  `players: HashMap<Uuid, PlayerStats>` (`src/analysis/player_stats.rs:265-266`).
  There is no *named* identity type; the raw `Uuid` is passed around.
- **No per-viewer redaction exists in-core.** `src/play/visibility.rs:28`
  models only `Up`/`Down` (face-up stud cards vs. concealed), and the
  module doc is explicit that "the `Card` type itself stays
  visibility-free" (`src/play/visibility.rs:1-6`). Redacting a table view
  *by who is looking* is planned only as EPIC-37's `SessionView::view(viewer)`
  sketch (`docs/EPIC-37_Mobile_Engine.md:238,250,265`) — `SessionView`
  and `SeatView` do **not** exist in `src/` yet.
- **pkcore is transport-pure and must stay so.** No `tonic`, `tower`,
  `axum`, `rustls`, or `jsonwebtoken` appears anywhere in `Cargo.toml`.
  `uuid` is already a dependency with `features = ["serde", "v4"]`
  (`Cargo.toml:115`; wasm variant adds `js` at `:128`) — but **not**
  `v5`.

**This EPIC does NOT:** add any transport, crypto, or token dependency to
pkcore (that is pgate's, and forbidden here); define token formats
(EPIC-51), scopes (EPIC-52), or client helper crates (EPIC-53); build the
pkgate repo itself; or change any existing pkcore public signature. Its
entire pkcore footprint is the `Principal` newtype, the `uuid` `v5`
feature flag, and a redaction method that rides on EPIC-37's `SessionView`.

---

## Goals

- Choose **one wire-protocol family** reachable from native, browser, and
  mobile clients through a single port with a single `Authorization`
  header — **Connect RPC** if the Phase-0 spike validates a Rust server
  path, else **Tonic + `tonic-web`** (same single-port, header-auth,
  browser-reach properties, minus Connect's third JSON protocol).
- Build **`pkgate_tower`**: a reusable Tower `Layer` stack (auth-header
  extraction → `TokenVerifier` → **`Principal`** injection → structured
  rejection → per-principal rate-limit hook → OTel span attributes) that
  mounts on both Tonic (gRPC) and Axum (SSE), since both are Tower
  services.
- Give pkcore a **named identity Thing** — `Principal`, a `Uuid` newtype
  — so the "distinguish, never authenticate" contract is expressed in the
  type system, and move the **hole-card redaction rule** out of pkdealer's
  handler into **one pure, unit-testable kernel function**.
- Preserve **domain-kernel purity**: the token → `Principal` mapping
  happens entirely at the pgate edge; the kernel never sees a token.

## Scope

- The gateway exposes exactly one listening port; a browser, a bot
  binary, and a mobile app all authenticate by presenting the same
  `Authorization: Bearer <token>` header, verified by the same code.
- Adopting `pkgate_tower` in pkdealer is **behavior-preserving**: the
  existing shared-secret clients keep working unchanged (auth is
  *relocated* into middleware, not *upgraded* — the strangler pattern).
  Upgrading the token format is EPIC-51's job.
- `Principal` is a newtype over the existing player `Uuid`
  (`src/casino/player.rs:11`); constructing one performs **no**
  authentication and reads **no** token. `StatsRegistry`, seat resume,
  and hand histories keep working because the underlying key is unchanged.
- Redaction is a **pure function** of `(SessionView, Principal)`:
  no view returned for any principal ever contains a non-owned hole card,
  and no view — for any principal, ever — contains the undealt deck.
- pkcore gains **no** transport/crypto/token dependency. The only new
  Cargo line is enabling `uuid`'s `v5` feature (consumed by EPIC-51).

---

## Domain map

| Concept | Code construct | Status |
|---|---|---|
| Settled actor identity | `Principal(Uuid)` — new, pkcore | ❌ this EPIC (seam) |
| Underlying identity atom | `Player.id: Uuid` (`src/casino/player.rs:11`) | ✅ exists |
| Identity as a stats key | `StatsRegistry: HashMap<Uuid, _>` (`src/analysis/player_stats.rs:266`) | ✅ exists |
| Per-viewer table read-out | `SessionView`/`SeatView` (EPIC-37, planned) | 🟡 planned elsewhere |
| Redact a view by viewer | `SessionView::for_principal` — new, pkcore | ❌ this EPIC (seam) |
| Unified wire transport | Connect RPC / `tonic-web` | ❌ pkgate (Phase 0) |
| Edge auth middleware | `pkgate_tower` Tower `Layer` | ❌ pkgate |
| Token → identity mapping | `TokenVerifier` (EPIC-51) | ❌ pkgate |

---

## Design

### `Principal` — the pure identity seam (pkcore)

`src/casino/principal.rs` (new, always compiled — costs nothing, gates
nothing):

```rust
//! The identity of an actor, as far as the domain kernel is concerned.
//!
//! A `Principal` names *who* is acting; it says nothing about *how* they
//! proved it. Authentication — tokens, claims, signatures — happens at the
//! transport edge (see the `pkgate` repo, EPIC-50–53). Constructing a
//! `Principal` never verifies anything; it only wraps a stable id.

use uuid::Uuid;

/// A stable, opaque actor identity: a newtype over the player `Uuid` that
/// already keys seating (`Player::id`) and `StatsRegistry`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub struct Principal(pub Uuid);

impl Principal {
    #[must_use]
    pub fn new(id: Uuid) -> Self { Self(id) }

    /// The underlying player id used by seating and stats.
    #[must_use]
    pub fn id(&self) -> Uuid { self.0 }
}

impl From<Uuid> for Principal { fn from(id: Uuid) -> Self { Self(id) } }
impl From<Principal> for Uuid { fn from(p: Principal) -> Uuid { p.0 } }
```

Rationale: a *newtype*, not a fresh id space. `Player.id`
(`src/casino/player.rs:11`) and `StatsRegistry`'s `HashMap<Uuid, _>`
(`src/analysis/player_stats.rs:266`) already use `Uuid`; `Principal`
gives that atom a name and a contract ("the thing the edge resolves a
token to") without changing any key. The derive set mirrors the wire
enums (`TableAction` `src/casino/action.rs:88`, `PKError` `src/lib.rs:444`)
so it is serde-stable from day one. Registered in the prelude beside the
other casino types (`src/prelude.rs:175-178`).

**Why in pkcore at all, if pkgate does auth?** Because the *consumers* of
identity are in pkcore — stats, seat resume, redaction. The edge resolves
a token to a `Principal` and hands only that inward. The kernel gains a
name for a thing it already had; it gains no dependency and no I/O.

### `uuid` `v5` feature — deterministic IdP mapping (pkcore, for EPIC-51)

`Cargo.toml:115` today:

```toml
uuid = { version = "1.22", features = ["serde", "v4"] }
```

becomes:

```toml
uuid = { version = "1.22", features = ["serde", "v4", "v5"] }
```

Rationale: EPIC-51 maps an OIDC `sub` (an opaque per-issuer string) to a
`Principal` via `Uuid::new_v5(namespace, issuer + sub)` — deterministic
and stateless, so the same login always yields the same `Principal` and
`StatsRegistry` accumulates across sessions. The *mapping function* lives
in pgate (`pkgate_tokens`), but the `v5` generator must be enabled on the
shared `uuid` dependency, and pkcore owns `Cargo.toml`. This is the only
dependency change in the entire suite. The wasm variant (`Cargo.toml:128`)
gains `v5` identically.

### Per-viewer redaction — `SessionView::for_principal` (pkcore, blocked on EPIC-37)

The authorization rule that lives in pkdealer's `GetStatus` handler today
(`ROADMAP.md:401-403`) becomes one pure kernel function on EPIC-37's
planned `SessionView` (`docs/EPIC-37_Mobile_Engine.md:238-265`):

```rust
impl SessionView {
    /// Redact this view for `viewer`: hole cards survive only on the
    /// seat this principal owns; every other seat's are `None`. A
    /// `None` principal is a spectator (all hole cards hidden). No
    /// resulting view — for any principal — ever contains the undealt
    /// deck (the view type carries board + seats only, never the deck).
    #[must_use]
    pub fn for_principal(&self, viewer: Option<Principal>) -> SessionView { /* … */ }
}
```

Rationale: today EPIC-37's sketch parameterizes redaction by **seat
number** (`view(viewer: Option<u8>)`, `docs/EPIC-37_Mobile_Engine.md:265`).
Keying on **`Principal`** instead makes the rule authorization-correct: a
network client presents an identity, not a seat index, and the function
looks up which seat (if any) that principal owns. This is the *fine* tier
of a two-tier authorization split — pgate's `pkgate_tower` decides the
*coarse* question ("does this token carry `player` scope at all?"),
pkcore decides the *fine* one ("which cards may this principal see"). The
fine half tests with zero network.

**Dependency note:** `SessionView`/`SeatView` do not exist yet
(confirmed absent from `src/`; planned in EPIC-37). This method therefore
**lands with or after EPIC-37**, and EPIC-37's `view` signature should be
authored as `view(&self, viewer: Option<Principal>)` from the start to
avoid a later break. Recorded as a hard dependency below.

### `pkgate_tower` — the edge middleware (pgate, not pkcore)

The reusable Tower `Layer` stack, living in the pgate repo:

```rust
// pkgate_tower (sketch — implemented downstream)
// A tower::Layer wrapping any tower::Service (Tonic RPC or Axum route):
//   1. extract `Authorization: Bearer <token>` (or the legacy shared secret)
//   2. call the injected `dyn TokenVerifier` (EPIC-51)
//   3. insert the resolved `Principal` + scopes into request extensions
//   4. reject with a structured, transport-appropriate status on failure
//   5. per-principal rate-limit hook (EPIC-52)
//   6. emit `principal.id` / `auth.scheme` span attributes nesting under
//      pkdealer's EPIC-22 service spans
```

Rationale: Tonic and Axum are both `tower::Service`s, so one `Layer`
secures gRPC RPCs *and* the spectator SSE endpoint — the "one auth layer,
every transport" goal falls out of the Tower abstraction rather than
being built twice. pkcore never sees this crate; it only sees the
`Principal` the layer eventually hands to a seated `PokerSession`.

---

## Work Items

Phases 0–2 are pgate/pkdealer work (tracked in pgate's own EPIC once the
repo exists); Phases 3–4 are the pkcore seam and are the only items that
touch this crate.

### Phase 0 — Transport decision spike (pgate)

- [ ] **0a.** Evaluate a Rust Connect server (`axum-connect`, community)
      against the **Tonic + `tonic-web`** fallback. Success bar: a browser
      `connect-es` client and a native gRPC client both reach one Axum
      port with a `Bearer` header; the existing `proto/dealer.proto` is
      unchanged. Output: a one-page decision record in pgate.
- [ ] **0b.** Confirm the chosen stack preserves every property in Scope
      (single port, header auth, browser + native reach). `tonic-web`
      loses only Connect's plain-JSON third protocol; note that in the
      record.

### Phase 1 — `pkgate_tower` (pgate)

- [ ] **1a.** Implement the `Layer` stack above with a trivial
      `SharedSecretVerifier` placeholder (real verifiers are EPIC-51).
- [ ] **1b.** Mounts cleanly on a Tonic server and an Axum router;
      integration test: a request with the correct secret injects a
      `Principal`; a wrong/absent one is rejected with the
      transport-appropriate status.

### Phase 2 — Behavior-preserving adoption (pkdealer, pkspectator)

- [ ] **2a.** pkdealer mounts `pkgate_tower`; the existing shared-secret
      client conversation is byte-identical (golden-diff test of the RPC
      exchange before/after the middleware move).
- [ ] **2b.** pkspectator consumes the same layer on its SSE route.

### Phase 3 — `Principal` seam (pkcore)

- [ ] **3a.** Add `src/casino/principal.rs` with `Principal` as above;
      declare `pub mod principal;` in `src/casino/mod.rs` and re-export
      from `src/prelude.rs` beside the casino block (`:175-178`). Doc test
      on every public item (house rule).
- [ ] **3b.** Enable `uuid`'s `v5` feature in `Cargo.toml:115` and the
      wasm variant `:128`; confirm `cargo build` and
      `cargo check --target wasm32-unknown-unknown --no-default-features`
      stay green.
- [ ] **3c.** Unit tests: `principal_round_trips_uuid` (`From`/`Into`
      both ways), `principal_serde_round_trip`, `principal_hashes_as_uuid`
      (a `HashMap<Principal, _>` and `HashMap<Uuid, _>` agree on the same
      id — proves it drops into `StatsRegistry` unchanged).

### Phase 4 — Redaction seam (pkcore, with EPIC-37)

- [ ] **4a.** Author EPIC-37's `SessionView::view` as
      `view(&self, viewer: Option<Principal>)` (not `Option<u8>`); add
      `SessionView::for_principal(viewer)`.
- [ ] **4b.** Tests: `for_principal_reveals_only_owned_seat_hole_cards`,
      `for_principal_none_is_spectator_hides_all`,
      `for_principal_never_contains_deck` (structural: the view type has
      no deck field — a compile-time-adjacent assertion plus a runtime
      check that no serialized field carries undealt cards).

### Phase 5 — Registration

- [ ] **5a.** Register `pkgate` and the EPIC-50–59 block in `ROADMAP.md`
      (repo table + Numbering Policy `:374-382`); add the EPIC-50–53 rows.
- [ ] **5b.** Flip this EPIC's pkcore Status rows as Phases 3–4 land.

---

## Test Plan

- `principal_round_trips_uuid` / `principal_serde_round_trip` /
  `principal_hashes_as_uuid` — the seam is a transparent, serde-stable
  newtype that drops into the existing `Uuid`-keyed machinery.
- `for_principal_reveals_only_owned_seat_hole_cards` /
  `for_principal_none_is_spectator_hides_all` — the authorization rule
  from `ROADMAP.md:401-403`, now a pure kernel function.
- `for_principal_never_contains_deck` — the secrecy invariant (no view
  leaks the future), generalizing EPIC-37's snapshot-privacy rule.
- Golden-diff RPC test (pgate/pkdealer, Phase 2a) — adoption changed no
  observable behavior.

Test naming per house convention (no `test_` prefix; colocated
`#[cfg(test)]` modules).

## Key Files

| File | Role |
|---|---|
| `src/casino/principal.rs` | New — `Principal(Uuid)` identity seam |
| `src/casino/mod.rs` | `pub mod principal;` |
| `src/prelude.rs` | Re-export `Principal` (`:175-178`) |
| `Cargo.toml` | `uuid` gains `v5` (`:115`, `:128`) |
| `src/casino/session.rs` | `SessionView::for_principal` (with EPIC-37) |
| `ROADMAP.md` | Register pkgate + EPIC-50–59 block |
| *(pgate repo)* `pkgate_tower` | Tower auth/observability `Layer` |

## Reuse (do NOT recreate)

- `Player.id: Uuid` (`src/casino/player.rs:11`) and
  `StatsRegistry: HashMap<Uuid, _>` (`src/analysis/player_stats.rs:266`)
  — `Principal` wraps this atom; do NOT mint a second id space.
- `TableAction::PlayerSeated(u8, Uuid)` (`src/casino/table.rs:352`) — the
  existing identity-into-log path; unchanged.
- EPIC-37's `SessionView`/`SeatView` (`docs/EPIC-37_Mobile_Engine.md:238-265`)
  — redaction extends these; do NOT define a parallel view type.
- The wire-enum derive/serde-stability pattern (`src/lib.rs:197-212`,
  `src/casino/action.rs:88-90`) — `Principal` matches it.
- `uuid` already a dependency (`Cargo.toml:115`) — enable `v5`, don't add
  a second UUID crate.

## Compatibility

- **Preserves** every existing public signature and all default-feature
  behavior; `Principal` is additive; enabling `uuid/v5` changes no
  existing code path; the shared-secret POC keeps working through Phase 2.
- **Adds** `Principal`, `uuid/v5`, `SessionView::for_principal` (with
  EPIC-37), and — downstream — `pkgate_tower`.
- **Breaks** nothing in pkcore. The one coordination point: EPIC-37 must
  key its `view` on `Principal` rather than `u8`; since neither has
  shipped, this is a design alignment, not a break.

## Dependencies

- **Blocks:** EPIC-51 (needs `Principal` + `uuid/v5` for the sub-mapping),
  EPIC-52 (redaction is its fine-tier enforcement), EPIC-53.
- **Built on:** the `Uuid` identity atom (EPIC-26 `StatsRegistry`), the
  EPIC-19/20 `PokerSession`, EPIC-22 (service spans the layer nests
  under), the shared-secret POC (`ROADMAP.md:710-715`).
- **Blocked on:** **EPIC-37** for `SessionView`/`SeatView` (Phase 4 only;
  Phases 3, 0–2 are independent).
- **Related:** EPIC-21 (pkspectator SSE — a Tower adopter), EPIC-79 (the
  trustless horizon that eventually replaces the `TokenVerifier` slot).

## Verification

pkcore-side (the only commands that run in this repo):

```bash
cargo build
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings
cargo check --target wasm32-unknown-unknown --no-default-features
# purity guard: no transport/crypto/token crate entered the graph
cargo tree -e no-dev | grep -Ei 'tonic|tower|axum|rustls|jsonwebtoken' ; # expect empty
```

pgate-side verification (single-port reach, middleware) lives in the
pgate repo's EPIC once the repo exists.

Exit criteria:

1. `Principal` round-trips its `Uuid` through serde and `From`/`Into`,
   and slots into `StatsRegistry` with no key change (Phase 3c green).
2. `uuid/v5` enabled; wasm and default builds stay green; the purity
   grep returns empty.
3. `SessionView::for_principal` reveals only the viewer's hole cards,
   hides all for a spectator, and never carries the deck (Phase 4b green,
   landed with EPIC-37).
4. pkdealer's shared-secret client conversation is byte-identical after
   `pkgate_tower` adoption (pgate/pkdealer golden-diff green).
5. `cargo publish --dry-run` clean; downstream release audit unaffected.
