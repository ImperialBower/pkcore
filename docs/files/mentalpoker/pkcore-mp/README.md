# pkcore-mp

A swappable **mental-poker** layer for [`pkcore`](https://github.com/ImperialBower/pkcore),
implementing the deal/reveal mechanics of the Barnett–Smart protocol behind two
traits so the poker engine, the cryptography, and the deployment topology stay
independent.

- **`CardCrypto`** — the verifiable *l*-out-of-*l* threshold masking scheme
  (`keygen` / `mask` / `remask` / `unmask`) plus a verifiable shuffle. Swap the
  impl to change security/curve without touching the engine.
- **`Coordinator`** — transport + total ordering of the signed event log. Each
  architecture (in-proc, relay, mesh, state channel, chain) is one impl.

The **engine boundary**: pkcore only ever sees a plaintext `Card` (returned by
`CardCrypto::decode` once a card is fully unmasked) and an `Action`. Ciphertexts
and proofs are verified at the node boundary and never enter the transition
function — so `pkcore`'s evaluation/analysis code is untouched by crypto.

## What ships

| Item | Role |
|------|------|
| `PlaintextCrypto` | Mock `CardCrypto`. Cards in the clear, but models the *l*-out-of-*l* padlock accounting faithfully (you need every seat's token to read a card). |
| `InProcCoordinator` | Mock `Coordinator` (architecture #1): one shared append-only log, per-reader cursor. |
| `tests/round.rs` | Two-seat round: keygen → mask → shuffle ×N → deal hole cards (reveal-to-one) → reveal board (reveal-to-all) → deck-integrity + threshold assertions. |

## Toolchain note

The default build uses a **local `Card` stub** so the crate compiles on older
toolchains (verified on rustc 1.75). Real `pkcore` requires **Rust ≥ 1.94.1 /
edition 2024**, so its dependency is gated behind the `pkcore` feature — see
the comments in `Cargo.toml` for how to enable it (place `pkcore` as a sibling
directory, uncomment the dependency, wire the feature).

`src/card.rs` re-exports `pkcore::card::Card` and `pkcore::deck::DECK_ARRAY`
under the feature, and a matching stub (same 52-card ordering) otherwise. The
rest of the crate is written against `card::Card` and never branches on it.

## Next steps toward a real game

See `docs/EPIC-01_Real_Cryptography_Backend.md` at the archive root for the
full staged plan. In brief:

1. **Real crypto backend.** Implement `CardCrypto` over arkworks with a
   Bayer–Groth shuffle argument (see `geometryxyz/mental-poker`). `MaskedCard`
   becomes an ElGamal ciphertext (two curve points); proofs replace the `()`s;
   thread an `rng: &mut impl RngCore` through `keygen`/`mask`/`remask`/`shuffle`/
   `reveal_token`.
2. **Engine glue.** Replace the local `Action` with
   `pkcore::casino::dealer::DealerAction`, and feed each `decode`d `Card` into
   `pkcore`'s `TableNoCell` (flipping the slot's `Visibility` to `Up`). Keep the
   transition function crypto-free.
3. **Real transport.** See `pktable` for relay and QR transports; make
   `publish` / `next_event` async, sign the `SignedEvent` envelope (ed25519),
   and hash-chain `prev_hash` for real.
4. **Liveness.** Add the timeout + forfeiture path where a `ToSeat`/`ToAll`
   reveal stalls because a required seat went offline.

Licensed under MIT OR Apache-2.0, matching pkcore.
