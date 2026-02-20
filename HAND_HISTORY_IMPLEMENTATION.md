# Poker Hand History File Format Implementation

## Overview

I've implemented a comprehensive Poker Hand History system that allows you to save and load poker game states from the `Table` struct into YAML format using Serde.

## What Was Added

### 1. **Dependencies**
- `serde_yaml = "0.9"` - For YAML serialization/deserialization
- `chrono = { version = "0.4", features = ["serde"] }` - For ISO 8601 timestamps

### 2. **New Module: `src/casino/hand_history.rs`**

A complete hand history implementation with the following structures:

#### Core Structures

**`PokerHandHistory`** - Main struct containing:
- `version`: Format version (currently "1.0")
- `hand_id`: Unique identifier (UUID from table)
- `timestamp`: ISO 8601 timestamp
- `table`: Table information
- `game`: Game configuration (variant, blinds)
- `players`: Vector of player info
- `button`: Button position
- `actions`: All betting actions by round
- `board`: Community cards (flop, turn, river)
- `results`: Hand results (optional)
- `metadata`: Custom metadata (optional)

**`TableInfo`** - Table details:
- Table name
- Maximum players
- Table ID

**`GameInfo`** - Game configuration:
- Variant (NoLimitHoldem, PLO, Razz)
- Blind structure (small, big, ante)

**`PlayerInfo`** - Player details:
- Seat number
- Player name/handle
- Starting stack
- Hole cards (if visible)
- Player ID

**`HandActions`** - Actions by street:
- Preflop actions
- Flop actions
- Turn actions
- River actions

**`PlayerAction`** - Individual action:
- Seat number
- Action type (fold, call, bet, raise, check, all-in)
- Amount (for bets/raises)
- Pot size after action

**`BoardCards`** - Community cards:
- Flop (3 cards)
- Turn (1 card)
- River (1 card)

**`HandResult`** - Result for each player:
- Seat number
- Player name
- Winnings (positive) or losses (negative)
- Hand description
- Best 5-card hand

### 3. **New Trait: `HandHistoryExt`**

Extension trait for `Table` providing:
- `to_hand_history()` - Convert table to hand history
- `save_hand_history(path)` - Save directly to YAML file

### 4. **Updated PKError**

Added three new error variants:
- `SerializationError` - For YAML serialization failures
- `DeserializationError` - For YAML parsing failures
- `IoError` - For file I/O operations

### 5. **Example Programs**

#### `examples/hand_history_demo.rs`
Basic demonstration showing:
- Converting a table to hand history
- Serializing to YAML
- Saving to file
- Loading from file
- Verifying round-trip serialization

#### `examples/complete_hand_history.rs`
Complete example using "The Hand" (Negreanu vs Hansen):
- Playing a complete hand
- Adding custom metadata
- Exporting to YAML
- Showing statistics

## Usage

### Basic Usage

```rust
use pkcore::prelude::*;
use pkcore::casino::hand_history::HandHistoryExt;

// Create a table
let table = Table::default();

// Play some poker...
table.act_shuffle_deck();
let _ = table.deal_cards_to_seats();
let _ = table.act_forced_bets();

// Convert to hand history
let history = table.to_hand_history()?;

// Save to YAML file
history.save_to_file("my_hand.yaml")?;

// Or get YAML string
let yaml_string = history.to_yaml()?;
println!("{}", yaml_string);
```

### Loading Hand History

```rust
use pkcore::casino::hand_history::PokerHandHistory;

// Load from file
let history = PokerHandHistory::load_from_file("my_hand.yaml")?;

// Or from YAML string
let history = PokerHandHistory::from_yaml(&yaml_str)?;

// Access data
println!("Hand ID: {}", history.hand_id);
println!("Players: {}", history.players.len());

for player in &history.players {
    println!("Seat {}: {} ({} chips)", 
        player.seat, player.name, player.stack);
}
```

### Using the Extension Trait

```rust
use pkcore::prelude::*;

let table = Table::default();

// Direct save
table.save_hand_history("hand.yaml")?;
```

## YAML Format Example

```yaml
version: "1.0"
hand_id: "550e8400-e29b-41d4-a716-446655440000"
timestamp: "2026-02-19T10:30:00Z"
table:
  name: "Default No Limit Hold'em Table"
  max_players: 6
  id: "550e8400-e29b-41d4-a716-446655440000"
game:
  variant: "NoLimitHoldem"
  blinds:
    small_blind: 50
    big_blind: 100
players:
  - seat: 0
    name: "Player1"
    stack: 10000
    hole_cards:
      - "A♠"
      - "K♠"
    player_id: "player-uuid-1"
  - seat: 1
    name: "Player2"
    stack: 10000
    hole_cards:
      - "Q♥"
      - "Q♦"
    player_id: "player-uuid-2"
button: 0
actions:
  preflop:
    - seat: 0
      action: "raise"
      amount: 300
    - seat: 1
      action: "call"
      amount: 300
  flop:
    - seat: 1
      action: "check"
    - seat: 0
      action: "bet"
      amount: 500
board:
  flop:
    - "K♥"
    - "J♥"
    - "T♥"
  turn: "9♥"
  river: "2♣"
metadata:
  event: "Example Hand"
  stakes: "50/100"
```

## Features

✅ **Complete Hand Capture** - All table state, players, actions, and results  
✅ **YAML Format** - Human-readable, industry-standard format  
✅ **Serde Integration** - Full serialization/deserialization support  
✅ **Type Safety** - Strongly typed structures with validation  
✅ **Extensible** - Metadata field for custom data  
✅ **Round-trip Safe** - Perfect serialization/deserialization fidelity  
✅ **Action Tracking** - Captures all player actions by street  
✅ **ISO 8601 Timestamps** - Standard timestamp format  
✅ **Optional Fields** - Efficient YAML with optional field skipping  

## API Reference

### `PokerHandHistory`

Methods:
- `new(hand_id)` - Create new hand history
- `to_yaml()` - Serialize to YAML string
- `from_yaml(yaml)` - Deserialize from YAML string
- `save_to_file(path)` - Save to file
- `load_from_file(path)` - Load from file

### `HandHistoryExt` (Trait for Table)

Methods:
- `to_hand_history()` - Convert table to hand history
- `save_hand_history(path)` - Save table as hand history file

## Run Examples

```bash
# Basic demo
cargo run --example hand_history_demo

# Complete hand example
cargo run --example complete_hand_history

# View generated YAML
cat generated/example_hand.yaml
cat generated/the_hand_history.yaml
```

## File Locations

- **Module**: `src/casino/hand_history.rs`
- **Examples**: `examples/hand_history_demo.rs`, `examples/complete_hand_history.rs`
- **Dependencies**: Added to `Cargo.toml`
- **Exports**: Added to `src/prelude.rs`
- **Module Declaration**: Updated `src/casino/mod.rs`

## Integration with Existing Code

The hand history system integrates seamlessly with your existing `Table` struct:

```rust
// Any table can be converted
let table = TestData::the_hand_table();
let history = table.to_hand_history()?;

// Works with any game type
let plo_table = /* ... */;
let plo_history = plo_table.to_hand_history()?; // variant will be "PLO"
```

## Future Enhancements

Possible additions:
- [ ] JSON format support (in addition to YAML)
- [ ] PokerStars hand history format compatibility
- [ ] Automatic result calculation and population
- [ ] Hand replay from history
- [ ] Statistics generation from hand history
- [ ] Batch export of multiple hands
- [ ] Compression for large hand history files

## Testing

The module includes comprehensive tests:
- `new_hand_history()` - Test creation
- `serialize_deserialize_yaml()` - Round-trip testing
- `player_action_serialization()` - Action serialization

Run tests:
```bash
cargo test --lib casino__hand_history_tests
```

---

**The Poker Hand History system is now fully integrated and ready to use!** 🎉

You can now save any poker game from your Table struct into a standardized, human-readable YAML format that can be:
- Stored in databases
- Shared with other applications
- Analyzed by external tools
- Converted to other formats
- Used for hand reviews and training

