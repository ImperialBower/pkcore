#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

// Example 1: Vector of structs with Cell fields (like your PhaseHoldemTracker)
#[derive(Debug)]
struct Player {
    id: u32,
    chips: Cell<u32>,
    is_active: Cell<bool>,
}

impl Player {
    fn new(id: u32, chips: u32) -> Self {
        Self {
            id,
            chips: Cell::new(chips),
            is_active: Cell::new(true),
        }
    }

    fn bet(&self, amount: u32) -> bool {
        let current_chips = self.chips.get();
        if current_chips >= amount {
            self.chips.set(current_chips - amount);
            true
        } else {
            false
        }
    }

    fn add_chips(&self, amount: u32) {
        self.chips.set(self.chips.get() + amount);
    }

    fn fold(&self) {
        self.is_active.set(false);
    }

    fn get_chips(&self) -> u32 {
        self.chips.get()
    }

    fn is_active(&self) -> bool {
        self.is_active.get()
    }
}

// Example 2: Vector of structs with RefCell fields (like your CardsCell pattern)
#[derive(Debug, Clone)]
struct GameState {
    round: u32,
    deck: RefCell<Vec<String>>, // Simplified cards as strings
    pot: RefCell<u32>,
    players_hands: RefCell<HashMap<u32, Vec<String>>>,
}

impl GameState {
    fn new(round: u32) -> Self {
        let mut deck = Vec::new();
        // Create a simple deck
        for suit in &["♠", "♥", "♦", "♣"] {
            for rank in &["A", "K", "Q", "J", "T", "9", "8", "7", "6", "5", "4", "3", "2"] {
                deck.push(format!("{}{}", rank, suit));
            }
        }

        Self {
            round,
            deck: RefCell::new(deck),
            pot: RefCell::new(0),
            players_hands: RefCell::new(HashMap::new()),
        }
    }

    fn deal_cards(&self, player_id: u32, num_cards: usize) -> Result<(), String> {
        let mut deck = self.deck.borrow_mut();
        let mut hands = self.players_hands.borrow_mut();

        if deck.len() < num_cards {
            return Err("Not enough cards in deck".to_string());
        }

        let player_hand = hands.entry(player_id).or_insert_with(Vec::new);
        for _ in 0..num_cards {
            if let Some(card) = deck.pop() {
                player_hand.push(card);
            }
        }
        Ok(())
    }

    fn add_to_pot(&self, amount: u32) {
        let mut pot = self.pot.borrow_mut();
        *pot += amount;
    }

    fn get_pot(&self) -> u32 {
        *self.pot.borrow()
    }

    fn get_player_hand(&self, player_id: u32) -> Vec<String> {
        self.players_hands.borrow().get(&player_id).cloned().unwrap_or_default()
    }

    fn cards_remaining(&self) -> usize {
        self.deck.borrow().len()
    }
}

// Example 3: RefCell containing a vector of structs
#[derive(Debug)]
struct Tournament {
    games: RefCell<Vec<GameState>>,
    current_game: Cell<usize>,
}

impl Tournament {
    fn new() -> Self {
        Self {
            games: RefCell::new(Vec::new()),
            current_game: Cell::new(0),
        }
    }

    fn add_game(&self, game: GameState) {
        self.games.borrow_mut().push(game);
    }

    fn get_current_game(&self) -> Option<GameState> {
        let games = self.games.borrow();
        let current_idx = self.current_game.get();
        games.get(current_idx).cloned()
    }

    fn next_game(&self) -> bool {
        let current = self.current_game.get();
        let games_len = self.games.borrow().len();
        if current + 1 < games_len {
            self.current_game.set(current + 1);
            true
        } else {
            false
        }
    }

    fn game_count(&self) -> usize {
        self.games.borrow().len()
    }
}

fn main() {
    println!("=== Example 1: Vector of structs with Cell fields ===");

    // Create a vector of players
    let players = vec![Player::new(1, 1000), Player::new(2, 1500), Player::new(3, 2000)];

    println!("Initial state:");
    for player in &players {
        println!(
            "Player {}: {} chips, active: {}",
            player.id,
            player.get_chips(),
            player.is_active()
        );
    }

    // Players make bets
    players[0].bet(100);
    players[1].bet(150);
    players[2].fold();

    println!("\nAfter betting:");
    for player in &players {
        println!(
            "Player {}: {} chips, active: {}",
            player.id,
            player.get_chips(),
            player.is_active()
        );
    }

    println!("\n=== Example 2: Structs with RefCell fields ===");

    let game = GameState::new(1);

    // Deal cards to players
    game.deal_cards(1, 2).unwrap();
    game.deal_cards(2, 2).unwrap();

    println!("Cards remaining in deck: {}", game.cards_remaining());
    println!("Player 1 hand: {:?}", game.get_player_hand(1));
    println!("Player 2 hand: {:?}", game.get_player_hand(2));

    // Add to pot
    game.add_to_pot(250);
    println!("Current pot: {}", game.get_pot());

    println!("\n=== Example 3: RefCell containing vector ===");

    let tournament = Tournament::new();

    // Add multiple games
    tournament.add_game(GameState::new(1));
    tournament.add_game(GameState::new(2));
    tournament.add_game(GameState::new(3));

    println!("Tournament has {} games", tournament.game_count());
    println!(
        "Current game round: {:?}",
        tournament.get_current_game().map(|g| g.round)
    );

    tournament.next_game();
    println!(
        "After next_game(), current game round: {:?}",
        tournament.get_current_game().map(|g| g.round)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_betting() {
        let player = Player::new(1, 1000);
        assert_eq!(player.get_chips(), 1000);
        assert!(player.bet(500));
        assert_eq!(player.get_chips(), 500);
        assert!(!player.bet(600)); // Should fail - not enough chips
        assert_eq!(player.get_chips(), 500); // Should remain unchanged
    }

    #[test]
    fn test_game_state() {
        let game = GameState::new(1);
        assert_eq!(game.cards_remaining(), 52);

        game.deal_cards(1, 2).unwrap();
        assert_eq!(game.cards_remaining(), 50);
        assert_eq!(game.get_player_hand(1).len(), 2);
    }

    #[test]
    fn test_tournament() {
        let tournament = Tournament::new();
        tournament.add_game(GameState::new(1));
        tournament.add_game(GameState::new(2));

        assert_eq!(tournament.game_count(), 2);
        assert_eq!(tournament.get_current_game().unwrap().round, 1);

        tournament.next_game();
        assert_eq!(tournament.get_current_game().unwrap().round, 2);
    }

    #[test]
    fn test_vector_of_players() {
        let players = vec![Player::new(1, 1000), Player::new(2, 1500)];

        // Test that we can modify through the vector
        players[0].bet(100);
        players[1].add_chips(200);

        assert_eq!(players[0].get_chips(), 900);
        assert_eq!(players[1].get_chips(), 1700);
    }
}
