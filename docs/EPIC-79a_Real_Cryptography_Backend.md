# EPIC-01: Real Cryptography Backend for pkcore-mp

**Repo:** `pkcore-mp` (touches `pktable`; no changes to `pkcore` / `tricktaking`)
**Status:** Proposed
**Depends on:** `CardCrypto` / `Coordinator` trait boundary (shipped), `PlaintextCrypto` mock (shipped), `pktable` relay/client/qrtable demos (shipped), `mp-toy` lesson-plan crate (companion reference)

---

## 1. Context

`pkcore-mp` defines the mental-poker layer behind two traits — `CardCrypto`
(the Barnett–Smart verifiable threshold masking scheme + shuffle) and
`Coordinator` (transport/ordering) — with mock implementations that model the
*accounting* of the protocol (l-out-of-l padlocks, staged unmask, chain
verification) but perform no real encryption. The `pktable` demos verified the
full event-log machinery end to end over three transports (in-proc, TCP relay,
QR-optical) against the mock.

This EPIC replaces the mock with real cryptography. The design goal that makes
it tractable: **the swap happens entirely behind `CardCrypto`.** The engine,
the duty machine, the transports, and the tests do not change shape; the
existing two-seat round test must pass unchanged against both backends.

The work is unevenly distributed: the VTMF (ElGamal, threshold keys, Schnorr,
Chaum–Pedersen, Fiat–Shamir) is a direct transliteration of `mp-toy` modules
3–6 onto a real curve library; the verifiable shuffle (Bayer–Groth) is a
multi-week, high-subtlety project on its own. The phases below stage the work
so everything except the shuffle proof becomes real first.

## 2. Goals

- A `RistrettoCrypto` (or arkworks-equivalent) backend implementing
  `CardCrypto` with real secrecy: hole cards unreadable without the staged
  unmask, deck ciphertext opaque to any relay.
- Real proofs at every verification point currently stubbed: Schnorr PoK at
  keygen (rogue-key gate), Chaum–Pedersen on every reveal token, and —
  ultimately — a shuffle argument on every shuffle.
- Real signatures and hashing in the `pktable` envelope: ed25519 over
  `(seq, prev, author, payload)`, SHA-256 chain fold.
- `wasm32` compatibility for the entire crypto path (web/mobile surfaces use
  the identical code).
- The existing `tests/round.rs` passing against both `PlaintextCrypto` and the
  real backend, plus new adversarial tests per phase.

## 3. Non-Goals

- Settlement, escrow, penalties (Kaleidoscope/Royale layer) — separate EPIC.
- Timeout/forfeiture protocol for dropouts — separate EPIC (interacts with
  this one only at the "which reveal is owed" level).
- N>2 seat betting logic in `pktable` — orthogonal.
- A from-scratch Bayer–Groth implementation (explicitly Phase-3-optional; see
  §6 Risks).
- Production security sign-off. Phase 4 is engineering hardening; an external
  review is budgeted separately before anything with stakes.

## 4. Phases

### Phase 0 — Survey and pin (short, do first)

The crate landscape moves; my last full picture needs re-verification.

- [ ] Assess `geometryxyz/mental-poker` (and forks): maintenance status,
      arkworks version pins, curve, license, test coverage. Identify the most
      modernized fork if any.
- [ ] Survey alternatives: zkShuffle lineage, any newer arkworks-native
      shuffle crates, any maintained Barnett–Smart crates.
- [ ] Pin current versions: `curve25519-dalek`, `merlin`, `ed25519-dalek`,
      `zeroize`, `rand_core`/`getrandom` (wasm feature flags), `sha2`.
- [ ] **Decision gate:** dalek-stack for Phases 1–2 with arkworks arriving in
      Phase 3, vs. all-arkworks from the start to match the shuffle library's
      types. Record as an ADR in `docs/`. Default recommendation: all-arkworks
      *if* the Phase-3 library choice is firmly Geometry-derived; otherwise
      dalek first (cleaner API, Ristretto's prime-order group, easier wasm).

**Acceptance:** ADR committed; `Cargo.toml` versions chosen; `cargo build`
on stable and `wasm32-unknown-unknown`.

### Phase 1 — `RistrettoCrypto`: the VTMF with an honest-shuffle assumption

The full Barnett–Smart VTMF, real in every respect except that `shuffle`'s
proof is a placeholder verified as `Ok(())`, documented loudly.

Work items:

- [ ] **Group + encoding.** `MaskedCard = (RistrettoPoint, RistrettoPoint)`
      compressed to 64 bytes on the wire. Card encoding table: 52 points via
      hash-to-curve on domain-separated labels (`"pkcore-mp/v1/card/{i}"`),
      derived identically by every client; reverse map for `decode`.
- [ ] **Keys.** `keygen` → `(Scalar, RistrettoPoint, SchnorrProof)`;
      `verify_key` checks the PoK (rogue-key gate); `aggregate` = point sum.
- [ ] **Mask / remask.** ElGamal encrypt / re-randomize under the aggregate
      key; `MaskProof` as Chaum–Pedersen-style correctness proof (or documented
      as unproven-in-v1 if Geometry's API shape is adopted — decide in Phase 0).
- [ ] **Reveal tokens.** `reveal_token` = `c1 * x_i` with a Chaum–Pedersen
      proof binding it to the player's registered public key;
      `verify_reveal_token` mandatory before `unmask` accepts a token.
- [ ] **Transcripts.** All Fiat–Shamir challenges via `merlin`, binding the
      full statement: protocol version, table id, hand id, deck position,
      ciphertext, public keys, and key-registration order. No ad-hoc hashing.
- [ ] **Randomness + hygiene.** `OsRng` only; `zeroize` on secret keys and
      masking randomness; `ToyRng` and all `mp-toy` code firewalled from this
      crate (dev-dependency at most, never in `src/`).
- [ ] **Tests.** Port `mp-toy` property tests (mask/remask/unmask roundtrips,
      threshold: n−1 tokens ≠ plaintext, tamper: every proof rejects a
      one-bit-flipped transcript). Run `tests/round.rs` against both backends
      via a test-generic harness.

**Acceptance:** round test green on both backends; adversarial tests green;
`#![forbid(unsafe_code)]`; builds on `wasm32-unknown-unknown`; README states
the honest-shuffle assumption in a way nobody can miss:
*"a cheating shuffler can substitute cards undetected in this phase."*

### Phase 2 — Real envelope in `pktable`

- [ ] ed25519 signatures (`ed25519-dalek`) over the canonical serialization of
      `(table_id, hand_id, seq, prev, author, payload)`; relay's one integrity
      duty (signature matches author) implemented for real; clients verify
      every event's signature before `apply`.
- [ ] SHA-256 chain fold replacing FNV-64 (keep the same head-beacon UX).
- [ ] Wire format: payloads as base64(compressed points / proof bytes);
      version byte in the envelope for forward compatibility.
- [ ] QR transport arithmetic revisited: masked deck ≈ 3.3 KB + proofs means
      animated frames are the norm for setup. Upgrade sequential chunking to
      fountain coding (`raptorq`), BC-UR-style, so any sufficient frame subset
      reconstructs.
- [ ] Re-run all three transport demos (relay, qrtable) end to end.

**Acceptance:** full hand over the relay and over QR with real crypto +
signatures; a tampered event (flipped byte, wrong author key, replayed event
from another hand) is rejected with a specific error naming the check that
failed.

### Phase 3 — The verifiable shuffle

Three options, in order of preference; Phase 0's survey picks:

- **3a (default): fork-and-vendor Geometry's Bayer–Groth.** Pin or port its
  arkworks stack; adapt curve choice per the Phase-0 ADR; wire behind
  `shuffle`/`verify_shuffle`. Treat the fork as owned code: read it against
  the Bayer–Groth paper (lesson-plan module 7 is the preparation), add our
  own test vectors, keep the diff from upstream minimal and documented.
- **3b: adopt a maintained alternative** surfaced in Phase 0 (zkShuffle-line
  or newer), same integration seam.
- **3c (only if 3a/3b fail): implement Bayer–Groth over the chosen stack.**
  Multi-week; requires the module-7 material cold; budget accordingly and
  descope elsewhere.

- [ ] Shuffle proof produced by each shuffler, verified by every peer before
      the deck advances; a bad shuffle is rejected with evidence.
- [ ] Negative tests: substituted card, duplicated card, non-permutation —
      all rejected.
- [ ] Benchmarks recorded in `docs/` (target from Geometry's published
      numbers: ~50 ms prove / <1 ms verify for 52 cards; verify on our
      hardware and on wasm).

**Acceptance:** honest-shuffle assumption removed from the README; the
end-to-end demos run with proofs on; benchmark doc committed.

### Phase 4 — Hardening pass

- [ ] Canonical-encoding rejection fuzz: feed malformed/non-canonical point
      encodings and truncated proofs into every deserializer (cargo-fuzz).
- [ ] Transcript audit: enumerate every challenge derivation; confirm each
      binds the full statement; write the enumeration into `docs/` so review
      is possible.
- [ ] Cross-context replay tests: proofs from hand A rejected in hand B;
      reveal token for slot i rejected for slot j.
- [ ] `cargo-deny` + `clippy` gates in CI; `zeroize` coverage check.
- [ ] Scope + commission external review **before any real-stakes use**.

**Acceptance:** fuzz targets run clean for a committed duration; audit doc in
`docs/`; CI gates green.

## 5. Size and shape estimates

Phase 1 is ~500–800 lines plus tests — one to two careful weeks for someone
through the lesson plan, "careful" being the operative word. Phase 2 is days.
Phase 3a is dominated by reading and porting, not writing (1–2 weeks); 3c is
the multi-week outlier to avoid. Phase 4 is a steady background week.
Performance is a non-issue for playability at every phase: point ops are
microseconds; the shuffle proof is the only >1 ms item.

Concrete wire sizes to design against: masked card 64 B; reveal token 32 B +
~96 B proof; masked deck ~3.3 KB; Bayer–Groth proof on the order of KBs.

## 6. Risks

- **Geometry bitrot** (Phase 3a): old arkworks pins may fight current
  toolchains; the fork may need real porting. Mitigation: Phase 0 sizes this
  before commitment; 3b/3c are the fallbacks; Phases 1–2 are independent of it.
- **Stack split** (dalek vs arkworks): maintaining a seam between two curve
  stacks invites conversion bugs. Mitigation: the Phase-0 ADR decides once;
  all-arkworks is acceptable if it simplifies Phase 3.
- **Composition bugs**: primitives are audited, our composition is not.
  Peer-reviewed protocol + transcript discipline + adversarial tests reduce
  but do not eliminate this — hence the external-review gate.
- **wasm randomness/size**: `getrandom` needs the right feature for wasm;
  proof code may bloat the bundle. Mitigation: wasm build in CI from Phase 1.
- **Toolchain**: current dalek/arkworks require a recent rustc; all pins in
  this EPIC assume the workspace toolchain (≥1.94), not the sandbox pins used
  in the demos.

## 7. Test gate

`OTEL_SDK_DISABLED=true cargo test --workspace` green at every phase
boundary, plus the wasm build check and (from Phase 4) the fuzz/CI gates.

## 8. References

- Barnett & Smart, *Mental Poker Revisited* (IMACC 2003) — the protocol.
- Bayer & Groth, *Efficient Zero-Knowledge Argument for Correctness of a
  Shuffle* (2012) — the Phase-3 argument.
- Mohnblatt, *Mental Poker in the Age of SNARKs* pts 1–2 — the implementation
  map; Geometry's `mental-poker` repo — the Phase-3a candidate.
- `mp-toy` (this workspace) — the executable specification of Phases 1's
  algebra; retire it from reference duty once Phase 1 lands.
- Lesson plan `mental-poker-crypto-lesson-plan.md` — modules 3–7 are the
  preparation for Phases 1 and 3 respectively.
