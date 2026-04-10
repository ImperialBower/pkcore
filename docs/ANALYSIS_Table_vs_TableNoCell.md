# Analysis: `Table` vs. `TableNoCell` — Interior Mutability vs. `&mut self`

**Date:** April 2026  
**Files:** `src/casino/table.rs`, `src/casino/table_no_cell.rs`  
**Examples:** `examples/the_hand.rs`, `examples/the_hand_no_cell.rs`

---

## Background

`casino::table::Table` is the core game engine for a poker hand. It manages the
deck, seat assignments, betting rounds, side pots, and showdown. It was built
using **interior mutability** — a Rust pattern where a type is mutated through a
shared `&self` reference rather than through `&mut self`.

`casino::table::TableNoCell` is a structurally identical reimplementation that
replaces every interior-mutability wrapper with plain fields and uses
conventional `&mut self` for every method that changes state. The two versions
exist to:

1. Make the interior-mutability design visible by contrast.
2. Provide a benchmark target (Cell reads/writes vs. direct field access).
3. Serve as a teaching tool — understanding one helps you understand the other.

---

## Interior Mutability in `Table`

Rust's ownership rules normally require `&mut T` to mutate `T`. Interior
mutability circumvents this with types from `std::cell` that perform a
runtime borrow check (or none at all, for `Cell<T: Copy>`):

| Wrapper | Runtime check | Use case |
|---|---|---|
| `Cell<T>` | None (`T: Copy` only) | Cheap scalar mutations |
| `RefCell<T>` | Panic on aliased `borrow_mut` | Heap-allocated mutations |

`Table` wraps every mutable piece of state in one of these:

```
Table {
    phase:          RefCell<GamePhase>          // set_phase() via borrow_mut
    seats:          Seats(Box<[SeatCell]>)      // SeatCell = RefCell<Seat>
    button:         BintCell                    // Cell<u8> with bounds checking
    deck:           CardsCell                   // RefCell<Cards>
    board:          CardsCell
    muck:           CardsCell
    pot:            Stack(Cell<usize>)          // chip stack via Cell
    bet:            Cell<usize>
    raise_increment: Cell<usize>
    event_log:      TableLog(RefCell<Vec<TableAction>>)
}
```

`Player` (inside each `SeatCell`) follows the same pattern:

```
Player {
    chips:          Stack(Cell<usize>)
    bet:            Stack(Cell<usize>)
    chips_in_play:  Cell<usize>
    state:          PlayerStateCell(Cell<PlayerState>)
}
```

The result: **every `Table` method can take `&self`**, even those that post
blinds, deal cards, log events, or collect bets.

---

## What `TableNoCell` Changes

Every wrapper is replaced with the plain type it wraps:

| `Table` field type | `TableNoCell` field type |
|---|---|
| `RefCell<GamePhase>` | `GamePhase` |
| `Seats(Box<[SeatCell]>)` | `SeatsNoCell(Vec<SeatNoCell>)` |
| `BintCell` | `u8` |
| `CardsCell` (deck/board/muck) | `Cards` |
| `Stack(Cell<usize>)` (pot/bet) | `usize` |
| `Cell<usize>` (raise_increment) | `usize` |
| `TableLog(RefCell<Vec<TableAction>>)` | `Vec<TableAction>` |
| `Player` (with Cell fields) | `PlayerNoCell` (plain `usize`/`PlayerState`) |
| `SeatCell(RefCell<Seat>)` | `SeatNoCell` |

Two interior-mutability utilities were also replaced with explicit equivalents:

- **`BintCell`** (a bounded circular counter for the dealer button) → `u8` with
  `% seats.size()` arithmetic in `button_up()`.
- **`DrainableBintCell`** (a circular iterator for dealing cards) → an explicit
  `for _ in 0..cards_per { for step in 0..seat_count { ... } }` loop.
- **`Stack::divvy_up`** (splits a pot using interior mutability) → a standalone
  `fn divvy_up(total: usize, by: usize) -> Vec<usize>` function.

---

## API Surface: `&self` vs. `&mut self`

The signature change is the most visible consequence of the design difference.

### `Table` (interior mutability)

```rust
// All mutating methods still take &self:
pub fn act_forced_bets(&self) -> Result<(), PKError>
pub fn deal_cards_to_seats(&self) -> Result<(), PKError>
pub fn act_bet(&self, seat_number: u8, amount: usize) -> Result<usize, PKError>
pub fn bring_it_in(&self) -> Result<usize, PKError>
pub fn end_hand(&self) -> Result<Winnings, PKError>
```

A single `&Table` reference can be passed to any number of functions, and each
can independently mutate through Cells without conflict — as long as no
`RefCell` is double-borrowed at runtime.

### `TableNoCell` (traditional mutability)

```rust
// Mutating methods require &mut self:
pub fn act_forced_bets(&mut self) -> Result<(), PKError>
pub fn deal_cards_to_seats(&mut self) -> Result<(), PKError>
pub fn act_bet(&mut self, seat_number: u8, amount: usize) -> Result<usize, PKError>
pub fn bring_it_in(&mut self) -> Result<usize, PKError>
pub fn end_hand(&mut self) -> Result<Winnings, PKError>
```

The borrow checker enforces at compile time that no other reference to the
table exists while a mutating method runs.

---

## Ergonomic Impact at Call Sites

The most concrete illustration is the phase functions in the two example files.

### `examples/the_hand.rs`

```rust
fn setup(table: &Table) -> Result<(), PKError> {
    table.act_forced_bets().expect("forced bets failed");
    table.deal_cards_to_seats().expect("failed to deal hole cards");
    // ...
}

fn preflop(table: &Table) -> Result<usize, PKError> {
    table.act_bet(3, 2_100)?;
    table.act_raise(4, 5_000)?;
    // ...
}
```

Every function takes a shared reference. The mutations happen invisibly through
Cell wrappers.

### `examples/the_hand_no_cell.rs`

```rust
fn setup(table: &mut TableNoCell) -> Result<(), PKError> {
    table.act_forced_bets().expect("forced bets failed");
    table.deal_cards_to_seats().expect("failed to deal hole cards");
    // ...
}

fn preflop(table: &mut TableNoCell) -> Result<usize, PKError> {
    table.act_bet(3, 2_100)?;
    table.act_raise(4, 5_000)?;
    // ...
}
```

Every function requires exclusive access. The `mut` at each call site is a
machine-verified annotation that this function will change the table's state.

---

## Borrow Checker Implications Inside `TableNoCell`

The shift to `&mut self` introduces a friction pattern that `Table` avoids
through Cells: **you cannot hold a mutable borrow on a sub-field while calling
a method that also needs `&mut self`**.

This arises most visibly in `player_mucks_cards`:

```rust
// TableNoCell must scope the borrow tightly:
pub fn player_mucks_cards(&mut self, seat_number: u8) {
    let result = {
        let seat = self.seats.get_seat_mut(seat_number)?;
        let bard = Bard::from(seat.cards.cards());
        let cards = seat.discard_cards();
        (bard, cards)
    }; // seat borrow released here
    // Now self.log() can take &mut self again:
    self.log(TableAction::MuckPlayerCards(seat_number, result.0));
    self.muck.insert_all(&result.1);
}
```

`Table` avoids this entirely. Because `RefCell` borrows are dynamic, `Table`
uses `drop(seat)` as a convention rather than a requirement:

```rust
// Table: drop() is idiomatic but the borrow checker doesn't require it
let seat = self.get_seat_mut(seat_number);
// ... do work ...
drop(seat); // explicit, but only for clarity
self.event_log.log(TableAction::MuckPlayerCards(...));
```

In `TableNoCell`, the compiler **enforces** the scope discipline. In `Table`,
it is a convention that can silently be forgotten without compilation failure —
only a runtime `RefCell` panic would reveal the mistake.

---

## Implementation Challenges

### The `DrainableBintCell` Replacement

`Table::deal_cards_to_seats` uses a `DrainableBintCell` — a circular counter
that steps through seat indices and drains as it goes. `TableNoCell` replaces
this with a direct double loop:

```rust
// TableNoCell deal_cards_to_seats
for _ in 0..cards_per {
    for step in 0..seat_count {
        let idx = u8::try_from(
            (button as usize + 1 + step) % seat_count
        ).unwrap_or(0);
        if self.seats.is_seat_in_hand(idx) {
            self.deal_card_to_seat(idx)?;
        }
    }
}
```

The result is equivalent and arguably more readable — the pattern is explicit
rather than encapsulated in a custom iterator type.

### Showdown Without Shared References

`Table::end_hand` delegates to `Showdown::process(table)`, which can call back
into `table.seats.bring_it_in()` and other `&self` methods freely because all
mutations go through Cells.

`TableNoCell::end_hand` inlines the three showdown cases as private methods:
`showdown_single_seat`, `showdown_headsup`, and `showdown_multiway`. The
borrow checker prevents passing `&mut self` into a free function that also
needs to call other `&mut self` methods on the same value — so the logic lives
inside `impl TableNoCell` rather than in a separate `Showdown` struct.

### Deck Injection

`Table` has `nlh_primed(seats, &CardsCell, forced)` — a constructor that accepts
a pre-built deck wrapped in `CardsCell`. `TableNoCell` has no equivalent, but
none is needed: because `deck` is a plain `pub Cards` field, the example simply
assigns to it after construction:

```rust
let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
table.deck = Cards::deck_primed(&TestData::the_hand_cards_dealable());
```

This is impossible with `Table` because `deck: CardsCell` is private. Interior
mutability required the private wrapper; plain mutability makes the field
directly accessible.

---

## Analysis of the Trade-offs

### Compile-time Safety

| | `Table` | `TableNoCell` |
|---|---|---|
| Aliased mutation | Allowed (Cells) | Prevented at compile time |
| Double-borrow panics | Possible at runtime (`RefCell`) | Impossible |
| Borrow scope discipline | Convention | Enforced |

`TableNoCell` trades a small amount of implementation friction (scoped borrows,
no free-function showdown) for a stronger correctness guarantee.

### Flexibility

| | `Table` | `TableNoCell` |
|---|---|---|
| Multiple `&Table` references | Yes | No (only one `&mut` at a time) |
| Callbacks into table from closures | Straightforward | Requires borrow juggling |
| Thread-safety | No (Cell/RefCell are `!Sync`) | No (would need `Mutex`/`RwLock`) |

Interior mutability is not automatically thread-safe — `Cell` and `RefCell`
are both `!Sync`. Neither design can be shared across threads without
additional synchronisation. The `&self` API of `Table` looks shared-friendly,
but it is not.

### Observability

`Table`'s `event_log: TableLog(RefCell<Vec<TableAction>>)` has a custom
`Display` implementation. `TableNoCell`'s `event_log: Vec<TableAction>` does
not — callers iterate directly:

```rust
// Table
println!("{}", table.event_log);

// TableNoCell
for action in &table.event_log { println!("{action}"); }
```

The `TableLog` wrapper also provides `entries()`, `last()`, and
`last_player_action()` helper methods. Without it, `TableNoCell` callers use
iterator adapters directly (`iter().rev().find(...)`). This is less convenient
but no less capable.

### Performance

The Cell-based approach has costs:

- **`Cell<T>`** — a `get()`/`set()` pair compiles to a direct load/store with
  `UnsafeCell` internally, but it prevents the compiler from keeping the value
  in a register across calls (the cell could have been written elsewhere).
- **`RefCell<T>`** — `borrow_mut()` increments an atomic counter, and the
  borrow guard decrements it on drop. For hot paths (inner loops over seats),
  this adds measurable overhead.

`TableNoCell` eliminates all of these: field reads and writes are direct memory
accesses that the compiler can freely hoist, eliminate, or register-allocate.
For the event-logging path (`Vec::push`) vs. `TableLog::log` (which goes
through `RefCell::borrow_mut()`), the plain `Vec::push` on `&mut self` is
strictly cheaper.

A formal benchmark has not yet been written. The expectation is that
`TableNoCell` will be faster on hot paths (dealing, bet collection, event
logging) with the margin depending on how many `RefCell` borrows occur per
hand.

---

## Recommendation

Neither design is universally superior.

**Use `Table` when:**
- The poker engine is embedded in a larger structure that holds `&Table`
  references across multiple components (e.g. an observer pattern where
  multiple readers watch the same table).
- The API boundary requires `&self` for trait object compatibility.

**Use `TableNoCell` when:**
- Correctness guarantees at compile time are preferred over runtime flexibility.
- The table is owned and mutated by a single controller (the most common case
  for a game loop or simulation).
- Performance on hot paths matters and profiling confirms Cell overhead.

For a straightforward game loop — which is the primary use case in this codebase
— `TableNoCell` is the cleaner design. The `&mut self` discipline makes every
mutation site visible and prevents the class of bugs that only appear as
runtime `RefCell` panics under aliased borrows.

---

## Files

| Purpose | Path |
|---|---|
| Interior mutability version | `src/casino/table.rs` |
| Traditional mutability version | `src/casino/table_no_cell.rs` |
| Interior mutability example | `examples/the_hand.rs` |
| Traditional mutability example | `examples/the_hand_no_cell.rs` |
| Showdown logic (for `Table`) | `src/casino/table/showdown.rs` |
| Player with interior mutability | `src/casino/player.rs` |
| Player without interior mutability | `src/casino/table_no_cell.rs` (`PlayerNoCell`) |
