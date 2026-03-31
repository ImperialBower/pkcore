//! River stage hand ranking.
//!
//! At the river the board is complete (five cards dealt), so there is exactly one outcome
//! per pair of hands — no runout enumeration is needed. [`RiverEval`] wraps that single
//! [`CaseEval`] in the same shape as [`FlopEval`] and [`TurnEval`].

use crate::PKError;
use crate::analysis::case_eval::CaseEval;
use crate::analysis::case_evals::CaseEvals;
use crate::analysis::eval::Eval;
use crate::arrays::HandRanker;
use crate::arrays::seven::Seven;
use crate::play::game::Game;
use crate::prelude::Table;
use crate::Pile;
use std::fmt::{Display, Formatter};
use wincounter::results::WinResults;
use wincounter::wins::Wins;

/// Hand ranking at the river stage.
///
/// The board is complete, so there is a single deterministic [`CaseEval`] rather than the
/// enumeration performed by [`FlopEval`](super::flop_eval::FlopEval) and
/// [`TurnEval`](super::turn_eval::TurnEval).
///
/// # Examples
/// ```
/// use pkcore::prelude::TestData;
/// use pkcore::play::stages::river_eval::RiverEval;
///
/// let game = TestData::the_hand();
/// let result = RiverEval::try_from(game);
/// assert!(result.is_ok());
/// ```
#[derive(Clone, Debug, Default)]
pub struct RiverEval {
    pub game: Game,
    pub case_eval: CaseEval,
    pub wins: Wins,
    pub results: WinResults,
}

impl RiverEval {
    /// Returns the ranked hand for the player at the given index on the complete board.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::Fubar`] if the index is out of range.
    ///
    /// # Examples
    /// ```
    /// use pkcore::prelude::TestData;
    /// use pkcore::play::stages::river_eval::RiverEval;
    ///
    /// let re = RiverEval::try_from(TestData::the_hand()).unwrap();
    /// assert!(re.rank_for_player(0).is_ok());
    /// assert!(re.rank_for_player(99).is_err());
    /// ```
    pub fn rank_for_player(&self, i: usize) -> Result<Eval, PKError> {
        match self.game.hands.get(i) {
            None => Err(PKError::Fubar),
            Some(two) => Ok(Seven::from_case_and_board(two, &self.game.board).eval()),
        }
    }
}

impl Display for RiverEval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "The River: {} {} {}",
            self.game.board.flop, self.game.board.turn, self.game.board.river
        )?;

        for (i, hole_cards) in self.game.hands.iter().enumerate() {
            if hole_cards.is_blank() {
                continue;
            }
            writeln!(
                f,
                "  Player #{} [{}] {}",
                i,
                hole_cards,
                self.results.player_to_string(i)
            )?;
            match self.rank_for_player(i) {
                Ok(ranked) => writeln!(f, "     {ranked}")?,
                Err(_) => writeln!(f, "     Error")?,
            }
        }

        Ok(())
    }
}

impl TryFrom<Game> for RiverEval {
    type Error = PKError;

    /// # Errors
    ///
    /// Returns [`PKError::NotDealt`] if the river card or any required board card is not yet dealt,
    /// or if no hands are present.
    fn try_from(game: Game) -> Result<Self, Self::Error> {
        if !game.board.flop.is_dealt() || !game.board.turn.is_dealt() || !game.board.river.is_dealt() {
            return Err(PKError::NotDealt);
        }
        if game.hands.is_empty() {
            return Err(PKError::NotDealt);
        }

        let case_eval = game.hands.river_case_eval(&game.board);
        let case_evals = CaseEvals::from(vec![case_eval.clone()]);
        let wins = case_evals.wins();
        let results = WinResults::from_wins(&wins, game.hands.len());

        Ok(RiverEval { game, case_eval, wins, results })
    }
}

impl TryFrom<&Table> for RiverEval {
    type Error = PKError;

    fn try_from(table: &Table) -> Result<Self, Self::Error> {
        RiverEval::try_from(Game::try_from(table)?)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__stages__river_eval_tests {
    use super::*;
    use crate::PKError;
    use crate::arrays::three::Three;
    use crate::card::Card;
    use crate::play::board::Board;
    use crate::play::game::Game;
    use crate::play::hole_cards::HoleCards;
    use crate::prelude::TestData;
    use std::str::FromStr;

    fn board_flop_and_turn_only() -> Board {
        Board::new(
            Three::from_str("9♣ 6♦ 5♥").unwrap(),
            Card::from_str("5♠").unwrap(),
            Card::default(),
        )
    }

    #[test]
    fn test_river_eval_try_from_game_ok() {
        let sut = RiverEval::try_from(TestData::the_hand());
        assert!(sut.is_ok());
    }

    #[test]
    fn test_river_eval_try_from_game_no_river() {
        let game = TestData::the_hand();
        let game = Game { hands: game.hands, board: board_flop_and_turn_only() };
        let result = RiverEval::try_from(game);
        assert!(result.is_err());
        assert_eq!(PKError::NotDealt, result.unwrap_err());
    }

    #[test]
    fn test_river_eval_try_from_game_no_hands() {
        let game = Game { hands: HoleCards::default(), board: TestData::the_hand().board };
        let result = RiverEval::try_from(game);
        assert!(result.is_err());
        assert_eq!(PKError::NotDealt, result.unwrap_err());
    }

    #[test]
    fn test_river_eval_rank_for_player_valid() {
        let sut = RiverEval::try_from(TestData::the_hand()).unwrap();
        assert!(sut.rank_for_player(0).is_ok());
        assert!(sut.rank_for_player(1).is_ok());
    }

    #[test]
    fn test_river_eval_rank_for_player_invalid() {
        let sut = RiverEval::try_from(TestData::the_hand()).unwrap();
        assert_eq!(Err(PKError::Fubar), sut.rank_for_player(99));
    }

    #[test]
    fn test_river_eval_player_one_wins_the_hand() {
        // Board: 9♣ 6♦ 5♥ 5♠ 8♠
        // Player 0 (6♠ 6♥): full house, sixes full of fives
        // Player 1 (5♦ 5♣): four fives — quads beat the full house
        let sut = RiverEval::try_from(TestData::the_hand()).unwrap();
        let p0 = sut.rank_for_player(0).unwrap();
        let p1 = sut.rank_for_player(1).unwrap();
        assert!(p1 > p0, "Player 1 should win with four fives");
    }

    #[test]
    fn test_river_eval_display() {
        let sut = RiverEval::try_from(TestData::the_hand()).unwrap();
        let s = sut.to_string();
        assert!(s.contains("The River:"));
        assert!(s.contains("Player #0"));
    }
}
