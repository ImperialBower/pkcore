# Feature: Always Log Dealt Hole Cards

## Problem

`TableNoCell::act_fold()` calls `player_mucks_cards()` immediately when a player
folds. This blanks the seat's `BoxedCards` field (`seat.cards`) by replacing
every card with `Card::BLANK`. Any caller that reads hole cards from seat state
after the hand ends sees `None` for every player who folded — producing
incomplete hand history YAMLs where only winners and showdown players have
`hole_cards` populated.

## Root Cause

```
act_fold(seat)
  └─ player_mucks_cards(seat)
       └─ seat.discard_cards()
            └─ BoxedCards::take()   ← replaces cards with BLANK in place
```

There was no mechanism to recover what was originally dealt once a player
mucked.

## Solution

`TableNoCell` now carries a `dealt_hole_cards: HashMap<u8, BoxedCards>` field.

- **Populated** by `deal_cards_to_seats()` immediately after all seats receive
  their cards from the deck, and by `inject_hole_cards()` for replay paths.
- **Never modified** by folds, mucks, or any mid-hand action.
- **Cleared** by `reset()` at the start of each new hand so stale data from a
  previous hand cannot bleed through.

The type is `BoxedCards` (the same type used by `SeatNoCell.cards`) rather than
`String`, keeping the field consistent with pkcore's internal card
representation. Callers convert to string at serialization time if needed.

## Usage

### Reading dealt cards for hand history

```rust
// After end_hand(), before calling HandHistory::from_table_state():
let player_snapshot: Vec<(u8, String, usize, Option<String>)> = occupied_seats
    .iter()
    .map(|(seat_num, name, starting_stack)| {
        let hole_str = table.dealt_hole_cards
            .get(seat_num)
            .map(|bc| bc.to_string());
        (*seat_num, name.clone(), *starting_stack, hole_str)
    })
    .collect();
```

### Checking cards after a fold

```rust
table.act_fold(utg).unwrap();

// seat.cards is now blanked:
assert!(!table.seats.get_seat(utg).unwrap().cards.is_dealt());

// but dealt_hole_cards still has what was originally dealt:
let original = table.dealt_hole_cards.get(&utg).unwrap();
println!("UTG was dealt: {original}");
```

## Files Changed

- `src/casino/table_no_cell.rs` (`TableNoCell`)
  - struct: added `dealt_hole_cards: HashMap<u8, BoxedCards>`
  - `nlh_from_seats()`: initialises field to `HashMap::new()`
  - `deal_cards_to_seats()`: clears and repopulates after dealing
  - `inject_hole_cards()`: clears and repopulates from injected entries
  - `reset()`: clears the map

- `src/casino/table.rs` (`TableCelled`)
  - struct: added `dealt_hole_cards: RefCell<HashMap<u8, BoxedCards>>`
  - `nlh_from_seats()` and `Default::default()`: initialise to empty
  - `deal_cards_to_seats()`: clears and repopulates after dealing
  - `reset()`: clears the map

## Tests Added

### `TableNoCell` (`src/casino/table_no_cell.rs`)
- `test_dealt_hole_cards_survive_fold` — deals 3 players, folds one, asserts
  `dealt_hole_cards` still contains that seat's original cards while the seat
  itself is blank.
- `test_dealt_hole_cards_cleared_on_reset` — verifies the map is empty after
  `reset()`.
- `test_dealt_hole_cards_inject` — verifies `inject_hole_cards()` populates the
  map correctly for the replay path.

### `TableCelled` (`src/casino/table.rs`)
- `test_celled_dealt_hole_cards_survive_fold` — same fold-survival invariant for
  the interior-mutability table.
- `test_celled_dealt_hole_cards_cleared_on_reset` — same reset-clears invariant.

## Version

Introduced in pkcore 0.0.47.
