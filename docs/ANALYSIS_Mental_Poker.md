# Mental Poker

> > Is there a way to play a distributed card game like no limit holdem where there isn’t a centralized server,
> but it is instead stateless, possibly using public/private key encryption?

Yes — this is a well-studied problem with a name: **mental poker**. The foundational question ("can we play a fair card game with no trusted dealer?") was posed by Shamir, Rivest, and Adleman in 1979, and the cryptographic machinery to do it properly has matured a lot since.

The key insight is to separate two concerns that often get conflated when people say "server":

**1. The state-coordination problem (the "stateless" part).** A poker hand obviously *has* state — deck, board, pot, whose turn it is — but that state doesn't need to live on an authoritative server. The clean model is a **replicated state machine over a signed, hash-linked event log**. Every player runs the *same deterministic* transition function: given the same ordered sequence of signed actions, everyone independently computes byte-identical state. The "server" effectively becomes (a) the protocol and (b) a pure engine — which is exactly the kind of deterministic core a poker library like yours already is. Each action (bet, fold, raise, reveal) is signed with the actor's private key and references the hash of the prior state, so the history is tamper-evident and any fork or equivocation is detectable. For turn-based games you can often skip heavy BFT consensus, because the rules already constrain *who* may legitimately act next; a player signing out of turn is simply rejected by everyone.

So "stateless" here really means "no authoritative party — state is the deterministic fold over a verifiable log." Public/private keys do the authentication and non-repudiation work.

**2. The card-secrecy problem (the genuinely hard cryptography).** This is where mental poker proper comes in. You need a shuffled deck where no one knows anyone's cards, no card is duplicated, the shuffle is provably a real permutation, and cards can be revealed selectively and verifiably. The modern approach (Barnett–Smart, 2003, building on the original commutative-encryption idea) uses:

- **ElGamal encryption** of the 52 cards, with the decryption key **split among all players** (threshold/distributed key generation). No single player can decrypt anything alone.
- **Verifiable shuffles**: each player shuffles and re-randomizes the encrypted deck in turn, attaching a zero-knowledge proof that they applied *some* valid permutation without revealing which. The composition of everyone's secret permutations is the real shuffle, and no one knows it.
- **Threshold/cooperative decryption** to deal: to show a hole card to just one player, every *other* player publishes their partial decryption, leaving only the recipient able to compute the final value. To reveal the board, everyone contributes.

Commutative-encryption schemes (the original SRA approach, where E_A(E_B(x)) = E_B(E_A(x))) also work and are conceptually simpler, but watch out — naive RSA-style versions leak quadratic-residue information about cards unless you encode the deck carefully.

**The honest caveats**, because they matter for whether this is worth building:

- **The drop-out problem.** With (n,n) threshold decryption, a player who's about to lose can stall the whole game by refusing to provide their partial decryption. You need timeouts, forfeiture rules, or recovery mechanisms, and this is a real, well-known pain point rather than a solved one.
- **Settlement is separate from dealing.** Mental poker makes the *cards* trustless, but not the *money*. Without a server, who holds the chips and enforces that the loser pays? For play-money or social-trust games this is fine. For real stakes you essentially need on-chain escrow / smart contracts, which is why most serverless real-money attempts end up on a blockchain (using the chain as a public bulletin board and escrow, not as a "server"). That reintroduces cost and latency.
- **Collusion is unsolvable by crypto.** Nothing stops two players from telling each other their hands over a side channel. That's true of all poker, but worth stating.

**On the Rust side**, the building blocks exist. The `arkworks` ecosystem (ark-ec, ark-ff, ark-ff-based curves) gives you the elliptic-curve and ZK primitives, and Geometry Research published open-source Rust crates implementing exactly this — a `mental-poker` / `barnett-smart-card-protocol` pair on top of arkworks. That's the most direct thing to look at as a reference implementation of the card layer. Newer work uses zk-SNARK-based shuffles (Zypher and others) that have pushed shuffle proofs down to sub-second, which makes a playable interactive table realistic. This area moves fast, though, so I'd verify the current maintenance status and benchmarks with a search before committing — my view here is roughly as of early 2026 and these libraries churn.

If you wanted to actually assemble this, the architecture would be: your deterministic engine as the state machine, a thin P2P/gossip transport (or a relay, or a chain if money is involved) carrying signed actions onto a hash-chained log, and the Barnett–Smart layer sitting on top to handle the deck. The engine you'd write is the stateless part; the crypto and transport are the wrapper. It also maps neatly onto a peer-to-peer or SSH-style table where each seat is just a keypair and a log-follower rather than a client talking to a host.

Want me to sketch the message/event schema for the signed log, or walk through the Barnett–Smart deal-and-reveal flow step by step?

> yes to both

Good — I'll do the schema first, then the protocol flow, and try to make them line up so you can see which events carry which cryptographic payloads.

## The signed event log

The whole game is a sequence of events. Every event shares an envelope; the payload varies. The envelope is what makes the log tamper-evident and lets every peer reconstruct identical state without anyone being in charge.

```rust
// Abstract crypto types — concretely these are arkworks types over
// some prime-order group (e.g. Ristretto, or a pairing-friendly curve
// if you want SNARK shuffles).
type Scalar = ...;   // field element: secret keys, randomness r
type Point  = ...;   // group element: public keys, card encodings
type Sig    = ...;   // Schnorr/Ed25519-style signature
type Hash   = [u8; 32];

/// ElGamal ciphertext = a "masked card". Underlying plaintext is a Point.
struct MaskedCard { c1: Point, c2: Point }

/// The envelope every peer signs and chains.
struct Event {
    table_id: TableId,
    hand_id:  HandId,
    seq:      u64,            // monotonic within the hand
    prev_hash: Hash,          // hash of the previous Event — the chain
    author:   PlayerPk,       // who is claiming to emit this
    payload:  Payload,
    sig:      Sig,            // author's signature over everything above
}
```

The `prev_hash` link is doing the heavy lifting. Each event commits to the entire history before it, so you can't reorder, drop, or splice. `seq + author` together constrain legality: the rules say whose turn it is, so an event from the wrong author at the wrong seq is rejected by everyone independently. That's how you get ordering without a consensus protocol — the turn structure *is* the ordering oracle, and the signature stops impersonation.

The payloads split into three phases — setup, shuffle, and play:

```rust
enum Payload {
    // ---- Setup ----
    TableCreate {
        group_params: GroupParams,     // (G, q, g) + the 52 card encodings
        seats: Vec<PlayerPk>,
        blinds: Blinds,
        starting_stacks: Vec<Chips>,
    },
    // Each player publishes their public key share + proves they know the secret.
    KeyShare {
        h_i: Point,                    // h_i = g^{x_i}
        knowledge_proof: SchnorrProof, // ZK proof of knowledge of x_i
    },

    // ---- Shuffle ----
    // The aggregate key h = ∏ h_i is *computed* by everyone, not emitted.
    Shuffle {
        deck_out: Vec<MaskedCard>,     // permuted + re-masked deck
        shuffle_proof: ShuffleProof,   // ZK: output is a valid shuffle of input
    },

    // ---- Play ----
    // Reveal machinery: a partial decryption of one deck position.
    PartialDecrypt {
        position: u8,                  // which deck slot
        target: RevealTarget,          // ToAll | ToSeat(seat)
        d_i: Point,                    // d_i = c1^{x_i} for this card
        proof: ChaumPedersenProof,     // proves d_i is correct vs h_i
    },
    // Betting actions.
    Action { kind: ActionKind },       // Fold | Check | Call | Bet(c) | Raise(c)

    // Showdown / end.
    HandResult { payouts: Vec<(PlayerPk, Chips)> }, // deterministic, so verifiable
}

enum RevealTarget { ToAll, ToSeat(SeatIdx) }
```

A few design notes that matter:

The deck assignment (which slot is the flop, which two slots are seat 3's hole cards) is *deterministic from the rules*, so it never needs its own event — every peer computes it. Same for `HandResult`: given the revealed cards and the betting log, the payout is a pure function, so it's verifiable rather than authoritative. If a peer emits a `HandResult` that doesn't match everyone's computation, it's simply rejected.

Betting (`Action`) and crypto (`PartialDecrypt`) events live in the *same* chain, interleaved in the order the hand actually progresses: shuffle, deal hole cards (partial decrypts targeted to each seat), preflop betting, deal flop (partial decrypts `ToAll`), and so on. One log, one fold-left, one state.

Equivocation — a malicious peer sending event A to you and a conflicting event A' to someone else at the same `seq` — is *detectable* because both are signed by the same key over the same `prev_hash` and `seq`. Any honest peer that sees both has a cryptographic proof of cheating. It's not *prevented* without a broadcast/consensus layer (a relay everyone reads, or a chain), so for serious play you'd want all events fanned out to all peers, not just point-to-point.

## The Barnett–Smart deal-and-reveal flow

Work in a cyclic group `G` of prime order `q` with generator `g`. The 52 cards are mapped to 52 fixed, public, distinct group elements `m_1 … m_52` (a precomputed lookup table — you'll invert it at the end to turn a recovered Point back into a card).

**Step 0 — Key setup.** Each player `i` picks a secret `x_i`, publishes `h_i = g^{x_i}` (the `KeyShare` event) with a Schnorr proof of knowledge of `x_i`. Everyone computes the shared public key:

```
h = ∏ h_i = g^(∑ x_i)
```

No one knows the corresponding secret `∑ x_i` — it's split `(n, n)`. This is the key all cards get encrypted under.

**Step 1 — Initialize the deck.** Start with trivial ElGamal encryptions of each card encoding: `(c1, c2) = (1, m_j)` for each `j`. These are public; nothing is hidden yet.

**Step 2 — Shuffle round-robin.** Each player, in turn, takes the current deck and does two things to *every* card:

- *Re-mask:* `(c1, c2) → (c1 · g^{r}, c2 · h^{r})` with fresh random `r` per card. ElGamal is homomorphic, so this changes the ciphertext's appearance without changing the plaintext underneath.
- *Permute:* apply a secret random permutation `π` to the deck order.

The player attaches a zero-knowledge shuffle proof (Neff, Bayer–Groth, or a SNARK circuit) showing the output really is a permutation-plus-re-masking of the input — without revealing `π` or any `r`. After all `n` players have done this, the deck is uniformly shuffled, fully masked under `h`, and the composite permutation is unknown to everyone. This is the `Shuffle` chain of events.

**Step 3 — Deal.** Deck positions map to roles by the rules (positions 0–1 → seat 0's hole cards, 2–3 → seat 1's, …, then burn/flop/turn/river slots). No card is decrypted yet; players just agree on which masked slot belongs where.

**Step 4 — Reveal a hole card to exactly one player.** Take the masked card at seat `j`'s slot: `(c1, c2) = (g^r, m · h^r) = (g^r, m · g^{r·∑x_i})`. To decrypt it you'd need to divide out `c1^{∑ x_i}`. So: **every player except `j`** publishes their partial decryption

```
d_i = c1^{x_i}
```

with a Chaum–Pedersen proof that `log_{c1}(d_i) = log_g(h_i)` — i.e. that `d_i` was computed with the *same* secret behind their public key. (Without this proof a player could hand you a bogus `d_i` and make you misread your own hand.) Player `j` then computes

```
m = c2 / ( (∏_{i≠j} d_i) · c1^{x_j} )
```

applying their own share last. Everyone else contributed and verified, but since only `j` knows `x_j`, only `j` finishes the division and learns `m`. Invert the lookup table: that Point is their card. These are the `PartialDecrypt { target: ToSeat(j) }` events.

**Step 5 — Reveal a community card to everyone.** Identical, except *all* `n` players publish `d_i` with proofs (`target: ToAll`), and anyone can compute `m = c2 / ∏ d_i`. That's the flop, turn, river.

**Step 6 — Showdown.** A player who has to show down reveals their hole cards by publishing the partial decryption they withheld in Step 4 — effectively their `x_j`-share *for those specific cards* (still proof-backed), so everyone can now complete the division and verify the cards were the ones dealt, not swapped. Folded players never reveal anything.

The proofs at every masking and decryption step are what make it *trustless* rather than merely *distributed*: a cheater can't shuffle in a duplicate card, can't peek (they only ever hold their own `x_i`), and can't lie about a partial decryption without producing a proof that fails verification — which becomes signed, on-chain evidence of cheating.

The one thing this flow doesn't fix, and I want to flag it again because it shapes the engineering: in Step 4/5, if a required player goes offline before publishing `d_i`, that card can't be opened and the hand stalls. So your event schema needs timeout events and a forfeiture rule (e.g. a player who fails to provide a partial decrypt within `T` is folded and forfeits, recoverable because the rest of you can still open *your* cards and settle). That recovery logic is a chunk of real work on top of the crypto.

This maps cleanly onto a deterministic engine, by the way: the engine is `fold(state, event) -> state` plus a legality check, and the crypto types are just opaque blobs the engine carries and the *peers* verify. The engine doesn't need to understand ElGamal — it needs to know "this slot is now revealed to seat 3" and refuse illegal transitions.

Want me to go deeper on any one piece — the verifiable-shuffle proof options and their cost/security tradeoffs, the Chaum–Pedersen proof construction, or how the timeout/forfeiture recovery would actually thread through the event log?