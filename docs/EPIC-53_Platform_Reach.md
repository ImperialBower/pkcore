# EPIC-53: Platform Reach (REACH)

The same login, compiled everywhere: one `pkgate_client` crate that
acquires and refreshes tokens for native agents, browser WASM, and mobile
apps — with a single secrecy rule for where those tokens (and pkcore's
session snapshots) are allowed to live.

> **The implementation of this EPIC lives in the sibling repo
> [`pkgate`](https://github.com/ImperialBower/pkgate) (to be created), in
> the `pkgate_client` crate, plus the future UniFFI mobile-binding repo
> that EPIC-37 designates.** pkcore contributes nothing new here — it
> consumes `Principal` (EPIC-50) and is otherwise a passenger. Design of
> record:
> `docs/superpowers/specs/2026-07-19-networking-security-epics-design.md`.

The kata: the **Thing** is a **credential at rest** — a token on a device.
The **Business Requirement** is that the token acquisition/refresh logic
is written once and runs on three platforms, and that tokens (and session
snapshots, which contain the deck) are stored only in platform-secure
locations, never in logs or shared storage. The **Business Logic** is the
`pkgate_client` state machines plus the storage-boundary rule, verified by
cross-target CI.

---

## Status

*As of 2026-07-19. All rows are pgate / downstream; pkcore gains nothing.*

| Component | Status | Repo |
|---|---|---|
| `pkgate_client` — acquisition/refresh state machines (native) | Planned | pkgate |
| PKCE / device-code / client-credentials flow drivers | Planned | pkgate |
| WASM target (`wasm32-unknown-unknown`, in-memory tokens) | Planned | pkgate |
| Mobile via UniFFI (Keychain / Keystore storage) | 🔒 Gated (EPIC-37 repo) | mobile-binding repo |
| Secure-storage rule (tokens + snapshots) | Planned | pkgate / apps |
| Cross-target CI (`cargo check` wasm + iOS/Android) | Planned | pkgate |

---

## Context

- **The four platforms have no shared client-auth code.** Native agents
  hand-roll shared-secret handling; the WASM apps (pkgto-web,
  pkarena0-web) are serverless and auth-free today; mobile is not built.
  EPIC-51 defines *three OAuth2 grants* but the *acquisition* halves —
  the redirect dance, the device-code poll, the client-credential fetch —
  are unwritten.
- **pkcore already disciplines its WASM and mobile surfaces.** wasm builds
  are first-class (target-gated deps, `uuid` gains `js` on wasm at
  `Cargo.toml:128`), and EPIC-37 makes the crate bindable via UniFFI with
  "zero binding code in pkcore." `pkgate_client` mirrors that discipline
  one layer up.
- **pkcore already has a secrecy rule this EPIC generalizes.** EPIC-37:
  session snapshots "contain the deck order — i.e., the future… store them
  only in the app's private storage, never transmit them to other
  players" (`docs/EPIC-37_Mobile_Engine.md`, snapshot section). A bearer
  token is exactly the same kind of secret. One rule covers both.
- **`Principal` is the shared vocabulary.** Every platform's client ends
  up presenting a token that resolves (EPIC-51) to the same `Principal`
  (`src/casino/principal.rs`, EPIC-50); nothing platform-specific reaches
  pkcore.

**This EPIC does NOT:** add anything to pkcore; retrofit auth onto the
serverless WASM apps (it provides a login story *for when* they grow
server-backed features, not a mandate to add one); build the mobile
binding repo (that is EPIC-37's downstream repo, which gains a
`pkgate_client` dependency); or own the IdP-side of the flows (EPIC-51).

---

## Goals

- **`pkgate_client`**: one crate holding only the genuinely shareable
  logic — OAuth2 acquisition state machines (PKCE, device-code, client-
  credentials), token refresh/expiry, and the `Authorization` header
  convention — compiled native, WASM, and into the mobile bindings.
- **A single secure-storage rule** covering both bearer tokens and
  pkcore's session snapshots: platform-secure at rest, never in logs or
  world-readable storage.
- **Cross-target proof**: `cargo check` for `wasm32-unknown-unknown` and
  the iOS/Android targets in CI, extending EPIC-37's pattern.

## Scope

- `pkgate_client` holds acquisition/refresh/header logic **only** — no
  transport client, no UI. It hands a valid `Authorization` value to
  whatever transport EPIC-50 chose.
- **Native** (agents, TUI): direct use; agents delete hand-rolled secret
  handling in favor of the client's device-code or client-credentials
  driver.
- **Web/WASM**: compiles on `wasm32-unknown-unknown` behind a feature,
  mirroring pkcore's wasm discipline (`Cargo.toml:128`). Tokens live **in
  memory only** — never `localStorage`/`sessionStorage`. The PKCE redirect
  is the page's responsibility; the client owns the code/verifier
  exchange.
- **Mobile**: consumed through the UniFFI bindings in EPIC-37's future
  downstream repo. Tokens **and** snapshots go to **Keychain (iOS) /
  Keystore (Android)** — never app documents, never logs.
- The secrecy rule is one sentence applied twice: *tokens and session
  snapshots are secrets; store them only in platform-secure storage.*
- pkcore gains **nothing**.

---

## Domain map

| Platform | Client path | Token storage |
|---|---|---|
| Native agent / TUI | `pkgate_client` direct (device-code / client-creds) | OS keyring / process memory |
| Browser (WASM) | `pkgate_client` (wasm feature) + page-driven PKCE redirect | **memory only** (never `localStorage`) |
| Mobile | `pkgate_client` via UniFFI (EPIC-37 repo) | **Keychain / Keystore** |
| Snapshot secrecy | pkcore `snapshot()` bytes (EPIC-37) | same secure store as tokens |

---

## Design

### `pkgate_client` — shareable client logic (pgate)

```rust
// pkgate_client (implemented downstream) — transport-free, UI-free
pub struct TokenStore { /* current access token + refresh token + expiry */ }

pub enum Flow {
    Pkce { authorize_url: Url, redirect: Url },   // humans; page/app drives the redirect
    DeviceCode { device_endpoint: Url },           // CLIs; poll until authorized
    ClientCredentials { client_id, client_secret },// bots; headless fetch
}

impl TokenStore {
    pub async fn acquire(&mut self, flow: Flow) -> Result<(), ClientError>;
    pub async fn refresh_if_needed(&mut self) -> Result<(), ClientError>;
    /// The header value EPIC-50's transport attaches: `Bearer <access_token>`.
    pub fn authorization(&self) -> Option<HeaderValue>;
}
```

Rationale: this crate holds the *portable* part of authentication — the
grant state machines and refresh clock — and nothing else. It deliberately
does **not** own storage (platform-specific) or the transport (EPIC-50) or
the UI redirect (the app's). Keeping it that thin is what lets one crate
compile to three targets; the moment it reached for `localStorage` or a
Keychain API it would fork per platform. Storage is injected as a trait the
host implements:

```rust
pub trait SecureStore { fn load(&self) -> Option<String>; fn save(&self, v: &str); fn clear(&self); }
// wasm: an in-memory impl; iOS: Keychain; Android: Keystore; native: OS keyring.
```

### The one secrecy rule (pgate + apps)

pkcore already says session **snapshots** are secret because they carry the
undealt deck (EPIC-37). A bearer **token** is the same shape of secret.
The rule, stated once and applied to both:

> Tokens and session snapshots are secrets. Persist them only in
> platform-secure storage — OS keyring / Keychain / Keystore, or process
> memory on the web. Never `localStorage`, never app documents, never
> logs, never transmitted to another player.

Rationale: unifying the two under one rule means the mobile app's secure-
storage plumbing serves both, and the review checklist is one line, not
two. The `SecureStore` trait above is the enforcement point for both.

### Cross-target CI (pgate)

Extend the EPIC-37 CI pattern (iOS/Android `cargo check`, `basic.yaml`
`wasm` job) to `pkgate_client`:

```yaml
- run: cargo check -p pkgate_client --target wasm32-unknown-unknown --no-default-features --features wasm
- run: cargo check -p pkgate_client --target aarch64-apple-ios
- run: cargo check -p pkgate_client --target aarch64-linux-android
```

Rationale: `cargo check` (no linking) proves the dependency graph and all
`cfg` gates resolve on every target without an NDK/Xcode — the same trick
EPIC-37 uses. Producing linked artifacts is the mobile-binding repo's job.

---

## Work Items

All items are pgate / mobile-repo; none touch pkcore.

### Phase 0 — Prerequisite

- [ ] **0a.** EPIC-51 flows defined; EPIC-50 transport chosen (the header
      convention `pkgate_client` targets).

### Phase 1 — Native client

- [ ] **1a.** `TokenStore`, `Flow`, `SecureStore` trait; device-code and
      client-credentials drivers (the bot/CLI grants).
- [ ] **1b.** Refresh clock: `refresh_if_needed` renews before `exp`.
      Tests: `device_code_polls_until_authorized`,
      `refresh_renews_before_expiry`, `authorization_is_bearer_token`.
- [ ] **1c.** Point one existing demo agent binary at `pkgate_client`,
      deleting its hand-rolled shared-secret handling.

### Phase 2 — WASM

- [ ] **2a.** `wasm` feature; in-memory `SecureStore`; PKCE code/verifier
      exchange (page owns the redirect). `cargo check
      --target wasm32-unknown-unknown` green.
- [ ] **2b.** Guard: a test/lint asserts no `localStorage`/`sessionStorage`
      write path exists in the wasm build.

### Phase 3 — Mobile (EPIC-37 downstream repo)

- [ ] **3a.** In the UniFFI binding repo, wrap `pkgate_client` and
      implement `SecureStore` over Keychain / Keystore.
- [ ] **3b.** Apply the secrecy rule to pkcore `snapshot()` bytes: they go
      to the same secure store, never app documents.
- [ ] **3c.** iOS/Android `cargo check` in CI.

### Phase 4 — Docs

- [ ] **4a.** A "Client integration" guide: the three grants, the storage
      rule, the per-platform `SecureStore` impls.

---

## Test Plan

- `device_code_polls_until_authorized` / `refresh_renews_before_expiry` /
  `authorization_is_bearer_token` — the portable acquisition/refresh core.
- WASM `cargo check` + the no-`localStorage` guard — tokens stay in memory
  on the web.
- iOS/Android `cargo check` — the crate compiles for mobile targets.
- Secrecy review: snapshots and tokens share the `SecureStore` path (a
  code-review checklist item, not a runtime test).

## Key Files

| File | Role |
|---|---|
| *(pgate)* `pkgate_client/src/token_store.rs` | `TokenStore`, `Flow`, refresh |
| *(pgate)* `pkgate_client/src/secure_store.rs` | `SecureStore` trait + impls |
| *(mobile repo)* Keychain/Keystore `SecureStore` | Mobile storage |
| *(pgate)* CI workflow | wasm + iOS/Android `cargo check` |
| `docs/EPIC-37_Mobile_Engine.md` (pkcore) | Snapshot-secrecy rule this EPIC generalizes |

## Reuse (do NOT recreate)

- `Principal` (EPIC-50) — the client acquires tokens that resolve to it;
  no client-side identity type.
- EPIC-51's grants and verifiers — `pkgate_client` drives the *client*
  half of the same flows; it does not re-specify them.
- EPIC-37's UniFFI binding repo + snapshot-secrecy rule
  (`docs/EPIC-37_Mobile_Engine.md`) — mobile auth rides in that repo and
  extends that rule; do NOT create a second mobile repo.
- pkcore's wasm target-gating (`Cargo.toml:128`) — the discipline
  `pkgate_client`'s wasm feature mirrors.

## Compatibility

- **Preserves** everything in pkcore (it gains nothing) and the serverless
  WASM apps (untouched unless they opt into server features).
- **Adds** `pkgate_client` (three targets), the `SecureStore` seam, and
  cross-target CI — all downstream.
- **Breaks** nothing.

## Dependencies

- **Blocks:** nothing in the suite (it is the leaf); enables real
  server-backed web/mobile apps.
- **Built on:** **EPIC-50** (`Principal`, transport/header convention),
  **EPIC-51** (the three grants), **EPIC-52** (scoped tokens, resume by
  identity), **EPIC-37** (the mobile-binding repo and snapshot-secrecy
  rule it extends).
- **Related:** EPIC-34 (pkarena0-web — a candidate first server-backed web
  adopter), EPIC-08 (the original web-service seed).

## Verification

pgate / mobile-repo side (no pkcore commands — pkcore is unchanged):

```bash
cargo test -p pkgate_client
cargo check -p pkgate_client --target wasm32-unknown-unknown --no-default-features --features wasm
rustup target add aarch64-apple-ios aarch64-linux-android
cargo check -p pkgate_client --target aarch64-apple-ios
cargo check -p pkgate_client --target aarch64-linux-android
# web secrecy guard
grep -rn 'localStorage\|sessionStorage' pkgate_client/src ; # expect empty
```

Exit criteria:

1. `pkgate_client` acquires and refreshes tokens via all three grants; the
   header is always `Bearer <token>` (Phase 1b green).
2. It `cargo check`s green on wasm and both mobile targets in CI.
3. The wasm build has no `localStorage`/`sessionStorage` write path; the
   mobile binding stores tokens **and** snapshots in Keychain/Keystore.
4. At least one demo agent runs on `pkgate_client` with its hand-rolled
   secret handling deleted.
5. pkcore is byte-for-byte unchanged by this EPIC.
