// Include the narrator helper module
mod poker_narrator_lib;

use pkcore::prelude::*;
use poker_narrator_lib::PokerNarrator;

/// Example showing how to use the PokerNarrator helper
///
/// This demonstrates a cleaner way to add speech to your poker applications
/// using the helper module.
///
/// Run with: cargo run --example narrator_helper_demo
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Poker Narrator Helper Demo ===\n");

    // Initialize the narrator
    let mut narrator = match PokerNarrator::new() {
        Ok(n) => n,
        Err(e) => {
            println!("Warning: TTS not available: {}", e);
            println!("Continuing in silent mode...\n");
            PokerNarrator::silent()
        }
    };

    // Optional: Adjust speech settings
    let _ = narrator.set_rate(1.2);  // Slightly faster
    let _ = narrator.set_volume(0.9);

    // Create and set up a table
    let table = Table::default();

    // Announce new hand
    narrator.announce_new_hand()?;
    println!("Starting new hand...");

    // Shuffle and deal
    table.act_shuffle_deck();
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Post blinds
    println!("\nPosting blinds...");
    narrator.announce_blinds(50, 100)?;
    let _ = table.act_forced_bets();

    std::thread::sleep(std::time::Duration::from_millis(800));

    // Announce current state
    let state = table.get_game_state();
    println!("\nCurrent game state:");
    println!("  Phase: {}", state.phase);
    println!("  Pot: {} chips", state.pot_size);
    println!("  Button: Seat {}", state.button_position);
    println!("  Action: Seat {}", state.next_to_act);

    narrator.announce_state(&state)?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Simulate some player actions
    println!("\nSimulating player actions...");

    narrator.announce_action(3, "call", Some(100))?;
    println!("  Seat 3 calls 100");
    std::thread::sleep(std::time::Duration::from_millis(800));

    narrator.announce_action(4, "raise", Some(300))?;
    println!("  Seat 4 raises to 300");
    std::thread::sleep(std::time::Duration::from_millis(800));

    narrator.announce_action(5, "fold", None)?;
    println!("  Seat 5 folds");
    std::thread::sleep(std::time::Duration::from_millis(800));

    // Announce pot update
    let new_pot = 800;
    narrator.announce_pot(new_pot)?;
    println!("\n  Pot: {} chips", new_pot);

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Simulate dealing a flop
    println!("\nDealing the flop...");
    use std::str::FromStr;
    use crate::card::Card;

    let flop_cards = vec![
        Bard::from(&Card::from_str("Ah").unwrap()),
        Bard::from(&Card::from_str("Kh").unwrap()),
        Bard::from(&Card::from_str("Qh").unwrap()),
    ];

    for card in &flop_cards {
        table.board.insert(Card::from(card));
    }

    narrator.announce_board("flop", &flop_cards)?;
    println!("  Board: A♥ K♥ Q♥");

    std::thread::sleep(std::time::Duration::from_millis(1500));

    // Announce winner (simulated)
    println!("\nShowdown...");
    narrator.announce_winner(4, 800, "a flush")?;
    println!("  Winner: Seat 4 with a flush!");

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Final message
    println!("\n=== Demo Complete ===");
    narrator.speak("Demo complete. Thank you for playing!")?;

    // Give TTS time to finish
    std::thread::sleep(std::time::Duration::from_millis(2000));

    Ok(())
}

