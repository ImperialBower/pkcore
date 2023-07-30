use crate::analysis::eval::Eval;
use crate::analysis::store::bcm::binary_card_map::SevenFiveBCM;
use crate::analysis::store::db::headsup_preflop_result::HUPResult;
use crate::arrays::five::Five;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::play::board::Board;
use crate::play::game::Game;
use crate::play::hole_cards::HoleCards;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;
use crate::Card;
use std::str::FromStr;

/// I am a classicist when it comes to testing. Martin Fowler, in his essay
/// [Mocks Aren't Stubs](https://martinfowler.com/articles/mocksArentStubs.html)
/// breaks down the styles of TDD into classical and mockist:
///
/// > The classical TDD style is to use real objects if possible and a double if it's awkward to use the real thing. So a classical `TDDer` would use a real warehouse and a double for the mail service. The kind of double doesn't really matter that much.
/// >
/// > A mockist TDD practitioner, however, will always use a mock for any object with interesting behavior. In this case for both the warehouse and the mail service.
///
/// Now, the norm where I work is to code in a mockist style. As a developer, I try to understand
/// the different styles and be able to do both. Even though I would much rather inject pure state
/// into my objects, in the classical style, it's useful to be able to do both.
///
/// Now one of my favorite programmers, [Dan Wiebe](https://github.com/dnwiebe), is a hard core
/// mockist, and has used his considerable fundamentalist will-to-power foo to make the challenge
/// that rust brings to mocking possible in the code bases that he has worked with.
///
/// * [`SubstratumNode`](https://github.com/robmoorman/SubstratumNode)
/// * [MASQ-Project/Node](https://github.com/MASQ-Project/Node)
///
///
#[allow(dead_code, clippy::module_name_repetitions)]
pub enum TestData {}

#[allow(dead_code)]
impl TestData {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn the_hand() -> Game {
        let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap();

        Game {
            hands: TestData::hole_cards_the_hand(),
            board,
        }
    }

    /// Based on HSP S04E08 Harman/Safai but with the river bringing quads
    /// `cargo run --example calc -- -d "A♣ Q♠ T♦ T♣ 6♦ 4♦ 2♥ 2♦" -b "J♦ J♠ J♥ A♥ J♣"`
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn the_board() -> Game {
        let hands = HoleCards::from(vec![
            Two::HAND_AC_QS,
            Two::HAND_TD_TC,
            Two::HAND_6D_4D,
            Two::HAND_2H_2D,
        ]);
        let board = Board::from_str("J♦ J♠ J♥ A♥ J♣").unwrap();
        Game { hands, board }
    }

    /// The 985th case at the flop when running `The Hand`:
    /// `RUST_LOG=trace cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"`
    #[must_use]
    pub fn case_985() -> [Card; 2] {
        [Card::SIX_CLUBS, Card::TREY_CLUBS]
    }

    /// # The Fold
    ///
    /// 5♠ 5♦ 9♠ 9♥ K♣ T♦ - 5♣ 9♦ T♥ T♣ Q♦
    /// HSP S09E13 Antonius, Negreanu, Ivey
    ///     <https://www.pokernews.com/news/2022/05/phil-ivey-negreanu-high-stakes-poker-41207.htm/>
    #[must_use]
    pub fn evals_the_fold() -> Vec<Eval> {
        let the_fold_hands = TestData::hole_cards_the_fold();
        let the_flop = Three::from([Card::FIVE_CLUBS, Card::NINE_DIAMONDS, Card::TEN_HEARTS]);
        the_fold_hands.three_into_evals(the_flop)
    }

    #[must_use]
    pub fn fives_the_fold() -> Vec<Five> {
        let the_fold_hands = TestData::hole_cards_the_fold();
        let the_flop = Three::from([Card::FIVE_CLUBS, Card::NINE_DIAMONDS, Card::TEN_HEARTS]);
        the_fold_hands.three_into_fives(the_flop)
    }

    /// I am deliberately keeping these hands out of order, to facilitate sorting tests
    /// later on.
    #[must_use]
    pub fn hole_cards_the_fold() -> HoleCards {
        HoleCards::from(vec![Two::HAND_5S_5D, Two::HAND_KC_TD, Two::HAND_9S_9H])
    }

    #[must_use]
    pub fn hole_cards_the_hand() -> HoleCards {
        HoleCards::from(vec![Two::HAND_6S_6H, Two::HAND_5D_5C])
    }

    #[must_use]
    pub fn the_flop() -> Three {
        Three::from([Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS])
    }

    #[must_use]
    pub fn daniel_eval_at_flop() -> Eval {
        Eval::from(TestData::daniel_hand_at_flop())
    }

    #[must_use]
    pub fn daniel_hand_at_flop() -> Five {
        Five::from_2and3(Two::HAND_6S_6H, TestData::the_flop())
    }

    /// DEFECT: Wrong hand. FIXED
    #[must_use]
    pub fn gus_eval_at_flop() -> Eval {
        Eval::from(TestData::gus_hand_at_flop())
    }

    #[must_use]
    pub fn gus_hand_at_flop() -> Five {
        Five::from_2and3(Two::HAND_5D_5C, TestData::the_flop())
    }

    #[must_use]
    pub fn wins_the_hand() -> Wins {
        let mut wins = Wins::default();

        wins.add_x(Win::FIRST, 1_365_284); // Daniel Wins
        wins.add_x(Win::SECOND, 314_904); // Gus Wins
        wins.add_x(Win::FIRST | Win::SECOND, 32_116); // Ties

        wins
    }

    /// # Panics
    ///
    /// ¯\_(ツ)_/¯
    #[must_use]
    pub fn spades_royal_flush_bcm() -> SevenFiveBCM {
        SevenFiveBCM::try_from(Seven::from_str("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠").unwrap()).unwrap()
    }

    /// # Panics
    ///
    /// ¯\_(ツ)_/¯
    #[must_use]
    pub fn spades_king_high_flush_bcm() -> SevenFiveBCM {
        SevenFiveBCM::try_from(Seven::from_str("K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠").unwrap()).unwrap()
    }

    /// This data comes from my old [Fudd hup example](https://github.com/ImperialBower/fudd/blob/main/examples/hup.rs)
    /// which was painstakingly slow.
    #[must_use]
    pub fn the_hand_as_hup_result() -> HUPResult {
        HUPResult {
            higher: Bard::SIX_SPADES | Bard::SIX_HEARTS,
            lower: Bard::FIVE_DIAMONDS | Bard::FIVE_CLUBS,
            higher_wins: 1_365_284,
            lower_wins: 314_904,
            ties: 32_116,
        }
    }

    #[must_use]
    pub fn the_hand_sorted_headsup() -> SortedHeadsUp {
        SortedHeadsUp::new(Two::HAND_6S_6H, Two::HAND_5D_5C)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__data_tests {
    use super::*;

    fn shu_hup_alignment() {}
}
