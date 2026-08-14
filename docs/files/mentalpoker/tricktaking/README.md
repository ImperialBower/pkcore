# tricktaking

A shared engine for **trick-taking card games** — bridge, spades, hearts,
euchre, whist. It owns the logic that is identical across all of them
(following suit, trump-aware trick resolution, lead rotation) and exposes hooks
for the parts that differ per game (bidding and scoring). A new game is a thin
`TrickTaking` impl, not a new engine.

```
$ cargo test
running 7 tests ... ok          # trick resolution, follow-suit, bridge & spades scoring
running 2 tests ... ok          # full hand through the generic engine
```

## Design

Don't unify trick-taking games on a single `Game` object — unify on the small
set of mechanics they truly share. The core provides:

- **`trick_winner`** — highest trump wins, else highest card of the led suit
  (no-trump = highest of the led suit). Off-suit, non-trump cards never win.
- **`must_follow`** — the follow-suit rule: play the led suit if you hold it,
  otherwise anything.
- **`resolve_trick`** — credit the winner, archive the trick, open the next one
  led by the winner.
- **`legal_plays`** — follow-suit intersected with each game's extra constraints.

A game then supplies only three things via the `TrickTaking` trait:

| Hook | Bridge | Spades |
|------|--------|--------|
| `trump(contract)` | strain → suit or no-trump | always Spades |
| `can_play(...)` | (default: follow-suit only) | can't *lead* spades until broken |
| `score(contract, tricks)` | undoubled contract scoring | bid / bags / nil, per partnership |

`Score` is an associated type on purpose — it is *not* unified. Bridge returns
an `i32` (declaring side); spades returns `[i32; 2]` (per partnership).

## Layout

```
src/
  card.rs     Card / Rank / Suit (Ace-high). Swap for `cardpack` on a real build.
  lib.rs      Trump, Trick, PlayState, must_follow, trick_winner, resolve_trick,
              legal_plays, and the `TrickTaking` trait.
  bridge.rs   Bridge: Strain/Contract + undoubled scoring.
  spades.rs   Spades: always-trump, spades-not-broken, bid/bag/nil scoring.
  engine.rs   The game-agnostic `GameRules` trait, the `TrickPlay` adapter that
              lifts any `TrickTaking` game into it, and the `run` driver.
tests/
  tricktaking.rs  Trick resolution, follow-suit, and scoring.
  full_hand.rs    A full spades hand played through the generic engine.
```

## Layering

```
card  →  TrickTaking + bridge/spades  →  TrickPlay adapter  →  GameRules  →  run
(substrate)   (family rules)              (lift to engine)    (generic)   (driver)
```

`engine.rs`'s `run` only sequences turns — "whose turn? legal actions? apply
one" — and contains no trick, trump, suit, or scoring logic. All of that arrives
through the `TrickTaking` impl behind the `TrickPlay` adapter, so the same
generic engine would drive poker or Go Fish given their own `GameRules` impls.

## Usage

```rust
use tricktaking::card::{Card, Rank, Suit};
use tricktaking::engine::{run, GameRules, TrickPlay, TrickState};
use tricktaking::spades::{Bid, Spades};
use tricktaking::{PlayState, TrickTaking};

let game = Spades;
let bids = [Bid { tricks: 3 }, Bid { tricks: 3 }, Bid { tricks: 3 }, Bid { tricks: 4 }];

let trump = game.trump(&bids);
let rules = TrickPlay { game, contract: bids };
let state = TrickState { play: PlayState::new(trump, hands /* Vec<Vec<Card>> */, 0) };

let done = run(&rules, state).unwrap();
let score = rules.outcome(&done).unwrap(); // [i32; 2]
```

Verified scoring (from the test suite):

- Bridge — 4♠ making = **420**, 2♣ part-score = **90**, 3NT making = **400**,
  4♥ down one = **−50**.
- Spades — bid 4 / take 5 = **41** (one bag), set = **−40**, nil made = **+100**,
  nil broken = **−100**.

## Adding a game

Hearts slots in with no new machinery: `trump` returns `Trump::NoTrump`, a
`can_play` enforces the first-trick and hearts-not-broken rules, and `score`
implements point avoidance with shoot-the-moon. Euchre is the same pattern with
a 24-card deck and a chosen-trump auction.

## Hidden information

`GameRules::view_for` is the per-seat projection: a seat sees its own hand and
public table state, but opponents' hands appear only as sizes. This is the seam
the distributed / mental-poker layer plugs into — "a hidden hand" becomes "a
vector of masked cards," and `view_for` is where a revealed card surfaces once
decoded. The plaintext engine and the cryptographic one implement the same
projection; only the representation of "hidden" differs.

## Toolchain & integration

The `card` module is a self-contained stub so the crate builds dependency-free
(verified on rustc 1.75, edition 2021). For real use, swap it for `cardpack`'s
`Rank`/`Suit`/`Card` — the only requirement the engine makes is **Ace-high**
ordering — and align the `cardpack` version with `pkcore` and `gfcore` so all
three sit on one card substrate.

Note a name clash to avoid: the `Spades` game struct collides with
`Suit::Spades` under glob imports. A module convention (e.g. `spades::Game`)
keeps downstream `use` statements clean.

## Known gaps

- **Auction.** `TrickPlay` starts from a finished `Contract`; the bidding phase
  is a separate sub-machine you compose first (a `GameRules` whose `Outcome` is
  the `Contract` that seeds the play phase). Bridge's auction is the bulk of
  bridge-specific code.
- **Scoring depth.** Bridge scoring is undoubled — no doubles/redoubles, slam
  bonuses, or honors. Spades omits cross-hand bag-penalty accumulation and
  blind nil. Both are additive.

## License

MIT OR Apache-2.0, matching pkcore and gfcore.
