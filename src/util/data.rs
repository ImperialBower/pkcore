use crate::analysis::eval::Eval;
use crate::analysis::gto::odds::WinLoseDraw;
use crate::analysis::store::bcm::binary_card_map::SevenFiveBCM;
use crate::analysis::store::db::hup::HUPResult;
use crate::arrays::five::Five;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::cards_cell::CardsCell;
use crate::casino::player::Player;
use crate::casino::table::seat::Seat;
use crate::play::board::Board;
use crate::play::game::Game;
use crate::play::hole_cards::HoleCards;
use crate::prelude::{BoxedCards, ForcedBets, Forgiving, Seats, Table};
use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;
use crate::{Card, Cards, Pile};
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
    pub fn the_hand() -> Game {
        let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap_or_default();

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
        let hands = HoleCards::from(vec![Two::HAND_AC_QS, Two::HAND_TD_TC, Two::HAND_6D_4D, Two::HAND_2H_2D]);
        let board = Board::from_str("J♦ J♠ J♥ A♥ J♣").unwrap_or_default();
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
    pub fn the_hand_as_wins() -> Wins {
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
        SevenFiveBCM::try_from(Seven::from_str("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠").unwrap_or_default()).unwrap_or_default()
    }

    /// # Panics
    ///
    /// ¯\_(ツ)_/¯
    #[must_use]
    pub fn spades_king_high_flush_bcm() -> SevenFiveBCM {
        SevenFiveBCM::try_from(Seven::from_str("K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠").unwrap_or_default()).unwrap_or_default()
    }

    /// This data comes from my old [Fudd hup example](https://github.com/ImperialBower/fudd/blob/main/examples/hup.rs)
    /// which was painstakingly slow.
    #[must_use]
    pub fn the_hand_as_hup_result() -> HUPResult {
        HUPResult {
            higher: Bard::SIX_SPADES | Bard::SIX_HEARTS,
            lower: Bard::FIVE_DIAMONDS | Bard::FIVE_CLUBS,
            odds: WinLoseDraw {
                wins: 1_365_284,
                losses: 314_904,
                draws: 32_116,
            },
        }
    }

    #[must_use]
    pub fn the_hand_sorted_headsup() -> SortedHeadsUp {
        SortedHeadsUp::new(Two::HAND_6S_6H, Two::HAND_5D_5C)
    }

    #[must_use]
    pub fn known_hups() -> Vec<HUPResult> {
        let mut hups: Vec<HUPResult> = vec![HUPResult {
            higher: Two::HAND_AS_AH.bard(),
            lower: Two::HAND_7D_7C.bard(),
            odds: WinLoseDraw {
                wins: 1364608,
                losses: 343300,
                draws: 4396,
            },
        }];

        hups.push(HUPResult {
            higher: Two::HAND_AS_AH.bard(),
            lower: Two::HAND_7D_7C.bard(),
            odds: WinLoseDraw {
                wins: 1364608,
                losses: 343300,
                draws: 4396,
            },
        });
        hups.push(HUPResult {
            higher: Two::HAND_AS_AH.bard(),
            lower: Two::HAND_6D_6C.bard(),
            odds: WinLoseDraw {
                wins: 1364608,
                losses: 343300,
                draws: 4396,
            },
        });
        hups.push(HUPResult {
            higher: Two::HAND_AS_AH.bard(),
            lower: Two::HAND_5D_5C.bard(),
            odds: WinLoseDraw {
                wins: 1364608,
                losses: 343300,
                draws: 4396,
            },
        });

        hups
    }

    #[must_use]
    pub fn the_hand_cards() -> Cards {
        cards!("T♠ 2♥ 8♣ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♦ 4♣ 7♣ 9♣ 9♣ 6♦ 5♥ 5♠ 8♠")
    }

    #[must_use]
    pub fn the_hand_cards_dealable() -> Cards {
        cards!("T♠ 8♣ A♦ 5♦ 6♠ K♠ 4♦ 7♣ 2♥ 3♥ Q♣ 5♣ 6♥ J♦ 4♣ 2♦ 9♣ 6♦ 5♥ 5♠ 8♠")
    }

    /// # Panics
    ///
    /// Because of `draw_from_the_bottom`, but this is test data so... ¯\_(ツ)_/¯
    #[must_use]
    pub fn deck_the_hand_dealable() -> Cards {
        // let mut dealt = TestData::the_hand_cards_dealable();
        // let mut minus = Cards::deck_minus(&dealt).shuffle();
        // let river = dealt.draw_from_the_bottom(1).unwrap();
        // let turn = dealt.draw_from_the_bottom(1).unwrap();
        // let flop = dealt.draw_from_the_bottom(3).unwrap();

        todo!()
    }

    /// # Panics
    ///
    /// Because of `CardsCell` usage, but this is test data so... ¯\_(ツ)_/¯
    #[must_use]
    pub fn the_hand_players() -> Vec<Seat> {
        let doyle_brunson = Seat {
            player: Player::new_with_chips("Doyle Brunson".to_string(), 1_000_000),
            cards: BoxedCards::blanks(2),
        };
        let eli_elezra = Seat {
            player: Player::new_with_chips("Eli Elezra".to_string(), 1_000_000),
            cards: BoxedCards::blanks(2),
        };
        let antonio_esfandiari = Seat {
            player: Player::new_with_chips("Antonio Esfandari".to_string(), 1_000_000),
            cards: BoxedCards::blanks(2),
        };
        let gus_hansen = Seat {
            player: Player::new_with_chips("Gus Hansen".to_string(), 1_000_000),
            cards: BoxedCards::blanks(2),
        };
        let daniel_negreanu = Seat {
            player: Player::new_with_chips("Daniel Negreanu".to_string(), 1_000_000),
            cards: BoxedCards::blanks(2),
        };
        let cory_zeidman = Seat {
            player: Player::new_with_chips("Cory Zeidman".to_string(), 1_000_000),
            cards: BoxedCards::blanks(2),
        };
        let barry_greenstein = Seat {
            player: Player::new_with_chips("Barry Greenstein".to_string(), 1_000_000),
            cards: BoxedCards::blanks(2),
        };
        let amnon_filippi = Seat {
            player: Player::new_with_chips("Amnon Filippi".to_string(), 1_000_000),
            cards: BoxedCards::blanks(2),
        };
        vec![
            doyle_brunson,
            eli_elezra,
            antonio_esfandiari,
            gus_hansen,
            daniel_negreanu,
            cory_zeidman,
            barry_greenstein,
            amnon_filippi,
        ]
    }

    /// # Panics
    ///
    /// Because of `CardsCell` usage, but this is test data so... ¯\_(ツ)_/¯
    #[must_use]
    pub fn the_hand_seats() -> Vec<Seat> {
        let doyle_brunson = Seat {
            player: Player::new_with_chips("Doyle Brunson".to_string(), 1_000_000),
            cards: boxed!("T♠ 2♥"),
        };
        let eli_elezra = Seat {
            player: Player::new_with_chips("Eli Elezra".to_string(), 1_000_000),
            cards: boxed!("8♠ 3♥"),
        };
        let antonio_esfandiari = Seat {
            player: Player::new_with_chips("Antonio Esfandari".to_string(), 1_000_000),
            cards: boxed!("A♦ Q♣"),
        };
        let gus_hansen = Seat {
            player: Player::new_with_chips("Gus Hansen".to_string(), 1_000_000),
            cards: boxed!("5♦ 5♣"),
        };
        let daniel_negreanu = Seat {
            player: Player::new_with_chips("Daniel Negreanu".to_string(), 1_000_000),
            cards: boxed!("6♠ 6♥"),
        };
        let cory_zeidman = Seat {
            player: Player::new_with_chips("Cory Zeidman".to_string(), 1_000_000),
            cards: boxed!("K♠ J♦"),
        };
        let barry_greenstein = Seat {
            player: Player::new_with_chips("Barry Greenstein".to_string(), 1_000_000),
            cards: boxed!("4♣ 4♦"),
        };
        let amnon_filippi = Seat {
            player: Player::new_with_chips("Amnon Filippi".to_string(), 1_000_000),
            cards: boxed!("7♣ 2♣"),
        };
        vec![
            doyle_brunson,
            eli_elezra,
            antonio_esfandiari,
            gus_hansen,
            daniel_negreanu,
            cory_zeidman,
            barry_greenstein,
            amnon_filippi,
        ]
    }

    #[must_use]
    pub fn min_players() -> Vec<Seat> {
        Vec::from(&TestData::the_hand_players()[2..5])
    }

    /// cargo run --example calc -- -d "A♦ Q♣ 6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"
    ///
    /// ```shell
    /// hole cards> A♦ Q♣ 5♦ 5♣ 6♠ 6♥
    /// Player #1 38.4% (38.15%/0.29%) [522929/4035]
    /// Player #2 16.7% (16.43%/0.29%) [225186/4035]
    /// Player #3 45.4% (45.13%/0.29%) [618604/4035]
    /// ```
    #[must_use]
    pub fn min_table() -> Table {
        let primed = cards!("A♦ 5♦ 6♠ Q♣ 5♣ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠");
        Table::nlh_primed(
            Seats::new(TestData::min_players()),
            &CardsCell::from(Cards::deck_primed(&primed)),
            ForcedBets::new(50, 100),
        )
    }

    #[must_use]
    pub fn the_hand_table() -> Table {
        Table::nlh_primed(
            Seats::new(TestData::the_hand_players()),
            &CardsCell::from(Cards::deck_primed(&TestData::the_hand_cards_dealable())),
            ForcedBets::new(50, 100),
        )
    }

    /// # Panics
    ///
    /// Because of `CardsCell` usage, but this is test data so... ¯\_(ツ)_/¯
    #[must_use]
    pub fn min_seats() -> Vec<Seat> {
        Vec::from(&TestData::the_hand_seats()[2..5])
    }

    #[must_use]
    pub fn four_seats() -> Vec<Seat> {
        Vec::from(&TestData::the_hand_seats()[2..6])
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__data_tests {
    use super::*;

    /// We want to make sure that our test data enforces the correct contract of the structs that
    /// we are validating with it.
    #[test]
    fn shu_hup_alignment() {
        let hup = TestData::the_hand_as_hup_result();
        let wins = TestData::the_hand_as_wins();
        let (first_wins, first_ties) = wins.wins_for(Win::FIRST);
        let (second_wins, second_ties) = wins.wins_for(Win::SECOND);

        assert_eq!(hup.odds.wins as usize, first_wins - first_ties);
        assert_eq!(hup.odds.losses as usize, second_wins - second_ties);
    }
}
