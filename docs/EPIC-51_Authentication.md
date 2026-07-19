# EPIC-51: Authentication (AUTH)

One pluggable way to answer "who is this, and did they prove it?" for
every client — humans on web/mobile/desktop, bot agents, and interactive
CLIs — so a single verifier turns any credential into the `Principal`
pkcore already understands (EPIC-50).

> **The implementation of this EPIC lives in the sibling repo
> [`pkgate`](https://github.com/ImperialBower/pkgate) (to be created), in
> the `pkgate_tokens` crate.** This document is the **contract**: the
> `TokenVerifier` trait, its two shipping implementations, the
> `sub → Principal` mapping, the OAuth2 flow matrix per client type, and
> the dual-IdP demo stack. pkcore's only contribution is the `Principal`
> seam and `uuid/v5`, both delivered by **EPIC-50**. Design of record:
> `docs/superpowers/specs/2026-07-19-networking-security-epics-design.md`.

The kata: the **Thing** is a **credential** (a shared secret, then a
JWT). The **Business Requirement** is that verifying it yields exactly a
`Principal` + a set of scopes, or a typed failure — nothing transport- or
game-specific leaks in. The **Business Logic** is the `TokenVerifier`
trait and its conformance suite, driven out test-first against a
compose-launched IdP.

---

## Status

*As of 2026-07-19. No pkgate repo exists yet; this is a contract doc. All
rows are pkgate-side — pkcore gains nothing here beyond what EPIC-50
ships.*

| Component | Status |
|---|---|
| `TokenVerifier` trait + `AuthContext` (`pkgate_tokens`) | Planned |
| `SharedSecretVerifier` (formalizes the POC) | Planned |
| `OidcVerifier` (JWKS, any spec-compliant issuer) | Planned |
| `sub → Principal` via `Uuid::new_v5` | Planned (needs EPIC-50 `uuid/v5`) |
| Flow matrix: PKCE / client-credentials / device-code | Planned |
| Dual self-hosted IdP compose profiles (Zitadel **and** Keycloak) | Planned |
| Trustless capability-proof verifier slot | 🔒 Gated (EPIC-79 horizon) |

---

## Context

- **The only auth decision on record is a shared secret.** `ROADMAP.md:710-715`:
  a single static token in gRPC metadata (players) or header/query param
  (spectator), chosen for the POC and explicitly flagged replaceable by
  "JWT + OAuth2 without restructuring." There is no token issuance, no
  expiry, no per-user identity — every player presents the same string.
- **`GRPC_DEALER.md` lists real auth as undesigned.** Its "Security
  Considerations" name JWT and mTLS only as future "production" items with
  no design behind them.
- **pkcore already resolves identity to a `Uuid`, and (via EPIC-50) to a
  `Principal`.** `Player.id: Uuid` (`src/casino/player.rs:11`),
  `StatsRegistry: HashMap<Uuid, PlayerStats>`
  (`src/analysis/player_stats.rs:266`). EPIC-50 names that atom
  `Principal` and enables `uuid`'s `v5` generator (`Cargo.toml:115`) for
  exactly the deterministic mapping this EPIC needs. `uuid` today carries
  `["serde", "v4"]` only — **not** `v5` — so this EPIC is blocked on
  EPIC-50's Cargo change.
- **pkcore stays out of it.** No `jsonwebtoken`, `rustls`, or HTTP client
  is in `Cargo.toml`, and none may be added; JWT verification is entirely
  a pgate concern. The kernel receives a resolved `Principal` and nothing
  else.

**This EPIC does NOT:** add any dependency to pkcore; implement scopes/
authorization (EPIC-52 consumes the `scopes` field this EPIC populates);
build client-side login helpers (EPIC-53); implement the trustless
verifier (EPIC-79); or own account UX — recovery, MFA policy, and
password rules are the IdP's configuration, not pgate code.

---

## Goals

- A single **`TokenVerifier`** trait: `token → AuthContext { principal,
  scopes, expires_at, raw_claims }` or a typed `AuthError`. Every
  transport (EPIC-50) and every trust layer plugs in behind it.
- **`SharedSecretVerifier`** — formalizes today's POC so existing demo
  clients need zero change while gaining a real seam.
- **`OidcVerifier`** — verifies any spec-compliant JWT via the issuer's
  JWKS, so **any** OIDC provider works unmodified (self-hosted or Google/
  GitHub/Auth0), mapping the token `sub` to a stable **`Principal`**.
- A **flow matrix** that makes "universal" concrete: PKCE for humans,
  client-credentials for bots, device-code for CLIs — all yielding the
  same JWT the same `OidcVerifier` checks.
- A **runnable login story in the demo stack** via **both Zitadel and
  Keycloak**, documented and launchable side by side, proving the
  verifier is genuinely IdP-agnostic.

## Scope

- `TokenVerifier::verify` is `async`, `Send + Sync`, and returns a
  `Principal` (EPIC-50) plus a `ScopeSet` (consumed by EPIC-52) — never a
  transport type, never a `Table` type.
- `OidcVerifier` enforces `iss` / `aud` / `exp` / `nbf`, caches JWKS with
  key-rotation awareness, and rejects tokens signed by unknown keys.
- Identity mapping is **deterministic and stateless**: the same
  `(issuer, sub)` always produces the same `Principal`, so
  `StatsRegistry` accumulates across logins and swapping IdPs never
  touches pkcore.
- `SharedSecretVerifier` uses **constant-time** comparison and maps its
  static secret to a fixed `Principal` + scope set from config.
- Both verifiers pass one shared conformance suite (expired,
  wrong-audience, tampered, absent, malformed).
- pkcore gains **nothing** from this EPIC (EPIC-50 already shipped the
  seam it relies on).

---

## Domain map

| Concept | Code construct | Status |
|---|---|---|
| Verify a credential | `TokenVerifier::verify` (`pkgate_tokens`) | ❌ this EPIC |
| Result of verification | `AuthContext { principal, scopes, .. }` | ❌ this EPIC |
| Legacy shared secret | `SharedSecretVerifier` | ❌ this EPIC (wraps POC) |
| Standards JWT check | `OidcVerifier` (JWKS) | ❌ this EPIC |
| Stable identity from `sub` | `Uuid::new_v5` → `Principal` | 🟡 needs EPIC-50 `uuid/v5` |
| Human login | Authorization Code + PKCE | ❌ this EPIC |
| Bot login | Client credentials | ❌ this EPIC |
| CLI login | Device code | ❌ this EPIC |
| Runnable IdP | Zitadel / Keycloak compose profiles | ❌ this EPIC |

---

## Design

### `TokenVerifier` — the one trait (`pkgate_tokens`)

```rust
// pkgate_tokens (implemented downstream; uses pkcore's Principal)
use pkcore::prelude::Principal;

pub struct AuthContext {
    pub principal: Principal,      // EPIC-50 seam
    pub scopes: ScopeSet,          // consumed by EPIC-52
    pub expires_at: Option<u64>,   // unix seconds; None for the static secret
    pub raw_claims: serde_json::Value,
}

#[derive(Debug)]
pub enum AuthError {
    Missing, Malformed, Expired, BadSignature,
    WrongAudience, UnknownIssuer, Revoked,
}

#[async_trait::async_trait]
pub trait TokenVerifier: Send + Sync {
    async fn verify(&self, token: &str) -> Result<AuthContext, AuthError>;
}
```

Rationale: one narrow async trait is the seam the whole suite pivots on —
EPIC-50's `pkgate_tower` holds a `dyn TokenVerifier` and cares only about
`AuthContext`; the transport, the token format, and the trust model all
vary behind it. `AuthContext` deliberately exposes `raw_claims` so
deployments can read custom claims (e.g. a table allowlist) without the
trait growing game knowledge.

### `SharedSecretVerifier` — the POC, formalized

```rust
pub struct SharedSecretVerifier {
    secret: SecretString,
    principal: Principal,
    scopes: ScopeSet,
}
// verify(): constant-time eq against `secret`; on match returns the
// configured principal + scopes with expires_at = None.
```

Rationale: ships first so the migration is behavior-preserving (EPIC-50
Phase 2 mounts exactly this). It turns the single string at
`ROADMAP.md:710-715` into a `TokenVerifier` with no client-visible change,
then becomes the trivial fallback for local/offline demos once OIDC
lands. Constant-time comparison closes the one real weakness of the POC
(timing-leaked secret).

### `OidcVerifier` — standards-based, IdP-agnostic

```rust
pub struct OidcVerifier {
    issuer: String,
    audience: String,
    jwks: JwksCache,          // fetched from issuer's /.well-known, rotation-aware
    namespace: Uuid,          // for the v5 sub mapping
}
// verify(): decode header → select JWKS key by `kid` → verify signature →
// check iss/aud/exp/nbf → map sub → Principal → read scopes claim.
```

**Identity mapping (the load-bearing decision):**

```rust
// sub is opaque and per-issuer; make it a stable Principal deterministically.
let principal = Principal::new(
    Uuid::new_v5(&namespace, format!("{issuer}|{sub}").as_bytes())
);
```

Rationale: `new_v5` is a pure hash — no database, no state — so the same
login always resolves to the same `Principal`, `StatsRegistry` keeps
accumulating, and switching from Zitadel to Keycloak to Google never
touches pkcore or migrates a single row. Namespacing on `issuer` prevents
a `sub` collision across IdPs from fusing two people. This is precisely
why EPIC-50 enables `uuid/v5` (`Cargo.toml:115`); without it this line
does not compile.

**Scopes from the token:** the `scope`/`roles` claim is parsed into the
`ScopeSet` EPIC-52 defines (`player` / `spectator` / `table:admin`). The
verifier only *reads* scopes; deciding what they *grant* is EPIC-52.

### Flow matrix — the "universal" story made concrete

Every client obtains a JWT the same `OidcVerifier` checks; only the OAuth2
grant differs by client shape:

| Client | Grant | Why |
|---|---|---|
| Human — web, mobile, desktop | Authorization Code + **PKCE** | Public clients, no secret to store; redirect-based |
| Bot / agent binary | **Client credentials** | Headless machine-to-machine; a service account per bot |
| Interactive CLI / TUI | **Device code** | "visit /device, enter ABCD-1234"; no local browser/redirect needed |

The acquisition state machines live in `pkgate_client` (EPIC-53); this
EPIC owns only the *verification* end and documents the matrix so the two
halves agree.

### Dual self-hosted IdP — both documented, both runnable

The demo compose stack ships **two profiles**, selected at launch:

```bash
docker compose --profile zitadel  up   # single binary, lighter footprint
docker compose --profile keycloak up   # broad ecosystem, realm import/export
```

Each is pre-seeded with demo human users and per-bot service accounts, and
exposes a standard `/.well-known/openid-configuration` + JWKS endpoint.
**The `OidcVerifier` code is byte-identical against both** — that identity
is the entire proof that the design is IdP-agnostic. EPIC-51's doc carries
a short comparison and a per-IdP setup appendix:

| | Zitadel | Keycloak |
|---|---|---|
| Footprint | Single Go binary | JVM, heavier |
| Config | API/Terraform-first | Realm JSON import/export |
| Passkeys | Built-in | Built-in (WebAuthn) |
| Best when | Minimal ops, cloud-native | Enterprise SSO, broad IdP federation |

Any external OIDC provider (Google, Auth0, GitHub via an OIDC bridge)
also works with the same verifier and no code change; the self-hosted
pair exists so the demo runs offline and reproducibly.

### Trustless verifier slot (design only, EPIC-79 horizon)

`TokenVerifier` is where EPIC-79's capability proofs plug in later: a
`CapabilityVerifier` that checks a signed capability against pkmental's
event log instead of an IdP-minted JWT, still yielding a `Principal` +
scopes. Recorded so the trait is shaped to accept it; not built here.

---

## Work Items

All items are pgate-side; none touch pkcore (its prerequisite is EPIC-50).

### Phase 0 — Prerequisite

- [ ] **0a.** Confirm EPIC-50 has shipped `Principal` and `uuid/v5`
      (`Cargo.toml:115`); `pkgate_tokens` depends on pkcore for
      `Principal`.

### Phase 1 — Trait + shared secret

- [ ] **1a.** `TokenVerifier`, `AuthContext`, `AuthError`, `ScopeSet`
      stub (EPIC-52 fills scope semantics).
- [ ] **1b.** `SharedSecretVerifier` with constant-time compare; unit
      tests: correct secret → configured principal/scopes; wrong/absent →
      `AuthError::Missing`/`BadSignature`.

### Phase 2 — OIDC verifier

- [ ] **2a.** `JwksCache` (fetch, `kid` lookup, rotation/TTL refresh).
- [ ] **2b.** `OidcVerifier::verify` — signature + `iss`/`aud`/`exp`/`nbf`
      + `sub → Principal` via `new_v5`; scope-claim parsing.
- [ ] **2c.** Conformance suite both verifiers pass: `expired_token`,
      `wrong_audience`, `tampered_signature`, `absent_token`,
      `malformed_token`, `unknown_kid`.
- [ ] **2d.** Determinism test: the same `(issuer, sub)` yields the same
      `Principal` across two independent verify calls
      (`sub_maps_to_stable_principal`).

### Phase 3 — Flows + IdP stack

- [ ] **3a.** Document + smoke-test the three grants (PKCE, client-
      credentials, device-code) against a running IdP; acquisition code
      is EPIC-53, so here assert only that each grant's token verifies.
- [ ] **3b.** `docker compose --profile zitadel` and `--profile keycloak`,
      each pre-seeded; a CI-optional integration test runs the OIDC
      conformance suite against **both** profiles and asserts identical
      `Principal` results.
- [ ] **3c.** Per-IdP setup appendix + the comparison table in the pgate
      EPIC.

---

## Test Plan

- `shared_secret_constant_time` — the POC's timing weakness is closed.
- OIDC conformance suite (`expired_token`, `wrong_audience`,
  `tampered_signature`, `absent_token`, `malformed_token`, `unknown_kid`)
  — both verifiers reject every malformed credential identically.
- `sub_maps_to_stable_principal` — deterministic, stateless identity;
  the reason `StatsRegistry` survives re-login and IdP swaps.
- Dual-profile integration test — the same `OidcVerifier` yields the same
  `Principal` under Zitadel and Keycloak (the IdP-agnostic proof).

## Key Files

| File | Role |
|---|---|
| *(pgate)* `pkgate_tokens/src/verifier.rs` | `TokenVerifier`, `AuthContext`, `AuthError` |
| *(pgate)* `pkgate_tokens/src/shared_secret.rs` | POC-formalizing verifier |
| *(pgate)* `pkgate_tokens/src/oidc.rs` | JWKS verifier + `new_v5` mapping |
| *(pgate)* `compose/` | Zitadel + Keycloak profiles, seed data |
| `Cargo.toml` (pkcore) | `uuid/v5` — delivered by EPIC-50, consumed here |

## Reuse (do NOT recreate)

- `Principal` + `uuid/v5` (EPIC-50, `src/casino/principal.rs`,
  `Cargo.toml:115`) — the mapping target and generator; do NOT mint a
  second identity type or add a second UUID crate.
- `StatsRegistry`'s `Uuid` keying (`src/analysis/player_stats.rs:266`) —
  the deterministic mapping exists so this keeps working untouched.
- The shared-secret POC (`ROADMAP.md:710-715`) — wrapped by
  `SharedSecretVerifier`, not discarded.
- Standard crates downstream (`jsonwebtoken`, `jwks`/`reqwest`) — in
  pgate only; never in pkcore.

## Compatibility

- **Preserves** all pkcore behavior (it gains nothing here) and the
  shared-secret client path (now a `TokenVerifier`).
- **Adds** the verifier trait, two implementations, the flow matrix, and
  the dual-IdP stack — all in pgate.
- **Breaks** nothing: OIDC is additive; a deployment can run
  shared-secret-only, OIDC-only, or both behind the same `pkgate_tower`.

## Dependencies

- **Blocks:** EPIC-52 (consumes the `scopes` this EPIC populates),
  EPIC-53 (client-side halves of these flows).
- **Built on:** **EPIC-50** (`Principal`, `uuid/v5`, `pkgate_tower`'s
  `dyn TokenVerifier` slot), the shared-secret POC.
- **Related:** EPIC-22 (verify latency as a span attribute), EPIC-79
  (the capability-proof verifier that later fills the trustless slot).

## Verification

pgate-side (in the pgate repo once created):

```bash
cargo test -p pkgate_tokens                     # trait + both verifiers + conformance
docker compose --profile zitadel  up -d && cargo test -p pkgate_tokens --features oidc-it
docker compose --profile keycloak up -d && cargo test -p pkgate_tokens --features oidc-it
```

pkcore-side (confirms the prerequisite and continued purity):

```bash
cargo tree -e no-dev | grep -Ei 'jsonwebtoken|reqwest|rustls' ; # expect empty in pkcore
```

Exit criteria:

1. Both verifiers pass the shared conformance suite; every malformed
   credential is rejected with a typed `AuthError`.
2. `(issuer, sub)` maps to a stable `Principal` across calls and across
   the two IdP profiles.
3. All three OAuth2 grants produce a token the `OidcVerifier` accepts.
4. The demo stack logs in a human (PKCE), a bot (client-credentials), and
   a CLI (device-code) under both Zitadel and Keycloak.
5. pkcore acquires no auth dependency (purity grep empty).
