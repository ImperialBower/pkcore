# Cell and RefCell with Vectors of Structs

This example demonstrates several patterns for using `Cell` and `RefCell` with vectors of structs in Rust. These patterns provide interior mutability, allowing you to modify data through shared references.

## Patterns Demonstrated

### 1. Vector of Structs with Cell/RefCell Fields

```rust
struct Player {
    id: u32,
    chips: Cell<u32>,        // Cell for Copy types
    is_active: Cell<bool>,   // Cell for simple values
}

let players = vec![
    Player::new(1, 1000),
    Player::new(2, 1500),
];

// Can modify through shared references
players[0].bet(100);  // Changes chips value internally
```

**Use when:** You have a collection of objects that need to modify their own state, but you don't want to make the entire vector mutable.

### 2. Structs with RefCell Fields

```rust
struct GameState {
    deck: RefCell<Vec<String>>,              // RefCell for non-Copy types
    players_hands: RefCell<HashMap<u32, Vec<String>>>,
}

// Can modify complex data structures
game.deal_cards(1, 2)?;  // Modifies both deck and hands
```

**Use when:** You need to modify complex data structures (like `Vec`, `HashMap`) through shared references.

### 3. RefCell Containing a Vector

```rust
struct Tournament {
    games: RefCell<Vec<GameState>>,  // Entire vector inside RefCell
    current_game: Cell<usize>,
}

// Can add/remove games from the vector
tournament.add_game(new_game);
```

**Use when:** You need to modify the vector itself (add/remove elements) through shared references.

## Key Differences: Cell vs RefCell

### Cell<T>

- **For:** `Copy` types (integers, booleans, small structs with Copy)
- **Access:** `get()` and `set()` methods
- **Borrowing:** No borrowing - always copies the value
- **Performance:** Faster, no runtime borrow checking
- **Thread safety:** Not thread-safe

### RefCell<T>

- **For:** Any type, especially non-`Copy` types
- **Access:** `borrow()` and `borrow_mut()` methods
- **Borrowing:** Runtime borrow checking (can panic!)
- **Performance:** Slight overhead from borrow checking
- **Thread safety:** Not thread-safe

## When to Use Each Pattern

### Pattern 1: Vector of structs with Cell/RefCell fields

- **Best for:** Game entities (players, cards, game objects)
- **Example:** Player stats, card states, entity properties
- **Benefit:** Each object manages its own state

### Pattern 2: Structs with RefCell fields  

- **Best for:** Complex state management
- **Example:** Game state, caches, collections that need modification
- **Benefit:** Can modify complex data structures through shared references

### Pattern 3: RefCell containing vector

- **Best for:** Dynamic collections
- **Example:** Player lists, game history, dynamic inventories
- **Benefit:** Can modify the collection structure itself

## Running the Example

```bash
# Run the example
cargo run --example cell_vector_examples

# Run the tests
cargo test --example cell_vector_examples
```

## Real-world Applications

Based on your poker library, these patterns are useful for:

1. **Player management** - Track chip counts, hands, actions
2. **Game state** - Manage deck, community cards, pot
3. **Tournament structure** - Multiple games, dynamic player lists
4. **Card dealing** - Modify deck and player hands simultaneously

## Safety Notes

- `RefCell` can panic if you violate borrowing rules at runtime
- Neither `Cell` nor `RefCell` are thread-safe (use `Mutex` or `RwLock` for threading)
- Always prefer immutable designs when possible - use interior mutability only when necessary
