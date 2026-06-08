# Mental Poker

> Is there a way to play a distributed card game like no limit holdem where there isn’t a centralized server,
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

> Can you cite references to papers links books etc?

I'll verify these rather than risk giving you stale or wrong links. Let me pull together the real references.That surfaced the canonical lineage. Let me verify the modern protocol papers, the shuffle proofs, and the Rust implementation.Now the modern SNARK-based work, the Rust implementations, and the blockchain poker protocols.Let me get the verifiable-shuffle primitives and the blockchain/financial-fairness poker protocols.Here's the literature, grouped so you can see how the pieces we discussed fit together. I've favored stable links (author-hosted PDFs, IACR ePrint, publisher DOIs).

**The founding papers**

- Shamir, Rivest & Adleman, "Mental Poker" (MIT tech report LCS/TM-125, 1979; reprinted in *The Mathematical Gardner*, 1981). The origin of the problem — they first prove a fair phone-based poker game is impossible, then give a working protocol anyway using commutative encryption. Rivest's copy: https://people.csail.mit.edu/rivest/pubs/SRA81.pdf and the original report at https://dspace.mit.edu/handle/1721.1/148953
- Coppersmith, "Cheating at Mental Poker" (CRYPTO '85). The classic attack: the SRA scheme leaks information when p−1 has a small prime divisor, or via cards encoded with attacker-chosen random-looking bits — the quadratic-residue leak I mentioned. https://doi.org/10.1007/3-540-39799-X_10
- Fortune & Merritt, "Poker Protocols" (CRYPTO '85). https://doi.org/10.1007/3-540-39568-7_36
- Crépeau, "A Secure Poker Protocol that Minimizes the Effect of Player Coalitions" (CRYPTO '85) and "A Zero-Knowledge Poker Protocol that Achieves Confidentiality of the Players' Strategy" (CRYPTO '86) — early treatment of collusion and strategy-hiding.

**The practical protocol we walked through**

- Barnett & Smart, "Mental Poker Revisited" (Cryptography and Coding / IMACC 2003, LNCS 2898, pp. 370–383). The threshold-ElGamal + verifiable-shuffle scheme. It needs a broadcast channel between players, and the bits to represent each card are independent of the number of players. DOI: https://doi.org/10.1007/978-3-540-40974-8_29
- Castellà-Roca, Domingo-Ferrer et al., "Practical Mental Poker Without a TTP Based on Homomorphic Encryption" (Indocrypt 2003), and Castellà-Roca's thesis "Contributions to Mental Poker" (2005) — a parallel practical line.
- Schindelhauer, "A Toolbox for Mental Card Games" (tech report, Univ. Lübeck, 1998) — generalizes beyond poker to arbitrary card games.
- Golle, "Dealing Cards in Poker Games" (ITCC 2005) — poker-specific dealing optimizations.
- Wei & Wang, "A Fast Mental Poker Protocol" (IACR ePrint 2009/439). Provably secure under DDH, and roughly twice as fast as Barnett–Smart and Castellà-Roca on the shuffle. https://eprint.iacr.org/2009/439

**The primitives underneath** (for the proofs in Steps 2/4/5)

- ElGamal, "A Public Key Cryptosystem and a Signature Scheme Based on Discrete Logarithms" (IEEE IT, 1985) — the encryption layer.
- Groth, "A Verifiable Secret Shuffle of Homomorphic Encryptions" (IACR ePrint 2005/246; J. Cryptology 2010) and the later Bayer–Groth shuffle argument — the ZK-shuffle proofs. The Geometry library below uses Bayer–Groth.
- Chaum–Pedersen (proof of equality of two discrete logs) underlies the partial-decryption proofs; Schnorr underlies proof-of-knowledge of secret keys; Fiat–Shamir ("How to Prove Yourself," CRYPTO '86) makes them non-interactive. Neff's "A Verifiable Secret Shuffle and its Application to E-Voting" (CCS 2001) is the other seminal shuffle proof if you want an alternative to Bayer–Groth.

**Settlement, penalties, and the drop-out problem** (the money layer)

This is the line that directly addresses the abort/forfeiture issue I flagged, using blockchain escrow:

- David, Dowsley & Larangeira, "Kaleidoscope: An Efficient Poker Protocol with Payment Distribution and Penalty Enforcement" (FC 2018). Provably secure in a comprehensive model, using blockchain techniques so winners get paid and cheaters are financially penalized. https://eprint.iacr.org/2017/899
- David, Dowsley & Larangeira, "ROYALE: A Framework for Universally Composable Card Games with Financial Rewards and Penalties Enforcement" (FC 2019). Generalizes Kaleidoscope to UC-secure general card games rather than just poker. https://eprint.iacr.org/2018/157
- Same authors, "21 — Bringing Down the Complexity: Fast Composable Protocols for Card Games Without Secret State" (ePrint 2018/303). Relevant to your "stateless" framing: for games with no secret state to maintain, protocols can be built from lightweight primitives like signatures and random-oracle commitments, far outperforming full mental-poker constructions. https://eprint.iacr.org/2018/303
- Bentov et al., "Instantaneous Decentralized Poker" (ASIACRYPT 2017) and Kumaresan et al. (CCS 2015) — earlier blockchain-poker work the above builds on.

**Implementations you can actually read**

- `geometryxyz/mental-poker` (Rust) — the one I mentioned, based on the Barnett–Smart protocol and the Bayer–Groth shuffle argument, built on arkworks. There's a runnable end-to-end example at `barnett-smart-card-protocol/examples/round.rs`. https://github.com/geometryxyz/mental-poker (formerly under `geometryresearch`). Note it's from ~2022 and may be unmaintained, but it's the cleanest Rust reference for the protocol.
- `akonradi/mental-poker` (Rust) — another arkworks/ElGamal `no_std` implementation. https://github.com/akonradi/mental-poker
- zkShuffle (Manta Network) — SNARK-based mental poker aimed at minimizing Ethereum gas; Geometry's implementation uses the Starknet curve and is expensive even ported to BN254, which zkShuffle tries to fix. Writeup: https://hackmd.io/@ZDZ-B3ktQlOiBE4iqOXVlg/BJA7Zoqns
- LibTMCG (Heiko Stamer) — the long-standing C library implementing Schindelhauer's toolbox; ships a curated bibliography PDF that's the best single index to this field: https://www.nongnu.org/libtmcg/

**Best starting tutorials**

- Nicolas Mohnblatt (Geometry), "Mental Poker in the Age of SNARKs," Part 1 and Part 2 — the gentlest accurate path into Barnett–Smart and the SNARK improvements. Part 1: https://nmohnblatt.me/mental-poker-1/ (also on the Geometry notebook). It frames the protocol around verifiable l-out-of-l threshold masking functions plus a zero-knowledge proof of correct shuffle — exactly the two abstractions in the flow above.
- Wikipedia "Mental poker" for the quick orientation and further-reading trail: https://en.wikipedia.org/wiki/Mental_poker

If you want the tightest path from where you are: read the Mohnblatt two-parter, then the Barnett–Smart paper, then clone `geometryxyz/mental-poker` and run the `round.rs` example — that triangle will get you a working mental model and runnable Rust fast. Kaleidoscope is the one to read once you start caring about real-money settlement and penalizing the player who rage-quits before revealing their partial decryption.

Want me to pull the Barnett–Smart PDF and map its notation onto the deal/reveal steps I wrote earlier, so the paper reads more easily?

> Yes

Let me pull the paper.The paper itself is behind Springer's paywall, so let me pull the most faithful open sources — Geometry's implementation README (which mirrors the paper's abstractions directly) and the protocol writeup.I pulled Geometry's writeup, which implements the paper directly and uses Barnett–Smart's own vocabulary, so I can line their terminology up against the steps I gave you earlier. (The original SRA paper, separately, is freely readable at DTIC: https://apps.dtic.mil/dtic/tr/fulltext/u2/a066331.pdf — the Barnett–Smart one stays paywalled, but the abstractions below are theirs.)

The key thing to internalize: Barnett–Smart don't talk about "ElGamal ciphertexts" and "partial decryptions" the way I did. They wrap everything in two named abstractions, and that's the vocabulary you'll meet in the paper and in `round.rs`:

1. a **VTMF** — a *verifiable l-out-of-l threshold masking function* — which bundles the key/encryption machinery, and
2. a **zero-knowledge proof of a correct shuffle**.

A VTMF exposes exactly four operations: key generation, mask, remask, and unmask, and the "discrete-log VTMF" they specify is just threshold ElGamal over an elliptic curve, where each card is represented by two elliptic-curve points — i.e. the `(c1, c2)` pair I was calling a masked card. So the mapping is:

**Key generation** ↔ my Step 0. Each player runs keygen to get `(sk_A, pk_A)`, publishes `pk_A` with a zero-knowledge proof of knowledge of the secret key, and everyone folds these into an **aggregate public key** `pk_Agg`. That's my `x_i`, `h_i = g^{x_i}`, the Schnorr proof, and `h = ∏ h_i`. The paper's `pk_Agg` is my `h`; their aggregate secret key stays unknown to all parties as long as at least one player keeps their secret key hidden. They stress the proof-of-knowledge isn't optional — it's there to stop **rogue-key attacks** (a player choosing `pk` as a function of others' keys to control `pk_Agg`), which I glossed.

**Mask** ↔ my Step 1, with one correction to what I told you. Mask is encryption under `pk_Agg`: plaintext card + randomness → masked card `(c1, c2)`. For the *initial* 52-card deck the paper masks with a public randomness value rather than the `r = 0` "trivial encryption" I described — same effect (everyone can check the masked card corresponds to the known plaintext encoding) but the honest framing is "mask with public, agreed randomness." Their box-with-padlocks analogy is the mental model: the ciphertext is an opaque box, one padlock per player, plus a unique label.

**Remask** ↔ the re-randomization inside my Step 2. Remask takes a masked card and produces a new masked card for the same underlying plaintext without decrypting, so any player can do it — my `(c1·g^r, c2·h^r)`. In the analogy it just swaps the label on the box, which is why a shuffle has to remask: reordering alone does nothing because each ciphertext is uniquely identifiable.

**Unmask** ↔ my Steps 4–6. Unmask is the threshold decryption, and the crucial property is that it can be performed in stages, letting players A, B, C partially unmask so that D alone can finish and privately learn the card. That's exactly my "everyone except seat j publishes their `d_i`, then j applies their own share." Each player's partial contribution — my `d_i = c1^{x_i}` plus its Chaum–Pedersen proof — is what the implementation calls a **reveal token**. Revealing to everyone (the board, showdown) is just a full unmask where all `l` players contribute; revealing a hole card is the staged version that stops one short.

**Shuffle proof** ↔ my Step 2's ZK shuffle. Here's the one place the modern implementation deliberately diverges from the 2003 paper, and it's worth knowing because it changes the security statement: Barnett–Smart's original used a zero-knowledge *proof* of correct shuffle, which was expensive. Geometry replaces the proof with an argument of knowledge — specifically the Bayer–Groth "Efficient Zero-Knowledge Argument for Correctness of a Shuffle" (2012). The distinction matters: a valid proof can never be forged, while a valid argument can be forged by a computationally unbounded adversary, so an argument is only sound against a bounded adversary — the same assumption public-key encryption already rests on, so in practice it's a fine trade for the speed. For scale, their naive 52-card implementation produces a shuffle proof in just over 50ms on a laptop and verifies in under 1ms, but the deck is only fair once every player has taken a shuffle turn.

So if you read `examples/round.rs`, the flow is literally: `key_gen` per player → compute aggregate key → `mask` the 52 encodings → each player `shuffle_and_remask` with a Bayer–Groth proof → produce `reveal_token`s (with proofs) and `unmask` per the dealing rules → final unmask at showdown. That's the same eight-step Texas Hold'em round I walked through, just named in the library's terms.

One honest gap to flag: this entire mapping covers the *card* layer only. The betting in both the paper's and Geometry's round is still "betting using physical chips" — settlement, the drop-out penalty, and the on-chain escrow are explicitly *out of scope* for Barnett–Smart and are exactly what the Kaleidoscope/Royale line adds on top. So in your architecture, VTMF + shuffle-argument fills the crypto payloads in the event log, and the penalty/forfeiture logic is separate work layered over it.

Want me to clone the Geometry repo and trace `round.rs` line by line so you can see which function emits which event in the log schema we sketched?

> I would like a list if possible architectures that would be used to implement this idea using the pkcore library.

Let me look at pkcore's actual structure so these architectures map onto its real shape rather than a guess.Good — I cloned it. The useful discovery is that pkcore already contains most of the substrate you'd need: a `TableLog` event log, a `DealerAction` enum that mixes player actions (`Bet/Call/Check/Raise/AllIn/Fold`) with dealer events (`DealHand/DealFlop/DealTurn/DealRiver/BringItIn/EndHand`), a `PlayerState` legality machine (`can_given`, `can_act_after`), a `PhaseHoldem` street machine, per-card `Visibility::{Down, Up}`, a canonical ordered `DECK_ARRAY`, and both a `TableCelled` (interior-mutable) and a `TableNoCell` (pure) table. That last pair matters: `TableNoCell` + `apply_action` is already shaped like the `fold(state, event) -> state` replicated state machine we discussed.

So the organizing insight for all the architectures below: **pkcore's role never changes — it's the deterministic domain engine.** What differs is where three things live: *coordination* (who orders events and prevents equivocation), *verification* (who checks the crypto proofs), and *settlement* (who holds and enforces the money). The mental-poker crypto (VTMF + shuffle argument) is the same plug-in regardless. Here are the viable shapes, roughly from easiest-to-build to most-trustless.

**1. In-process reference harness (the build-it-first architecture).** All N player engines run in one process; the Geometry `mental-poker` crate provides keygen/mask/remask/unmask/shuffle; transport is just function calls or channels. pkcore's `TableNoCell::apply_action` is the transition function, `DealerAction` is your event payload, and you replace the local `Deck` shuffle with a masked deck. This is essentially `round.rs` wired into pkcore. No networking, no signatures yet — it exists to prove the crypto and the engine compose correctly and to nail the `Card(u32)` ↔ group-element bijection. Every later architecture is this plus a transport and a settlement story.

**2. Full-mesh P2P, no server.** Each peer runs the pkcore engine + crypto + a keypair, and broadcasts signed `DealerAction`s to all other peers over a mesh (libp2p/gossipsub or QUIC). Ordering comes from the turn structure plus hash-chaining each event onto the prior — your `TableLog` becomes the signed, chained log. Every peer independently verifies every shuffle argument and reveal token. Maximally decentralized; no trusted party at all. The pain is operational: NAT traversal, equivocation is only *detectable* not *prevented* (a peer can send conflicting events to different peers), and a dropout during unmask stalls the table. Best for small trusted groups (friends), play-money.

**3. Stateless relay / bulletin board.** Same client engine as (2), but instead of a mesh, a single dumb relay fans out and totally-orders messages. The relay holds *no authoritative game state*, can't read cards (threshold VTMF), and can't forge signatures — it's a message bus, not a dealer. This is the literal "broadcast channel between all players" that Barnett–Smart assumes, and it kills equivocation (one ordering point) and the NAT problem without reintroducing a trusted dealer. This is also where your SSH-served-TUI interest fits cleanly: the SSH host is the relay + a thin renderer, explicitly *not* the source of fairness. Settlement still off-chain/play-money or bonded by social trust.

**4. Semi-trusted coordinator ("trust for liveness, not for fairness").** Here you keep your existing `Dealer`/`PokerSession` orchestrator largely intact and running server-side — it sequences events, runs the betting state machine, presents state, and can verify proofs on clients' behalf — but it is cryptographically prevented from seeing hole cards or rigging the shuffle, because secrecy and shuffle-fairness rest on the threshold VTMF that no single party (including the server) can break. Clients still independently verify. This is the most pragmatic migration from pkcore-as-it-is: you bolt the crypto layer onto the current `Dealer` so it can no longer peek or stack the deck, rather than rewriting the orchestration. You're trusting the coordinator for uptime and honest ordering, not for the integrity of the cards.

**5. State channel with on-chain settlement (off-chain play, chain only for money/disputes).** Play runs at full speed off-chain exactly like (3) or (4). The blockchain is touched only to escrow stakes when the channel opens, to settle at close, and — crucially — to *adjudicate*: if a player disputes or drops out before revealing, the signed `TableLog` is replayed on-chain (or against a verifier contract) to assign fault and slash the offender's bond. This is the architecture that solves the two things mental poker alone doesn't: real-money settlement and the dropout/penalty problem. It's the Kaleidoscope/Royale design pattern. Cost is contained because you only pay gas on open/close/dispute, not per action.

**6. Fully on-chain coordinated.** The smart contract *is* the broadcast channel and the verifier: it stores every event, verifies each shuffle/reveal proof, holds escrow, and enforces payout and penalties directly. Maximally trustless, but you pay gas per action and need EVM-friendly choices — a SNARK shuffle (zkShuffle on BN254) rather than Bayer–Groth on an arbitrary curve, and either a WASM build of pkcore's evaluator for client/verifier use or settling only disputed showdowns on-chain to avoid reimplementing Cactus-Kev evaluation in Solidity. Heaviest option; rarely worth it over (5) unless you need full public auditability with no liveness assumptions.

Three cross-cutting changes to pkcore that every option above needs, regardless of which you pick:

The dealing path is the main refactor. Today `Deck` holds concrete `Card`s and dealing reveals them; in a mental-poker design the deck is a vector of masked cards, and a `Card(u32)` only materializes after the unmask protocol completes for that slot. So `Dealer`'s `DealHand/DealFlop/...` stop drawing from a local shuffled `Deck` and instead trigger reveal-token collection, with the resulting plaintext fed back into the engine. Your `DECK_ARRAY` is perfect as the canonical, public, agreed ordering that defines the fixed bijection between the 52 `Card` values and the 52 group elements each table uses.

`Visibility` wants a richer notion than `Down`/`Up`. The protocol has three states, not two: masked (no one knows), known-to-owner (unmasked to seat j only — your hole cards mid-hand), and public (`Up`, fully unmasked — the board or a showdown card). The cleanest move is to keep `Card` visibility-free as the comment in `visibility.rs` intends, and track the mask/reveal status in a parallel structure keyed by deck slot, letting `Visibility::Up` mean "plaintext now lives in the engine."

And the engine should stay pure and crypto-agnostic. `TableNoCell::apply_action` + `PlayerState` legality is the deterministic core every peer replays; it should treat shuffle proofs and reveal tokens as opaque blobs carried on the event and verified *outside* the transition function. That keeps your evaluation/analysis code (the `arrays/*`, `lookups/*`, `analysis` modules) completely untouched by the cryptography.

If you're building toward something real, the sequence that minimizes risk is 1 → 3 → 5: prove the crypto in-process against pkcore, get a playable serverless table over a dumb relay, then add the state-channel settlement once you want stakes to matter. Options 4 and 6 are branches off that spine depending on whether you prioritize a smooth migration from your current `Dealer` (4) or maximal trustlessness (6).

Want me to draft the concrete trait boundary — something like a `CardCrypto` trait wrapping the VTMF operations and a `Coordinator` trait for the transport — so the engine, the crypto, and the chosen architecture stay swappable behind interfaces?

> Sure

The seam principle that makes everything swappable: **pkcore's engine only ever sees plaintext `Card`s and `DealerAction`s. All crypto is verified at the node boundary and never crosses into the transition function.** So you get two traits — one for the card crypto, one for transport/ordering — and pkcore stays untouched behind them.

**The crypto trait.** This wraps the Barnett–Smart VTMF (`keygen/mask/remask/unmask`) plus the shuffle argument. Holding the group params in `self` keeps the signatures clean.

```rust
use rand::RngCore;
use pkcore::card::Card;

/// The mental-poker layer: a verifiable l-out-of-l threshold masking scheme
/// (Barnett–Smart VTMF) plus a zero-knowledge shuffle argument.
///
/// Swap implementations to change security/perf, not the engine:
///   - `BayerGrothCrypto` — arkworks/Geometry, ZK *argument* of shuffle
///   - `SnarkCrypto`      — BN254 + a SNARK shuffle for on-chain verification
///   - `PlaintextCrypto`  — no-op mock: cards in the clear, for testing arch #1
pub trait CardCrypto {
    type SecretKey;
    type PublicKey: Clone;
    type AggregateKey: Clone;
    type MaskedCard: Clone + Eq;          // an ElGamal ciphertext: two curve points
    type RevealToken: Clone;              // one player's partial unmask + its proof
    type KeyProof;
    type MaskProof;                       // covers mask and remask
    type ShuffleProof;
    type Error: std::error::Error;

    // --- Step 0: key setup ---
    fn keygen(&self, rng: &mut impl RngCore)
        -> (Self::SecretKey, Self::PublicKey, Self::KeyProof);
    fn verify_key(&self, pk: &Self::PublicKey, proof: &Self::KeyProof)
        -> Result<(), Self::Error>;
    fn aggregate(&self, pks: &[Self::PublicKey]) -> Self::AggregateKey;

    // --- Card <-> group-element bijection, fixed by pkcore's DECK_ARRAY order ---
    fn encode(&self, card: Card) -> Self::MaskedCard;                 // plaintext encoding
    fn decode(&self, unmasked: &Self::MaskedCard) -> Result<Card, Self::Error>;

    // --- Steps 1–2: mask / remask ---
    fn mask(&self, agg: &Self::AggregateKey, m: &Self::MaskedCard, rng: &mut impl RngCore)
        -> (Self::MaskedCard, Self::MaskProof);
    fn remask(&self, agg: &Self::AggregateKey, c: &Self::MaskedCard, rng: &mut impl RngCore)
        -> (Self::MaskedCard, Self::MaskProof);
    fn verify_mask(&self, agg: &Self::AggregateKey,
                   input: &Self::MaskedCard, output: &Self::MaskedCard,
                   proof: &Self::MaskProof) -> Result<(), Self::Error>;

    // --- Step 2: verifiable shuffle of the whole deck ---
    fn shuffle(&self, agg: &Self::AggregateKey, deck: &[Self::MaskedCard],
               rng: &mut impl RngCore) -> (Vec<Self::MaskedCard>, Self::ShuffleProof);
    fn verify_shuffle(&self, agg: &Self::AggregateKey,
                      input: &[Self::MaskedCard], output: &[Self::MaskedCard],
                      proof: &Self::ShuffleProof) -> Result<(), Self::Error>;

    // --- Steps 4–6: staged unmask ---
    fn reveal_token(&self, sk: &Self::SecretKey, pk: &Self::PublicKey,
                    c: &Self::MaskedCard, rng: &mut impl RngCore) -> Self::RevealToken;
    fn verify_reveal_token(&self, pk: &Self::PublicKey, c: &Self::MaskedCard,
                           t: &Self::RevealToken) -> Result<(), Self::Error>;
    /// Apply reveal tokens. A *subset* leaves a still-locked card (reveal-to-one);
    /// the *full* set yields a plaintext that `decode` accepts.
    fn unmask(&self, c: &Self::MaskedCard, tokens: &[Self::RevealToken])
        -> Result<Self::MaskedCard, Self::Error>;
}
```

The payoff of this trait is architecture #1 from before: a `PlaintextCrypto` whose `MaskedCard = Card`, where `mask`/`remask`/`shuffle` are identity-plus-`Vec::shuffle` and all proofs are `()`. You wire the entire game loop against it first, prove the pkcore integration works, then swap in `BayerGrothCrypto` with zero changes to the engine or transport.

**The transport trait.** This is the *only* thing that differs across architectures 2–6. Each becomes one impl.

```rust
pub type Hash = [u8; 32];

pub trait Coordinator {
    type Error: std::error::Error;

    /// Append a signed event to the shared, totally-ordered log. Returns once
    /// the event is durably ordered for everyone (gossip-acked / relayed /
    /// chain-confirmed / pushed to the in-proc queue).
    async fn publish(&mut self, event: SignedEvent) -> Result<(), Self::Error>;

    /// Next event in canonical order. Drives every peer's identical replay.
    async fn next_event(&mut self) -> Result<SignedEvent, Self::Error>;

    /// Hash of the current log head, for chaining the next event's `prev_hash`.
    async fn head(&self) -> Result<Hash, Self::Error>;
}
```

The mapping to your architecture list is direct: `MeshCoordinator` (libp2p gossipsub) is #2; `RelayCoordinator` (one socket to a dumb bulletin board) is #3; `CoordinatorServer` that also runs validation is #4; `ChannelCoordinator` (off-chain + on-chain adjudication) is #5; `ContractCoordinator` (chain log subscription) is #6; and `InProcCoordinator` (a `tokio::mpsc` everyone shares) is #1. `async fn` in traits is fine on current stable; for the in-process one the bodies are just channel sends that never actually await.

**The glue.** The event payload extends your `DealerAction` with the crypto messages, parameterized over the scheme so the masked-card and proof types stay abstract:

```rust
use pkcore::casino::dealer::DealerAction;

pub enum RevealTarget { ToAll, ToSeat(u8) }

pub enum EventPayload<C: CardCrypto> {
    KeyShare  { pk: C::PublicKey, proof: C::KeyProof },
    DeckInit  { deck: Vec<C::MaskedCard>, proofs: Vec<C::MaskProof> },
    Shuffle   { deck: Vec<C::MaskedCard>, proof: C::ShuffleProof },
    Reveal    { slot: u8, target: RevealTarget, token: C::RevealToken },
    Action(DealerAction),   // your existing betting + dealer-phase enum
}

pub struct SignedEvent { /* table_id, hand_id, seq, prev_hash, author, payload bytes, sig */ }
```

And the node that ties the three together. Note where the boundary falls — `verify_*` runs here; the engine call below it only ever receives a plaintext `Card` or a `DealerAction`:

```rust
pub struct GameNode<C: CardCrypto, T: Coordinator> {
    crypto: C,
    transport: T,
    seat: u8,
    sk: C::SecretKey,
    agg: C::AggregateKey,
    masked_deck: Vec<C::MaskedCard>,
    pending: std::collections::HashMap<u8, Vec<C::RevealToken>>, // slot -> tokens
    engine: pkcore::casino::table_no_cell::TableNoCell,          // pure pkcore state
}

impl<C: CardCrypto, T: Coordinator> GameNode<C, T> {
    /// Verify at the boundary, then feed only plaintext into pkcore.
    pub fn apply(&mut self, ev: &EventPayload<C>) -> Result<(), C::Error> {
        match ev {
            EventPayload::Shuffle { deck, proof } => {
                self.crypto.verify_shuffle(&self.agg, &self.masked_deck, deck, proof)?;
                self.masked_deck = deck.clone();
            }
            EventPayload::Reveal { slot, target, token } => {
                let c = &self.masked_deck[*slot as usize];
                self.crypto.verify_reveal_token(/* author pk */ todo!(), c, token)?;
                let acc = self.pending.entry(*slot).or_default();
                acc.push(token.clone());
                // once enough tokens exist for this slot's target, finish + decode
                if ready(*slot, target, acc.len()) {
                    let plain = self.crypto.unmask(c, acc)?;
                    let card: Card = self.crypto.decode(&plain)?;     // <-- only plaintext past here
                    self.engine.reveal_slot(*slot, card);             // sets Visibility::Up
                }
            }
            EventPayload::Action(a) => {
                // pkcore's PlayerState legality runs inside apply_action; crypto never enters.
                self.engine.apply_action(*a).map_err(/* ... */ todo!())?;
            }
            EventPayload::KeyShare { .. } | EventPayload::DeckInit { .. } => { /* setup */ }
        }
        Ok(())
    }
}
```

That `apply` is the whole architecture in miniature: signature + chain checks (in the `SignedEvent` wrapper), then `crypto.verify_*` at the boundary, then a plaintext `Card` or `DealerAction` handed to the engine — which is exactly your existing `TableNoCell` with its `PlayerState` legality intact. Swapping `C` changes the security/curve; swapping `T` changes the deployment topology (mesh vs relay vs chain); neither touches the engine, and your `arrays/*`, `lookups/*`, and `analysis` modules never see a ciphertext.

The two `todo!()`s flag the real remaining work: threading the *author's* public key through to `verify_reveal_token` (the envelope carries `author`, so it's a lookup into the per-seat key map), and the `ready()` policy — `ToSeat(j)` is ready at `n−1` tokens (everyone but j), `ToAll` at `n`. That `ready` function plus a timeout is also where the dropout/forfeiture logic from architecture #5 hooks in.

Want me to package this as a starter `pkcore-mp` crate skeleton — the two traits, the `PlaintextCrypto` and `InProcCoordinator` mock impls, and a runnable two-seat test that deals and reveals — so you have something that compiles against pkcore to build on?

> Sure

This one I'll actually build and test so you get something that compiles. Let me check pkcore's manifest first.No Rust here, and pkcore needs a very recent toolchain (1.94.1, edition 2024) that I can't pull in this sandbox — rustup's host isn't reachable. Let me get whatever `cargo` apt offers so I can at least compile and *run* the crate logic against a small `Card` stub, then hand you the version wired to real pkcore.cargo 1.75 it is — too old for real pkcore (1.94.1), but enough to *run* the logic against a small `Card` stub so the trait design is verified, not just asserted. I'll wire the real pkcore dependency behind a feature flag so flipping it on is a one-liner on your toolchain. Building now.Even as an optional dep, cargo 1.75 parses pkcore's edition-2024 manifest and refuses. I'll comment the dep out to verify the logic here, then restore it in what I hand you (your 1.94.1 toolchain handles it fine).Both tests pass. Let me capture the actual dealt output and restore the real pkcore dependency for delivery.Done — a `pkcore-mp` crate skeleton that compiles and whose two-seat round actually runs:

```
seat0 hole: 2h 6s
seat1 hole: 7d Th
board: 5s Js 7s Qd Jd
test two_seat_deal_and_reveal ... ok
test coordinator_orders_and_chains_events ... ok
```

Reconstruct the layout under `pkcore-mp/` as `src/lib.rs`, `src/card.rs`, `tests/round.rs`, plus `Cargo.toml` and `README.md`.

What's in there and what I verified:

`CardCrypto` and `Coordinator` are the two traits from the last message, made concrete. `PlaintextCrypto` is the mock backend — cards in the clear, but it models the *l*-out-of-*l* padlock accounting faithfully: a card carries a set of seat "padlocks," each `reveal_token` removes one, and `decode` only succeeds when the set is empty. That's what makes the test meaningful rather than trivial — the `two_seat_deal_and_reveal` test asserts that with only the *other* seat's token a hole card is still `StillMasked`, and only the recipient adding their own token unlocks it. It also checks all dealt + board cards are distinct (deck integrity through the shuffle). `InProcCoordinator` is architecture #1: one shared append-only log with per-reader cursors, and the second test confirms ordering and that `prev_hash` advances between publishes.

The honest caveat, spelled out in the README: I verified this on rustc 1.75 (what apt gave me) against a local `Card` stub, because real pkcore needs Rust ≥ 1.94.1 / edition 2024 and this sandbox can't reach rustup to install it. The real dependency is wired and ready — `src/card.rs` re-exports `pkcore::card::Card` and `pkcore::deck::DECK_ARRAY` behind the `pkcore` feature, with a matching stub otherwise, and the rest of the crate never branches on which one is active. On your machine:

```bash
cargo test                    # local stub, any recent rustc
cargo test --features pkcore  # real pkcore::card::Card
```

One thing to know going in: cargo parses an optional dependency's manifest even when the feature is off, so on a pre-1.94 toolchain the mere presence of the `pkcore` line errors on edition 2024. On 1.94.1 that's a non-issue; I only hit it because of the sandbox's old cargo.

The crate is deliberately the *crypto + coordination* layer only — it stops at handing a plaintext `Card` to the boundary. The README's "next steps" list the four pieces that turn it into a game: a real arkworks/Bayer–Groth `CardCrypto`, the `DealerAction` + `TableNoCell` engine glue, an async signed transport, and the timeout/forfeiture path for the dropout problem.

If it'd help, I can sketch the arkworks `CardCrypto` impl next — the ElGamal `MaskedCard` as two curve points, and `reveal_token` as a Chaum–Pedersen-proved partial decryption — so you can see exactly which `()` proof types become real and where the RNG threads through.

[mentalpoker files](files/mentalpoker)

> Is there a way to use QR codes to hold state at any given time that players can scan to get up to date?

Yes, but the useful version of "yes" depends hard on *which* state you mean, because a QR code's capacity is small and fixed by the spec, while your cryptographic state is large and growing. So the answer splits into "what fits" and "what doesn't."

The hard ceiling: a single QR code maxes out at Version 40 (177×177 modules), which holds about 2,953 bytes of binary at the lowest error-correction level — and far less if you want it scannable by a phone camera across a table, where you realistically stay under ~1–1.5 KB. Compare that to what your state actually weighs:

- Public engine state (phase, pot, the up-to-5 board `Card`s, per-seat stack/bet/`PlayerState` for 9 seats): well under ~300 bytes serialized with postcard — which pkcore already uses. Fits trivially, even at high error correction.
- A single reveal token + its Chaum–Pedersen proof + a signature: ~150–200 bytes. One QR.
- A head-hash + sequence number + relay address + signature (a *sync pointer*): ~100 bytes. One QR.
- The full masked deck (52 cards × two curve points ≈ 64 bytes): ~3.3 KB. Over the single-code limit.
- A Bayer–Groth shuffle proof: on the order of kilobytes. Over the limit.

So the masked deck and the proofs — the heavy one-time setup at the start of a hand — do not fit in a static QR. Everything else does. That gives you two clean, legitimate roles for QR in the architecture:

**As a sync beacon / commitment.** A "table" QR encodes the current `Coordinator::head()` hash, the sequence number, a signature, and a bootstrap address. Players scan it to confirm they're on the same chain head and to find where to pull the actual log from. This is exactly how hardware wallets and Matrix/WhatsApp pairing use QR — it carries keys and pointers, not bulk data. It answers "am I up to date?" and "where do I sync from?", not "give me the whole state." It maps directly onto the `head()` method already in the `Coordinator` trait.

**As the actual transport, when you want no network at all.** This is the interesting one for an offline/in-the-room table. The small per-turn deltas — a bet, a fold, one reveal token — each fit in one QR, so a player whose turn it is displays a QR of their signed event and the others scan it. For the big objects (the initial masked deck and shuffle proofs) you use *animated / fountain-coded* QR: split the blob across a rotating sequence of frames encoded with a fountain code (RaptorQ / Luby transform) so a scanner reconstructs the whole thing from any sufficient subset of frames it happens to catch. This is not exotic — it's exactly how air-gapped Bitcoin hardware wallets move multi-kilobyte signed transactions across QR (the BC-UR "Uniform Resources" standard). Slower than a socket, but it genuinely needs zero connectivity, which fits the constrained/offline direction.

There's an elegant bonus here that's worth calling out: a QR shown to the whole table at once is a *physical broadcast*. That directly attacks the equivocation problem I flagged earlier — a player can't easily show one event to you and a conflicting event to someone else when everyone is photographing the same screen. The "broadcast channel between all players" that Barnett–Smart assumes is, at a physical table, literally a screen everyone can see.

The honest limit to keep in view: QR is only a display format for bytes — it carries no trust of its own. A QR-delivered event is exactly as trustworthy as the same signed, chained bytes over a network: every recipient still verifies the signature and that it links onto the known head. And because the log only grows, you can never QR the *whole* history — which is fine, since state is the deterministic fold over the log. What you'd QR for a late-joiner or a resync is a signed *checkpoint*: `postcard::to_allocvec(&public_state)` plus the head hash. They re-anchor the public betting state from that, and pull only the masked-card data they're missing from a peer.

If you want to prototype it, the Rust path is small: `qrcode` for generation, `rqrr` or `bardecoder` for scanning, `raptorq` for the fountain chunking, and postcard (already in your stack) for compact encoding — and you could add it to `pkcore-mp` as a `QrCoordinator` impl alongside `InProcCoordinator`, where `publish` renders frames and `next_event` decodes scanned ones. Worth checking those crates' current maintenance before committing, as that part of the ecosystem moves.

Want me to sketch the `QrCoordinator` — the checkpoint/delta split, and where the fountain-coded animated frames kick in for the deck setup?

> I want to explore alternative architectures to add support for cardcames other than poker. We currently have a go fish library called gfcore. Want to support games like bridge, spades etc.

Let me look at how gfcore is structured relative to pkcore — the right factoring depends on what's already shared versus duplicated between them.Cloning both was worth it — it shows exactly where you are. `cardpack` is already your shared substrate (gfcore on 0.7.0, pkcore on 0.6.9 — worth aligning those), and that's the *only* thing the two cores share. Above cardpack, pkcore and gfcore have independently reinvented the same four things under different names: an event log + replay (`hand_history`/`TableLog` vs `history/replay`), a bot harness (`bot/` in both), a hidden-information projection (`Visibility` vs `player/view`), and a rules state machine (`casino`/`play` vs `game` + a pluggable `rules/{standard,quartet,happy_families}`). That duplication is the thing your question is really about: keep federating, or extract a shared engine.

The reframe that makes this tractable: **don't try to unify on a "Game" object.** Poker (betting + showdown), Go Fish (set collection), and bridge/spades (trick-taking) have no honest common shape at that level — forcing Go Fish through a poker interface, or vice versa, produces a leaky abstraction. Unify one level lower, on two things they genuinely all share:

A small **algebra of card operations over zones** — `shuffle`, `deal(card, from_zone → to_zone)`, `move(card, zone → zone)` (play to trick, discard, pass), `reveal(zone/card, to: audience)`, `peek(card, seat)` — and a **zone + visibility model** attached to it (a zone is private-to-seat, public, hidden-to-all, or revealed-to-a-subset). Every game you named decomposes into these. Your `Visibility::{Down,Up}` and gfcore's `player::view` are both special cases of that visibility model, which is the highest-value abstraction to extract because it's also exactly what the mental-poker layer operates on (masking = hidden-to-all, reveal-to-one = private-to-seat, reveal-to-all = public). The game-specific part is then only two things: *which operations are legal when* (the rules state machine) and *how a finished hand scores*.

With that lens, here are the architectures, roughly from least to most shared:

1. **Federated cores (status quo, extended).** `bridgecore`, `spadescore`, etc., each its own crate on `cardpack`, each with its own engine. Pro: every domain model stays idiomatic, zero abstraction tax. Con: you reimplement event-log/replay, bots, view, and — critically — the entire distributed/crypto layer per game. Fine if the games stay isolated and you never want shared infra. Given you've already built two cores this way, this is the path of least change but maximum future duplication.

2. **Shared engine over a `GameRules` trait.** Extract one crate (`tablecore`?) holding the deterministic `fold(state, action) -> state` machine, the zone/visibility model, the event log + replay, the bot harness, and serialization — parameterized over a trait each game implements:

   ```rust
   trait GameRules {
       type State; type Action; type View; type Outcome;
       fn deal(&self, deck: Deck, seats: &Seats) -> Self::State;
       fn legal_actions(&self, s: &Self::State, seat: Seat) -> Vec<Self::Action>;
       fn apply(&self, s: Self::State, a: Self::Action) -> Result<Self::State>;
       fn view_for(&self, s: &Self::State, seat: Seat) -> Self::View;  // the hidden-info projection
       fn outcome(&self, s: &Self::State) -> Option<Self::Outcome>;
   }
   ```

   pkcore and gfcore each implement it; the infra is written once. The evidence this fits *you* specifically: gfcore already treats rules as pluggable (`standard`/`quartet`/`happy_families`), so the trait-per-ruleset pattern is one you've validated. `view_for` is the subtle method — it's where each game says what a seat is allowed to see — and it's the seam the crypto layer plugs into.

3. **Family mid-layers on top of (2).** Below the generic engine but above specific games, add crates where reuse is real *within a family*. The big win is `tricktaking`, shared by bridge, spades, hearts, euchre, whist: it owns trick state, follow-suit legality, trump-aware trick resolution, and lead rotation — about 70–80% of those games — exposing hooks for the two parts that differ, *bidding* and *scoring*. Bridge fills in a full auction + contract scoring + vulnerability; spades fills in trick-count bidding + bags + nil; hearts fills in the passing phase + point avoidance + shooting the moon. A separate `betting` mid-layer backs the poker variants (pkcore), and a `collection` one backs Go Fish/Rummy (gfcore). This is the architecture I'd actually aim for: `cardpack` → generic engine (2) → family layer (3) → specific game.

4. **Data-driven rule specs (a DSL).** Express the regular parts of a game — deck, deal pattern, phases, follow-suit, trick-winner — as data interpreted by one engine, so adding a *variant* is writing a spec rather than code. Powerful for the families' many minor variants, but it breaks down on the irregular logic (bridge scoring, conventions, shoot-the-moon) where you'll need code escape hatches. Best used as a thin declarative layer over (2)/(3) for the regular bits, not as the whole system.

I'd combine 2 and 3, keep cardpack as the universal layer, and refactor pkcore/gfcore to implement the shared `GameRules` rather than carry their own engines — pulling their duplicated history/bot/view code up into the shared crate as you go.

The payoff for the distributed work we built: because the crypto layer only ever touches the operations algebra and the zone/visibility model — never poker rules — `pkcore-mp` generalizes to `cardgame-mp` by parameterizing over that model instead of over poker. `CardCrypto`'s `mask`/`reveal`/`shuffle` already speak in "hidden / revealed-to-seat / public," which is game-agnostic. This isn't speculative: the Royale paper is precisely the generalization of the poker-specific Kaleidoscope protocol to UC-secure general card games built from a set of core card operations — the same move at the protocol level that the operations algebra is at the engine level.

The hard parts to scope before committing:

Bidding/auction phases don't exist in poker or Go Fish, so your current phase machines (`PhaseHoldem`) won't transfer — the trick-taking family needs a distinct phase shape (auction → play → score), and bridge's auction is genuinely complex. Partnerships/teams (bridge, spades are 2v2) add a layer your seat model probably assumes away today. Scoring is where the per-game code concentrates and resists abstraction, so design the `Outcome` type to be game-specific rather than trying to unify it. Decks vary — euchre is 24 cards, pinochle a 48-card double deck — so the engine must take a `DeckSpec` rather than assume 52; cardpack handles this, but your dealing code shouldn't hardcode `DECK_ARRAY`. And align the cardpack versions first, since a shared engine can't depend on two of them.

Want me to sketch the `tricktaking` crate's core — the trick/follow-suit/trump-resolution types plus the bidding and scoring hooks — and show bridge and spades as two thin implementations of it?

> Sure

I'll build it so the trick logic and the bridge/spades scoring are verified, not just asserted — same approach as before, with a local card model so it compiles on this toolchain (swap in `cardpack` types on yours).Name clash — the `Spades` game struct and the `Spades` suit variant collide under glob import. Aliasing the game type fixes it.All seven pass. Reconstruct under `tricktaking/`: `src/lib.rs`, `src/card.rs`, `src/bridge.rs`, `src/spades.rs`, `tests/tricktaking.rs`, `Cargo.toml`.The crate compiles and all seven tests pass:

```
test highest_of_led_suit_wins_no_trump ... ok
test trump_beats_higher_led_card ... ok
test highest_trump_wins_when_several_ruff ... ok
test must_follow_the_led_suit ... ok
test cannot_lead_spades_until_broken ... ok
test bridge_scores_match_known_contracts ... ok
test spades_scores_bids_bags_and_nil ... ok
```

How the split lands: the shared core in `lib.rs` owns everything identical across trick-taking games — `trick_value`/`trick_winner` (trump beats led suit beats off-suit, rank within a category), `must_follow`, `resolve_trick`, and `legal_plays` (follow-suit intersected with per-game constraints). A game implements `TrickTaking` and supplies only three things: `trump` (how a contract maps to the trump suit), `can_play` (extra legality beyond follow-suit), and `score`.

That's what makes bridge and spades genuinely thin. `bridge.rs` is mostly a `Strain → Trump` match plus undoubled contract scoring — verified against known results: 4♠ making = 420, 2♣ part-score = 90, 3NT = 400, 4♥ down one = −50. `spades.rs` is a one-line `trump` (always Spades), a `can_play` that reads `completed` tricks to enforce spades-not-broken, and partnership bid/bag/nil scoring — bid 4 take 5 = 41, set = −40, nil made = +130, nil broken = −70. Neither game touches trick resolution or follow-suit; they inherit it.

The design choices worth flagging. The auction is deliberately *not* modeled — a game produces a `Contract` (bridge) or `[Bid; 4]` (spades) and the core takes it from there; that's honest because bidding, especially bridge's, is where the real per-game volume lives and it doesn't share much. `PlayState` stays game-agnostic by keeping `completed: Vec<Trick>` so games derive their own flags (spades-broken) rather than the core carrying game-specific state. And `Score` is an associated type precisely so it *doesn't* unify — bridge returns an `i32` for the declaring side, spades returns `[i32; 2]` per partnership. Hearts would slot in the same way: `Trump::NoTrump`, a `can_play` for its first-trick and hearts-not-broken rules, and avoidance scoring with shoot-the-moon.

Two things to do on your side: swap the local `card` module for `cardpack`'s `Rank`/`Suit`/`Card` (uncomment the dep, map the types — the only requirement the core makes is Ace-high ordering), and note the `Spades` struct vs `Suit::Spades` name clash I hit in the test — worth a module convention (e.g. game structs as `spades::Game`) so downstream glob-imports don't collide.

This `tricktaking` crate sits at family-layer (#3) above the generic engine: `legal_plays` and `resolve_trick` are the trick-taking realization of the operations algebra, and the same `view_for`/zone model feeds the crypto layer when you want bridge or spades played without a trusted dealer.

If you want to keep going, I could wire one game end-to-end through a full hand — deal 13 each, run an auction stub, play all tricks via `legal_plays`/`resolve_trick`, and score — so you can see the `PlayState` loop in motion, or show how `TrickTaking` plugs into the generic `GameRules` engine from the previous message.

> Sure

Both at once, then verified end to end. First two small additions: `ALL` arrays for dealing, and the generic `GameRules` engine the trick layer plugs into.All nine tests pass, and the full hand runs end to end through the generic engine:

```
tricks won by seat: [0, 0, 0, 13]
spades score [team {0,2}, team {1,3}]: [-60, 76]
```

(That `[0,0,0,13]` is the degenerate suit-major deal — seat 3 gets all 13 spades and ruffs everything; team {1,3} bid 7, took 13 → 70 + 6 bags = 76, team {0,2} set −60. The scoring is the same `score` you saw verified earlier, now reached by playing rather than by hand.)

Here are the new and updated files — `src/engine.rs` and `tests/full_hand.rs` are new; `src/lib.rs` and `src/card.rs` gained a module declaration and the `ALL` arrays.What this demonstrates, both at once:

The full-hand loop is in `engine.rs`'s `run` — and the point is what it *doesn't* contain. It only asks "whose turn? what are the legal actions? apply one," then repeats until `to_act` returns `None`. No mention of tricks, trump, suits, or scoring. That's the generic engine: it would drive poker or Go Fish identically given their own `GameRules` impls. The trick-taking knowledge lives entirely behind `TrickPlay`, the adapter that lifts any `TrickTaking` game into `GameRules`.

So the layering is now concrete: `card` (substrate) → `TrickTaking` + `bridge`/`spades` (family rules) → `TrickPlay` adapter → `GameRules` (generic engine) → `run` (driver). Spades reaches the generic engine without a single spades-specific line in `engine.rs`; its trump rule, its spades-not-broken constraint, and its bid/bag/nil scoring all arrive through the trait.

Two things in the adapter are worth calling out because they're the seams to the rest of the system. `TrickState`'s `apply` re-checks both turn order and legality before mutating — that's the same defensive boundary the distributed engine needs, since in a P2P setting `apply` runs on bytes that arrived from an untrusted peer. And `view_for` is the hidden-information projection made literal: it hands seat 0 its own 13 cards but only the *sizes* of the others' hands. The test asserts exactly that (`opponent_hand_sizes == [13,13,13,13]`). That method is the hook the mental-poker layer fills — "an opponent's hand is hidden" becomes "an opponent's hand is a vector of masked cards," and `view_for` is where a `decode`d card would surface once revealed. The plaintext engine and the cryptographic one implement the same projection; only the representation of "hidden" differs.

The honest gaps remain the auction and turn-1 specifics: `TrickPlay` starts from a finished `Contract`, so the bridge/spades bidding phase is still a separate sub-machine you'd compose before the play phase (a `GameRules` whose `Outcome` is the `Contract` that seeds this one), and hearts' first-trick rules and dummy exposure in bridge are `can_play`/`view_for` refinements rather than new machinery.

A natural next step in either direction: compose the auction as its own `GameRules` phase feeding `TrickPlay` (so a full game is auction-engine → play-engine), or swap the local `card` module for `cardpack` and align the versions so `tricktaking`, `pkcore`, and `gfcore` all sit on one card substrate. Say which and I'll build it.

[github.com/ImperialBower/tricktaking](https://github.com/ImperialBower/tricktaking)
