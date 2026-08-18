# EPIC-52: Authorization & Session Security (AUTHZ)

Once a client is *authenticated* (EPIC-51), decide what it may *do*: a
small scope model, seat ownership bound to a `Principal`, per-principal
rate limiting, and TLS end to end — with the coarse "may they?" decided at
the edge and the fine "which card, whose turn?" decided in the pure
kernel.

> **The edge enforcement of this EPIC lives in the sibling repo
> [`pkgate`](https://github.com/ImperialBower/pkgate) (to be created), in
> `pkgate_tower`.** The **fine-grained enforcement is pkcore's**, delivered
> as EPIC-50's `SessionView::for_principal` redaction and seat ownership.
> This document is the contract binding the two tiers. Design of record:
> `docs/superpowers/specs/2026-07-19-networking-security-epics-design.md`.

The kata: the **Thing** is a **scope** — a capability a `Principal`
carries. The **Business Requirement** is that a player may act only at
their own seat and see only their own cards; a spectator sees all cards
but touches nothing; an admin runs the table. The **Business Logic** is a
two-tier check — coarse scope gate at the edge, seat/visibility gate in
the kernel — driven out test-first as an exhaustive scope × RPC matrix.

---

## Status

*As of 2026-07-19. Edge rows are pgate-side; the seat-binding and
redaction rows are pkcore-side and delivered by EPIC-50.*

| Component | Status | Repo |
|---|---|---|
| `ScopeSet` — `player` / `spectator` / `table:admin` | Planned | pkgate |
| Coarse scope gate (edge, before handlers) | Planned | pkgate |
| Per-principal rate-limit `Layer` | Planned | pkgate |
| TLS end-to-end (rustls; ingress-termination note) | Planned | pkgate |
| **Seat binding to `Principal` + resume** | 🔒 Gated (design only) | **pkcore** |
| **Fine gate — `for_principal` visibility** | 🔒 Gated (EPIC-50) | **pkcore** |
| Per-layer threat-model tables | Planned | pkgate |

---

## Context

- **Authorization today is one hardcoded rule, downstream.** "The server
  knows all cards; `GetStatus` returns hole cards only for the requesting
  player's seat; a separate admin/spectator token reveals all"
  (`ROADMAP.md:401-403`). It lives inside pkdealer's RPC handler, so every
  transport re-implements it and there is no reusable notion of a scope.
- **Seat resume is ad-hoc.** Reconnection uses a `client_secret`
  (`ROADMAP.md:367`) rather than the authenticated identity — a separate
  secret to issue, store, and match.
- **pkcore binds seats to a `Uuid` already.** `Seat.player: Player` with
  `Player.id: Uuid` (`src/casino/table/seat.rs:26-27`,
  `src/casino/player.rs:11`), and seating is logged as
  `TableAction::PlayerSeated(u8, Uuid)` (`src/casino/table.rs:352`). Once
  EPIC-50 names that `Principal`, "which seat does this principal own?" is
  a lookup pkcore can answer with no network.
- **No visibility-by-viewer in the kernel yet.** `src/play/visibility.rs`
  models only face-up/face-down stud cards, not per-viewer redaction
  (`src/play/visibility.rs:1-6,28`); the per-viewer rule is EPIC-50's
  `SessionView::for_principal`.
- **No TLS or rate limiting anywhere.** `GRPC_DEALER.md` lists both as
  undesigned production TODOs; the POC binds plaintext `localhost:50051`.

**This EPIC does NOT:** implement authentication (EPIC-51 hands it a
verified `Principal` + `ScopeSet`); add anything to pkcore beyond EPIC-50's
already-planned seat lookup and redaction (no new dependency); defend
against a *malicious server* — a compromised dealer still sees all cards;
that threat is EPIC-79's trustless territory, named here but not solved;
or implement multi-table routing (future ROADMAP phase).

---

## Goals

- A **minimal scope model** — `player`, `spectator`, `table:admin` — read
  off the token by EPIC-51, enforced coarsely at the edge before any
  handler runs.
- **Seat ownership bound to a `Principal`**, replacing the ad-hoc
  `client_secret`: reconnection is "present any valid token for the same
  principal."
- The **fine visibility gate in the kernel**: EPIC-50's
  `SessionView::for_principal` is the single point where hole-card
  redaction happens, so every transport inherits it identically.
- An **operational security floor**: per-principal rate limiting as a
  Tower layer, and **TLS end to end** via rustls (with a documented
  ingress-termination alternative).
- A **stated threat model per layer** — each EPIC in the suite says what
  it does and does not defend against.

## Scope

- The three scopes are exhaustive for v1: `player` (act at own seat, see
  own hole cards), `spectator` (read-only, all hole cards visible —
  the PokerGo view), `table:admin` (seat/kick, start hands).
- The **coarse** gate (edge): a request whose token lacks the required
  scope is rejected before the handler runs, with a transport-appropriate
  status — never reaching pkcore.
- The **fine** gate (kernel): even with `player` scope, a principal may
  act only at the seat they own and may see only their own hole cards;
  enforced by seat lookup + `for_principal`, tested with no network.
- Seat binding: `PokerSession` seating records the `Principal`; resume
  matches on it. No `client_secret` is issued.
- Rate limiting is **per-principal**, not per-connection (a principal
  cannot multiply its budget by reconnecting).
- TLS: `demo.sh` generates local certs (mkcert/self-signed); production
  may terminate TLS at an ingress — both documented, neither assumed.
- pkcore gains **no new dependency**; its share is the EPIC-50 seat
  lookup + redaction.

---

## Domain map

| Concept | Code construct | Status |
|---|---|---|
| Capability a principal holds | `ScopeSet` (`pkgate`) | ❌ this EPIC |
| Coarse "may they call this?" | edge scope gate (`pkgate_tower`) | ❌ this EPIC |
| Fine "may they see this card?" | `SessionView::for_principal` (EPIC-50) | 🟡 EPIC-50 |
| Which seat a principal owns | `Seat.player.id` lookup (`src/casino/table/seat.rs:27`) | 🟡 exists as `Uuid`; keyed on `Principal` this EPIC |
| Reconnect / resume | principal match (replaces `client_secret` `ROADMAP.md:367`) | ❌ this EPIC |
| Abuse control | per-principal rate-limit `Layer` | ❌ this EPIC |
| Wire confidentiality | rustls TLS | ❌ this EPIC |

---

## Design

### Scope model (`pkgate`)

```rust
// pkgate — read off the token by EPIC-51's OidcVerifier/SharedSecretVerifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scope { Player, Spectator, TableAdmin }

pub struct ScopeSet(/* bitset or HashSet<Scope> */);
```

Rationale: deliberately three. Poker's authorization surface is small —
act, watch, administer — and a big RBAC model would be YAGNI. The set is
`#[non_exhaustive]`-spirited (adding a scope later is additive); the mapping
from an OIDC `scope`/`roles` claim to this set is EPIC-51's parsing step.

### Two-tier enforcement — the load-bearing split

```
 request ──▶ pkgate_tower  ── coarse: does the token carry the scope this RPC needs?
                 │                     (reject here → never reaches pkcore)
                 ▼
            handler seats/acts on PokerSession
                 │
                 ▼
            pkcore  ── fine: is it this Principal's seat? may they see this card?
                        (Seat.player lookup + SessionView::for_principal)
```

Rationale: coarse policy needs no game state, so it belongs at the edge
where it can shed load cheaply; fine policy *is* game state
("whose turn, which hole cards"), so it belongs in the kernel where it is
pure and unit-testable. Splitting them means the expensive, security-
critical half (`for_principal`, EPIC-50) is tested with zero network and
reused by every transport — the "one redaction, all platforms" property.

### Seat binding & resume (pkcore, on EPIC-50's `Principal`)

`Seat` already holds `Player { id: Uuid, .. }`
(`src/casino/table/seat.rs:26-27`). Seating records the `Principal`
(EPIC-50 names the `Uuid`), and resume is:

```rust
// which seat does this principal own? (pure lookup, no network)
fn seat_of(table: &Table, who: Principal) -> Option<u8>; // scans seats for player.id == who.id()
```

Rationale: reconnection becomes "authenticate (EPIC-51) → look up the
seat you own → resume," deleting the `client_secret` (`ROADMAP.md:367`)
entirely. The identity that proves who you are is the identity that owns
the seat — one secret, not two. Because the underlying key is unchanged,
`StatsRegistry` and the event log are untouched.

### Rate limiting & TLS (`pkgate`)

- A per-`Principal` token-bucket `tower::Layer` sits after auth in the
  EPIC-50 stack, keyed on the resolved `Principal` so reconnecting does
  not reset the bucket.
- **rustls** terminates TLS in-process for the single-binary demo
  (`demo.sh` mints local certs); a production note documents terminating
  at an ingress/load balancer instead. rustls, like every transport
  crate, is a **pgate** dependency — never pkcore.

### Threat model, per layer (documented in each EPIC)

| Layer | Defends against | Does NOT defend against |
|---|---|---|
| EPIC-51 auth | Forged/absent/expired identity | A valid user acting maliciously in-game |
| EPIC-52 authz | Wrong-scope calls; peeking at others' cards; abuse floods | **A malicious/compromised server** (it sees all cards) |
| EPIC-79 (horizon) | A dishonest server / dealer | (its own scope) |

Rationale: stating the ceiling honestly is the point — Layer 2 gives real
identity on a *trusted* server; it is not zero-trust. The row that names
EPIC-79 as the answer to "malicious server" is the suite's on-ramp to the
trustless layer, not a promise this EPIC keeps.

---

## Work Items

### Phase 0 — Prerequisite

- [ ] **0a.** EPIC-50 (`Principal`, `for_principal`) and EPIC-51
      (`AuthContext.scopes`) shipped.

### Phase 1 — Scope model + coarse gate (pgate)

- [ ] **1a.** `Scope` / `ScopeSet`; EPIC-51's verifiers populate it.
- [ ] **1b.** Edge scope gate in `pkgate_tower`: each RPC declares its
      required scope; missing → transport-appropriate reject before the
      handler. Test: the full **scope × RPC matrix** (every scope against
      every RPC, asserting allow/deny).

### Phase 2 — Seat binding & fine gate (pkcore)

- [ ] **2a.** `seat_of(table, principal)` lookup (scans
      `Seat.player.id`, `src/casino/table/seat.rs:27`); `PokerSession`
      resume matches on `Principal`, deleting `client_secret` usage.
- [ ] **2b.** Wire the fine gate: acting requires `seat_of(..) == Some(seat)`;
      viewing goes through `SessionView::for_principal` (EPIC-50).
- [ ] **2c.** pkcore tests: `player_can_act_only_at_owned_seat`,
      `resume_matches_on_principal_not_secret`,
      `spectator_sees_all_player_sees_own` (drives `for_principal`).

### Phase 3 — Operational floor (pgate)

- [ ] **3a.** Per-principal token-bucket rate-limit `Layer`; test that
      reconnecting does not refill the bucket.
- [ ] **3b.** rustls TLS in the demo; `demo.sh` cert generation; ingress-
      termination note.
- [ ] **3c.** Threat-model tables added to EPIC-50/51/52/53 docs.

---

## Test Plan

- Scope × RPC matrix (edge) — every (scope, RPC) pair allows or denies
  exactly as specified; wrong-scope calls never reach pkcore.
- `player_can_act_only_at_owned_seat` — the fine gate: `player` scope is
  necessary but not sufficient; seat ownership is checked in-kernel.
- `spectator_sees_all_player_sees_own` — the `ROADMAP.md:401-403` rule,
  now the pure `for_principal` function (EPIC-50).
- `resume_matches_on_principal_not_secret` — `client_secret` retired;
  identity owns the seat.
- Rate-limit reconnection test — budget is per-principal, not
  per-connection.

## Key Files

| File | Role |
|---|---|
| *(pgate)* `pkgate_tower/src/scope_gate.rs` | Coarse scope enforcement |
| *(pgate)* `pkgate_tower/src/rate_limit.rs` | Per-principal token bucket |
| *(pgate)* `compose/`, `demo.sh` | rustls certs |
| `src/casino/session.rs` (pkcore) | Seat binding + resume on `Principal`; fine gate via `for_principal` |
| `src/casino/table/seat.rs` (pkcore) | `Seat.player.id` — the ownership lookup source (`:27`) |

## Reuse (do NOT recreate)

- `Seat.player: Player` / `Player.id: Uuid`
  (`src/casino/table/seat.rs:26-27`, `src/casino/player.rs:11`) — seat
  ownership is a lookup over this; do NOT add a parallel seat-owner map.
- `SessionView::for_principal` + `Principal` (EPIC-50) — the fine gate
  *is* this; do NOT re-implement redaction in a handler.
- The shared-secret + scope population from EPIC-51 — this EPIC consumes
  `AuthContext.scopes`, does not re-parse tokens.
- `TableAction::PlayerSeated(u8, Uuid)` (`src/casino/table.rs:352`) — the
  existing seat-identity record; resume reads it, does not replace it.

## Compatibility

- **Preserves** all pkcore behavior except that seat resume now keys on
  `Principal` instead of `client_secret` — a downstream (pkdealer) change,
  invisible to library consumers not using resume.
- **Adds** the scope model, edge gate, rate limiting, TLS (pgate), and the
  pkcore seat lookup / fine gate.
- **Breaks** nothing in the library API; the `client_secret` retirement is
  a pkdealer-internal migration guarded by EPIC-50 Phase 2's golden-diff.

## Dependencies

- **Blocks:** EPIC-53 (clients present scoped tokens and resume by
  identity).
- **Built on:** **EPIC-50** (`Principal`, `for_principal`, `pkgate_tower`),
  **EPIC-51** (`AuthContext.scopes`), the seat-identity model
  (`src/casino/table/seat.rs:27`), the visibility rule (`ROADMAP.md:401-403`).
- **Related:** EPIC-22 (rate-limit / authz decisions as span attributes),
  EPIC-79 (the "malicious server" row's eventual answer), EPIC-37
  (`SessionView` — the redaction substrate).

## Verification

pkcore-side:

```bash
cargo test --all-features   # seat binding, resume, for_principal gate
cargo clippy --all-features -- -D warnings
cargo tree -e no-dev | grep -Ei 'rustls|tower|governor' ; # expect empty in pkcore
```

pgate-side (in pgate once created): scope-matrix, rate-limit, and TLS
tests.

Exit criteria:

1. The scope × RPC matrix denies every wrong-scope call at the edge; none
   reaches a pkcore handler.
2. A `player`-scoped principal can act only at its owned seat and see only
   its own hole cards; a spectator sees all; verified in-kernel with no
   network (Phase 2c green).
3. Reconnection succeeds by presenting any valid token for the same
   principal; no `client_secret` is issued.
4. Rate-limit budget is per-principal and survives reconnection; TLS is on
   end-to-end in the demo.
5. pkcore acquires no new dependency (purity grep empty); downstream
   release audit clean.
