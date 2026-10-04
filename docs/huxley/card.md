# Huxley: Card (current representations)

Report date: 2026-09-27

Repos searched, each at its current HEAD:

- [pkcore](https://github.com/ImperialBower/pkcore) @ `6c10b52c831de0c9ce0d2ea10957ae32b5492c27`
- [cardpack.rs](https://github.com/ImperialBower/cardpack.rs) @ `e4c91c57a0bd06239ce11a9eaa4e33cb5ce40203`
- [ckc-rs](https://github.com/ImperialBower/ckc-rs) @ `8699e1d2841647654712b2daeff0a80f9b93a494`
- [fudd](https://github.com/ImperialBower/fudd) @ `9d20005ccf85a6108a718e6f71e29a6c8a6b5854`
- [wincounter](https://github.com/ImperialBower/wincounter) @ `7f88f60e3de91066d27a588f0705cee017b83461`
- [pkcore.py](https://github.com/ImperialBower/pkcore.py) @ `3723183fc1183f9e10d621e8c1446c7b7b6658f4`
- [pkcore.js](https://github.com/ImperialBower/pkcore.js) @ `8a7aa05f7961ee6a4514d54e130eacfb8867f249`
- [pknotebook](https://github.com/ImperialBower/pknotebook) @ `579cfa10b74ec2b9f527094dbeb876699461b5b9`
- [pktui](https://github.com/ImperialBower/pktui) @ `8521ce750e4c433b2101de742f0d6c9a67c36021`
- [pkdealer](https://github.com/ImperialBower/pkdealer) @ `f7f7cdee215914a3d5089f0c749c042afb51dbaf`
- [pkgto-web](https://github.com/ImperialBower/pkgto-web) @ `064015e01560e5bfef176ccfd4f400cc53fdc1cb`
- [pkkuhn-web](https://github.com/ImperialBower/pkkuhn-web) @ `c853b723fab3d3abdfd864394cd28b53883032b8`
- [pkarena0-web](https://github.com/ImperialBower/pkarena0-web) @ `fbbeb42d54c550ab3a3ad633930591652572ca30`

This is a breadth/snapshot report — how the family represents a single playing
card *today*, in one pass across all thirteen clones — not a lineage report.
For the historical, commit-by-commit story of the related `Suit` type (born as
`Anzug` in cardpack.rs, ported through ckc-rs into pkcore, then wrapped for
Python/JS), see [`suit.md`](./suit.md) in this same folder.

## Summary

Across the family there are four genuinely distinct "shapes" for a single
card: **(1)** a `Copy`-able newtype wrapping a Cactus-Kev-style `u32` bitmask
(`pkcore::Card`, `fudd::PlayingCard`, and ckc-rs's bare `CKCNumber` constants
with no wrapper at all); **(2)** a heavier `struct` of `Rank` + `Suit`-like
fields meant for display/i18n (`cardpack.rs`'s `BasicCard` = `{ suit: Pip,
rank: Pip }`, generic-wrapped again by `Card<DeckType>`); **(3)** a
game-specific closed enum with no suit concept at all (pkcore's own
`KuhnCard::{Jack, Queen, King}` for the 3-card Kuhn variant); and **(4)** thin
FFI newtypes that just forward to shape (1) (`pkcore.py`'s and `pkcore.js`'s
`Card(PkCard)`). The bitmask newtype is dominant and canonical: it is the one
type actually named `Card` in the majority of these repos, either defined
directly (pkcore, ckc-rs) or imported by every downstream consumer (fudd,
pktui, pkdealer, pkarena0-web, pkgto-web transitively via `Two`). No repo in
this family represents a card as a plain `(Rank, Suit)` tuple or a bare
0–51 index — those two conventions, common elsewhere in the Rust card-game
ecosystem, are absent here.

## Comparison table

| Repo | Type name | Kind | Representation | Key traits | Notes |
|---|---|---|---|---|---|
| pkcore | `Card` | newtype struct | `u32` Cactus-Kev bitmask, built from `Rank`+`Suit` bits | `Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd`, `Deserialize`/`Serialize` (custom), `Display`, `FromStr` | Canonical type; `CardNumber` enum (`#[repr(u32)]`) supplies the 52 named bit patterns |
| cardpack.rs | `BasicCard` (data) / `Card<DeckType>` (generic wrapper) | struct / generic newtype-ish struct | `BasicCard { suit: Pip, rank: Pip }`; `Pip { weight, pip_type, index, symbol, value }` | `Clone, Copy, Debug, Default, Eq, Hash, PartialEq`, `Ord`/`PartialOrd` (customized, reverse order), `Serialize`/`Deserialize` (feature-gated) | Rank/suit are both instances of a shared `Pip` type, not distinct types |
| ckc-rs | `CKCNumber` (= `u32`), named via unit struct `CardNumber` | plain integer convention | raw `u32` bitmask, no wrapper type | `Copy` (primitive), whatever traits `u32` has | The "card" is just a `u32` value; `CardNumber` is a namespacing unit struct holding consts, not a card type itself |
| fudd | `PlayingCard` | newtype struct | wraps ckc-rs's `U32Card` (= `u32`) `CKCNumber` | `Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd` | Own newtype, but built directly on **ckc-rs**, not pkcore — same shape as `pkcore::Card`, independently duplicated |
| wincounter | — | — | — | — | No card concept; see "Nothing card-specific" |
| pkcore.py | `Card` | PyO3 newtype (`pyclass`) | `Card(PkCard)` wraps `pkcore::card::Card` | `Clone` (Rust side); Python-visible via `#[pymethods]` | Thin wrapper, forwards to pkcore |
| pkcore.js | `Card` | napi-rs newtype | `Card(PkCard)` wraps `pkcore::card::Card` | `Clone, Copy` | Thin wrapper, forwards to pkcore |
| pknotebook | — | — | — | — | Not a Rust/JS project at all; see "Nothing card-specific" |
| pktui | — | — | imports `pkcore::card::Card` | — | No type of its own |
| pkdealer | — | — | imports `pkcore::card::Card` | — | No type of its own, across all workspace crates |
| pkgto-web | — | — | imports `pkcore::prelude::*` (`Two` = `[Card; 2]`, `Rank`, `Suit`) | — | Never names `Card` directly, but every hand it parses is built from pkcore `Card`s under `Two` |
| pkkuhn-web | — | — | imports `pkcore::games::kuhn::KuhnCard` | — | Uses pkcore's own game-specific `KuhnCard` enum, not the bitmask `Card` |
| pkarena0-web | — | — | imports `pkcore::card::Card` | — | No type of its own |

## Details per repo

### pkcore — `Card`, the canonical bitmask newtype

[view on GitHub](https://github.com/ImperialBower/pkcore/blob/6c10b52c831de0c9ce0d2ea10957ae32b5492c27/src/card.rs#L36-L37)

```rust
#[derive(Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Card(#[serde(deserialize_with = "deserialize_card_index")] u32);
```

Built via `Card::new(rank: Rank, suit: Suit)` at
[`src/card.rs:124-127`](https://github.com/ImperialBower/pkcore/blob/6c10b52c831de0c9ce0d2ea10957ae32b5492c27/src/card.rs#L124-L127), which composes the u32 from
`rank.bits() | rank.prime() | rank.shift8() | suit.binary_signature()` — i.e.
`Rank` and `Suit` remain separate types, but a `Card` is their *encoded*
combination, not a struct holding both. The doc comment above the struct spells
out the Cactus-Kev-derived bit layout (`mmmbbbbb bbbbbbbb SHDCrrrr xxpppppp`)
and explicitly calls this a suit-bit-order variant of the original scheme "for
easier sorting." The 52 named constants (`Card::ACE_SPADES`, etc.) are built
from a companion `#[repr(u32)] enum CardNumber` at
[`src/card_number.rs:71-73`](https://github.com/ImperialBower/pkcore/blob/6c10b52c831de0c9ce0d2ea10957ae32b5492c27/src/card_number.rs#L71-L73), whose discriminants are the raw bit
patterns. `Card::BLANK` is `Card(0)`.

pkcore also defines a second, unrelated single-card type for one specific
game — `KuhnCard`, a closed 3-value enum with no suit at all:

[view on GitHub](https://github.com/ImperialBower/pkcore/blob/6c10b52c831de0c9ce0d2ea10957ae32b5492c27/src/games/kuhn.rs#L55-L60)

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum KuhnCard {
    Jack,
    Queen,
    King,
}
```

This exists because Kuhn poker's whole deck is three ranked cards with no
suits or bitmask math needed — reusing the 52-card `Card` bitmask here would
be overkill, so pkcore ships a purpose-built enum instead.

### cardpack.rs — `BasicCard` (data) wrapped by generic `Card<DeckType>`

[view on GitHub](https://github.com/ImperialBower/cardpack.rs/blob/e4c91c57a0bd06239ce11a9eaa4e33cb5ce40203/src/basic/types/basic_card.rs#L43-L48)

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BasicCard {
    pub suit: Pip,
    pub rank: Pip,
}
```

`Pip` (the shared type behind both fields) is itself a struct, not a bare enum:

[view on GitHub](https://github.com/ImperialBower/cardpack.rs/blob/e4c91c57a0bd06239ce11a9eaa4e33cb5ce40203/src/basic/types/pips.rs#L77-L85)

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pip {
    pub weight: usize,
    pub pip_type: PipType,
    pub index: char,
    pub symbol: char,
    pub value: usize,
}
```

The doc comment on `BasicCard` explains the field order deliberately: suit
first, then rank, "so that the default sorting for a collection is done suit
first," and notes the `Ord`/`PartialOrd` impls are customized to sort in
reverse. A second, generic wrapper type — `Card<DeckType>` — adds
deck-specific behaviour (`color()`, `fluent_name()`, `from_str()`) on top:

[view on GitHub](https://github.com/ImperialBower/cardpack.rs/blob/e4c91c57a0bd06239ce11a9eaa4e33cb5ce40203/src/basic/types/card.rs#L32-L40)

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Card<DeckType>
where
    DeckType: DeckedBase,
{
    pub base_card: BasicCard,
    pub deck: PhantomData<DeckType>,
}
```

This is the design that `suit.md` calls the "Pip-based model," born
2025-02-19 as a replacement for the old field-based `Suit` struct — it exists
so one card shape (`BasicCard`) can serve many games (French, Canasta, etc.)
while `Card<DeckType>` supplies per-deck superpowers via a zero-sized
`PhantomData<DeckType>` marker, at no runtime cost.

### ckc-rs — no wrapper type; a card is a raw `u32` (`CKCNumber`)

[view on GitHub](https://github.com/ImperialBower/ckc-rs/blob/8699e1d2841647654712b2daeff0a80f9b93a494/src/lib.rs#L33-L39)

```rust
pub type CKCNumber = u32;

/// u32 constants for all 52 cards in a standard poker deck.
pub struct CardNumber;

#[rustfmt::skip]
impl CardNumber {
```

There is no struct or enum literally named `Card` in ckc-rs — `CKCNumber` is a
plain type alias for `u32`, and `CardNumber` is a zero-field unit struct used
purely as a namespace for the 52 `pub const` bit patterns (e.g.
`CardNumber::ACE_SPADES`). This is the leanest representation in the family: a
"card" here is whatever bit pattern a `u32` happens to hold, evaluated through
the crate's `PokerCard` trait (`src/lib.rs:465`) rather than through any
inherent methods on a type.

### fudd — `PlayingCard`, its own bitmask newtype over ckc-rs

[view on GitHub](https://github.com/ImperialBower/fudd/blob/9d20005ccf85a6108a718e6f71e29a6c8a6b5854/src/types/playing_card.rs#L1-L9)

```rust
use crate::types::U32Card;
use ckc_rs::{CKCNumber, CardNumber, PokerCard};
use serde::de::Deserializer;
use serde::ser::{Serialize, Serializer};
use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlayingCard(#[serde(deserialize_with = "deserialize_card_index")] U32Card);
```

`U32Card` is fudd's own alias for `u32` (`src/types/mod.rs:30`). fudd depends
on **ckc-rs**, not pkcore (`ckc-rs = "0.1.14"` in its `Cargo.toml`), so
`PlayingCard` independently re-derives the same newtype-over-bitmask shape
that pkcore also arrived at — same derive list, same constructor pattern
(`PlayingCard(CardNumber::ACE_SPADES)`), but a separate type built against a
different upstream crate. fudd layers small fixed-size wrappers on top —
`ThreeCard([U32Card; 3])`, `FiveCard([U32Card; 5])`, `SevenCard([U32Card; 7])`,
etc. — for hand-sized collections, plus a `PlayingCards(IndexSet<PlayingCard>)`
for variable-sized ones.

### pkcore.py — `Card(PkCard)`, a PyO3 newtype

[view on GitHub](https://github.com/ImperialBower/pkcore.py/blob/3723183fc1183f9e10d621e8c1446c7b7b6658f4/src/lib.rs#L302-L304)

```rust
#[pyclass(from_py_object, name = "Card")]
#[derive(Clone)]
pub struct Card(PkCard);
```

`PkCard` is pkcore's `Card` imported under an alias
(`use pkcore::card::Card as PkCard;`). The wrapper exposes `Card.parse("As")`,
`Card.from_rank_suit(rank, suit)` (which calls straight through to
`PkCard::new(rank.0, suit.0)`), and a `Display` that renders suited unicode
(`'A♠'`, per the module's doctest). No card logic is reimplemented — this is a
pure forwarding shim, matching the pattern `suit.md` already documents for
`pkcore.py`'s `Suit`.

### pkcore.js — `Card(PkCard)`, a napi-rs newtype

[view on GitHub](https://github.com/ImperialBower/pkcore.js/blob/8a7aa05f7961ee6a4514d54e130eacfb8867f249/src/lib.rs#L274-L290)

```rust
/// A single playing card.
#[napi]
#[derive(Clone, Copy)]
pub struct Card(PkCard);

#[napi]
impl Card {
    /// Parses a card such as `"As"`, `"Kh"`, `"Q♦"`, `"2c"`.
    #[napi(factory)]
    pub fn parse(text: String) -> Result<Self, napi::Error<String>> {
        PkCard::from_str(&text).map(Card).map_err(pk_err)
    }

    #[napi(factory)]
    pub fn from_rank_suit(rank: &Rank, suit: &Suit) -> Self {
        Card(PkCard::new(rank.0, suit.0))
    }
```

Same wrapper shape as the Python binding, one derive stronger (`Copy`, since
napi-rs's generated bindings favor by-value handles), exposing `#[napi(factory)]`
constructors instead of Python `@staticmethod`s. Both bindings independently
arrived at the identical `Card(PkCard)` newtype pattern already established
for `Suit` in the sibling report.

## Nothing card-specific

- **wincounter** — a standalone "binary win counter that supports ties" for
  reporting game results (`description` in its `Cargo.toml`); its only
  dependency is `percent-encoding`, and the sole match for "Card" in the whole
  tree is a doc comment mentioning an unrelated historical `Card` struct in
  prose. It has no notion of a playing card at all.
- **pknotebook** — not a Rust or JS project; it is a Docker Compose /
  Jupyter / PySpark environment ("Jupyter All Spark Poker Notebook") with no
  `Cargo.toml`, no `package.json`, and no source files matching a card concept.

## Cross-repo notes

- **Bitmask vs. struct is the real fault line.** `pkcore::Card`, `ckc-rs`'s
  bare `CKCNumber`, and `fudd::PlayingCard` are all `Copy`-friendly integers
  (or thin wrappers over one), optimized for fast poker hand evaluation.
  `cardpack.rs::BasicCard`/`Card<DeckType>` is a heavier, field-carrying
  struct optimized for display, i18n, and per-game customization. Both
  families are alive simultaneously today — nothing has consolidated them.
- **The same shape was invented twice, independently.** `fudd::PlayingCard`
  and `pkcore::Card` are structurally identical (same derive list, same
  bit-encoding, same `CardNumber`-driven constant list) but wrap *different*
  upstream crates (`ckc-rs` vs. pkcore's own `card_number` module) and share
  no code — a duplicated invention rather than a shared dependency.
- **`BLANK`/joker handling differs by family.** The bitmask family represents
  "no card" as the sentinel value `0` (`pkcore::Card::BLANK`, ckc-rs's
  `CardNumber` has no blank constant of its own but `0` is treated the same
  way by convention). `cardpack.rs::BasicCard` instead treats "blank" as
  `Pip::BLANK_INDEX` (`'_'`) on a `Default`-constructed value — a
  string/char sentinel rather than a numeric zero. Neither family models a
  joker as a first-class variant.
- **0–51 index and raw `(Rank, Suit)` tuples are absent.** Despite being
  common conventions elsewhere in card-game code, no repo in this family uses
  a plain array index 0–51 or a bare `(Rank, Suit)` tuple as its atomic card
  representation — every repo either encodes into a bitmask, wraps a
  `Rank`+`Suit`-bearing struct, or (for pkcore's Kuhn variant) uses a
  dedicated closed enum.
- **A pkcore-internal split most consumers never see.** pkcore itself hosts
  two unrelated single-card types — the 52-card bitmask `Card` and the
  3-value `KuhnCard` enum for its Kuhn poker game — and `pkkuhn-web` is the
  only downstream repo that reaches for `KuhnCard` instead of `Card`. Every
  other consumer (pktui, pkdealer, pkarena0-web, and pkgto-web transitively
  via `Two`) uses the bitmask `Card`.
