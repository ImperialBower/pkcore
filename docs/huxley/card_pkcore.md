# Huxley: Card (pkcore)

**Report date:** 2026-09-25  
**Repo:** [pkcore](https://github.com/ImperialBower/pkcore) (HEAD: [`78a45ab1`](https://github.com/ImperialBower/pkcore/commit/78a45ab121e05221ac2ede3d1ea95f4260aa4e2f))

## Summary

`Card` is a newtype wrapper around `u32` that encodes poker cards using the Cactus Kev binary format. Born in pkcore on 2022-05-15 as `Card::from(u32)`, it evolved rapidly over 2 days with `is_blank()`, `get_rank()`, and private card number constants. The design is stable: a zero-cost wrapper with bit-mask accessors for rank, suit, and prime factorization, used throughout hand evaluation.

## Timeline

| Date | Commit | What changed |
|------|--------|--------------|
| 2022-05-15 | [`80cccc64`](https://github.com/ImperialBower/pkcore/commit/80cccc6436a3c46ba50f6406c9ad347f4d5952e2) | **Card born** — `impl From<u32>` adds validation |
| 2022-05-15 | [`01e2401f`](https://github.com/ImperialBower/pkcore/commit/01e2401f4cf446a6496d477566153aed77159814) | `is_blank()` method detects invalid cards |
| 2022-05-16 | [`7f649537`](https://github.com/ImperialBower/pkcore/commit/7f649537e0cfd8701f989dc82e6c8b6e90b8340e) | `get_rank()` extracts rank from bit encoding |
| 2022-05-16 | [`1b388d0f`](https://github.com/ImperialBower/pkcore/commit/1b388d0f3740a6ae30dc90b41aa4d4f6ab810a75) | Card number constants moved to private module |
| 2022-05-17 | [`a6541a1d`](https://github.com/ImperialBower/pkcore/commit/a6541a1d6368172723d2ea9c524cd501000c24f0) | Card example use in repl — feature complete |

## Eras

### Era 1: Birth and API Solidification (2022-05-15 to 2022-05-17)

**Birth: Card::from(u32)**

[view on GitHub](https://github.com/ImperialBower/pkcore/blob/80cccc6436a3c46ba50f6406c9ad347f4d5952e2/src/card.rs#L590-L620)

```rust
/// Filters u32 so that only valid Cactus Kev Card values are set.
impl From<u32> for Card {
    fn from(ckc_number: u32) -> Self {
        let ckc_number = match ckc_number {
            Card::ACE_SPADES
            | Card::KING_SPADES
            // ... all 52 valid cards
        };
        Card(ckc_number)
    }
}
```

Card enters as a validation wrapper. Only valid Cactus Kev values pass through `From<u32>`. All 52 card constants are public at this point.

**is_blank() helper added:**

[view on GitHub](https://github.com/ImperialBower/pkcore/blob/01e2401f4cf446a6496d477566153aed77159814/src/card.rs#L57-L59)

```rust
fn is_blank(&self) -> bool {
    self.0 == Card::BLANK_NUMBER
}
```

One-liner to detect the sentinel invalid card. Used in collections that accept blank spots.

**get_rank() extraction:**

[view on GitHub](https://github.com/ImperialBower/pkcore/blob/7f649537e0cfd8701f989dc82e6c8b6e90b8340e/src/card.rs#L70-L72)

```rust
pub fn get_rank(&self) -> Rank {
    Rank::from_prime(self.get_rank_prime())
}
```

Unpacks rank from the bit encoding. Rank is computed from the prime field baked into every card.

**Card numbers made private:**

[view on GitHub](https://github.com/ImperialBower/pkcore/blob/1b388d0f3740a6ae30dc90b41aa4d4f6ab810a75/src/card.rs#L55-L110)

```rust
// Before
pub const ACE_SPADES_NUMBER: u32 = 268_471_337;
pub const ACE_SPADES: Card = Card(Card::ACE_SPADES_NUMBER);

// After
const ACE_SPADES_NUMBER: u32 = 268_471_337;
pub const ACE_SPADES: Card = Card(ACE_SPADES_NUMBER);
```

Internal constants hidden. Public API is now `Card::ACE_SPADES`, not the raw u32 numbers. Signals that raw numbers are an implementation detail.

### Era 2: Stable API (2022-05-17 onwards)

After the initial 2-day sprint, the API stabilizes. Card remains a thin, zero-cost wrapper:
- **Bit masks** for rank, suit, prime extraction
- **Constants** for all 52 cards + blank
- **Display**, **From<u32>**, **Copy/Clone** traits
- No allocation, no heap use

Recent changes are test formatting only — the type is feature-complete.

## Today

**Live code:** [`src/card.rs`](https://github.com/ImperialBower/pkcore/blob/78a45ab121e05221ac2ede3d1ea95f4260aa4e2f/src/card.rs)

Card is heavily used throughout:
- `Five::hand_rank_value()` extracts suit and rank flags
- `Cards` collection uses Card constants
- `Rank` and `Suit` traits depend on Card's bit encoding
- Serialization uses custom deserialize to validate

The Cactus Kev encoding — baked into Card on birth — has never changed.
