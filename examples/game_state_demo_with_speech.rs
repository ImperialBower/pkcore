use pkcore::prelude::*;
use tts::Tts;

/// This example demonstrates how to use the `get_game_state()` function
/// with text-to-speech to make your poker game speak out loud!
///
/// Run with: cargo run --example game_state_demo_with_speech
///
/// The TTS library provides cross-platform text-to-speech:
/// - macOS: Uses AVFoundation
/// - Windows: Uses SAPI/Speech Platform
/// - Linux: Uses Speech Dispatcher
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize text-to-speech
    let mut tts = Tts::default()?;

    println!("=== GameState Demo with Speech ===\n");
    tts.speak("Game State Demo with Speech", false)?;

    // Create a default 6-handed No Limit Hold'em table
    let table = Table::default();

    // Get and display the initial game state
    println!("Initial State:");
    let state = table.get_game_state();
    println!("{}", state);

    // Speak the initial state
    let announcement = format!(
        "Welcome to {}. This is a {} game with blinds of {} and {}. \
         We have {} players seated. Let's begin!",
        state.table_name,
        format!("{:?}", state.game_type).replace("NoLimitHoldem", "No Limit Hold'em"),
        state.small_blind,
        state.big_blind,
        state.total_players
    );
    tts.speak(&announcement, false)?;

    // Simulate some game actions
    println!("\n=== Simulating Game Progress ===\n");
    tts.speak("Shuffling the deck and posting blinds", false)?;

    // Shuffle and post blinds
    table.act_shuffle_deck();
    let _ = table.act_forced_bets();

    println!("After Forced Bets:");
    let state_after_bets = table.get_game_state();
    println!("{}", state_after_bets);

    // Announce the blind posting
    if state_after_bets.pot_size > 0 {
        let blind_announcement = format!(
            "The blinds have been posted. The pot now contains {} chips. \
             Current bet to call is {} chips.",
            state_after_bets.pot_size,
            state_after_bets.current_bet
        );
        println!("\n✓ {}", blind_announcement);
        tts.speak(&blind_announcement, false)?;
    }

    // Announce the next action
    let next_action = format!(
        "We are in the {} phase. Seat {} is on the button. \
         Action is on seat {}.",
        state_after_bets.phase,
        state_after_bets.button_position,
        state_after_bets.next_to_act
    );
    println!("\n{}", next_action);
    tts.speak(&next_action, false)?;

    // Final announcement
    println!("\n=== Demo Complete ===");
    tts.speak("Demo complete. Good luck at the tables!", false)?;

    Ok(())
}

