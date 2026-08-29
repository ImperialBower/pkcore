# EPIC-87: Pluribus-Format Hand Export (UNUM)

> **Provenance.** Drafted 2026-08-29 against `pkcore` `main` @ `28f214d`
> (version `0.9.1`, `Cargo.toml:4`), working tree clean. Corpus counts quoted
> below were measured at that commit against `data/pluribus/raw/*.log`
> (92 files, 10,000 `STATE:` lines).
>
> **Built 2026-08-29 on branch `EPIC-87`, shipped as `0.10.0`.** Every phase
> landed. Two findings changed the exit criteria and one changed the design;
> all three are recorded in the [Corrigendum](#corrigendum) at the foot of this
> document rather than edited silently into the text above them.

## Context

`pkcore` reads the Pluribus log format and cannot write it. The asymmetry is
structural, not accidental — it is baked into the trait:

```rust
// src/lib.rs:977-985
pub trait Plurable {
    fn from_pluribus(s: &str) -> Result<Self, PKError>
    where
        Self: Sized;
}
```

One method, one direction. Six types implement it — `Two`
(`src/arrays/two.rs:1552`), `Three` (`src/arrays/three.rs:71`), `Four`
(`src/arrays/four.rs:151`), `Five` (`src/arrays/five.rs:283`), `Board`
(`src/play/board.rs:70`), `HoleCards` (`src/play/hole_cards.rs:274`) — and not
one of them can render back out.

The same holds one level up. `Pluribus` parses a full log line
(`FromStr`, `src/analysis/nubibus.rs:764-789`), but its `Display`
(`src/analysis/nubibus.rs:754-762`) is a debugging shape, not the format:

```rust
"#{} rounds: {:?} HANDS: {} BOARD: {} WINNINGS: {:?} PLAYERS: {:?}"
```

So today the round trip `line → Pluribus → line` is impossible, and the longer
trip `line → Table → line` — which is what would actually prove the replay
engine correct — has never been attempted.

### Why this is worth doing now

The replay engine has been wrong three times, and each time the bug was found
by accident rather than by a test.

- [`DEFECT_020`](../defects/DEFECT_020_nubificus_act_discards_results.md) —
  `Nubificus::act` discarded every action `Result`. Propagating them
  immediately failed **291 of the 10,000 corpus hands**.
- [`DEFECT_021`](../defects/DEFECT_021_pluribus_cumulative_amounts.md) — logged
  raise amounts are cumulative per-hand totals, not per-street bets. The two
  readings coincide on the first street with action, which is the only shape any
  unit fixture had (`src/analysis/nubibus.rs:77-106`).
- [`DEFECT_022`](../defects/DEFECT_022_next_to_act_restarts_under_the_gun.md) —
  found in the same sweep.

The pattern is the tell: every one of these survived because replay had no
*output* to compare against. A hand that replays into a wrong table still
replays silently. An exporter changes that — it turns 10,000 archived hands
into 10,000 assertions, because each one carries its own expected answer in
`Pluribus.raw` (`src/analysis/nubibus.rs:504`), the untouched original line.

This is the Gold Standard applied to a parser: a real behavioural change to the
replay engine *should* make a previously-passing test fail. Right now it does
not, because there is no test that can see it.

### The author's own open question

`Pluribus::parse_all_rounds` carries this note
(`src/analysis/nubibus.rs:528-530`):

> *"I have a theory that the divider between rounds isn't needed. That we can
> just take a vector of all the actions, and they pause when the round is
> over."*

The theory has never been tested, because nothing ever needed to reconstruct
the dividers. An exporter needs exactly that: given a flat
`VecDeque<PluribusEvent>` (`src/analysis/nubibus.rs:499`) and a table, re-derive
where each `/` goes. **Phase 3 settles the theory across all 10,000 hands, one
way or the other.** That alone justifies the EPIC.

### What this EPIC does NOT do

- **Not PHH or OHH.** [EPIC-19a](EPIC-19a_SIDEQUEST_Mutants.md) evaluates those
  standards against `HandHistory`. Untouched here.
- **Not the compact `HE:` format.** That is [EPIC-66](EPIC-66_Serialization.md),
  still a 12-line stub. Untouched here.
- **Not the PokerStars form.** `data/pluribus/converted_logs/*.txt` is a
  third-party conversion to PokerStars hand histories. Read-only reference data;
  this EPIC neither reads nor writes it.
- **No new dependency, no new feature flag, no I/O in the kernel.** Export is
  pure `String` formatting. File writing lives in an example, not in `src/`.
  `make check-purity` must stay green with nothing added to it.
- **Not a `HandHistory` bridge.** `HandHistory` (`src/hand_history.rs:128`) is
  serde/YAML behind the `hand-histories` feature. Converting between it and the
  Pluribus format is a reasonable follow-on and is explicitly deferred.
- **Six-max no-limit hold'em only.** The format has no vocabulary for other
  seat counts, other variants, antes, or straddles. Anything else is an error,
  not a best-effort render.

---

## Status

| Component | Status |
|---|---|
| `Unumable` trait (`to_pluribus`) | ✅ Done — `src/lib.rs:988` |
| `Card` → `As` primitive (lowercase suit) | ✅ Done — `src/card.rs:270` |
| `Unumable` for `Two`/`Three`/`Four`/`Five` | ✅ Done |
| `Unumable` for `HoleCards` (re-inserts `\|`) | ✅ Done |
| `Unumable` for `Board` (truncates at blanks) | ✅ Done |
| `PluribusEvent` → `f` / `c` / `r<n>` | ✅ Done |
| Street-divider re-derivation (the `/`) | ✅ Done — both strategies built |
| `Pluribus::to_pluribus()` — full `STATE:` line | ✅ Done |
| **Tier 1** textual round-trip, 10,000 hands | ✅ **9,992 / 10,000** |
| `Table` → `Pluribus` (cumulative-amount inversion) | ✅ Done |
| **Tier 2** semantic round-trip via `Nubificus` | ✅ **9,901 / 10,000**, zero unexplained |
| **Tier 3** novel export of a pkcore-dealt hand | ✅ Done — `dealt_hand_exports_and_reimports` |
| Half-chip payoffs (8 corpus hands) | ⚖️ **Design option 3** — accepted, named, excluded |
| Hole-card order within a player | ⚖️ **New finding** — cannot round-trip, see C-1 |
| All-in run-out (92 hands) | 🐛 **New finding** — engine gap, see C-3 |
| Divider theory (`nubibus.rs:528-530`) | ✅ **Confirmed 10,000 / 10,000** |
| File-level writer (4 header lines + N states) | ✅ Done — `Pluribus::write_log` |
| `examples/unum.rs` corpus verifier | ✅ Done |
| `HandHistory` ⇄ Pluribus bridge | 🔒 Deferred — separate EPIC |

---

## Goals

- Give `pkcore` a **byte-exact writer** for the Pluribus log format, symmetric
  with the reader it already has.
- Turn the **10,000-hand corpus into a regression suite** by round-tripping it,
  so the replay engine can no longer be silently wrong.
- Settle the **street-divider theory** stated at
  `src/analysis/nubibus.rs:528-530` with evidence rather than intuition.
- Let a **pkcore-dealt hand** be written out in a format the wider poker-AI
  research world already reads.
- Add all of it **inside the domain kernel** — no serde, no filesystem, no new
  crate.

## Scope

The format is specified by `data/pluribus/raw/README.md`. These are the rules
the writer must obey, each with the line of the spec that states it:

1. **Six colon-delimited fields**, exactly:
   `STATE:<index>:<actions>:<cards>:<payoffs>:<players>`. The reader enforces
   the count already (`parse_string`, `src/analysis/nubibus.rs:589-596`).
2. **Action alphabet is `f`, `c`, `r<digits>`, `/`** and nothing else —
   confirmed empirically: stripping digits from all 10,000 action fields yields
   the character set `{/, c, f, r}` and no other.
3. **Raise amounts are cumulative per-hand totals**, "the total number of chips
   that player has in the pot after the raise (including money from all prior
   betting rounds in this hand)" (`data/pluribus/raw/README.md:15-18`). The
   writer must invert `Nubificus::street_bet_target`
   (`src/analysis/nubibus.rs:96-106`), which converts the other way.
4. **`/` separates betting rounds**, not streets-with-cards. A round with no
   action still terminates.
5. **Cards**: two chars per card, rank in `23456789TJQKA`, suit in **lowercase**
   `shdc`. Hole cards pairwise, `|`-separated; then `/flop3/turn/river`
   (`data/pluribus/raw/README.md:20-32`).
6. **A board card the hand never reached is not written.** 4,662 of the 10,000
   corpus hands ended pre-flop and carry no `/` at all in the cards field.
7. **Payoffs** are `|`-separated signed amounts, net per player
   (`data/pluribus/raw/README.md:34-35`). Split pots are logged to half a chip
   (`287.5`) — 8 hands in the corpus.
8. **Player order is positional, not seat-indexed**: index 0 is the small
   blind, index 1 the big blind, index 2 first to act pre-flop
   (`data/pluribus/raw/README.md:37-41`). The button is therefore the *last*
   index — which is precisely what the importer encodes with
   `table.button = table.seats.size().saturating_sub(1)`
   (`src/analysis/nubibus.rs:450`, documented at `:398-408`).

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| A card, `As` | `Card` + new `Unumable` | ❌ absent (`get_letter_index` is close, wrong case) |
| A player's hole cards | `Two` : `Plurable` | 🟡 read-only (`src/arrays/two.rs:1552`) |
| All six players' hole cards | `HoleCards` : `Plurable` | 🟡 read-only (`src/play/hole_cards.rs:274`) |
| The board, street by street | `Board` : `Plurable` | 🟡 read-only (`src/play/board.rs:70`) |
| One logged action | `PluribusEvent` | 🟡 `Display` is `"Raise(200)"`, not `"r200"` (`:825-833`) |
| The action sequence | `Pluribus.actions` (flat) + `.rounds` (split) | 🟡 both parsed, neither rendered |
| One logged hand | `Pluribus` | 🟡 `FromStr` yes (`:764`), writer no |
| A replayed hand | `Nubificus` + `Table` | 🟡 replays; cannot report what it replayed |
| A log file | — | ❌ absent (reading is `read_in_log`, `:739`) |

---

## Design

### `Unumable` — the write half of `Plurable`

`src/lib.rs` (extend), sitting immediately below `Plurable`:

```rust
/// The other half of [`Plurable`]. *E pluribus unum* — out of many, one:
/// `Plurable` takes one Pluribus string apart, `Unumable` puts one back
/// together.
pub trait Unumable {
    /// Renders `self` as its fragment of a Pluribus log line.
    ///
    /// Infallible by construction: every implementor is a valid poker object,
    /// and every valid poker object has a rendering. Blank cards are the one
    /// wrinkle, and they are *elided*, not errors — see [`Board`].
    #[must_use]
    fn to_pluribus(&self) -> String;
}
```

**Why a sibling trait and not a second method on `Plurable`.** `Plurable` is
public and exported through the prelude; adding a required method breaks any
downstream implementor at compile time. A defaulted method that returns an
empty string or an `Err` would be worse — it would let a type silently claim a
capability it does not have, which is the exact failure mode `DEFECT_020` was.
Two traits keeps "can be read" and "can be written" separately provable, and
the six existing `Plurable` impls stay untouched.

**Why infallible.** A fallible `to_pluribus` pushes a `Result` through every
call site of a pure formatter, and there is no genuine failure: the invalid
cases (wrong seat count, wrong variant) are caught one level up, at
`Pluribus::try_from(&Table)`, where a `Result` is honest.

### `Card` — the primitive that does not exist yet

`src/card.rs` (extend):

```rust
impl Unumable for Card {
    /// `Ah`, `Ts`, `2c`. Note the **lowercase** suit.
    fn to_pluribus(&self) -> String {
        format!(
            "{}{}",
            self.get_rank().to_char(),
            self.get_suit().to_char_letter().to_ascii_lowercase()
        )
    }
}
```

`Card::get_letter_index` (`src/card.rs:184-186`) is one `to_ascii_lowercase`
away from being this — it emits `AS`, because `Suit::to_char_letter` returns
uppercase (`src/suit.rs:34-42`). **Do not reuse `get_letter_index` and hope.**
It is used elsewhere for a different purpose, and the near-miss is exactly the
kind of thing that produces a writer which is 99.9% right — the worst possible
outcome for a format whose only test is byte equality.

### `HoleCards` — re-inserting what parsing threw away

`src/play/hole_cards.rs` (extend):

```rust
impl Unumable for HoleCards {
    /// `Qc4h|Tc9c|8sAs|Qh7c|JcQd|5h5d`
    fn to_pluribus(&self) -> String {
        self.iter().map(Two::to_pluribus).collect::<Vec<_>>().join("|")
    }
}
```

Worth stating plainly, because it is the first place round-tripping bites:
`HoleCards::from_pluribus` (`src/play/hole_cards.rs:276-282`) **discards the
`|` separators** — it splits on `|`, joins the pieces back together, and then
re-splits every 2 characters. The player boundaries survive only because every
player holds exactly two cards. The writer must re-impose that structure from
the `Two`-per-player shape of the type, and the round-trip test is what proves
the reconstruction lands in the same places.

### `Board` — truncation, not padding

`src/play/board.rs` (extend):

```rust
impl Unumable for Board {
    /// `3h7s5c/Qs/6c`, `3h7s5c/Qs`, `3h7s5c`, or `""` for a hand that
    /// never saw a flop.
    fn to_pluribus(&self) -> String { /* stop at the first BLANK */ }
}
```

`Board::from_pluribus` (`src/play/board.rs:70-99`) pads absent streets with
`Card::BLANK`. The writer inverts that: emit up to the first blank and stop. It
must never render `_` (`Suit::BLANK.to_char_letter()`, `src/suit.rs:40`) into a
log line. 4,662 of 10,000 corpus hands exercise the empty case, so this path is
not an edge case — it is the plurality.

### `PluribusEvent` — and the dividers

`src/analysis/nubibus.rs` (extend):

```rust
impl Unumable for PluribusEvent {
    fn to_pluribus(&self) -> String {
        match self {
            PluribusEvent::Fold => "f".to_string(),
            PluribusEvent::Call => "c".to_string(),
            PluribusEvent::Raise(amount) => format!("r{amount}"),
        }
    }
}
```

Deliberately *not* a change to `Display` (`src/analysis/nubibus.rs:825-833`),
which renders `Raise(200)` for humans and log lines. Two renderings, two
traits; changing `Display` would break every existing debug output and the
`Nubificus` `Display` that embeds it (`:486-488`).

The dividers are the real work, and they are **not** recoverable from
`PluribusEvent` alone — a `c` that closes a street and a `c` that continues one
are the same token. Deriving them needs the table:

```rust
impl Pluribus {
    /// Renders the action field: events joined, with `/` inserted wherever a
    /// betting round closed.
    ///
    /// # Errors
    ///
    /// `PKError::InvalidPluribusIndex` if the event sequence does not
    /// correspond to a legal hand — i.e. the dividers cannot be placed.
    fn actions_to_pluribus(&self) -> Result<String, PKError>;
}
```

**Two candidate implementations, and the EPIC must pick by experiment:**

1. **Re-simulate.** Feed the events through a `Table` exactly as
   `Nubificus::do_action` does (`src/analysis/nubibus.rs:236-363`) and emit `/`
   whenever `seats.is_betting_complete()` flips. Correct by construction;
   couples the writer to the whole engine.
2. **Trust the theory** at `src/analysis/nubibus.rs:528-530` — that round
   boundaries fall out of the action sequence itself — and place dividers from
   a fold/call/raise counter alone. Cheap and decoupled, if it holds.

Build (1) first because it is verifiable, then run (2) against all 10,000 hands
as a *hypothesis test* in Phase 3. If (2) agrees with (1) everywhere, the
theory is confirmed and the note at `:528` gets rewritten from theory to fact.
If it disagrees, record where and how — that counterexample is worth more than
the optimisation.

> **Do not reconstruct the action field from `Pluribus.rounds`**
> (`src/analysis/nubibus.rs:498`). Those are the pre-split source substrings;
> `rounds.join("/")` reproduces the input by remembering it, and a round-trip
> test built on it asserts nothing. The writer renders from `actions`
> (`:499`) — the parsed form — or it is not a test.

### `Pluribus::to_pluribus` — the whole line

```rust
impl Unumable for Pluribus {
    /// The full `STATE:` line. Byte-identical to `self.raw` for any
    /// `Pluribus` parsed from a well-formed line, half-chip payoffs excepted.
    fn to_pluribus(&self) -> String;
}
```

The oracle is free: `Pluribus.raw` (`src/analysis/nubibus.rs:504`) already
holds the original line, kept per hand by `FromStr` (`:783`). Every parsed hand
is therefore a self-checking fixture, and 10,000 of them are already in the
repo.

### The half-chip problem — name it, gate it

`Pluribus.winnings` is `Vec<isize>` (`src/analysis/nubibus.rs:502`), and
`parse_isizes` (`:514-526`) truncates `287.5` to `287` on purpose, with the
reasoning recorded in place. That is fine for replay — a chip is a chip — but
it makes a byte-exact round trip **impossible** for the 8 corpus hands that
split a pot to half a chip.

Three options, in preference order:

1. **Store the payoff in half-chips** — `winnings: Vec<isize>` counting halves,
   rendered `/2` with a `.5` when odd. Exact, contained, and touches only
   `parse_isizes` and the writer.
2. **Keep the raw string** alongside the parsed value and re-emit it.
   Trivially correct, and dishonest — the writer would be transcribing, not
   rendering, which is the `rounds.join("/")` mistake wearing a hat.
3. **Accept the divergence**, exclude those 8 hands from Tier 1 by name, and
   record it as a known limitation the way `DEFECT_008`'s `D8-6` was.

Take (1) unless it forces a public-API break; fall back to (3), never (2). The
Status row stays `🔒 Gated` until this is decided against real code.

### `Table` → `Pluribus` — inverting DEFECT_021

`src/analysis/nubibus.rs` (extend), the mirror of
`TryFrom<&Pluribus> for Table` (`:413-458`):

```rust
impl TryFrom<&Table> for Pluribus {
    type Error = PKError;

    /// Rebuilds the log line a finished hand would have produced.
    ///
    /// # Errors
    ///
    /// `PKError::InvalidPluribusIndex` if the table is not a finished six-max
    /// NLH hand — the only shape the format can express.
    fn try_from(table: &Table) -> Result<Self, Self::Error>;
}
```

Source of truth is `Table::event_log: Vec<TableAction>`
(`src/casino/table.rs:103`). The mapping is where the sharp edges live:

| `TableAction` | Pluribus | Note |
|---|---|---|
| `Fold(seat)` | `f` | |
| `Check(seat)` | `c` | the format has no separate check token |
| `Call(seat, _)` | `c` | |
| `Bet(seat, n)` / `Raise(seat, n)` | `r<cumulative>` | **not** `n` — see below |
| `AllIn(seat, n)` | `r<cumulative>` | indistinguishable from a raise in the format |
| `ForcedBetSmallBlind` / `ForcedBetBigBlind` | *(nothing)* | blinds are implied, never written |
| everything else | *(nothing)* | deal/pot/showdown actions have no representation |

The amount conversion is `street_bet_target`
(`src/analysis/nubibus.rs:96-106`) run backwards: that function subtracts what
earlier streets took (`chips_in_play - bet`) to get a street target from a
logged total, so the writer adds it back. This is the single most likely place
for a `DEFECT_021`-shaped bug to reappear in mirror image, and Tier 2 is aimed
squarely at it.

`TableAction` is `#[non_exhaustive]` (`src/casino/action.rs:87`), so the match
needs a `_ =>` arm. Make that arm *silent-skip*, and enumerate every skipped
variant explicitly above it — a wildcard that quietly swallows a new betting
variant is how a future action goes missing from an exported hand.

### File level

```rust
impl Pluribus {
    /// The four `#` header lines a Pluribus log file opens with, plus one
    /// `STATE:` line per hand.
    #[must_use]
    pub fn write_log(session: &str, hands: &[Pluribus]) -> String;
}
```

Returns a `String`. **It does not touch the filesystem** — `read_in_log`
(`src/analysis/nubibus.rs:739-751`) reads because reading was already there;
writing does not get to add an I/O path to the kernel. `examples/unum.rs` owns
the `fs::write`.

---

## Work Items

### Phase 0 — The trait seam

- [x] **0a.** Add `Unumable` to `src/lib.rs` directly below `Plurable`
      (`src/lib.rs:977-985`), with the *e pluribus unum* doc comment.
- [x] **0b.** Export it from `src/prelude.rs:70`, alongside `Plurable`.
- [x] **0c.** Confirm `cargo check --no-default-features` and
      `make check-purity` are both green — the trait must add nothing to the
      kernel's dependency surface.

### Phase 1 — Card primitives

- [x] **1a.** `impl Unumable for Card` (`src/card.rs`, near
      `get_letter_index` at `:184`), lowercasing the suit letter.
- [x] **1b.** `impl Unumable for Two` / `Three` / `Four` / `Five`, each placed
      immediately after its existing `Plurable` impl
      (`src/arrays/two.rs:1552`, `three.rs:71`, `four.rs:151`, `five.rs:283`).
- [x] **1c.** `impl Unumable for HoleCards` (`src/play/hole_cards.rs`, after
      `:274`) — `|` between players.
- [x] **1d.** `impl Unumable for Board` (`src/play/board.rs`, after `:70`) —
      `/` between streets, stop at the first `Card::BLANK`, empty string for an
      empty board.
- [x] **1e.** Unit tests per impl (see Test Plan), plus a doc test on every new
      public method — required by `CLAUDE.md`, not optional.
- [x] **1f.** Symmetry tests: for each of the six types, a case asserting
      `T::from_pluribus(s).to_pluribus() == s`.

### Phase 2 — Actions

- [x] **2a.** `impl Unumable for PluribusEvent`
      (`src/analysis/nubibus.rs`, near `:825`). Leave `Display` alone.
- [x] **2b.** Implement `Pluribus::actions_to_pluribus` by **re-simulation**
      (design option 1), reusing the phase transitions from
      `Nubificus::do_action` (`:278-361`) rather than restating them.
- [x] **2c.** Implement the counter-only divider placement (design option 2)
      as a private `fn divider_hypothesis`, used by tests only.
- [x] **2d.** Assert in tests that `actions_to_pluribus` never reads
      `self.rounds` (`:498`) — a grep-level guard in the test, plus a comment
      at the field explaining why.

### Phase 3 — The line, and Tier 1

- [x] **3a.** `impl Unumable for Pluribus` — assemble all six fields.
- [x] **3b.** Decide the half-chip question (Design, three options). If option
      1: change `parse_isizes` (`:514-526`) to half-chips and update its
      comment, which currently documents the truncation as intentional.
- [x] **3c.** **Tier 1 corpus round-trip**: for all 10,000 hands across the 92
      files, `Pluribus::from_str(line).to_pluribus() == line`. Report the
      failure count, not just pass/fail — a count is a number you can watch go
      down.
- [x] **3d.** Run the divider hypothesis (2c) against all 10,000 and record the
      result in this EPIC's corrigendum. Rewrite the note at
      `src/analysis/nubibus.rs:528-530` from theory to finding, either way.
- [x] **3e.** Mark Tier 1 as a `#[ignore]`d heavy test if it is slow, and wire
      it into `tests/heavy_tests.rs` where the other corpus-scale work lives.

### Phase 4 — Table → line, and Tiers 2 and 3

- [x] **4a.** `impl TryFrom<&Table> for Pluribus`, mirroring
      `TryFrom<&Pluribus> for Table` (`:413-458`) and inverting
      `street_bet_target` (`:96-106`).
- [x] **4b.** Enumerate every `TableAction` variant explicitly before the
      `_ =>` arm required by `#[non_exhaustive]` (`src/casino/action.rs:87`).
- [x] **4c.** **Tier 2 semantic round-trip**: parse → `Nubificus` → replay to
      completion → `Pluribus::try_from(&table)` → render → compare to `raw`.
      Across the full corpus.
- [x] **4d.** **Tier 3 novel export**: deal a hand with `Table`/`Dealer` that
      never came from a log, export it, re-import it, and assert the two tables
      agree on button, stacks, board, and payoffs.

### Phase 5 — File level and docs

- [x] **5a.** `Pluribus::write_log` — 4 header lines + N states, returning
      `String`.
- [x] **5b.** `examples/unum.rs` — the round-trip verifier, modelled on
      `examples/pluribus.rs`; owns the only `fs::write` in this EPIC.
- [x] **5c.** `CHANGELOG.md` under `## [Unreleased] / Added`, and a `minor`
      version bump in `Cargo.toml:4` (new public trait + new public API), then
      `cargo build` so `Cargo.lock` picks it up.
- [x] **5d.** Update [`EPIC_Pluribus.md`](EPIC_Pluribus.md) — it currently
      describes a read-only module — and the `EPIC_Pluribus` entry in
      [`BACKLOG.md`](../BACKLOG.md) (`:125-127`).
- [x] **5e.** Register EPIC-87 in `ROADMAP.md` and note the 80-block in the
      "EPIC Numbering Policy" section (`ROADMAP.md:406-417`), which does not
      yet mention EPIC-81 through EPIC-87 at all.

---

## Test Plan

Module `analysis__nubibus__unum_tests`, colocated in
`src/analysis/nubibus.rs` beside the existing `store_pluribus_tests`
(`:837`), with `#[cfg(test)]` + `#[allow(non_snake_case)]`.

- `card_renders_lowercase_suit` — `Card::from_str("As").to_pluribus() == "as"`
  is **wrong**; asserts `"As"`. Pins the `get_letter_index` near-miss
  (`src/card.rs:184`).
- `board_omits_unreached_streets` — flop-only board renders `3h7s5c`, no
  trailing `/`; an empty board renders `""`.
- `board_never_renders_blank_suit` — no output of any `Board` contains `_`
  (`src/suit.rs:40`).
- `hole_cards_restore_player_boundaries` — six `Two`s render with exactly five
  `|`, inverting the join-and-resplit at `src/play/hole_cards.rs:276-282`.
- `pluribus_event_renders_log_token_not_display` — `Raise(200)` →`"r200"`,
  while `Display` still yields `"Raise(200)"`. Guards `:825-833` from drift.
- `raise_amounts_are_cumulative_not_per_street` — the `DEFECT_021` worked
  example (`STATE:154:fr250ffr1150fc/r2050c/r3750c/r6250f`, quoted at
  `:86-91`) exports with its logged totals, not the per-street sums.
- `dividers_are_derived_not_remembered` — a `Pluribus` with `rounds` cleared
  still renders the correct action field.
- `preflop_only_hand_round_trips` — the 4,662-hand shape: no `/` in the cards
  field, action field ends without a divider.
- `split_pot_payoff_round_trips` — one of the 8 half-chip hands. Passes under
  Design option 1; `#[ignore]`d with a comment naming this EPIC under option 3.
- `corpus_round_trips_byte_exact` *(heavy)* — Tier 1, all 10,000 hands,
  asserting a zero failure count and printing the count when non-zero.
- `corpus_replays_and_re_exports` *(heavy)* — Tier 2, the same 10,000 through
  `Nubificus`.
- `dealt_hand_exports_and_reimports` — Tier 3, no log file involved.
- `six_max_only_is_enforced` — `Pluribus::try_from(&table)` on a 9-handed or
  non-NLH table returns `Err`, and does not render something plausible-looking.

---

## Key Files

| File | Role |
|---|---|
| `src/lib.rs:977` | `Plurable` lives here; `Unumable` joins it |
| `src/prelude.rs:70` | export the new trait |
| `src/card.rs:184` | `get_letter_index` — the near-miss to *not* reuse |
| `src/suit.rs:34-42` | `to_char_letter` returns uppercase; source of the case bug |
| `src/arrays/{two,three,four,five}.rs` | four `Plurable` impls gain `Unumable` siblings |
| `src/play/hole_cards.rs:274` | `\|` reconstruction |
| `src/play/board.rs:70` | street truncation at `Card::BLANK` |
| `src/analysis/nubibus.rs:495-833` | `Pluribus`, `PluribusEvent`, the writer, `TryFrom<&Table>` |
| `src/casino/table.rs:103` | `event_log` — the source of truth for Tier 3 |
| `src/casino/action.rs:88` | `TableAction`, `#[non_exhaustive]` |
| `tests/heavy_tests.rs` | home for the two corpus-scale tests |
| `examples/unum.rs` (new) | the verifier; the only file that writes to disk |
| `data/pluribus/raw/*.log` | 92 files, 10,000 hands, the entire test oracle |

## Reuse (do NOT recreate)

- `src/analysis/nubibus.rs:504` — `Pluribus.raw` already holds every original
  line. The expected values for 10,000 assertions are sitting in memory; do not
  build a fixture directory.
- `src/analysis/nubibus.rs:96-106` — `street_bet_target` is the forward
  conversion, with the `DEFECT_021` reasoning in its doc comment. Invert it;
  do not re-derive the rule from the README.
- `src/analysis/nubibus.rs:413-458` — `TryFrom<&Pluribus> for Table` fixes the
  seat/button convention (`table.button = size - 1`, `:450`). The exporter must
  read that convention from here, not restate it.
- `src/analysis/nubibus.rs:278-361` — `do_action` already knows where betting
  rounds close. Divider derivation reuses those transitions.
- `src/analysis/nubibus.rs:368-387` / `:739-751` — `get_log_files` and
  `read_in_log` enumerate and parse the corpus. Both round-trip tiers use them
  as-is.
- `examples/pluribus.rs` — the corpus-walking harness. `examples/unum.rs` is
  the same loop with a comparison instead of a print.
- `src/casino/action.rs:162` — `TableAction::commentary` shows the established
  pattern for rendering actions to text. Same shape, different vocabulary.

## Compatibility

- **Preserves** every existing public item. `Plurable` is untouched, so all six
  downstream-visible `from_pluribus` impls keep their signatures. `Display for
  Pluribus` (`:754`), `Display for PluribusEvent` (`:825`) and `Display for
  Nubificus` (`:478`) are all deliberately left alone.
- **Adds** the `Unumable` trait, eight impls, `Pluribus::write_log`, and
  `TryFrom<&Table> for Pluribus`. Minor version bump.
- **Breaks** nothing — with one asterisk: Design option 1 for half-chip payoffs
  changes the *units* of the public field `Pluribus.winnings`
  (`:502`) from chips to half-chips. That is a silent semantic break for any
  downstream reader, so if option 1 is taken it needs either a rename
  (`winnings_half_chips`) or a major bump. **Decide this in 3b, in the open.**
- **No new dependency and no new feature flag.** Pure formatting, kernel-safe.

## Dependencies

- **Blocks:** nothing.
- **Built on:** [`EPIC_Pluribus`](EPIC_Pluribus.md) (the reader);
  [EPIC-83](EPIC-83_Table_Decelled.md) (`Table.event_log` as a plain
  `Vec<TableAction>` after `TableLog` was retired — `src/casino/action.rs:13`);
  [`DEFECT_020`](../defects/DEFECT_020_nubificus_act_discards_results.md),
  [`DEFECT_021`](../defects/DEFECT_021_pluribus_cumulative_amounts.md),
  [`DEFECT_022`](../defects/DEFECT_022_next_to_act_restarts_under_the_gun.md)
  (without those three fixes, Tier 2 could not pass at all).
- **Related:** [EPIC-66](EPIC-66_Serialization.md) and
  [EPIC-19a](EPIC-19a_SIDEQUEST_Mutants.md) — the other two unbuilt hand-format
  proposals. `BACKLOG.md:115-121` notes they overlap and asks for one format to
  be picked before building. **This EPIC does not resolve that** and does not
  compete with it: Pluribus is an *existing external* format with 10,000
  archived hands, so it is an interchange target, not a house format.
  [EPIC-38](EPIC-38_Observability.md) — same `event_log` source, different sink.

## Verification

```bash
cargo test --all-features                      # unit + doc tests
cargo test --doc --all-features                # doc tests, explicitly
cargo test --all-features -- --ignored         # the two heavy corpus tiers
cargo clippy --all-features -- -D warnings
cargo run --example unum                       # corpus verifier, prints counts
make check-purity                              # no new kernel dependencies
make ayce                                      # the full local gate
```

Exit criteria:

1. `corpus_round_trips_byte_exact` passes over all **10,000** hands with a zero
   failure count — or, under Design option 3, over 9,992 with the 8 half-chip
   hands excluded **by name and by test comment**, never by a silent filter.
2. `corpus_replays_and_re_exports` passes — the replay engine's output is now
   checkable, which is the whole point of the EPIC.
3. `dealt_hand_exports_and_reimports` passes — export works for hands pkcore
   dealt itself, not only for hands it read.
4. The divider theory at `src/analysis/nubibus.rs:528-530` is resolved in
   writing, with the corpus count that settled it.
5. `make check-purity` is green and `Cargo.toml`'s dependency list is byte-identical
   to `28f214d` — the kernel gained a capability and no weight.
6. No existing test changed its expected value. If one did, that is a
   `DEFECT_0NN`, not an edit.

---

## Corrigendum

Written 2026-08-29, after the build. Three things this EPIC got wrong or did
not know when it was drafted. Recorded here rather than edited into the text
above, so the difference between what was planned and what was found stays
visible.

### C-1 — Hole-card order within a player cannot round-trip

**The EPIC assumed byte-exactness was achievable modulo half-chips. It is not.**

`Two` normalizes its two cards high-to-low on construction
(`From<[Card; 2]>`, `src/arrays/two.rs:1498`), because `As8s` and `8sAs` are
the same poker hand and must compare equal. `HoleCards` is a `Vec<Two>`, so a
writer built on it renders the canonical order, not the logged one.

Measured against the corpus: **9,843 of 10,000 hands (98.4%)** log at least one
player low-card-first; **30,057 of 60,000** individual holdings are affected.
This is not an edge case, it is the norm, and it was invisible until the first
`Two` round-trip assertion ran.

This is the type doing its job, not a bug. The resolution is that Tier 1's
oracle is a **canonicalized** line — the raw line with each player's two cards
re-ordered high-to-low by string surgery, independent of the writer — rather
than the raw line itself. Player boundaries, board, action field and payoffs
are all still held to byte equality.

The same reasoning applies to `Four`, which also sorts. `Three` does not, which
is what lets a `Board`'s flop round-trip exactly in Tier 1.

### C-2 — The dividers are not "one per round with action"

The first implementation emitted a `/` after any action that closed a betting
round, when more actions followed. That is wrong for **91 hands**: when the
last caller is all-in, the remaining rounds happen with no action in them at
all, and the log still terminates every one of them. `r10000c///` is a real
corpus line.

The rule is instead: **one `/` per betting round that occurred**, and the number
of rounds is what the board says — 0 dividers for a hand that never saw a flop,
3 for one that reached the river. Both the re-simulation
(`Pluribus::actions_to_pluribus`, from the parsed `Board`) and the pure
hypothesis (`Pluribus::divider_hypothesis`, from the fact that two players are
still live when the actions run out) now derive it that way.

### C-3 — The engine cannot finish an all-in run-out

**A new defect, surfaced by Tier 2 — which is exactly what this EPIC was for.**

When every remaining player is all-in, `Table` deals one more street and then
stalls. `is_game_over` requires `is_last_street()` (`src/casino/table.rs:1009`),
the board never reaches five cards, `end_hand` never runs, and the pot is never
awarded. Draining the state machine with repeated `Table::act()` does not help:
it advances one street and then makes no further progress.

**92 of 10,000 corpus hands** hit this. Tier 2 detects them by chip
conservation — a hand that actually finished pays out exactly what it took in,
so the net payoff column sums to zero — counts them, and asserts the count, so
a fix shows up as the number going down.

Tier 2's Tier-1-style comparison is otherwise clean: of the 10,000 hands,
**9,901 match exactly**, 91 are these stalls, 8 are half-chip splits (one hand
is both), and **zero are unexplained**.

This is not an exporter bug and was not fixed here. Tier 2 is the first thing
in the codebase that ever asked the engine to run a board out; before EPIC-87
nothing could tell that it did not. Filed as
[`DEFECT_025`](../defects/DEFECT_025_all_in_run_out_never_completes.md).

### C-4 — `Unumable for Pluribus` cannot be the re-simulation

The Design section specifies an infallible `to_pluribus` for every implementor
*and* a divider derivation that replays the hand through a `Table`, which can
fail. Those two cannot both be true of one method.

Resolved by C-2's confirmation of the divider theory, which is what makes an
infallible whole-line writer possible at all:

- `Unumable::to_pluribus` uses `divider_hypothesis` — pure, table-free,
  infallible, and now known correct on all 10,000 hands.
- `Pluribus::try_to_pluribus` uses the re-simulation and returns a `Result`.
  It is the verifier, and `to_pluribus_agrees_with_the_re_simulated_render`
  asserts the two do not drift.

### What the exit criteria actually came out as

1. `pluribus__corpus_round_trips_byte_exact` — **9,992 of 10,000**, the 8
   half-chip hands excluded by name in `HALF_CHIP_HANDS`, against the C-1
   canonicalized oracle. ✅ (the EPIC's own option-3 fallback)
2. `pluribus__corpus_replays_and_re_exports` — **9,901 exact, 0 unexplained**,
   with 91 stalls counted and asserted. ✅
3. `dealt_hand_exports_and_reimports` — ✅
4. The divider theory — **confirmed, 10,000 / 10,000**, by
   `pluribus__divider_hypothesis_matches_the_replay`. The note at
   `Pluribus::parse_all_rounds` is rewritten from theory to finding. ✅
5. `make check-purity` green; `Cargo.toml`'s dependency list byte-identical to
   `28f214d` — only the version line moved, `0.9.1` → `0.10.0`. ✅
6. No existing test changed its expected value. ✅

`make ayce` fails on two clippy lints in `src/analysis/gto/strategy_profile.rs`
and `src/arrays/matchups/masks/suit_texture.rs`. Both pre-date this branch —
verified by stashing EPIC-87 and re-running — and neither file is touched here.

