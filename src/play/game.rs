use crate::analysis::player_wins::PlayerWins;
use crate::analysis::PlayOut;
use crate::arrays::five::Five;
use crate::play::board::Board;
use crate::play::hands::Hands;
use crate::{Cards, PKError, Pile};
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

    /// There is a point in your code where you reach the crux of the system you are trying to
    /// build. Where all of the thin slices start to come together and you can feel your program
    /// leveling up. For me with this journey the idea of playing out the probabilities is one
    /// of those places. I need this to be clear. I need it to be flexible. I need it to be
    /// extendable.
    ///
    /// ## The Play Out Saga
    ///
    /// * Book 1: Play out at flop
    /// * Book 2: Play out at turn
    /// * Book 3: Play out at river
    /// * Book 4: DUN DUN DUNNNNNNNNNN - The reckoning: Play out preflop.
    ///
    /// ### Book 1
    ///
    /// One of the things that I watch out for is if I start feeling the need to add a lot of print
    /// statements to my code to keep track of what it's doing.
    ///
    /// Introducing a Big Idea: Observability.
    ///
    /// Now for me as a software developer, I want to master the craft of making my code as
    /// observable as possible. Observability comes from the mathematical principal. From Wikipedia:
    ///
    /// Observability is a measure of how well internal states of a system can be inferred from knowledge of its external outputs. In control theory, the observability and controllability of a linear system are mathematical duals. The concept of observability was introduced by the Hungarian-American engineer Rudolf E. Kálmán for linear dynamic systems. A dynamical system designed to estimate the state of a system from measurements of the outputs is called a state observer or simply an observer for that system.
    ///
    /// I'm a huge fan of those in the `DevOps` movement who have been pioneering the Observability
    /// movement in software development.
    ///
    /// ### ~~Big Idea: Controllability~~
    ///
    /// ### Dimensions
    ///
    /// What are the different ways that we can view the information on the flop?
    ///
    /// * Board texture
    /// * Per player
    ///   * Counts of Hand Class
    ///   * Chances of winning
    ///
    /// ### `PlayOut` Trait Idea
    /// It would be nice if I could plug an analysis type into the iterator to give me flexibility
    /// on what I do with the information from the cases.
    ///
    /// # BOOM!!! post `PlayOut`
    ///
    /// We've moved all this logic over to the `PlayerWins` struct implementing our super amazing
    /// `PlayOut` trait plugin. Now we can inject different types of analysis depending on our needs.
    /// TBH, this is HAF.
    ///
    /// I'll be honest with you. I'm really proud of myself for this refactoring. This is above and
    /// beyond anything I did in the original fudd spike.
    ///
    /// Being able to pull off these optimizations largely depends on the clock. As a hack imposter
    /// you have to watch out if you have the time to spend on these quests for aesthetic beauty.
    /// Luckily for us, this work is all about self expression. as Joseph Campbell said,
    /// _"Find a place inside where there's joy, and the joy will burn out the pain."_ For me, this
    /// is one of those places. I can't control the world, but I can control the universe that is
    /// my art.
    ///
    /// # Panics
    ///
    /// Shouldn't be possible, knock on wood.
    pub fn play_out_flop(&self) {
        let mut wins = PlayerWins::default();
        self.pof::<PlayerWins>(&mut wins);
    }

    /// Could this actually work? It's trying to do stuff like this that I really start feeling
    /// like an imposter.
    pub fn pof<T>(&self, po: &mut T)
    where
        T: PlayOut,
    {
        po.play_out_flop(self.hands.clone(), self.board.flop);
    }

    /// REFACTORING: OK, we're moving this over to Hands for greater flexibility. Now that we've are
    /// trying out the `PlayOut` generic trait we need to be able to determine how many `Cards` are
    /// remaining at a specific point in the hand. This method locks it into the flop, and we
    /// really don't need that.
    ///
    /// TODO: deprecate
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

    fn the_hand() -> Game {
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
        let game = the_hand();

        assert_eq!(game, Game::new(game.hands.clone(), game.board));
    }

    #[test]
    fn five_at_flop() {
        let game = the_hand();

        assert_eq!(2185, game.five_at_flop(0).unwrap().hand_rank().value());
        assert_eq!(2251, game.five_at_flop(1).unwrap().hand_rank().value());
        assert!(game.five_at_flop(2).is_err());
    }

    #[test]
    fn remaining_cards_at_flop() {
        // Crude but effective. https://www.youtube.com/watch?v=UKkjknFwPac
        assert_eq!(
            the_hand().remaining_cards_at_flop().to_string(),
            "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 8♣ 7♣ 6♣ 4♣ 3♣ 2♣"
        );
    }

    /// I really like this test, even though it asserts nothing. It's just making sure that we
    /// really can inject a `PlayOut` struct and that the code will play nice. Maybe that's the
    /// imposter in me that I want to leave it in. The old java hacker in me would never leave this
    /// in that kind of codebase, but for now, I will let this sign of my lack of experience stay.
    /// After all, it's just a test. It's not like it's production code.
    #[test]
    fn pof() {
        let mut wins = PlayerWins::default();
        let game = the_hand();

        game.pof::<PlayerWins>(&mut wins);
    }

    #[test]
    fn display() {
        let game = the_hand();

        assert_eq!(
            "DEALT: [6♠ 6♥, 5♦ 5♣] FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
            game.to_string()
        );
    }
}
