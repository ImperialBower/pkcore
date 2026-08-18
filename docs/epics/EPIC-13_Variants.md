# EPIC-13: Poker Variants

Side quest features — not essential to the pkdealer platform, but valuable for
making pkcore a complete poker analysis library. Each variant is a self-contained
unit of work that can be picked up independently.

**Current state summary:**

| Variant | File | Status |
|---------|------|--------|
| Omaha Hi | `src/games/omaha.rs` | Working — `OmahaHigh` struct, permutations, hand ranking |
| Omaha Hi-Lo (8 or better) | `src/games/omaha.rs` | Not started |
| Razz | `src/games/razz.rs` | Stub — `CaliforniaHandRank` enum exists in `razz/california.rs` |
| Limit Hold'em | — | Not started |
| Seven-Card Stud | `src/games/stud.rs` | Empty file |
| Stud Hi-Lo (8 or better) | — | Not started |

---

## Omaha Hi-Lo (8 or Better)

**What makes it different from Omaha Hi:**
The pot is split between the best high hand and the best low hand. A low hand
qualifies only if it contains five unpaired cards all ranked 8 or lower (aces
count low). The same player can win both halves (a "scoop").

**Key evaluation rules:**
- High hand: same as `OmahaHigh` — best 5-card hand from exactly 2 hole cards
  and 3 board cards
- Low hand: best 5-card low from exactly 2 hole cards and 3 board cards, where
  all 5 cards are 8 or lower and unpaired; if no qualifying low exists, the
  high hand wins the whole pot
- Ace plays low for the low hand (A-2-3-4-5 = the wheel, best possible low)
- Straights and flushes do not count against a low hand

**Design notes:**
- `CaliforniaHandRank` in `razz/california.rs` already encodes A-5 low rankings
  and could be reused or adapted for the low-hand qualifier check
- A new `OmahaLow` struct (or extension of `OmahaHigh`) that returns
  `Option<LowHandRank>` — `None` if no qualifying low exists
- Pot splitting logic belongs in the dealer/game layer, not in hand ranking;
  `OmahaHiLo` should just return `(high: Eval, low: Option<LowEval>)`
- The 8-or-better qualifier means you need a rank filter before ranking —
  reject any hand where any of the 5 cards is ranked 9 or higher

**Known complexity:**
A player can use different hole card combinations for their high and low hands —
they are ranked independently. This means iterating all 60 permutations twice.

---

## Razz

**What it is:** Seven-Card Stud played for low. Best 5-card A-5 low hand wins.
No 8-or-better qualifier — any low hand wins.

**Current state:**
- `src/games/razz.rs` and `src/games/razz/california.rs` exist
- `CaliforniaHandRank` is a fully enumerated list of all valid Razz hands
  (hundreds of variants from WHEEL up through high-card hands)
- `src/games/razz.rs` itself is a one-line stub (`pub mod california;`)

**Design notes:**
- The `CaliforniaHandRank` enum is the ranking table — the hard work is done
- Need a `RazzHand` struct wrapping `Five` that maps to a `CaliforniaHandRank`
- Hand ranking: given a `Seven`, find the best (lowest) 5-card combination —
  iterate C(7,5) = 21 combinations, rank each with `CaliforniaHandRank`,
  return the minimum
- `CaliforniaHandRank` uses `Ord` (derived) so finding the minimum is
  straightforward — lower enum variant = better Razz hand
- Street structure is Stud (see below), not community cards — Razz hand
  ranking itself is independent of the street model

**Open question:**
Does `CaliforniaHandRank` cover all 21 possible 5-of-7 combinations, including
hands with pairs/trips/quads (which are ranked worst in Razz)? If not, a
fallback for paired boards is needed.

---

## Limit Hold'em

**What makes it different from NLHE:**
Betting structure only — hand ranking is identical to No-Limit Hold'em.
The difference is entirely in the dealer/game layer, not in card ranking.

**Design notes:**
- `pkcore` already handles hand ranking for NLHE; no new ranking structs needed
- The change is in `TableCelled` / `Dealer` betting logic:
  - Pre-flop and flop: bets/raises are fixed at one small bet (e.g. 2x BB)
  - Turn and river: bets/raises are fixed at one big bet (2x small bet)
  - Maximum 4 raises per street (cap), except heads-up where raises are unlimited
- `DealerAction` may need a `Bet(u32)` variant constrained to fixed amounts,
  or the dealer layer enforces the limits when validating actions
- `ForcedBets` already exists — extend with a `BettingStructure` enum:
  `NoLimit`, `PotLimit`, `FixedLimit { small_bet, big_bet }`

**Lowest-hanging fruit:** Since hand ranking is unchanged, Limit Hold'em is
primarily a pkdealer concern (betting enforcement in `pkdealer_service`) rather
than a pkcore concern. pkcore just needs the `BettingStructure` type.

---

## Seven-Card Stud

**What it is:** Each player receives 7 cards (3 down, 4 up) with no community
cards. Best 5-card hand from personal 7 cards wins.

**Current state:** `src/games/stud.rs` is an empty file.

**Design notes:**
- Hand ranking: given a player's 7 cards, find the best 5-card hand —
  same C(7,5) = 21 combination search as NLHE `Seven`, just without
  a shared board
- A `StudHand` struct wrapping `Seven` would mirror the existing `Seven` API
  closely
- **Street model is fundamentally different from Hold'em:**
  - Third Street: 2 hole cards down, 1 up — bring-in bet from lowest up-card
  - Fourth–Sixth Street: 1 card dealt face-up per street
  - Seventh Street ("The River"): final card dealt face-down
  - Action order determined by best visible hand each street (not position)
- The bring-in and up-card visibility rules mean `TableCelled`/`Dealer` need a Stud
  mode — this is the significant complexity, not the hand ranking
- `pkcore`'s `Board` type is Hold'em-centric (flop/turn/river); Stud needs a
  different state representation for per-player up-cards

**Suggested phasing:**
1. Implement `StudHand` ranking (easy — reuses existing `Seven` machinery)
2. Stud street model and `TableCelled` support (hard — new state machine)

---

## Stud Hi-Lo (Eight or Better)

**What it is:** Seven-Card Stud with a split pot — same high/low rules as
Omaha Hi-Lo. Low hand must qualify (five unpaired cards 8 or lower).

**Design notes:**
- Depends on both Razz (for low-hand ranking) and Seven-Card Stud
  (for street model)
- Once `StudHand` and `RazzHand` exist, `StudHiLo` is largely composition:
  rank high with `StudHand` and low with `RazzHand`, return
  `(Eval, Option<LowEval>)`
- Pot splitting logic same as Omaha Hi-Lo — belongs in the dealer layer

**Implement last** — this is the most complex variant and depends on everything
above.

---

## Suggested Implementation Order

1. **Razz** — `CaliforniaHandRank` is already there; just needs `RazzHand` and
   the C(7,5) ranking loop
2. **Omaha Hi-Lo** — `OmahaHigh` is solid; extend with low qualifier
3. **Limit Hold'em** — add `BettingStructure` to pkcore; enforcement in pkdealer
4. **Seven-Card Stud** — hand ranking is easy; street model is the hard part
5. **Stud Hi-Lo** — compose from Razz + Stud

---

## Shared Infrastructure Needed

- **`LowEval` / low-hand ranking type** — reusable across Razz, Omaha Hi-Lo,
  and Stud Hi-Lo; wraps `CaliforniaHandRank`
- **`BettingStructure` enum** — needed for Limit Hold'em; useful for Pot-Limit
  Omaha too
- **C(7,5) best-hand iterator** — already exists in `Seven`; confirm it can be
  reused directly for Stud without modification
