# Networking & Security Wrapper EPICs — Design

**Date:** 2026-07-19
**Status:** Approved design; EPIC documents to follow
**Scope:** Defines the EPIC-50–59 suite (`pkgate` sibling repo) plus the
minimal pkcore identity seam. Produced from a full review of the existing
EPIC corpus.

---

## 1. Context — what the EPIC review found

A survey of every EPIC, GRPC doc, and the ROADMAP established:

- **No existing EPIC owns networking/security wrapper territory.** pkcore
  is a pure domain kernel; all transport lives downstream (pkdealer gRPC,
  pkspectator SSE, serverless WASM web apps, planned UniFFI mobile).
- **Auth today is a POC placeholder.** ROADMAP Open Question #6: shared
  secret token in gRPC metadata (players) / header or query param
  (spectator SSE), plus a `client_secret` UUID for seat resume. JWT/OAuth2
  explicitly deferred. `GRPC_DEALER.md` lists TLS, mTLS, and rate limiting
  as undesigned "production" TODOs.
- **Authorization is one rule:** `GetStatus` returns hole cards only for
  the requesting seat; a spectator/admin token sees all. Implemented
  ad-hoc in pkdealer, re-implemented per transport.
- **The trust spectrum has two occupied ends and an empty middle:**
  pkdealer (trusted server, shared secret) and EPIC-79 pkmental
  (trustless, Ed25519-signed hash-linked event log, Barnett–Smart card
  crypto). Nothing provides *real identity on a trusted server*.
- **Transport is fragmented:** four platforms, four stories, no shared
  security layer. Plain gRPC is unreachable from browsers (no HTTP/2
  trailer support), which is why pkspectator bridges via SSE.
- **One existing data-secrecy rule** (EPIC-37): session snapshots contain
  the deck order and must never leave device-private storage. This design
  generalizes it.
- **Numbering:** pkcore's block (00–39) is saturated (35 soft-reserved
  for Hi-Lo/HORSE); 40–49 belongs to pkdealer. Per the ROADMAP ten-block
  policy, the new repo claims **EPIC-50–59**.

## 2. Decisions (settled during brainstorming)

| Decision | Choice |
|---|---|
| Security posture | **Layered progression**: demo-grade → real identity → cryptographic/trustless (EPIC-79 horizon) |
| Ownership | **New sibling repo `pkgate`** + a minimal pure seam in pkcore; contract/pointer docs live in pkcore `docs/` (EPIC-20–24 pattern) |
| Transport | **Connect RPC everywhere**, with a bounded Phase-0 spike; fallback is Tonic + `tonic-web` (same single-port, header-auth, browser-reach properties) |
| Identity | **OIDC-agnostic** verification via JWKS; demo stack self-hosts an IdP with **both Zitadel and Keycloak documented and supported via compose profiles** |
| Numbering | `pkgate` claims **EPIC-50–59** |
| EPIC structure | **Four layered EPICs** (50 transport, 51 authentication, 52 authorization/session, 53 platform reach) |

## 3. Architecture

```
                        ┌──────────── pkgate (new repo) ────────────┐
                        │  pkgate_tower   — Connect RPC + auth       │
                        │                  middleware (server side)  │
                        │  pkgate_tokens  — TokenVerifier impls      │
                        │  pkgate_client  — universal login helpers  │
                        │                  (native / WASM / mobile)  │
                        └──────┬────────────────────┬───────────────┘
                               │ mounted by         │ used by
                 ┌─────────────┴──────┐   ┌─────────┴─────────────────┐
                 │ pkdealer_service   │   │ agents, pkspectator,       │
                 │ (Connect endpoint) │   │ pkarena0-web, mobile apps  │
                 └─────────┬──────────┘   └───────────────────────────┘
                           │ pkcore (pure — knows Principal, never tokens)
```

**`pkgate` workspace crates:**

- **`pkgate_tower`** — Tower `Layer` stack: auth-header extraction,
  `TokenVerifier` invocation, principal injection into request
  extensions, structured rejections, per-principal rate limiting, OTel
  span attributes (`principal.id`, `auth.scheme`) nesting under EPIC-22
  service spans. Because Tonic and Axum are both Tower services, the same
  layer covers gRPC RPCs and the spectator SSE endpoint.
- **`pkgate_tokens`** — the `TokenVerifier` trait and implementations
  (shared-secret, OIDC/JWKS). No transport deps.
- **`pkgate_client`** — token acquisition state machines (PKCE,
  device-code, client-credentials), refresh/expiry, header-injection
  convention. Compiles native, `wasm32-unknown-unknown` (feature-gated),
  and into the future UniFFI mobile bindings repo.

### 3.1 The pkcore seam (the only in-core change)

- **`Principal`** — opaque serializable newtype over the existing `Uuid`
  (the `StatsRegistry` / `SeatPlayer` key). Contract: *pkcore never
  authenticates; it only distinguishes.* Token → `Principal` mapping
  happens at the edge; the kernel never sees a token, claim, or secret.
  Being a newtype over `Uuid`, `StatsRegistry`, seat resume, and hand
  histories work unchanged.
- **View redaction as a pure function** —
  `SessionView::for_principal(principal) -> SeatView`, reusing EPIC-37's
  planned `SessionView`/`SeatView` types. The "hole cards only for your
  seat" rule moves from ad-hoc per-transport code into one unit-testable
  kernel function; every transport (gRPC, SSE, mobile FFI) gets identical
  redaction. Invariant: no view, for any principal, ever contains the
  undealt deck.
- No crypto, token, or network dependency enters pkcore.

**Two-tier authorization split:** coarse policy at the edge in
`pkgate_tower` ("does this token carry `player` scope at all?"), fine
policy in the kernel ("is it this seat's turn; may this principal see
this card?"). The fine half tests without any network.

## 4. EPIC-50 — Transport Unification & Gateway Foundation

**Goal:** one wire protocol family, one port, one auth header
(`Authorization: Bearer <token>`), every platform. Strictly
behavior-preserving for security semantics (auth *relocation*, not
*upgrade* — strangler pattern).

**Client story (mature, official):** `connect-es` (browsers/TypeScript),
`connect-swift` (iOS), `connect-kotlin` (Android); native gRPC clients
keep working because Connect servers also speak gRPC.

**Server story (the honest risk):** Rust has no official Connect server
implementation.

- **Phase 0 — spike, decision gate.** Evaluate `axum-connect`
  (community) vs. fallback **Tonic + `tonic-web`** (gRPC + gRPC-Web from
  one port, no Envoy). Fallback preserves every chosen property except
  Connect's third plain-JSON protocol. Exit: one-page decision record.
  Protos unchanged either way.
- **Phase 1 — `pkgate_tower`.** The auth/observability `Layer` stack
  described in §3.
- **Phase 2 — adoption.** pkdealer mounts the layer; pkspectator consumes
  it via Axum. Shared-secret behavior preserved bit-for-bit.
- **Phase 3 — pkcore seam.** `Principal` + `for_principal` redaction land
  in pkcore under the normal doc-test/unit-test regime.

**Non-goals:** new token formats (EPIC-51), scopes (EPIC-52), client
helper crates (EPIC-53).

**Verification:** golden-diff test (identical RPC conversations before/
after the middleware move); browser smoke test against the single port.

## 5. EPIC-51 — Authentication

**Core trait** (in `pkgate_tokens`):

```rust
pub trait TokenVerifier: Send + Sync {
    async fn verify(&self, token: &str) -> Result<AuthContext, AuthError>;
}
// AuthContext = { principal: Principal, scopes: ScopeSet,
//                 expires_at, raw_claims }
```

**Implementations (one per trust layer):**

1. **`SharedSecretVerifier`** — formalizes the POC: constant-time
   comparison, static principal + scope mapping from config. Ships first;
   existing demo clients need zero changes.
2. **`OidcVerifier`** — verifies any spec-compliant JWT via the issuer's
   JWKS endpoint (cached, key-rotation-aware); enforces
   `iss`/`aud`/`exp`/`nbf`. **Identity mapping:** OIDC `sub` (string) →
   `Principal` (Uuid) via **UUIDv5(namespace, issuer + sub)** —
   deterministic and stateless, so the same login always yields the same
   principal, `StatsRegistry` accumulates across sessions, and swapping
   IdPs never touches pkcore.
3. **Trustless slot (documented, not implemented):** where EPIC-79's
   capability proofs plug in later.

**Token acquisition matrix (the "universal" story):**

| Client type | OAuth2/OIDC flow |
|---|---|
| Humans — web, mobile, desktop | Authorization Code + **PKCE** |
| Bot/agent binaries | **Client credentials** (machine-to-machine) |
| Interactive CLIs / TUI | **Device code** ("visit /device, enter ABCD-1234") |

All flows yield the same kind of JWT, verified by the same
`OidcVerifier`.

**Self-hosted IdP — both documented and runnable.** The demo compose
stack supports **two profiles**: `docker compose --profile zitadel` and
`--profile keycloak`, each pre-seeded with demo users and agent service
accounts. EPIC-51 carries a short comparison (Zitadel: single binary,
lighter footprint; Keycloak: broader ecosystem, realm import/export) and
a setup appendix for each. The `OidcVerifier` code is identical for both
— that is the point of JWKS-based verification. Any external OIDC
provider (Google, GitHub via bridge, Auth0) also works unmodified.

**Verification:** a `TokenVerifier` conformance suite both
implementations must pass (expired, wrong-audience, tampered, absent,
malformed); OIDC integration test against a compose-launched IdP, run
for **both profiles**, CI-optional.

## 6. EPIC-52 — Authorization & Session Security

**Scope model (deliberately small):**

| Scope | Grants |
|---|---|
| `player` | Act at own seat; see own hole cards |
| `spectator` | Read-only; all hole cards visible (PokerGo view) |
| `table:admin` | Seat/kick players, start hands |

Enforced in `pkgate_tower` before handlers run (coarse tier); seat
ownership and card visibility enforced in pkcore (fine tier, §3.1).

**Seat binding & resume:** `SeatPlayer` records the authenticated
`Principal`; reconnection = presenting any valid token for the same
principal. Replaces the ad-hoc `client_secret`.

**Operational floor:** per-principal rate limiting as a Tower layer;
TLS via rustls end-to-end (`demo.sh` generates local self-signed/mkcert
certs; deployment note: terminating at an ingress is equally supported).

**Threat-model table:** each EPIC doc states what its layer does and does
not defend against (e.g., EPIC-52 does *not* defend against a malicious
server — that is EPIC-79's territory).

**Verification:** full authorization matrix test (every scope × RPC
pair); pkcore property test — spectator view contains all hole cards,
player view only its own, no view ever contains the deck.

## 7. EPIC-53 — Platform Reach

`pkgate_client` compiled three ways; holds only what is genuinely
shareable (acquisition state machines, refresh, header convention).

- **Native (agents, TUI):** direct use; agents drop hand-rolled secret
  handling.
- **Web/WASM:** `wasm32-unknown-unknown` behind a feature, mirroring
  pkcore's WASM discipline. Tokens in memory only — never
  `localStorage`. PKCE redirect handled by the page. Note: pkarena0-web /
  pkgto-web stay serverless and auth-free today; EPIC-53 provides a login
  story only for when they grow server-backed features.
- **Mobile:** UniFFI bindings in the same future downstream repo EPIC-37
  designates. Generalized secrecy rule: *tokens and session snapshots are
  secrets — Keychain (iOS) / Keystore (Android), never app documents or
  logs.*

**Verification:** WASM + iOS/Android `cargo check` in CI (extends
EPIC-37's pattern); device-code flow exercised by demo agent binaries.

## 8. The EPIC-79 bridge (horizon, not work)

The layered model ends with the trusted server dissolving: pkmental's
signed event log replaces server authority, and the `TokenVerifier` slot
accepts capability proofs instead of IdP-minted JWTs. Recorded here so
Layer 3 has a named on-ramp. Constraint on EPIC-50–53: no public API
contract may assume "there will always be a trusted server."

## 9. Deliverables & sequencing

1. **EPIC-50** — Transport Unification & Gateway Foundation
   (spike → `pkgate_tower` → adoption → pkcore seam)
2. **EPIC-51** — Authentication (`TokenVerifier`, shared-secret + OIDC,
   flow matrix, dual-IdP compose profiles)
3. **EPIC-52** — Authorization & Session Security (scopes, seat binding,
   rate limiting, TLS)
4. **EPIC-53** — Platform Reach (`pkgate_client` tri-platform, storage
   rules, CI targets)

Strict order 50 → 51 → 52; 53 can begin once 51's flows are stable.
pkcore-side docs: pointer/contract docs `EPIC-50`–`EPIC-53` in
`docs/`, plus a ROADMAP update claiming the 50-block and recording the
architecture. Implementation EPICs live in the `pkgate` repo.

## 10. Out of scope

- Real-money considerations of any kind (consistent with EPIC-79).
- Mental-poker cryptography implementation (EPIC-79 / pkmental).
- Multi-table routing/sharding (future phase per ROADMAP).
- Account UX beyond what the IdPs provide (recovery, MFA policy — IdP
  configuration, not pkgate code).
