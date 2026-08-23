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

It is grounded entirely in [`docs/ANALYSIS_Mental_Poker.md`](../ANALYSIS_Mental_Poker.md),
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

This is a spike. Nothing here is shipped in pkcore; the table records
*design* maturity and the decisions a future implementation epic must
resolve. Rows marked **Prototyped** were built and tested as a throwaway
`pkcore-mp` spike (`docs/files/mentalpoker/pkcore-mp/`) — a swappable
`CardCrypto` / `Coordinator` skeleton with a mock backend and a passing
two-seat deal/reveal round. That spike validates the design; the `pkmental`
crate productionizes it. (On naming: the spike is `pkcore-mp`; the shipped
sibling crate is `pkmental`, matching the `pk*` convention of `pkdealer` /
`pkspectator`.)

*Consolidation note (2026-08-14):* the full exploration workspace —
`pkcore-mp`, `tricktaking`, `mp-toy`, and `pktable` — was archived into
`docs/files/mentalpoker/` from the temporary `ImperialBower/mp` holding
repo; see that directory's `README.md` for crate roles and the known gap
(`pktable`'s three demo binaries were lost in packaging).

| Component | Status |
|---|---|
| Problem framing & engine mapping | **Spike — documented here** |
| Architecture split (pkcore log layer vs. `pkmental` crypto crate) | **Prototyped** (`pkcore-mp` spike) |
| `CardCrypto` / `Coordinator` trait seam | **Prototyped** (`pkcore-mp` spike) |
| Signed `Event` envelope schema | **Prototyped** (`pkcore-mp` spike) |
| Barnett–Smart deal/reveal flow | **Prototyped** (`PlaintextCrypto`, 2-seat round) |
| `mental-log` pkcore feature gate | **Designed, not built** |
| `pkmental` sibling crate | **Designed** (productionizes the `pkcore-mp` spike) |
| Deployment topology (6-architecture taxonomy) | **Decision pending** (recommended `1→3→5`) |
| Crypto crate selection (`arkworks` / Geometry Research / Zypher) | **Decision pending** |
| Curve / group choice | **Decision pending** |
| Verifiable-shuffle proof scheme (proof vs. argument) | **Decision pending** |
| Transport (relay vs. gossip vs. chain) | **Decision pending** |
| Dropout / forfeiture recovery model | **Decision pending** |
| Card-game generalization (`GameRules` / `tricktaking`) | **Designed** (prototype archived at `docs/files/mentalpoker/tricktaking/`) |
| QR-code state / transport | **Exploratory** (experimental idea only) |
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
- Establish the **two swappable seams** — `CardCrypto` (the mental-poker
  cryptography) and `Coordinator` (transport + ordering) — so the engine,
  the crypto scheme, and the deployment topology can each change without
  touching the other two. The `pkcore-mp` spike proves this composes.
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
  make equivocation *detectable* — and at a physical table the broadcast can
  be literal: a QR code shown to everyone at once is a broadcast channel a
  player cannot equivocate over (see *QR transport* under Design).

---

## Design

### The engine boundary (organizing principle)

One invariant holds across *every* architecture below and is the thing that
keeps pkcore untouched by cryptography:

> **pkcore's engine only ever sees plaintext `Card`s and `TableAction`s. All
> crypto is verified at the node boundary and never crosses into the
> transition function.**

A node verifies signatures and chain-linkage on the envelope, then verifies
the crypto proofs (shuffle argument, reveal-token proof) at its boundary, and
only *then* hands a plaintext `Card` or a `TableAction` to the pure engine
(`TableNoCell::apply_action`). Swapping the crypto changes the security/curve;
swapping the transport changes the deployment topology; neither touches the
engine, and pkcore's `arrays/*`, `lookups/*`, and `analysis` modules never see
a ciphertext. The whole design is an exercise in keeping that boundary clean.

### The two seams

Everything that *isn't* pkcore lives behind two traits. The `pkcore-mp` spike
implements both, with a mock backend, to prove they compose against the real
engine.

**`CardCrypto` — the mental-poker layer.** Wraps the Barnett–Smart *verifiable
l-out-of-l threshold masking function* (VTMF) — `keygen` / `mask` / `remask` /
`unmask` — plus a verifiable shuffle and a per-card *reveal token* (one
player's partial unmask with its proof). Masked cards and every proof are
**associated types**, so the engine and the event schema stay generic over the
scheme:

```rust
pub trait CardCrypto {
    type SecretKey;  type PublicKey;  type AggregateKey;
    type MaskedCard;                  // an ElGamal ciphertext: two curve points
    type RevealToken;                 // a player's partial unmask + its proof
    type KeyProof;  type MaskProof;  type ShuffleProof;  type Error;

    fn keygen(&self, rng: &mut impl RngCore) -> (Self::SecretKey, Self::PublicKey, Self::KeyProof);
    fn aggregate(&self, pks: &[Self::PublicKey]) -> Self::AggregateKey;
    fn encode(&self, card: Card) -> Self::MaskedCard;          // Card → group element
    fn decode(&self, m: &Self::MaskedCard) -> Result<Card, Self::Error>;
    fn mask(&self, agg: &Self::AggregateKey, m: &Self::MaskedCard, rng: &mut impl RngCore) -> (Self::MaskedCard, Self::MaskProof);
    fn remask(&self, agg: &Self::AggregateKey, c: &Self::MaskedCard, rng: &mut impl RngCore) -> (Self::MaskedCard, Self::MaskProof);
    fn shuffle(&self, agg: &Self::AggregateKey, deck: &[Self::MaskedCard], rng: &mut impl RngCore) -> (Vec<Self::MaskedCard>, Self::ShuffleProof);
    fn reveal_token(&self, sk: &Self::SecretKey, pk: &Self::PublicKey, c: &Self::MaskedCard, rng: &mut impl RngCore) -> Self::RevealToken;
    /// A *subset* of tokens leaves a card still locked (reveal-to-one); the
    /// *full* set yields a plaintext that `decode` accepts (reveal-to-all).
    fn unmask(&self, c: &Self::MaskedCard, tokens: &[Self::RevealToken]) -> Result<Self::MaskedCard, Self::Error>;
    // verify_key / verify_mask / verify_shuffle / verify_reveal_token elided.
}
```

**`Coordinator` — transport + ordering.** Append-and-replay over a signed,
hash-linked log. This is the *only* thing that differs across the deployment
topologies below; each is one impl.

```rust
pub trait Coordinator {
    type Error;
    async fn publish(&mut self, event: SignedEvent) -> Result<(), Self::Error>;
    async fn next_event(&mut self) -> Result<SignedEvent, Self::Error>;  // drives every peer's identical replay
    async fn head(&self) -> Result<Hash, Self::Error>;                   // log head, for chaining prev_hash
}
```

**The mock pair.** The spike's `PlaintextCrypto` sets `MaskedCard = Card` and
all proofs to `()`, but *faithfully models the l-out-of-l padlock accounting*:
a card carries a set of seat padlocks, each `reveal_token` removes one, and
`decode` only succeeds when the set is empty — so the two-seat test meaningfully
asserts that a hole card stays locked with only the *other* seat's token and
unlocks only when its owner adds theirs. `InProcCoordinator` is one shared
append-only log with per-reader cursors (architecture #1). Together they let
the entire game loop be wired and tested before any real curve arithmetic
exists; the real backend swaps in with zero engine or transport changes. See
`docs/files/mentalpoker/pkcore-mp/{src/lib.rs,tests/round.rs,README.md}`.

### Crate-location split (orthogonal to topology)

The load-bearing *packaging* decision, independent of which topology ships.
Concern (1) is deterministic and dependency-light; concern (2) is crypto-heavy.
They live in different crates so pkcore never grows an `arkworks` dependency.

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
sibling-repo pattern in [`ROADMAP.md`](../../ROADMAP.md): pkcore owns the
logic, the sibling owns the transport. The contrast with pkdealer is the
trust model — pkdealer is a *trusted-server* gRPC dealer; pkmental is
*trustless / serverless*.

### Deployment topology (the six architectures)

The crate split above says *where the code lives*; this says *how the table is
deployed*. Each is one `Coordinator` impl, from easiest-to-build to most
trustless. pkcore's role never changes — it is always the deterministic domain
engine. What differs is where three things live: **coordination** (who orders
events and prevents equivocation), **verification** (who checks the crypto
proofs), and **settlement** (who holds and enforces the money).

| # | Topology | `Coordinator` impl | Trust / notes |
|---|---|---|---|
| 1 | **In-process reference harness** | `InProcCoordinator` (shared queue) | No network, no signatures. Proves crypto + engine compose. This is the `pkcore-mp` spike. |
| 2 | **Full-mesh P2P** | `MeshCoordinator` (libp2p/gossipsub) | No server at all. Equivocation only *detectable*, not prevented; NAT + dropout pain. Friends / play-money. |
| 3 | **Stateless relay / bulletin board** | `RelayCoordinator` (one dumb fan-out) | The literal "broadcast channel" Barnett–Smart assumes. One ordering point kills equivocation; relay can't read cards or forge sigs. SSH-served TUI fits here. |
| 4 | **Semi-trusted coordinator** | `CoordinatorServer` (also validates) | "Trust for liveness, not fairness." Smoothest migration from today's `Dealer`: bolt crypto on so the server can't peek or stack the deck. |
| 5 | **State channel + on-chain settlement** | `ChannelCoordinator` | Play off-chain (like #3/#4); chain only escrows, settles, and *adjudicates* a dispute/dropout by replaying the signed log. Solves real-money + the dropout penalty (Kaleidoscope/Royale). |
| 6 | **Fully on-chain** | `ContractCoordinator` | Contract *is* the broadcast channel + verifier + escrow. Maximally trustless; pay gas per action; needs a SNARK shuffle (zkShuffle/BN254). Rarely worth it over #5. |

**Recommended path: `1 → 3 → 5`.** Prove the crypto in-process against pkcore,
get a playable serverless table over a dumb relay, then add state-channel
settlement once stakes matter. #4 and #6 are branches off that spine —
depending on whether you prioritize a smooth migration from today's `Dealer`
(#4) or maximal trustlessness (#6).

### Three cross-cutting pkcore changes

Independent of topology, every option needs the same three refactors:

1. **The deck becomes a vector of masked cards.**
   **Status 2026-08-23: landed in pkcore `0.8.0` as
   [EPIC-79b](./EPIC-79b_Sealed_Deck.md).** `pkcore::seal` ships `CardSeal`,
   `SlotId`, `SealedCard<S>` and `SealedDeck<S>` — a deck that shuffles, cuts
   and deals blind (Phases 0–2), plus `TableAction::SealedDealt` / `Revealed`
   so the event log stops leaking hole cards (Phases 4a–4c). Wiring it into
   `Table` is EPIC-79b Phase 3, approved as *Option A′* — the deck is
   `SealedDeck<S>` always, with a `NullSeal` identity scheme for solvers and
   bots. See [Implementing `CardSeal` in `pkmental`](./EPIC-79b_Sealed_Deck.md)
   for the backend mapping.

   Today `Deck` holds concrete `Card`s and dealing reveals them. In a mental-poker design a `Card` only
   materializes after the unmask protocol completes for its slot, so
   `DealHand` / `DealFlop` / … stop drawing from a local shuffled `Deck` and
   instead trigger reveal-token collection, with the resulting plaintext fed
   back into the engine. `DECK_ARRAY` (`src/deck.rs:13`) is perfect as the
   canonical public ordering fixing the 52-`Card` ↔ 52-group-element bijection.
2. **`Visibility` wants three states, not two.** The protocol distinguishes
   *masked* (no one knows), *known-to-owner* (unmasked to seat `j` only — your
   hole cards mid-hand), and *public* (`Up`). The clean move keeps `Card`
   visibility-free as `src/play/visibility.rs:1` already intends, and tracks
   mask/reveal status in a parallel structure keyed by deck slot — letting
   `Visibility::Up` mean "plaintext now lives in the engine."
3. **The engine stays pure and crypto-agnostic.** `TableNoCell::apply_action`
   + `PlayerState` legality is the deterministic core every peer replays; it
   treats shuffle proofs and reveal tokens as opaque blobs verified *outside*
   the transition function — which is exactly the engine boundary above.

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

In the `pkcore-mp` spike this `Payload` is the trait-parameterized
`EventPayload<C: CardCrypto>`: `MaskedCard`, `ShuffleProof`, `RevealToken`,
and the rest become `C::MaskedCard`, `C::ShuffleProof`, `C::RevealToken`, so a
single schema serves the mock backend, an arkworks backend, and a SNARK
backend without change. The `Action` variant carries today's `TableAction`
verbatim, and the envelope is the `SignedEvent { table_id, hand_id, seq,
prev_hash, author, payload, sig }` struct the `Coordinator` total-orders.

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
pkcore's existing per-hand event log. Today `HandHistory`
(`src/hand_history.rs:128`) slices `table.event_log` (a `Vec<TableAction>`)
per hand and feeds it to `Streets::from_event_log`
(`src/hand_history.rs:1673`) to derive per-street structure. The mental-poker `Event` is that same idea with a
cryptographic envelope: the engine-relevant `Payload::Action` /
`HandResult` variants correspond directly to today's `TableAction`
variants (`Bet`, `Call`, `Raise`, `Fold`, `Dealt`, `DealtFlop`, …,
`src/casino/table/event.rs:39-66`). The future `mental-log` work (Phase 2)
maps `TableAction` into `Payload` and adds the `{ seq, prev_hash, author,
sig }` envelope.

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

> **VTMF vocabulary.** The steps below are spelled out in raw ElGamal for
> intuition, but the paper and the reference implementation (`round.rs`) wrap
> them in two named abstractions — the same ones the `CardCrypto` trait
> exposes. The map is: **keygen** ↔ Step 0, **mask** ↔ Step 1, **remask** ↔
> the re-randomization in Step 2, **shuffle proof** ↔ Step 2's ZK argument,
> and **unmask** ↔ Steps 4–6 (each player's partial contribution `d_i` + its
> Chaum–Pedersen proof is a *reveal token*; a hole card is a staged unmask
> that stops one short, the board is a full unmask). Reading `round.rs`, the
> flow is literally `keygen` → aggregate → `mask` the 52 encodings →
> `shuffle_and_remask` per player → `reveal_token`s + `unmask` per the dealing
> rules → full unmask at showdown.

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

> **Proof vs. argument (a real decision-gate subtlety).** Barnett–Smart's
> original Step 2 used a zero-knowledge *proof* of correct shuffle, which is
> expensive. The modern reference (Geometry) deliberately swaps it for an
> *argument of knowledge* — the Bayer–Groth shuffle argument (2012). The
> distinction changes the security statement: a valid *proof* can never be
> forged, while a valid *argument* can be forged by a computationally
> *unbounded* adversary, so an argument is sound only against a *bounded*
> adversary — the same assumption public-key encryption already rests on, so
> in practice it's a fine trade for the speed (~50 ms to prove, <1 ms to
> verify for a 52-card shuffle). The deck is only fair once *every* player has
> taken a shuffle turn. Accepting "argument, not proof" is one of the gate
> decisions below.

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
- **`akonradi/mental-poker`** — another arkworks / ElGamal `no_std` Rust
  implementation; a second reference point.
- **zk-SNARK shuffle work (Zypher, zkShuffle / Manta, and others)** — has
  pushed shuffle proofs to sub-second and targets cheap *on-chain*
  verification (zkShuffle uses BN254 to cut Ethereum gas vs. Geometry's
  Starknet-curve cost). Relevant for topology #6, or if Neff / Bayer–Groth
  proofs prove too slow.
- **LibTMCG** — the long-standing C library implementing Schindelhauer's
  toolbox; ships a curated bibliography that is the best single index to the
  field. Not a Rust dependency, but the canonical reference.

See the **References** section below for the papers behind each of these.

### QR transport (experimental)

> **Status: an experimental idea, not a decision gate.** Captured here so it
> isn't lost; nothing in the phased roadmap depends on it. It is one possible
> `Coordinator` impl for the offline / in-the-room case.

A QR code is just a display format for bytes, so it can carry the same signed,
chained events as any network transport — every recipient still verifies the
signature and that the event links onto the known head. The constraint is
capacity: a single QR maxes out at Version 40 (~2,953 bytes at the lowest error
correction, realistically ~1–1.5 KB scannable across a table). That splits the
state cleanly into *what fits* and *what doesn't*:

| Object | Size | Fits one QR? |
|---|---|---|
| Sync pointer (head hash + seq + sig + relay addr) | ~100 B | ✅ |
| Public engine state (phase, pot, board, per-seat stack/bet) via postcard | <300 B | ✅ |
| One reveal token + Chaum–Pedersen proof + sig | ~150–200 B | ✅ |
| Full masked deck (52 × two curve points) | ~3.3 KB | ❌ |
| Bayer–Groth shuffle proof | kilobytes | ❌ |

So QR has two legitimate roles. **(a) Sync beacon / commitment** — a "table"
QR encodes `Coordinator::head()`, the seq, a signature, and a bootstrap
address; players scan it to confirm they're on the same chain head and to find
where to pull the bulk log from (exactly how hardware-wallet / Matrix pairing
QR works). **(b) Actual transport with no network** — small per-turn deltas (a
bet, a fold, one reveal token) each fit in one QR; the big one-time objects
(masked deck, shuffle proofs) use *fountain-coded animated* QR (RaptorQ / Luby
transform across a rotating frame sequence, reconstructed from any sufficient
subset), exactly as air-gapped Bitcoin wallets move multi-KB transactions
(BC-UR). The bonus, already noted under Non-Goals: a QR shown to the whole
table is a **physical broadcast** that directly attacks equivocation — you
can't show one event to one player and a conflicting one to another when
everyone photographs the same screen. Implementation would be a `QrCoordinator`
alongside `InProcCoordinator`: `publish` renders frames, `next_event` decodes
scanned ones (`qrcode` + `rqrr`/`bardecoder` + `raptorq` + postcard).

### Generalization to other card games

The mental-poker work is poker-specific only at the *rules* layer; the crypto
and the engine boundary are not. pkcore already has a sibling, **gfcore** (Go
Fish), and the target includes bridge, spades, hearts. The two cores share only
`cardpack` today and have independently reinvented the same four things — an
event log + replay, a bot harness, a hidden-information projection
(`Visibility` vs gfcore's `player/view`), and a rules state machine. The clean
factoring is **not** a universal "Game" object (poker's betting, Go Fish's set
collection, and bridge's trick-taking share no honest shape there) but a layer
*below* it, on the two things every card game genuinely shares:

1. a small **algebra of card operations over zones** — `shuffle`, `deal`,
   `move` (play to trick / discard / pass), `reveal(to: audience)`,
   `peek(seat)`; and
2. a **zone + visibility model** — a zone is private-to-seat, public,
   hidden-to-all, or revealed-to-a-subset.

The recommended layering is `cardpack` → a generic engine over a `GameRules`
trait → family mid-layers (`tricktaking` for bridge/spades/hearts; `betting`
for poker; `collection` for Go Fish/Rummy) → specific games. A prototype
`tricktaking` crate ([github.com/ImperialBower/tricktaking](https://github.com/ImperialBower/tricktaking))
already realizes this: shared trick/follow-suit/trump resolution with bridge
and spades as thin impls supplying only `trump`, `can_play`, and `score`, all
driven by a generic `run` loop that mentions no game-specific concept.

The payoff for *this* EPIC: because `CardCrypto` already speaks
*hidden / revealed-to-seat / public*, the `pkmental` crypto layer generalizes
to a `cardgame-mp` simply by parameterizing over that visibility model instead
of over poker — `view_for` is the seam where a `decode`d card surfaces once
revealed, and the plaintext engine and the cryptographic one implement the
*same* projection; only the representation of "hidden" differs. This is not
speculative: **the Royale protocol is exactly the generalization of the
poker-specific Kaleidoscope to UC-secure general card games**, the same move at
the protocol level that the operations algebra is at the engine level.

The hard parts to scope (deferred, not solved here): **bidding/auction phases**
don't exist in poker or Go Fish, so `PhaseHoldem` won't transfer and bridge's
auction is genuinely complex; **partnerships/teams** (bridge, spades are 2v2)
add a layer the seat model assumes away; **scoring** is where per-game code
concentrates and resists abstraction (keep `Outcome` game-specific); **deck
size varies** (euchre 24, pinochle 48), so the engine must take a `DeckSpec`
rather than hardcode `DECK_ARRAY`; and the two **`cardpack` versions** must be
aligned first (gfcore 0.7.0 vs pkcore 0.6.9) since a shared engine can't depend
on both.

### Phased roadmap with decision gates

The spike's recommended path, reframed around the two seams and the `1 → 3 → 5`
topology sequence. **This document commits only to Phase 0.** Each later phase
is a *future, separately-approved* epic.

- **Phase 0 — this document.** Design + decision gate.
- **Phase 1 — seams + reference harness (`pkmental`, topology #1).** Define
  the `CardCrypto` and `Coordinator` traits; ship `PlaintextCrypto` +
  `InProcCoordinator` wired to pkcore's `TableNoCell` / `TableAction`. This is
  the `pkcore-mp` spike promoted into `pkmental`: deterministic, fully
  unit-testable, *no real crypto*. Validates that the crypto layer and the
  engine compose, and nails the `Card` ↔ group-element bijection.
- **Phase 2 — `mental-log` + relay (topology #3).** The `SignedEvent` envelope
  (Ed25519 sign + SHA-256 hash-chain) over the existing `TableAction`, behind
  a pkcore `mental-log` feature gate (matching `bot-profiles` / `equity`); plus
  a `RelayCoordinator` so a serverless table is playable. No card crypto yet.
- **Phase 3 — real `CardCrypto` over arkworks.** Key setup (`KeyShare` +
  Schnorr PoK), aggregate key `h`, trivial deck init — the first real backend
  swapped in behind the Phase-1 trait with no engine change.
- **Phase 4 — verifiable shuffle.** Round-robin re-mask + permute + Bayer–Groth
  shuffle *argument*; verification by all peers.
- **Phase 5 — threshold deal/reveal.** `PartialDecrypt` + Chaum–Pedersen
  proofs / reveal tokens for hole cards (`ToSeat`) and board (`ToAll`);
  showdown reveal.
- **Phase 6 — recovery + settlement (topology #5).** Timeout/forfeiture events;
  a `ChannelCoordinator` adding on-chain escrow + dispute adjudication.
  Topologies #4 (semi-trusted coordinator) and #6 (fully on-chain) are branches
  off this spine, not required steps.

---

## Key Files

This EPIC writes **no code**. The files below are the *existing* pkcore
touch-points a future implementation would build on — listed for orientation,
not modified by this spike.

| File | Role in a future implementation |
|---|---|
| `src/casino/table/event.rs` | `TableAction` — the event enum the `Event` envelope would wrap |
| `src/hand_history.rs` | `HandHistory`, `event_log`, `Streets::from_event_log` — today's chain/replay this generalizes |
| `src/deck.rs` | `DECK_ARRAY` / `POKER_DECK` — seed for the 52 group-element card encodings |
| `src/card.rs` | `Card::as_u32` — integer encoding feeding the `m_1…m_52` lookup table |
| `src/play/visibility.rs` | `Visibility` (`Down`/`Up`) — the two-state model the masked/known/public refactor extends |
| `Cargo.toml` | feature-gate pattern (`bot-profiles`, `hand-histories`, `equity`) the future `mental-log` gate would follow |
| `ROADMAP.md` | the pkcore → pkdealer sibling-repo pattern `pkmental` mirrors |
| `docs/ANALYSIS_Mental_Poker.md` | source analysis this EPIC distills |
| `docs/files/mentalpoker/` | the archived exploration workspace: `pkcore-mp/` (the reference-harness spike `pkmental` Phase 1 productionizes), `tricktaking/`, `mp-toy/`, and `pktable/` — see its `README.md` |

---

## Dependencies

- **Source:** [`docs/ANALYSIS_Mental_Poker.md`](../ANALYSIS_Mental_Poker.md).
- **Relates to ROADMAP Phase 4** (the distributed platform) as an
  *alternative, trustless* transport model.
- **Contrast with pkdealer:** pkdealer is the trusted-server gRPC dealer
  (centralized authority); `pkmental` is the trustless/serverless dealer
  (no authority). They are parallel transports over the same pkcore
  engine, not competitors.
- **Generalization track:** the engine/crypto layering relates to **gfcore**
  (Go Fish) and a prototype **`tricktaking`** crate
  ([github.com/ImperialBower/tricktaking](https://github.com/ImperialBower/tricktaking))
  for bridge/spades/hearts. pkcore and gfcore share only `cardpack` today; a
  shared card-game engine (`GameRules` + family mid-layers) is the natural
  home for the `cardgame-mp` generalization of `pkmental`. Aligning the two
  `cardpack` versions (gfcore 0.7.0, pkcore 0.6.9) is a prerequisite.
- **Not a blocker for, nor blocked by,** the variant epics (EPIC-29
  through EPIC-34) or the equity work (EPIC-41). It is an independent
  distributed-systems track.

---

## References

The literature behind the design, grouped as in the analysis. Links favor
stable sources (author PDFs, IACR ePrint, publisher DOIs).

**Founding papers**

- Shamir, Rivest & Adleman, *"Mental Poker"* (MIT LCS/TM-125, 1979; repr. *The
  Mathematical Gardner*, 1981). The origin of the problem.
  <https://people.csail.mit.edu/rivest/pubs/SRA81.pdf>
- Coppersmith, *"Cheating at Mental Poker"* (CRYPTO '85) — the quadratic-residue
  leak in SRA. <https://doi.org/10.1007/3-540-39799-X_10>
- Fortune & Merritt, *"Poker Protocols"* (CRYPTO '85).
  <https://doi.org/10.1007/3-540-39568-7_36>
- Crépeau, *"A Secure Poker Protocol that Minimizes the Effect of Player
  Coalitions"* (CRYPTO '85) and *"…Confidentiality of the Players' Strategy"*
  (CRYPTO '86) — early collusion / strategy-hiding treatment.

**The practical protocol (the one walked through above)**

- Barnett & Smart, *"Mental Poker Revisited"* (IMACC 2003, LNCS 2898) — the
  threshold-ElGamal + verifiable-shuffle scheme.
  <https://doi.org/10.1007/978-3-540-40974-8_29>
- Castellà-Roca, Domingo-Ferrer et al., *"Practical Mental Poker Without a TTP
  Based on Homomorphic Encryption"* (Indocrypt 2003); Castellà-Roca,
  *"Contributions to Mental Poker"* (thesis, 2005).
- Wei & Wang, *"A Fast Mental Poker Protocol"* (IACR ePrint 2009/439).
  <https://eprint.iacr.org/2009/439>

**Primitives (the proofs in Steps 2/4/5)**

- ElGamal, *"A Public Key Cryptosystem … Based on Discrete Logarithms"* (IEEE
  IT, 1985) — the encryption layer.
- Groth, *"A Verifiable Secret Shuffle of Homomorphic Encryptions"* (ePrint
  2005/246; J. Cryptology 2010) and the later **Bayer–Groth** shuffle argument
  (2012) — what Geometry uses.
- Chaum–Pedersen (equality of discrete logs, the partial-decryption proof);
  Schnorr (PoK of secret keys); Fiat–Shamir, *"How to Prove Yourself"* (CRYPTO
  '86) for non-interactivity; Neff, *"A Verifiable Secret Shuffle…"* (CCS 2001)
  as the alternative shuffle proof.

**Settlement, penalties, the drop-out problem (the money layer)**

- David, Dowsley & Larangeira, *"Kaleidoscope: An Efficient Poker Protocol with
  Payment Distribution and Penalty Enforcement"* (FC 2018).
  <https://eprint.iacr.org/2017/899>
- Same authors, *"ROYALE: A Framework for Universally Composable Card Games…"*
  (FC 2019) — the UC generalization to general card games.
  <https://eprint.iacr.org/2018/157>
- Same authors, *"21 — Bringing Down the Complexity: Fast Composable Protocols
  for Card Games Without Secret State"* (ePrint 2018/303).
  <https://eprint.iacr.org/2018/303>
- Bentov et al., *"Instantaneous Decentralized Poker"* (ASIACRYPT 2017);
  Kumaresan et al. (CCS 2015) — the blockchain-poker work the above builds on.

**Implementations**

- `geometryxyz/mental-poker` (Rust) — Barnett–Smart + Bayer–Groth on arkworks;
  runnable `barnett-smart-card-protocol/examples/round.rs`.
  <https://github.com/geometryxyz/mental-poker>
- `akonradi/mental-poker` (Rust) — arkworks/ElGamal `no_std`.
  <https://github.com/akonradi/mental-poker>
- zkShuffle (Manta) — SNARK-based mental poker minimizing on-chain gas.
- LibTMCG (Heiko Stamer) — C library implementing Schindelhauer's toolbox;
  ships the field's best bibliography. <https://www.nongnu.org/libtmcg/>

**Tutorials (the gentlest accurate path in)**

- Nicolas Mohnblatt (Geometry), *"Mental Poker in the Age of SNARKs,"* Parts 1 &
  2. <https://nmohnblatt.me/mental-poker-1/>
- Wikipedia, *"Mental poker"* — orientation + further-reading trail.

---

## Verification

This deliverable is a Markdown design document; verification is
review-based, not build-based.

- The document follows the house EPIC structure (Context / Status /
  Goals / Design / Key Files / Dependencies / References / Verification).
- Every Status row reflects spike/design maturity — no false
  "Complete" / "Shipped" claims.
- All `src/...` references cited resolve to real code:
  `TableAction` (`src/casino/table/event.rs:11`),
  `HandHistory` (`src/hand_history.rs:128`) /
  `Streets::from_event_log` (`src/hand_history.rs:1673`),
  `DECK_ARRAY` (`src/deck.rs:13`), `Card::as_u32` (`src/card.rs:131`),
  `Visibility` (`src/play/visibility.rs:28`).
- The `pkcore-mp` spike it cites lives at `docs/files/mentalpoker/pkcore-mp/`
  and its two-seat round passes (`two_seat_deal_and_reveal`,
  `coordinator_orders_and_chains_events`).
- No `cargo` build/test step applies to pkcore — there are no code changes.

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
   (Zypher-style sub-second proofs)? — and the **proof-vs-argument**
   acceptance (a Bayer–Groth *argument* is sound only against a bounded
   adversary; confirm that's acceptable).
4. **Deployment topology** — which of the six architectures, and confirm the
   `1 → 3 → 5` build sequence (in-process → relay → state-channel)?
5. **Dropout/forfeiture model** — timeout values and the exact
   forfeiture-and-settle recovery semantics.
