# Mental-poker spike archive

The consolidated output of the distributed / mental-poker exploration
(EPIC-79), archived here 2026-08-14 from the temporary `ImperialBower/mp`
holding repo. Four crates, all built and test-verified before packaging.
The through-line: **serverless card games with cryptographic fairness** — a
replicated state machine over a signed hash-chained event log, with the
Barnett–Smart mental-poker protocol supplying the deck.

This directory is documentation, not code pkcore builds: `docs/*` is in the
manifest's `exclude` list and none of these crates are cargo targets. Each
crate keeps its own `Cargo.toml` so it can be lifted out wholesale when its
production repo is created.

```
docs/files/mentalpoker/
├── README.md      ← this file
├── pkcore-mp/     the trait boundary + mocks           (2 tests)
├── tricktaking/   shared trick-taking engine           (9 tests)
├── mp-toy/        lesson-plan exercise crate           (22 tests)
└── pktable/       relay + terminal + QR transports     (⚠ binaries lost — see below)
```

## The crates and how they relate

**`pkcore-mp`** is the keystone: two traits that keep the poker engine, the
cryptography, and the deployment topology independent. `CardCrypto` wraps the
Barnett–Smart VTMF (mask/remask/staged-unmask + verifiable shuffle);
`Coordinator` wraps transport and total ordering of the event log. Ships with
mock impls (`PlaintextCrypto`, `InProcCoordinator`) that model the protocol's
*accounting* faithfully — the l-out-of-l threshold, the staged hole-card
reveal — with cards in the clear. The engine boundary is the design: pkcore
only ever sees plaintext `Card`s; crypto is verified outside the transition
function. This is the spike that `pkmental` (EPIC-79 Phase 1) productionizes.

**`pktable`** proves the machinery over real transports. Its `src/lib.rs`
(the pure `GameState` fold over `WireEvent`s, plus the `'|'`-separated wire
format and FNV chain fold) is intact. Three binaries were declared and
demo-verified before packaging — `relay` (TCP bulletin board), `client`
(full protocol node), and `qrtable` (every event crosses between seats as a
rendered-then-decoded QR image) — with verified runs of a full heads-up hand
over TCP and the same hand over 32 optically round-tripped QR codes.

> **⚠ Known gap:** `pktable/src/bin/{relay,client,qrtable}.rs` did not
> survive the packaging into `ImperialBower/mp` — the `[[bin]]` targets are
> declared in its `Cargo.toml` but the sources are missing, so the crate does
> not build as archived and the QR/TCP demos exist only as prose in
> `pktable/README.md`. The `qrcode`/`rqrr`/`image` dependencies belong to the
> missing `qrtable` binary. EPIC-79a Phase 2 is written against these
> binaries; they must be recovered or rewritten.

**`tricktaking`** answers the "beyond poker" question: a shared engine for
bridge/spades/hearts/euchre owning follow-suit, trump resolution, and lead
rotation, with per-game hooks for bidding and scoring. Its `engine.rs` holds
the generic `GameRules` trait — the seam the whole card-game family shares —
and `view_for`, the hidden-information projection the crypto layer realizes.
Uses its own ace-high `Card`; real integration maps to `cardpack` (0.7.0 —
note pkcore is on 0.6.9, the version-alignment prerequisite EPIC-79 names).

**`mp-toy`** is the learning companion to
[`docs/LESSON_PLAN-mental_poker_crypto.md`](../../LESSON_PLAN-mental_poker_crypto.md):
six modules of fill-in-the-`todo!()` exercises (deliberately insecure small
numbers) building the whole protocol — groups, DH + breaking it, ElGamal,
threshold, sigma protocols, Fiat–Shamir — with reference solutions behind
`--features solutions`. Its API maps ~1:1 onto `CardCrypto` by design.

## Companion documents (canonical copies live in `docs/`)

The holding repo's two documents were consolidated earlier and renumbered;
the copies in this repo's `docs/` are canonical:

| In the holding repo | Canonical location here |
|---|---|
| `EPIC-01_Real_Cryptography_Backend.md` | [`../../epics/EPIC-79a_Real_Cryptography_Backend.md`](../../epics/EPIC-79a_Real_Cryptography_Backend.md) |
| `mental-poker-crypto-lesson-plan.md` | [`docs/LESSON_PLAN-mental_poker_crypto.md`](../../LESSON_PLAN-mental_poker_crypto.md) |

Parent design docs: [`../../epics/EPIC-79_Mental_Poker.md`](../../epics/EPIC-79_Mental_Poker.md)
(the spike/decision-gate EPIC) and
[`docs/ANALYSIS_Mental_Poker.md`](../../ANALYSIS_Mental_Poker.md) (the source
analysis).

## Verification status (at packaging, rustc 1.75 / edition 2021)

| Crate | `cargo test` | Notes |
|---|---|---|
| pkcore-mp | 2/2 pass | round + coordinator tests |
| tricktaking | 9/9 pass | incl. full hand through the generic engine |
| mp-toy | 22/22 with `--features solutions`; 22 `todo!` failures by default (intended) | exercise crate |
| pktable | lib compiles; demos were verified by execution | **binaries now missing** — not reproducible from this archive |

Everything was kept dependency-free (pure std) for old-toolchain
portability; the QR pins in `pktable/Cargo.toml` exist only for that reason
and can be bumped on a modern rustc.

## Where each crate goes from here

- **`pkcore-mp` → `pkmental`** (new sibling repo, EPIC-79 Phase 1): real
  `pkcore` dependency re-exporting `Card`/`DECK_ARRAY` (deleting the
  `Card(u8)` stub in `src/card.rs`), edition 2024 / MSRV aligned with pkcore,
  an `rng: &mut impl RngCore` threaded through `CardCrypto`, docs and tests
  to house standards.
- **`pktable`** → `pkmental`'s workspace binaries, after the missing bins
  are recovered or rewritten.
- **`tricktaking`** → its own repo (prototyped at
  `github.com/ImperialBower/tricktaking`); not poker, stays out of pkcore.
- **`mp-toy`** → travels with the lesson plan wherever that lands; never a
  production dependency (it is `todo!()`-based by design).

pkcore itself takes **no code** from this archive — per EPIC-79's crate-split
decision, the only future in-pkcore work is the feature-gated `mental-log`
envelope.
