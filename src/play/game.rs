use crate::play::board::Board;
use crate::play::hands::Hands;
use std::fmt::{Display, Formatter};

/// A `Game` is a type that represents a single, abstraction of a game of `Texas hold 'em`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Game {
    hands: Hands,
    board: Board,
}

impl Game {
    #[must_use]
    pub fn new(hands: Hands, board: Board) -> Self {
        Game { hands, board }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DEALT: {} {}", self.hands, self.board)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play_game_tests {
    use super::*;
    use std::str::FromStr;

    fn state() -> (Hands, Board, Game) {
        let hands = Hands::from_str("6♠ 6♥ 5♦ 5♣").unwrap();
        let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap();

        let game = Game {
            hands: hands.clone(),
            board,
        };

        (hands, board, game)
    }

    #[test]
    fn new() {
        let (hands, board, game) = state();

        assert_eq!(game, Game::new(hands, board));
    }

    #[test]
    fn display() {
        let (_, _, game) = state();

        assert_eq!(
            "DEALT: [6♠ 6♥, 5♦ 5♣] FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
            game.to_string()
        );
    }
}
