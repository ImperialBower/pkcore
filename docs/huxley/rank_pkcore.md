# Huxley: Rank (pkcore)

**Report date:** 2026-09-25  
**Repo:** [pkcore](https://github.com/ImperialBower/pkcore) (HEAD: [`78a45ab1`](https://github.com/ImperialBower/pkcore/commit/78a45ab121e05221ac2ede3d1ea95f4260aa4e2f))

## Summary

`Rank` is an enum representing the 13 poker ranks (ACE through TWO) plus BLANK for invalid slots. Born on 2022-05-15 with full API—no evolution since. Provides bit masks, Cactus Kev number encoding, prime factorization, and parsing from char. Immutable and zero-cost like `Card`.

## Timeline

| Date | Commit | What changed |
|------|--------|--------------|
| 2022-05-15 | [`f004888e`](https://github.com/ImperialBower/pkcore/commit/f004888e1fd777e2d17629f385c366f10ee3fa74) | **Rank born** — enum with bits(), number(), prime(); From<char> and FromStr |
| 2022-05-16 | [`755a7abe`](https://github.com/ImperialBower/pkcore/commit/755a7abebd1556604149454b9e3beda43f1cc141) | Full test coverage added |

## Eras

### Era 1: Complete Birth (2022-05-15)

**Rank enum and API:**

[view on GitHub](https://github.com/ImperialBower/pkcore/blob/f004888e1fd777e2d17629f385c366f10ee3fa74/src/rank.rs#L6-L21)

```rust
#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum Rank {
    ACE = 14,
    KING = 13,
    QUEEN = 12,
    JACK = 11,
    TEN = 10,
    NINE = 9,
    EIGHT = 8,
    SEVEN = 7,
    SIX = 6,
    FIVE = 5,
    FOUR = 4,
    THREE = 3,
    TWO = 2,
    BLANK = 0,
}
```

Rank is born complete. 14 variants (13 poker ranks + blank). Values are poker hand order (`ACE=14` high). Derives `EnumIter` for table iteration.

**Encoding methods:**

[view on GitHub](https://github.com/ImperialBower/pkcore/blob/f004888e1fd777e2d17629f385c366f10ee3fa74/src/rank.rs#L23-L60)

```rust
impl Rank {
    pub fn bits(self) -> u32 {
        1 << (16 + self.number())
    }

    pub fn number(self) -> u32 {
        match self {
            Rank::ACE => 12,
            Rank::KING => 11,
            // ... maps to Cactus Kev 0-12 encoding
        }
    }

    pub fn prime(self) -> u32 {
        match self {
            Rank::ACE => 41,
            Rank::KING => 37,
            Rank::QUEEN => 31,
            // ... prime numbers for fast hand eval
        }
    }
}
```

Three methods that power hand evaluation:
- `bits()` — converts to bitmask position used in `Card` and `Five`
- `number()` — Cactus Kev encoding (0-12 for deuce-ace)
- `prime()` — prime factorization. Only one rank has any given prime, enabling fast hand typing

**Parsing:**

[view on GitHub](https://github.com/ImperialBower/pkcore/blob/f004888e1fd777e2d17629f385c366f10ee3fa74/src/rank.rs#L62-L85)

```rust
impl From<char> for Rank {
    fn from(char: char) -> Self {
        match char {
            'A' | 'a' => Rank::ACE,
            'K' | 'k' => Rank::KING,
            // ... case-insensitive single-char parsing
        }
    }
}

impl FromStr for Rank {
    type Err = PKError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse single-character rank, error on multi-char input
    }
}
```

Two parsing paths: `From<char>` for direct char (used in `Card::from_char`), `FromStr` for error handling (used in user input).

### Era 2: Stable (2022-05-16 onwards)

Tests added next day. No shape changes since. Rank is stable, immutable API.

## Today

**Live code:** [`src/rank.rs`](https://github.com/ImperialBower/pkcore/blob/78a45ab121e05221ac2ede3d1ea95f4260aa4e2f/src/rank.rs)

Rank is used everywhere:
- `Card::get_rank()` returns a Rank
- `Five::hand_rank_value()` uses `Rank::prime()` for hand classification
- `Suit` companion type (parallel structure)
- Display and serialization via prime/bits encoding

The design—primes for fast hand typing, bits for masking—is Cactus Kev. Never changed.
