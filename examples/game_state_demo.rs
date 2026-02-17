use pkcore::prelude::*;

/// This example demonstrates how to use the `get_game_state()` function
/// to retrieve and display the current state of a poker game.
///
/// The GameState provides a snapshot of all relevant game information including:
/// - Table details (ID, name, game type)
/// - Current phase of the game
/// - Button position and next player to act
/// - Pot size and current bet
/// - Board cards
/// - Player counts
/// - Blind structure
fn main() {
    println!("=== GameState Demo ===\n");
    
    // Create a default 6-handed No Limit Hold'em table
    let table = Table::default();
    
    // Get and display the initial game state
    println!("Initial State:");
    let state = table.get_game_state();
    println!("{}", state);
    
    // Access individual fields for custom logic
    println!("\n--- Individual Field Access ---");
    println!("Table Name: {}", state.table_name);
    println!("Game Type: {:?}", state.game_type);
    println!("Current Phase: {}", state.phase);
    println!("Button Position: Seat {}", state.button_position);
    println!("Next to Act: Seat {}", state.next_to_act);
    println!("Blinds: {}/{}", state.small_blind, state.big_blind);
    println!("Players: {} active out of {} total", state.active_players, state.total_players);
    println!("Deck: {} cards remaining", state.deck_remaining);
    
    // Simulate some game actions
    println!("\n=== Simulating Game Progress ===\n");
    
    // Shuffle and post blinds
    table.act_shuffle_deck();
    let _ = table.act_forced_bets();
    
    println!("After Forced Bets:");
    let state_after_bets = table.get_game_state();
    println!("{}", state_after_bets);
    
    // You can use GameState for decision making
    if state_after_bets.pot_size > 0 {
        println!("\n✓ Blinds have been posted!");
        println!("  Pot now contains {} chips", state_after_bets.pot_size);
    }
    
    if state_after_bets.current_bet > 0 {
        println!("  Current bet to call: {} chips", state_after_bets.current_bet);
    }
    
    // GameState is useful for:
    // - Logging game progress
    // - Making AI decisions based on game state
    // - Debugging game flow
    // - Saving/restoring game state
    // - Displaying game state to players
    
    println!("\n=== Demo Complete ===");
}

