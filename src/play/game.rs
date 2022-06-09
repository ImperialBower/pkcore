use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::two::Two;
use crate::play::board::Board;
use crate::play::hands::Hands;
use crate::{Card, Cards, PKError, Pile};
use std::fmt::{Display, Formatter};

/// A `Game` is a type that represents a single, abstraction of a game of `Texas hold 'em`.
///
/// ## PHASE 2.2: Display winning percentages
/// This is a big feature for me, and one that I've been struggling over for a while.
/// I originally completed this feature in
/// [Fudd](https://github.com/ContractBridge/fudd/blob/main/src/games/holdem/table.rs#L284),
/// but I found the solution convoluted, and impossible to extend.
///
/// I think the reason this is because I coded it backwards. I started with the most complex type,
/// the `Table`, and tried to drill down into the situations, instead of building things from
/// the bottom up.
///
/// A HUGE plus was when I can upon the idea for `WinCounter`. Obsessing over a way to deal with
/// counting wins against all possible combinations, I stumbled upon the idea of simply using
/// bitwise operations. If more than one player wins for a specific card combination, just set the
/// flag for each of them. That way I can have as many possible combination of winners as I need.
///
/// If I haven't said if before, I really love bitwise operations. I've been in love with them
/// since I first saw them used in PHP code for my first programming gig at the now defunct
/// [XOOM.com](https://en.wikipedia.org/wiki/Xoom_(web_hosting)), most famous for hosting
/// [Mahir Çağrı](https://en.wikipedia.org/wiki/Mahir_%C3%87a%C4%9Fr%C4%B1)'s website.
/// _[I KISS YOU!](https://web.archive.org/web/20050206024432/http://www.ikissyou.org/indeks2.html)_
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

    /// Returns the `Five` `Card` hand combining the hole cards from the passed in index
    /// combined with the `Three` Cards on the flop.
    ///
    /// # Errors
    ///
    /// Returns `PKError::Fubar` if invalid index is passed in.
    pub fn five_at_flop(&self, index: usize) -> Result<Five, PKError> {
        match self.hands.get(index) {
            None => Err(PKError::Fubar),
            Some(two) => Ok(Five::from_2and3(*two, self.board.flop)),
        }
    }

    /// # Panics
    ///
    /// Shouldn't be possible, knock on wood.
    pub fn play_out_flop(&self) {
        for case in self.remaining_cards_at_flop().combinations(2) {
            for player in self.hands.iter() {
                let seven = self.case_seven(*player, &case).unwrap();
            }
        }
    }

    /// I have coined the term `case` for a specific instance of analysis when iterating through
    /// all possible combinations of hands for a specific game of poker. For instance: Given
    /// `THE HAND` between Daniel Nergeanu and Gus Hansen, where Daniel held `6♠ 6♥` and Gus held
    ///  `5♦ 5♣`, with the flop of `9♣ 6♦ 5♥`
    fn case_seven(&self, player: Two, case: &[Card]) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            self.board.flop.first(),
            self.board.flop.second(),
            self.board.flop.third(),
            *case.get(0).ok_or(PKError::InvalidCard)?,
            *case.get(1).ok_or(PKError::InvalidCard)?,
        ]))
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
        // Crude but effective. https://www.youtube.com/watch?v=UKkjknFwPac
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
