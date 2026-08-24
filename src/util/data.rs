use crate::analysis::eval::Eval;
use crate::analysis::gto::odds::WinLoseDraw;
#[cfg(all(feature = "store", not(target_arch = "wasm32")))]
use crate::analysis::store::bcm::binary_card_map::SevenFiveBCM;
use crate::analysis::store::db::hup::HUPResult;
use crate::arrays::five::Five;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
#[cfg(all(feature = "store", not(target_arch = "wasm32")))]
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::casino::table::Player;
use crate::casino::table::Seat;
use crate::play::board::Board;
use crate::play::game::Game;
use crate::play::hole_cards::HoleCards;
use crate::prelude::{BoxedCards, ForcedBets, Forgiving, Seats, Table};
use crate::{Card, Cards, Pile};
use std::str::FromStr;
use wincounter::win::Win;
use wincounter::wins::Wins;

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
#[allow(non_snake_case, clippy::unwrap_used, clippy::expect_used)]
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
    #[cfg(all(feature = "store", not(target_arch = "wasm32")))]
    pub fn spades_royal_flush_bcm() -> SevenFiveBCM {
        SevenFiveBCM::try_from(Seven::from_str("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠").unwrap_or_default()).unwrap_or_default()
    }

    /// # Panics
    ///
    /// ¯\_(ツ)_/¯
    #[must_use]
    #[cfg(all(feature = "store", not(target_arch = "wasm32")))]
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

    /// A full 52-card deck with [`TestData::the_hand_cards_dealable`] on top, in
    /// dealing order, and the rest of the deck behind it.
    #[must_use]
    pub fn deck_the_hand_dealable() -> Cards {
        Cards::deck_primed(&TestData::the_hand_cards_dealable())
    }

    /// The eight players of "The Hand", seated in order, holding nothing.
    ///
    /// The roster lives here and nowhere else;
    /// [`the_hand_seats`](TestData::the_hand_seats) is the same eight with
    /// their cards.
    #[must_use]
    pub fn the_hand_players() -> Vec<Seat> {
        vec![
            Seat::new_with_cards(
                Player::new_with_chips("Doyle Brunson".to_string(), 1_000_000),
                BoxedCards::blanks(2),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Eli Elezra".to_string(), 1_000_000),
                BoxedCards::blanks(2),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Antonio Esfandari".to_string(), 1_000_000),
                BoxedCards::blanks(2),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Gus Hansen".to_string(), 1_000_000),
                BoxedCards::blanks(2),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Daniel Negreanu".to_string(), 1_000_000),
                BoxedCards::blanks(2),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Cory Zeidman".to_string(), 1_000_000),
                BoxedCards::blanks(2),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Barry Greenstein".to_string(), 1_000_000),
                BoxedCards::blanks(2),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Amnon Filippi".to_string(), 1_000_000),
                BoxedCards::blanks(2),
            ),
        ]
    }

    /// The eight players of "The Hand", each already holding the two cards
    /// they held on the night.
    #[must_use]
    pub fn the_hand_seats() -> Vec<Seat> {
        vec![
            Seat::new_with_cards(
                Player::new_with_chips("Doyle Brunson".to_string(), 1_000_000),
                boxed!("T♠ 2♥"),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Eli Elezra".to_string(), 1_000_000),
                boxed!("8♠ 3♥"),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Antonio Esfandari".to_string(), 1_000_000),
                boxed!("A♦ Q♣"),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Gus Hansen".to_string(), 1_000_000),
                boxed!("5♦ 5♣"),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Daniel Negreanu".to_string(), 1_000_000),
                boxed!("6♠ 6♥"),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Cory Zeidman".to_string(), 1_000_000),
                boxed!("K♠ J♦"),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Barry Greenstein".to_string(), 1_000_000),
                boxed!("4♣ 4♦"),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("Amnon Filippi".to_string(), 1_000_000),
                boxed!("7♣ 2♣"),
            ),
        ]
    }

    /// [`the_hand_seats`](TestData::the_hand_seats) as a ring.
    #[must_use]
    pub fn the_hand_dealt_seats() -> Seats {
        Seats::new(TestData::the_hand_seats())
    }

    /// The three seats [`min_table`](TestData::min_table) uses, holding nothing.
    #[must_use]
    pub fn min_players() -> Vec<Seat> {
        Vec::from(&TestData::the_hand_players()[2..5])
    }

    /// Re-orders a stacked dealing list built for `TableCelled` so the plain
    /// [`Table`] deals the same cards to the same seats.
    ///
    /// The two engines start dealing from different seats. `TableCelled` begins
    /// **at** the button (`src/casino/table_celled.rs`, via
    /// `DrainableBintCell::new_with_value(.., button)`); `Table` begins **one
    /// seat to its left** (`(button + 1 + step) % seat_count`), which is the
    /// actual poker rule. Each dealing pass therefore rotates left by one; the
    /// burn and board cards that follow the hole cards are untouched.
    #[must_use]
    fn rotated_for_plain_deal(dealable: &Cards, seat_count: usize, cards_per: usize) -> Cards {
        let all: Vec<Card> = dealable.iter().copied().collect();
        let hole_count = seat_count * cards_per;
        let mut out: Vec<Card> = Vec::with_capacity(all.len());

        for pass in 0..cards_per {
            let start = pass * seat_count;
            let end = (start + seat_count).min(all.len());
            let mut slice: Vec<Card> = all[start..end].to_vec();
            slice.rotate_left(1);
            out.extend(slice);
        }
        out.extend_from_slice(&all[hole_count.min(all.len())..]);

        Cards::from(out)
    }

    /// cargo run --example calc -- -d "A♦ Q♣ 6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"
    ///
    /// ```shell
    /// hole cards> A♦ Q♣ 5♦ 5♣ 6♠ 6♥
    /// Player #1 38.4% (38.15%/0.29%) [522929/4035]
    /// Player #2 16.7% (16.43%/0.29%) [225186/4035]
    /// Player #3 45.4% (45.13%/0.29%) [618604/4035]
    /// ```
    /// A three-handed table with a stacked deck: Antonio, Gus and Daniel,
    /// a 9♣ 6♦ 5♥ 5♠ 8♠ board, and the burns in between.
    #[must_use]
    pub fn min_table() -> Table {
        // Layout: [hole×6] [burn1] [flop×3] [burn2] [turn] [burn3] [river]
        // Burns 2♦ 3♦ 4♦ are arbitrary cards not in hole cards or the board.
        let primed = cards!("A♦ 5♦ 6♠ Q♣ 5♣ 6♥ 2♦ 9♣ 6♦ 5♥ 3♦ 5♠ 4♦ 8♠");
        let primed = TestData::rotated_for_plain_deal(&primed, 3, 2);
        Table::nlh_primed(
            Seats::new(TestData::min_players()),
            &Cards::deck_primed(&primed),
            ForcedBets::new(50, 100),
        )
    }

    /// "The Hand" fixture on the plain [`Table`] engine.
    #[must_use]
    pub fn the_hand_table() -> Table {
        let primed = TestData::rotated_for_plain_deal(&TestData::the_hand_cards_dealable(), 8, 2);
        Table::nlh_primed(
            Seats::new(TestData::the_hand_players()),
            &Cards::deck_primed(&primed),
            ForcedBets::new(50, 100),
        )
    }

    /// The three seats [`min_table`](TestData::min_table) uses, with cards.
    #[must_use]
    pub fn min_seats() -> Vec<Seat> {
        Vec::from(&TestData::the_hand_seats()[2..5])
    }

    #[must_use]
    pub fn four_seats() -> Vec<Seat> {
        Vec::from(&TestData::the_hand_seats()[2..6])
    }

    /// Three stacks of different sizes, each already holding its cards, on a
    /// deck stacked to run out `cards`.
    ///
    /// | Seat | Name           | Stack  | Holds |
    /// |------|----------------|--------|-------|
    /// | 0    | Rich Man       | 10,000 | Q♦ Q♣ |
    /// | 1    | Poor Man       |  5,000 | A♠ A♥ |
    /// | 2    | Average Person |  9,000 | 4♣ 4♦ |
    ///
    /// Three unequal stacks all-in is the shortest route to a side pot, which
    /// is what the fixture exists to exercise.
    #[must_use]
    pub fn split_pot_table(cards: &Cards) -> Table {
        let seats = Seats::new(vec![
            Seat::new_with_cards(Player::new_with_chips("Rich Man".to_string(), 10_000), boxed!("Q♦ Q♣")),
            Seat::new_with_cards(Player::new_with_chips("Poor Man".to_string(), 5_000), boxed!("A♠ A♥")),
            Seat::new_with_cards(
                Player::new_with_chips("Average Person".to_string(), 9_000),
                boxed!("4♣ 4♦"),
            ),
        ]);

        Table::nlh_primed(seats, cards, ForcedBets::new(50, 100))
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

    #[test]
    fn deck_the_hand_dealable__is_a_full_deck_primed_with_the_hand() {
        let deck = TestData::deck_the_hand_dealable();
        let primed = TestData::the_hand_cards_dealable();

        assert_eq!(52, deck.len());
        assert!(deck.are_unique());
        assert_eq!(primed.to_vec().as_slice(), &deck.to_vec()[..primed.len()]);
    }
}
