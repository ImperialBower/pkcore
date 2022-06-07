use crate::arrays::five::Five;
use crate::play::board::Board;
use crate::play::hands::Hands;
use crate::{Cards, PKError, Pile};
use std::fmt::{Display, Formatter};

/// A `Game` is a type that represents a single, abstraction of a game of `Texas hold 'em`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Game {
    pub hands: Hands,
    pub board: Board,
}

impl Game {
    #[must_use]
    pub fn new(hands: Hands, board: Board) -> Self {
        Game { hands, board }
    }

    /// # Errors
    ///
    /// Returns `PKError::Fubar` if invalid index is passed in.
    pub fn five_at_flop(&self, index: usize) -> Result<Five, PKError> {
        match self.hands.get(index) {
            None => Err(PKError::Fubar),
            Some(two) => Ok(Five::from_2and3(*two, self.board.flop)),
        }
    }

    #[must_use]
    pub fn remaining_cards_at_flop(&self) -> Cards {
        let mut cards = self.hands.cards();
        cards.add(&self.board.flop.cards());
        Cards::deck_minus(&cards)
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
    use crate::arrays::HandRanker;
    use std::str::FromStr;

    fn state() -> Game {
        let hands = Hands::from_str("6♠ 6♥ 5♦ 5♣").unwrap();
        let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap();

        let game = Game {
            hands: hands.clone(),
            board,
        };

        game
    }

    #[test]
    fn new() {
        let game = state();

        assert_eq!(game, Game::new(game.hands.clone(), game.board));
    }

    #[test]
    fn five_at_flop() {
        let game = state();

        assert_eq!(2185, game.five_at_flop(0).unwrap().hand_rank().value());
        assert_eq!(2251, game.five_at_flop(1).unwrap().hand_rank().value());
        assert!(game.five_at_flop(2).is_err());
    }

    #[test]
    fn remaining_cards_at_flop() {
        assert_eq!(
            state().remaining_cards_at_flop().to_string(),
            "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 8♣ 7♣ 6♣ 4♣ 3♣ 2♣"
        );
    }

    #[test]
    fn display() {
        let game = state();

        assert_eq!(
            "DEALT: [6♠ 6♥, 5♦ 5♣] FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
            game.to_string()
        );
    }
}
