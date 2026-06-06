# EPIC-79: Mental Poker

> Thematic number: a nod to Shamir, Rivest & Adleman's 1979 paper
> *"Mental Poker"* — the founding question, *can we play a fair card
> game with no trusted dealer?*

## Context

This EPIC is a **research / design spike** — a decision-gate document,
not an implementation commitment. Its job is to capture the
architecture, prove the deterministic-engine mapping, evaluate the
candidate crypto crates, define the signed-event schema, and lay out a
phased path with explicit gates the user can approve **before any code
is written**.

It is grounded entirely in [`docs/ANALYSIS_Mental_Poker.md`](./ANALYSIS_Mental_Poker.md),
which explores serverless, trustless distributed poker. That analysis
separates two concerns people conflate when they say "server":

1. **State coordination (the "stateless" part).** A poker hand has state
   — deck, board, pot, whose turn it is — but that state need not live on
   an authoritative server. The clean model is a **replicated state
   machine over a signed, hash-linked event log**: every player runs the
   *same deterministic* transition function and, given the same ordered
   sequence of signed actions, independently computes byte-identical
   state. "Stateless" really means *no authoritative party — state is the
   deterministic fold over a verifiable log.*

2. **Card secrecy (the genuinely hard cryptography).** A shuffled deck
   where no one knows anyone's cards, no card is duplicated, the shuffle
   is provably a real permutation, and cards can be revealed selectively
   and verifiably. This is **mental poker proper** — the Barnett–Smart
   (2003) protocol over ElGamal: a distributed key, verifiable shuffles
   with zero-knowledge proofs, and threshold/cooperative decryption.

The reason this is worth an EPIC: concern (1) maps unusually cleanly onto
what `pkcore` **already is**. `TableNoCell` is a deterministic
`fold(state, event) -> state` engine; `TableAction`
(`src/casino/table/event.rs:11`) is already a `Serialize`/`Deserialize`
event enum; and `HandHistory` (`src/hand_history.rs:128`) already chains
a per-hand `event_log` slice and replays it
(`HandHistory::from_event_log`, `src/hand_history.rs:1673`). The analysis
itself observes: *"this maps cleanly onto a deterministic engine — the
engine is `fold(state, event) -> state` plus a legality check, and the
crypto types are just opaque blobs the engine carries and the peers
verify."*

Concern (2), by contrast, needs an entire elliptic-curve / zero-knowledge
stack (`arkworks`) that pkcore — deliberately pure-compute — has **zero**
dependencies on today. That asymmetry drives the architecture below.

---

## Status

This is a spike. Nothing here is shipped; the table records *design*
maturity and the decisions a future implementation epic must resolve.

| Component | Status |
|---|---|
| Problem framing & engine mapping | **Spike — documented here** |
| Architecture split (pkcore log layer vs. `pkmental` crypto crate) | **Designed, not built** |
| Signed `Event` envelope schema | **Designed, not built** |
| `mental-log` pkcore feature gate | **Designed, not built** |
| `pkmental` sibling crate | **Designed, not built** |
| Barnett–Smart deal/reveal flow | **Designed, not built** |
| Crypto crate selection (`arkworks` / Geometry Research / Zypher) | **Decision pending** |
| Curve / group choice | **Decision pending** |
| Verifiable-shuffle proof scheme | **Decision pending** |
| Transport (relay vs. gossip vs. chain) | **Decision pending** |
| Dropout / forfeiture recovery model | **Decision pending** |
| Settlement (real-money escrow) | **Out of scope (see Non-Goals)** |

---

## Goals & Non-Goals

### Goals

- Decide **whether and how** to build serverless mental poker on top of
  pkcore, and produce an approvable **phased** design.
- Prove the mapping between the analysis's signed event log and pkcore's
  existing `TableAction` / `HandHistory` machinery.
- Establish the **location split**: what belongs in pkcore vs. a new
  sibling crate, so pkcore stays crypto-free.
- Surface the hard problems (dropout, settlement, collusion) as
  first-class engineering risks rather than footnotes.

### Non-Goals

- **No real-stakes settlement.** Mental poker makes the *cards*
  trustless, not the *money*. Without a server, enforcing that the loser
  pays needs on-chain escrow / smart contracts (per the analysis). v1 is
  play-money or social-trust; real-money settlement is explicitly out of
  scope.
- **No collusion prevention.** Nothing stops two players sharing their
  hands over a side channel. That is true of all poker and is not a
  cryptographic problem to solve here.
- **No BFT / heavy consensus in v1.** For turn-based games the rules
  already constrain *who* may act next, so `seq + author` legality is the
  ordering oracle. A player signing out of turn is simply rejected by
  every honest peer. A broadcast/relay layer (not consensus) is enough to
  make equivocation *detectable*.

---

## Design

### Architecture: the split

The load-bearing decision. Concern (1) is deterministic and dependency-
light; concern (2) is crypto-heavy. They live in different places.

```
┌──────────────────────────────────────────────────────────────────────┐
│  pkcore  (existing, deliberately crypto-free)                          │
│                                                                        │
│   TableNoCell  ──fold(state, event)──▶  deterministic state            │
│   TableAction  (src/casino/table/event.rs)   ← engine-relevant events  │
│   HandHistory  (src/hand_history.rs)         ← chain + replay today    │
│                                                                        │
│   ┌──────────────────────────────────────────────────────────────┐   │
│   │  mental-log   (NEW, future feature gate — signing only)        │   │
│   │  Event { table_id, hand_id, seq, prev_hash, author, payload,   │   │
│   │          sig }   — tamper-evident envelope over TableAction     │   │
│   └──────────────────────────────────────────────────────────────┘   │
└───────────────────────────────▲────────────────────────────────────────┘
                                 │ depends on pkcore (engine + Card encodings)
┌───────────────────────────────┴────────────────────────────────────────┐
│  pkmental  (NEW sibling crate — depends on arkworks)                     │
│                                                                          │
│   Barnett–Smart card layer:                                              │
│     • ElGamal masking of the 52 Card encodings                           │
│     • verifiable shuffle (re-mask + permute + ZK proof)                  │
│     • threshold / cooperative decryption (deal & reveal)                 │
│   Transport: thin P2P / relay (or chain) carrying signed Events          │
└──────────────────────────────────────────────────────────────────────────┘
```

**pkcore (`mental-log`, future feature gate).** Houses the signed event
envelope and the deterministic state machine. Crucially it *wraps* the
existing `TableAction` rather than replacing it. The crypto payloads
(`KeyShare`, `Shuffle`, `PartialDecrypt`) are carried as **opaque blobs**
— the engine never interprets ElGamal; it only needs to know "this slot
is now revealed to seat 3" and to refuse illegal transitions. Signing and
hashing add no heavy dependency (an Ed25519/Schnorr signer + SHA-256),
keeping pkcore's lean-by-default posture intact. This layer would sit
behind a feature gate (e.g. `mental-log`), matching the existing
`bot-profiles` / `hand-histories` / `equity` pattern in `Cargo.toml`.

**pkmental (new sibling crate).** Houses the Barnett–Smart card layer and
a thin transport. Depends on pkcore for the engine and the 52 fixed
`Card` encodings; depends on `arkworks` for the elliptic-curve and
zero-knowledge primitives. This mirrors the existing pkcore → **pkdealer**
sibling-repo pattern in [`ROADMAP.md`](../ROADMAP.md): pkcore owns the
logic, the sibling owns the transport. The contrast with pkdealer is the
trust model — pkdealer is a *trusted-server* gRPC dealer; pkmental is
*trustless / serverless*.

### Signed event log schema

The whole game is a sequence of events sharing one envelope; the payload
varies. The envelope makes the log tamper-evident and lets every peer
reconstruct identical state with no one in charge. (Transcribed and
lightly refined from the analysis.)

```rust
// Abstract crypto types — concretely arkworks types over a prime-order
// group (e.g. Ristretto, or a pairing-friendly curve for SNARK shuffles).
type Scalar = ...;   // field element: secret keys, randomness r
type Point  = ...;   // group element: public keys, card encodings
type Sig    = ...;   // Schnorr / Ed25519-style signature
type Hash   = [u8; 32];

/// ElGamal ciphertext = a "masked card". Underlying plaintext is a Point.
struct MaskedCard { c1: Point, c2: Point }

/// The envelope every peer signs and chains.
struct Event {
    table_id:  TableId,
    hand_id:   HandId,
    seq:       u64,        // monotonic within the hand
    prev_hash: Hash,       // hash of the previous Event — the chain
    author:    PlayerPk,   // who is claiming to emit this
    payload:   Payload,
    sig:       Sig,        // author's signature over everything above
}

enum Payload {
    // ---- Setup ----
    TableCreate { group_params: GroupParams, seats: Vec<PlayerPk>,
                  blinds: Blinds, starting_stacks: Vec<Chips> },
    KeyShare    { h_i: Point, knowledge_proof: SchnorrProof },

    // ---- Shuffle ----
    Shuffle     { deck_out: Vec<MaskedCard>, shuffle_proof: ShuffleProof },

    // ---- Play ----
    PartialDecrypt { position: u8, target: RevealTarget,
                     d_i: Point, proof: ChaumPedersenProof },
    Action      { kind: ActionKind },   // Fold | Check | Call | Bet | Raise
    HandResult  { payouts: Vec<(PlayerPk, Chips)> },  // deterministic, verifiable
}

enum RevealTarget { ToAll, ToSeat(SeatIdx) }
```

`prev_hash` does the heavy lifting: each event commits to the entire
history before it, so the log can't be reordered, dropped, or spliced.
`seq + author` together constrain *legality* — the rules say whose turn
it is, so an event from the wrong author at the wrong `seq` is rejected
independently by everyone. That is how you get ordering without a
consensus protocol: **the turn structure is the ordering oracle**, and
the signature stops impersonation. Equivocation (sending event A to one
peer and a conflicting A' to another at the same `seq`) is *detectable*
because both are signed over the same `prev_hash` and `seq` — which is
why serious play fans every event out to all peers (a relay or chain),
not point-to-point.

**Relationship to existing code.** This is the *generalization* of
pkcore's existing per-hand event log. Today `HandHistory` slices
`table.event_log` (a `Vec<TableAction>`) per hand and feeds it to
`HandHistory::from_event_log` (`src/hand_history.rs:1673`) to derive
per-street structure. The mental-poker `Event` is that same idea with a
cryptographic envelope: the engine-relevant `Payload::Action` /
`HandResult` variants correspond directly to today's `TableAction`
variants (`Bet`, `Call`, `Raise`, `Fold`, `Dealt`, `DealtFlop`, …,
`src/casino/table/event.rs:39-66`). A future Phase 1 maps `TableAction`
into `Payload` and adds the `{ seq, prev_hash, author, sig }` envelope.

Two design notes from the analysis worth preserving:

- **Deck assignment is deterministic from the rules** (positions 0–1 →
  seat 0's hole cards, etc.), so it never needs its own event — every
  peer computes it. Same for `HandResult`: given the revealed cards and
  the betting log, the payout is a pure function, so it is *verifiable*
  rather than authoritative. A `HandResult` that doesn't match everyone's
  computation is rejected.
- **Betting and crypto events live in the same chain**, interleaved in
  the order the hand progresses: shuffle → deal hole cards (partial
  decrypts targeted per seat) → preflop betting → deal flop (partial
  decrypts `ToAll`) → … One log, one fold-left, one state.

### Barnett–Smart deal-and-reveal flow

Work in a cyclic group `G` of prime order `q` with generator `g`. The 52
cards map to 52 fixed, public, distinct group elements `m_1 … m_52`.

> **Mapping to pkcore.** pkcore already has a fixed, ordered 52-card
> universe: `DECK_ARRAY` / `POKER_DECK` (`src/deck.rs:13,68`) and the
> integer encoding `Card::as_u32` (`src/card.rs:131`). Those become the
> seed for the 52 public encodings `m_1 … m_52`; the precomputed
> lookup-table inversion turns a decrypted `Point` back into a `Card`.

- **Step 0 — Key setup.** Each player `i` picks secret `x_i`, publishes
  `h_i = g^{x_i}` (`KeyShare`) with a Schnorr proof of knowledge of
  `x_i`. Everyone computes the shared key `h = ∏ h_i = g^(∑ x_i)`. No one
  knows `∑ x_i` — it is split `(n, n)`.
- **Step 1 — Init deck.** Trivial ElGamal encryptions `(c1, c2) = (1, m_j)`
  per card. Public; nothing hidden yet.
- **Step 2 — Shuffle round-robin.** Each player, in turn, re-masks every
  card `(c1, c2) → (c1·g^r, c2·h^r)` (fresh `r` per card) and applies a
  secret permutation `π`, attaching a zero-knowledge shuffle proof. After
  all `n` players the deck is uniformly shuffled, fully masked under `h`,
  and the composite permutation is unknown to everyone (`Shuffle` events).
- **Step 3 — Deal.** Deck positions map to roles by the rules; no card is
  decrypted, players just agree which masked slot belongs where.
- **Step 4 — Reveal a hole card to one player `j`.** Every player *except*
  `j` publishes `d_i = c1^{x_i}` with a Chaum–Pedersen proof that
  `log_{c1}(d_i) = log_g(h_i)`. Player `j` computes
  `m = c2 / ((∏_{i≠j} d_i) · c1^{x_j})`, applying their own share last —
  only `j` finishes the division and learns the card
  (`PartialDecrypt { target: ToSeat(j) }`).
- **Step 5 — Reveal a community card.** Identical, but *all* `n` players
  publish `d_i` (`target: ToAll`) and anyone computes `m = c2 / ∏ d_i`.
  Flop, turn, river.
- **Step 6 — Showdown.** A player who must show down publishes the
  partial decryption they withheld in Step 4 (still proof-backed), so
  everyone verifies the cards were the ones dealt, not swapped. Folded
  players reveal nothing.

The proofs at every masking and decryption step are what make this
*trustless* rather than merely *distributed*: a cheater can't shuffle in
a duplicate, can't peek (they only ever hold their own `x_i`), and can't
lie about a partial decryption without producing a proof that fails
verification — which becomes signed evidence of cheating.

### Hard problems / risks

These are first-class engineering risks, not footnotes (per the
analysis's honest caveats):

1. **The drop-out problem.** With `(n, n)` threshold decryption, a player
   about to lose can stall the whole hand by refusing to publish their
   partial decryption. The schema needs **timeout events and a
   forfeiture rule** (e.g. a player who fails to provide a partial decrypt
   within `T` is folded and forfeits; the rest can still open *their*
   cards and settle). This recovery logic is a real chunk of work on top
   of the crypto, and threads through the event log.
2. **Settlement is separate from dealing.** Trustless cards ≠ trustless
   money. Real stakes need on-chain escrow, reintroducing cost and
   latency. Out of scope for v1 (see Non-Goals).
3. **Collusion is unsolvable by crypto.** Side-channel hand-sharing is
   undetectable and unpreventable. Stated, not solved.

### Candidate crypto stack

> The analysis flags that this area moves fast — *"verify the current
> maintenance status and benchmarks with a search before committing."*
> The following is research input, not a locked choice; confirming it is
> a decision gate.

- **`arkworks` ecosystem** (`ark-ec`, `ark-ff`, curve crates) — the
  elliptic-curve and ZK primitives the rest builds on.
- **Geometry Research's `mental-poker` / `barnett-smart-card-protocol`**
  — open-source Rust implementing exactly this card layer on top of
  arkworks; the most direct reference implementation. Decision: depend on
  vs. vendor vs. reimplement.
- **zk-SNARK shuffle work (Zypher and others)** — has pushed shuffle
  proofs to sub-second, which is what makes an interactive table
  realistic. Relevant if Neff / Bayer–Groth proofs prove too slow.

### Phased roadmap with decision gates

The spike's recommended path. **This document commits only to Phase 0.**
Each later phase is a *future, separately-approved* epic.

- **Phase 0 — this document.** Design + decision gate.
- **Phase 1 — `mental-log` in pkcore.** `Event` envelope + signature +
  hash-chain verification over the existing `TableAction`. No card
  crypto. Deterministic and unit-testable; adds only a signer + hash (no
  heavy deps). Validates the "stateless state machine" half in isolation.
- **Phase 2 — `pkmental` skeleton.** New sibling crate; key setup
  (`KeyShare` + Schnorr PoK), aggregate key `h`, trivial deck init.
- **Phase 3 — verifiable shuffle.** Round-robin re-mask + permute + ZK
  shuffle proof; proof verification by all peers.
- **Phase 4 — threshold deal/reveal.** `PartialDecrypt` + Chaum–Pedersen
  proofs for hole cards (`ToSeat`) and board (`ToAll`); showdown reveal.
- **Phase 5 — recovery + transport.** Timeout/forfeiture events; a thin
  relay/gossip (or chain) transport fanning signed Events to all peers.

---

## Key Files

This EPIC writes **no code**. The files below are the *existing* pkcore
touch-points a future Phase 1 would build on — listed for orientation,
not modified by this spike.

| File | Role in a future implementation |
|---|---|
| `src/casino/table/event.rs` | `TableAction` — the event enum the `Event` envelope would wrap |
| `src/hand_history.rs` | `HandHistory`, `event_log`, `from_event_log` — today's chain/replay this generalizes |
| `src/deck.rs` | `DECK_ARRAY` / `POKER_DECK` — seed for the 52 group-element card encodings |
| `src/card.rs` | `Card::as_u32` — integer encoding feeding the `m_1…m_52` lookup table |
| `Cargo.toml` | feature-gate pattern (`bot-profiles`, `hand-histories`, `equity`) the future `mental-log` gate would follow |
| `ROADMAP.md` | the pkcore → pkdealer sibling-repo pattern `pkmental` mirrors |
| `docs/ANALYSIS_Mental_Poker.md` | source analysis this EPIC distills |

---

## Dependencies

- **Source:** [`docs/ANALYSIS_Mental_Poker.md`](./ANALYSIS_Mental_Poker.md).
- **Relates to ROADMAP Phase 4** (the distributed platform) as an
  *alternative, trustless* transport model.
- **Contrast with pkdealer:** pkdealer is the trusted-server gRPC dealer
  (centralized authority); `pkmental` is the trustless/serverless dealer
  (no authority). They are parallel transports over the same pkcore
  engine, not competitors.
- **Not a blocker for, nor blocked by,** the variant epics (EPIC-29
  through EPIC-34) or the equity work (EPIC-41). It is an independent
  distributed-systems track.

---

## Verification

This deliverable is a Markdown design document; verification is
review-based, not build-based.

- The document follows the house EPIC structure (Context / Status /
  Goals / Design / Key Files / Dependencies / Verification).
- Every Status row reflects spike/design maturity — no false
  "Complete" / "Shipped" claims.
- All `src/...` references cited resolve to real code:
  `TableAction` (`src/casino/table/event.rs:11`),
  `HandHistory::from_event_log` (`src/hand_history.rs:1673`),
  `DECK_ARRAY` (`src/deck.rs:13`), `Card::as_u32` (`src/card.rs:131`).
- No `cargo` build/test step applies — there are no code changes.

### Exit criteria (the decision gate)

Phase 0 is complete when the user reviews this document and decides
whether to advance to Phase 1. To advance, the gate must resolve these
open decisions (all currently **Decision pending** in the Status table):

1. **Crypto crate strategy** — depend on Geometry Research's
   `mental-poker` / `barnett-smart-card-protocol`, vendor them, or
   reimplement on raw `arkworks`?
2. **Curve / group** — Ristretto (simpler) vs. a pairing-friendly curve
   (enables SNARK shuffles)?
3. **Shuffle-proof scheme** — Neff / Bayer–Groth vs. a zk-SNARK circuit
   (Zypher-style sub-second proofs)?
4. **Transport** — relay, gossip, or chain-as-bulletin-board?
5. **Dropout/forfeiture model** — timeout values and the exact
   forfeiture-and-settle recovery semantics.
