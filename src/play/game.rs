use crate::arrays::five::Five;
use crate::play::board::Board;
use crate::play::hands::Hands;
use crate::SOK;
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

    #[must_use]
    pub fn five_at_flop(&self, index: usize) -> Five {
        let hand = self.hands.get(index);
        if hand.salright() {
            Five::from_2and3(hand, self.board.flop)
        } else {
            Five::default()
        }
    }
    //
    // #[must_use]
    // pub fn hand_rank_at_flop(&self, hand: usize) -> HandRank {
    //     let hand = self.hands.get(hand);
    //     if hand.salright() {
    //         Five::from_2and3(hand, self.board.flop).hand_rank()
    //     } else {
    //         HandRank::default()
    //     }
    // }
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

        assert_eq!(2185, game.five_at_flop(0).hand_rank().value());
        assert_eq!(2251, game.five_at_flop(1).hand_rank().value());
        assert_eq!(0, game.five_at_flop(2).hand_rank().value());
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
