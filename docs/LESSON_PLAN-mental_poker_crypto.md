# Learning the Cryptography Behind Mental Poker

A self-study plan for a working programmer with little cryptography background.
The destination is concrete: understand every primitive used in the
Barnett–Smart protocol well enough to implement a real `CardCrypto` backend —
ElGamal masking, threshold decryption with Chaum–Pedersen proofs, and a
verifiable shuffle — and to know *why* each piece is there.

**Design of the plan.** Each module has a goal, the core ideas, reading, and a
hands-on Rust exercise. The exercises build one artifact across the whole plan:
a toy (insecure, small-number) implementation of the full protocol, which you
then swap piece-by-piece for a real curve library. Building the insecure
version first is the point — the math is visible when the numbers are small.

**Pace.** Each module is roughly a week of evenings, but they're checkpoints,
not a schedule. Modules 1–4 are the foundation and shouldn't be rushed;
modules 7–8 can be skimmed on first pass and revisited.

**Primary texts** (all free):

- Boneh & Shoup, *A Graduate Course in Applied Cryptography* —
  https://toc.cryptobook.us/ (the reference; dip in per-topic, don't read linearly)
- Thaler, *Proofs, Arguments, and Zero-Knowledge* —
  https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.html (for modules 5–8)
- Mohnblatt, *Mental Poker in the Age of SNARKs*, parts 1–2 (the map of the
  destination; read part 1 now, understand it fully by module 6)
- Dan Boneh's *Cryptography I* (Coursera, free to audit) — optional lecture
  companion for modules 1–3

---

## Module 0 — Orientation (one evening)

Read the Shamir–Rivest–Adleman "Mental Poker" paper (1979). It's short,
readable, and needs no math you don't have: they prove a fair deal is
"impossible," then do it anyway with locked boxes. Don't study it — just absorb
the problem statement and the box-and-padlock intuition. Everything that
follows is machinery to make those padlocks real and *verifiable*.

Write down, in your own words, the four things a mental poker deal must
guarantee (secrecy, uniqueness, verifiable shuffle, selective reveal). You'll
grade every primitive against this list.

## Module 1 — Modular arithmetic and finite groups

**Goal:** be comfortable computing in ℤ/pℤ and know what a cyclic group, a
generator, and a group order are.

**Core ideas:** modular add/multiply/exponentiate; Fermat's little theorem;
multiplicative groups mod p; cyclic groups and generators; subgroups and
Lagrange's theorem (order divides group order); why we work in a *prime-order*
subgroup. The mantra to internalize: exponents live mod q (the group order),
elements live mod p.

**Reading:** Boneh–Shoup appendix on number theory; or any discrete-math
refresher. Supplement: search "cyclic group generator tutorial" — this level is
covered everywhere.

**Exercise:** in Rust, with `u128` arithmetic (no crypto crates), implement
`mod_exp` (square-and-multiply), find a generator of the multiplicative group
mod a small prime (p = 467 works), and verify its powers cycle through all
elements. Then find a prime-order subgroup (p = 2q+1 with q prime — a "safe
prime") and confirm every element of the subgroup has order q.

**Checkpoint:** you can explain why `g^x mod p` for random x lands anywhere in
the subgroup with equal probability.

## Module 2 — The discrete logarithm problem and Diffie–Hellman

**Goal:** understand the hardness assumption the entire stack rests on, and the
first protocol built on it.

**Core ideas:** the discrete log problem (given g and g^x, find x); why
exponentiation is easy but the inverse is believed hard; Diffie–Hellman key
exchange; the CDH and DDH assumptions (DDH — "you can't even *recognize* g^xy"
— is the one ElGamal needs); brute force vs. baby-step giant-step, so you feel
why group size matters.

**Reading:** Boneh–Shoup ch. 10 (key exchange) and the DDH discussion. The
Wikipedia articles on DLP and DDH are genuinely adequate here.

**Exercise:** implement Diffie–Hellman with your Module 1 code. Then implement
baby-step giant-step and *break* your own exchange at p ≈ 2^32. Measure how the
attack scales; extrapolate to 2^256. This one exercise inoculates you against
ever rolling toy parameters in production.

**Checkpoint:** you can state DDH precisely and explain the difference between
"can't compute the secret" and "can't distinguish it from random."

## Module 3 — ElGamal encryption and its homomorphism

**Goal:** master the encryption scheme that *is* the masked card.

**Core ideas:** ElGamal keygen/encrypt/decrypt: `(c1, c2) = (g^r, m·h^r)`; why
fresh randomness r per encryption matters (semantic security); the
multiplicative homomorphism; and the property everything hinges on —
**re-randomization**: `(c1·g^s, c2·h^s)` is a fresh-looking ciphertext of the
*same* plaintext. That operation is `remask`. Also: encoding messages as group
elements (your 52-card lookup table) and why you never ElGamal-encrypt "the
number 7" directly.

**Reading:** Boneh–Shoup ch. 11–12 (public-key encryption, ElGamal). Then
reread Mohnblatt part 1 — the mask/remask API will now be transparent.

**Exercise:** extend your toy crate with ElGamal over your safe-prime group.
Implement `encode` (52 cards → 52 subgroup elements), `mask`, `remask`,
`unmask` (with the full secret key, for now). Property-test: remask 1000 times,
decrypt, always the same card; and no two remasked ciphertexts collide.

**Checkpoint:** you can explain to someone else why remasking makes a shuffle
meaningful — why permuting *without* remasking hides nothing.

## Module 4 — Threshold ElGamal: splitting the key

**Goal:** turn one decryptor into n players, none of whom can decrypt alone.

**Core ideas:** additive key sharing: each player holds x_i, the effective
secret is Σx_i, the public key is h = Πh_i = g^(Σx_i); partial decryption
d_i = c1^(x_i); combining partials; the **staged unmask** — apply n−1 partials
and one player finishes privately (deal a hole card), apply all n (reveal the
board). Also the rogue-key attack: why each player must *prove knowledge* of
their x_i before their h_i is accepted (the proof itself arrives in module 5 —
here, just understand the attack).

**Reading:** Boneh–Shoup on threshold decryption; the Barnett–Smart paper
§ on the VTMF (it will now be readable); Mohnblatt part 1's reveal-token flow.

**Exercise:** extend the toy crate to n players with additive shares. Implement
`reveal_token` (no proof yet) and staged `unmask`. Reproduce, with real (small)
math, the two-seat deal test from your `pkcore-mp` mock: hole card locked with
n−1 tokens, open with n. This is the moment the mock's "padlock set" becomes
actual algebra.

**Checkpoint:** you can trace, on paper, a hole-card deal for 3 players with
p = 23 — every value computed by every player.

## Module 5 — Sigma protocols: Schnorr and Chaum–Pedersen

**Goal:** understand the two zero-knowledge proofs the protocol uses, deeply
enough to implement them.

**Core ideas:** what a proof of knowledge is; the three-move sigma shape
(commit, challenge, respond); the Schnorr protocol (prove knowledge of x in
h = g^x) — this fixes module 4's rogue-key hole; the **Chaum–Pedersen** protocol
(prove log_g(h_i) = log_c1(d_i), i.e. "my partial decryption used the same
secret as my public key") — this is what stops a player from feeding you a
bogus d_i and making you misread your own hand; special soundness and honest-
verifier zero-knowledge, at the level of "extract the witness from two
transcripts" and "simulate transcripts without the witness."

**Reading:** Thaler ch. on sigma protocols; Boneh–Shoup ch. 19–20. Schnorr
first, then Chaum–Pedersen as "Schnorr, twice, with the same response."

**Exercise:** implement interactive Schnorr and Chaum–Pedersen in the toy
crate (challenge = a random number you type in). Then implement the *cheating*
prover for a fixed known challenge, to see with your own eyes why the challenge
must be unpredictable. Attach Chaum–Pedersen proofs to your reveal tokens and
Schnorr proofs to keygen.

**Checkpoint:** you can explain what specifically goes wrong in the deal if
reveal tokens are unproven, and what the rogue-key attack does at keygen.

## Module 6 — Fiat–Shamir and the assembled protocol

**Goal:** make the proofs non-interactive, and assemble everything you have
into the actual Barnett–Smart flow.

**Core ideas:** hash functions as random oracles; the Fiat–Shamir transform
(challenge := hash of the transcript); what must go into the hash (the full
statement — omitting context enables replay and malleability attacks);
signatures as Fiat–Shamir'd Schnorr (which also demystifies ed25519). Then the
assembly: keygen+proof → aggregate key → mask deck → each player
shuffle+remask → deal via proven reveal tokens → staged/full unmask. Exactly
the eight steps of the round, now with real algebra end to end.

**Reading:** Thaler on Fiat–Shamir; reread Mohnblatt parts 1–2 and the
Barnett–Smart paper in full. It should now read as a description of code you've
mostly written.

**Exercise:** make every proof in the toy crate non-interactive with SHA-256
challenges. Then run a complete 2-player hand — the toy crate now implements
the entire protocol *except* the shuffle proof. Diff your API against the
`CardCrypto` trait: it should match almost 1:1.

**Checkpoint:** the whole toy protocol runs, and you can articulate the one
remaining hole: nothing yet proves the shuffler didn't swap the deck.

## Module 7 — Verifiable shuffles

**Goal:** understand what a shuffle argument guarantees and how the main
constructions work at block-diagram level. (Implementing Bayer–Groth from
scratch is a project in itself — the goal here is *informed use*, not
reimplementation.)

**Core ideas:** the statement being proven ("output = permutation + remasking
of input, and I know which"); Pedersen commitments (the missing primitive:
commitments to values with hiding + binding, and commitments to *permutations*);
the Neff and Bayer–Groth approaches, at the level of "commit to the
permutation, prove a polynomial identity that only holds for permutations";
proof vs. argument (bounded vs. unbounded provers) and why an argument is an
acceptable trade; costs — Bayer–Groth for 52 cards runs ~50ms to prove,
~1ms to verify.

**Reading:** Thaler on commitments; the Bayer–Groth paper's introduction and
protocol overview (skip the security proofs on first pass); Mohnblatt part 2;
the `geometryxyz/mental-poker` shuffle module as a reading exercise.

**Exercise:** implement Pedersen commitments in the toy crate, plus the
simplest possible shuffle *check* (your mock's multiset test) to make the
contrast concrete: the multiset check requires seeing the plaintexts; the whole
point of the ZK argument is achieving that guarantee over *ciphertexts*. Then
read Geometry's implementation and map its functions onto the paper's phases.

**Checkpoint:** you can explain to a fellow engineer what a shuffle argument
proves, what it costs, and why "argument" rather than "proof" is fine here.

## Module 8 — Real curves, real libraries: the production backend

**Goal:** move from toy integers to elliptic curves and implement the real
`CardCrypto`.

**Core ideas:** elliptic curve groups as a drop-in replacement (same group
axioms, same DDH-style assumptions, dramatically smaller elements — this is
why everything transfers unchanged from your toy crate); point encoding and
hashing-to-curve (the real version of your card lookup table); curve/library
choices in Rust: `curve25519-dalek` (Ristretto — clean prime-order group,
ideal first target) and the arkworks ecosystem (what `geometryxyz/mental-poker`
uses); constant-time discipline and why you never write production curve
arithmetic yourself.

**Reading:** the Ristretto explanation at ristretto.group;
`curve25519-dalek` docs; the RareSkills ZK book's elliptic-curve chapters if
you want more depth; `geometryxyz/mental-poker` as the reference
implementation to study end to end.

**Exercise (capstone):** implement `CardCrypto` for real — ElGamal over
Ristretto, Schnorr keygen proofs, Chaum–Pedersen reveal tokens, Fiat–Shamir
throughout, and Geometry's Bayer–Groth (or your binding to it) for the shuffle.
Port your toy crate's property tests. Wire it into `pkcore-mp` beside
`PlaintextCrypto` and run the two-seat round test against both backends: the
test should pass unchanged. That's the whole point of the trait boundary — and
the proof you've arrived.

## Module 9 (optional) — The frontier

Where to go once the core is solid, in rough order of relevance: SNARK-based
shuffles (zkShuffle; why on-chain verification changes the constraints);
universal composability, skimming the Kaleidoscope and Royale papers for the
security-model vocabulary; the financial layer (penalties, escrow, dropout
recovery); and side-channel awareness (timing, serialization leaks) for
anything that touches real stakes.

---

## Habits that make this stick

Keep the toy crate insecure and *small* on purpose — p = 23 on paper, p ≈ 2^32
in tests — so every intermediate value is inspectable. Never mix it with real
code; its job is X-ray vision, and it retires after module 8.

For every primitive, write the attack that its absence permits (module 2's
BSGS, module 5's fixed-challenge cheat, module 7's deck swap). The proofs stop
feeling like ceremony once you've played the adversary.

Explain each module's checkpoint out loud or in a paragraph before moving on.
The material compounds: a wobbly module 3 makes module 5 opaque and module 7
impossible, so when something later feels like magic, the fix is almost always
one or two modules back.
