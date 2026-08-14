# pktable

A runnable skeleton of **architecture #1** for distributed play: one dumb
relay, N full protocol nodes. Two binaries, std-only (no dependencies beyond
`pkcore-mp` as a sibling directory), verified end to end with two bot clients
playing a complete heads-up hand.

```
relay 127.0.0.1:4747          # the bulletin board
client 127.0.0.1:4747         # interactive seat (c=check/call, b=bet 10, f=fold)
client 127.0.0.1:4747 --bot   # auto check/call seat
```

## What the run demonstrates

```
[seat 1] all key shares in; aggregate key formed
[seat 1] shuffle 2/2 verified
[seat 1] hole card revealed to me: 7d
[seat 1] hole card revealed to me: Th
[seat 1] flop: 5s Js 7s
...
[seat 1] showdown, pot 0:
[seat 1]   seat 1: 7d Th
[seat 1]   seat 0: 2h 6s
[seat 1] chain head: 15a6aad6093a5fa2
```

Every step of the protocol flows as ordered events through the relay: joins,
key shares, two shuffle rounds, hole-card reveal tokens (each seat decodes its
own holes by adding its never-published token), betting, board reveals street
by street, and showdown reveals. Both clients replay the identical log through
the same `GameState::apply` fold and independently arrive at the same state.

## The division of trust

**The relay** (`src/bin/relay.rs`) assigns seats, totally orders events,
maintains the hash chain, replays the log to late joiners, and fans out. That
is all. It holds no keys and validates nothing about the game; with real
crypto the deck is ciphertext to it. Its one real integrity duty — checking
that a proposal's signature matches its author — is stubbed to an author-byte
check and marked where ed25519 goes.

**The client** (`src/bin/client.rs`) is the full node: it verifies the chain
link on every event (`prev` must equal its own head), verifies shuffles and
reveal tokens (mock verifiers, marked where Bayer–Groth / Chaum–Pedersen go),
derives whose turn it is purely from the replayed log, and performs whatever
`duties()` the state says it owes — key share, shuffle, reveal tokens, or a
betting prompt. A verification failure is treated as evidence of a bad peer
and exits loudly.

**The shared library** (`src/lib.rs`) is the replicated state machine: wire
format, FNV-64 chain fold (stand-in for SHA-256 over signed envelopes), the
`GameState` fold, and the duty engine. `pkcore`'s evaluator plugs in at the
showdown to name the winner; the mock crypto comes from `pkcore-mp`'s
`PlaintextCrypto`.

## Two findings from the verification run

**The chain check caught a real bug.** The first run failed with
`REJECTED event: chain break at seq 1` — the relay initially broadcast only
*live* events, so a late-joining seat never saw seq 0 and correctly refused
seq 1. That is the late-join/resync problem arriving on schedule, and the fix
is the resync mechanism itself: the relay now replays the stored log to every
new connection before going live. (The production version sends the suffix
after the client's last known head; genesis replay is the degenerate case.)

**Clients may exit at different prefix heads — that's not divergence.** Each
seat's hand completes at a different log position (seat 0 finishes when seat
1's last showdown token arrives, and vice versa), so their final printed heads
differ. Recomputing the relay's rolling chain confirms both are prefix heads
of the same single chain (positions 30 and 32 of 32 in the verified run).

## Known mock limitations

- `PlaintextCrypto`'s shuffle permutation is deterministic (same seed every
  run), so every hand deals the same cards. Real randomness arrives with the
  real `CardCrypto`; the protocol flow is unaffected.
- No signatures, no proofs — every verification point is present and marked,
  but trivially satisfied. The structure is the deliverable.
- Heads-up only, fixed bet size, no raises, no blinds, no hand evaluation.

## qrtable: the QR-code table

A third binary, `qrtable`, runs the same hand with **no relay and no socket**:
every event crosses between the two seats as a rendered-then-decoded QR image.
Each seat's "phone screen" is a directory; publishing renders the event as a
QR PNG onto its own screen, and the peer's "camera" (rqrr) decodes it. The run
is a real optical round trip for all 32 events of the hand:

```
[seat 0] showed 3-frame animated QR (378 bytes) for oversized event
[seat 1] hole card revealed to me: 7d
...
[seat 0] hand over. frames shown: 18, frames scanned: 18, chain head: 2909039c1f7f288a
[seat 1] hand over. frames shown: 18, frames scanned: 18, chain head: 2909039c1f7f288a
chain heads MATCH — both seats replayed the identical hand
total events: 32  (every one crossed as a scanned QR image)
```

Design points, in the order they bit:

- **Invite**: seat 0's genesis is a deep-link QR printed in ASCII in the
  terminal (actually scannable) — the trust-establishment role QR plays in
  device-pairing flows.
- **Ordering without a relay**: strict alternation — event seq N must be
  authored by seat N mod 2, and a seat with nothing to publish emits a PASS
  (the new no-op `'N'` event kind in the shared lib). The relay's ordering
  job moves into the protocol itself. First run stalled instantly because
  nothing announced joins (the relay used to); each seat now announces itself
  as its first duty.
- **Animated frames**: the two shuffle events (378-byte decks, over one QR's
  comfortable capacity) split into `F|id|i|n|chunk` sequences and reassemble
  on the scanner side. Production swaps sequential chunks for fountain coding
  (RaptorQ / BC-UR) so any sufficient subset of frames suffices.
- **The chain is the sync beacon**: every QR chains onto the head, so a missed
  scan is detected at the very next decode, and matching final heads prove the
  replicas agree.

Dependency pins (`qrcode =0.13`, `rqrr =0.6`, `image =0.24.9`) exist only for
old-toolchain compatibility; bump freely on a modern rustc.

## Where each surface picks this up

This is the terminal surface of the multi-surface plan. The **web** client is
this node compiled to `wasm32` with the TCP transport swapped for WebSocket
(the relay's line protocol is transport-agnostic); the **mobile** client is
the same node behind a PWA or UniFFI shell, with reconnect-and-resync as the
normal state. The relay is the piece that grows into the axum/tokio WebSocket
gateway shared with the arena spectator path.

Layout: `Cargo.toml`, `src/lib.rs`, `src/bin/relay.rs`, `src/bin/client.rs`,
with `pkcore-mp/` as a sibling directory.
