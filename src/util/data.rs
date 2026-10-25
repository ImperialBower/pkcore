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
use crate::casino::action::PlayerAction;
use crate::casino::table::Player;
use crate::casino::table::Seat;
use crate::hand_history::ReplayStreet;
use crate::play::board::Board;
use crate::play::game::Game;
use crate::play::hole_cards::HoleCards;
use crate::prelude::{BoxedCards, ForcedBets, Forgiving, Seats, Table};
use crate::{Card, Cards, Pile};
use std::str::FromStr;
use uuid::Uuid;
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
    /// A fixture player with a fixed id, so every fixture table is identical run
    /// to run. Ids are stable per name: the same person is the same id in every
    /// fixture.
    // `docs/KERNEL_PURITY_AUDIT.md` fix 8: fixtures used to mint random ids.
    fn fixture_player(id: u128, handle: &str, stack: usize) -> Player {
        Player::with_id(Uuid::from_u128(id), handle.to_string(), stack)
    }

    #[must_use]
    pub fn the_hand() -> Game {
        let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap_or_default();

        Game {
            hands: TestData::hole_cards_the_hand(),
            board,
            dead: Cards::default(),
        }
    }

    /// Based on HSP S04E08 Harman/Safai but with the river bringing quads
    /// `cargo run --example calc -- -d "A♣ Q♠ T♦ T♣ 6♦ 4♦ 2♥ 2♦" -b "J♦ J♠ J♥ A♥ J♣"`
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn the_board() -> Game {
        let hands = HoleCards::from(vec![Two::HAND_AC_QS, Two::HAND_TD_TC, Two::HAND_6D_4D, Two::HAND_2H_2D]);
        let board = Board::from_str("J♦ J♠ J♥ A♥ J♣").unwrap_or_default();
        Game {
            hands,
            board,
            dead: Cards::default(),
        }
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
                wins: 1_364_608,
                losses: 343_300,
                draws: 4396,
            },
        }];

        hups.push(HUPResult {
            higher: Two::HAND_AS_AH.bard(),
            lower: Two::HAND_7D_7C.bard(),
            odds: WinLoseDraw {
                wins: 1_364_608,
                losses: 343_300,
                draws: 4396,
            },
        });
        hups.push(HUPResult {
            higher: Two::HAND_AS_AH.bard(),
            lower: Two::HAND_6D_6C.bard(),
            odds: WinLoseDraw {
                wins: 1_364_608,
                losses: 343_300,
                draws: 4396,
            },
        });
        hups.push(HUPResult {
            higher: Two::HAND_AS_AH.bard(),
            lower: Two::HAND_5D_5C.bard(),
            odds: WinLoseDraw {
                wins: 1_364_608,
                losses: 343_300,
                draws: 4396,
            },
        });

        hups
    }

    /// "The Hand"'s 21 cards: each seat's hole cards, seat 0 to seat 7, then
    /// the board.
    #[must_use]
    pub fn the_hand_cards() -> Cards {
        cards!("T♠ 2♥ 8♣ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♦ 4♣ 7♣ 2♣ 9♣ 6♦ 5♥ 5♠ 8♠")
    }

    /// "The Hand" in the order a dealer starting at the button deals it: the
    /// hole cards seat 0 to seat 7, two passes, then burn, flop, burn, turn,
    /// burn, river. The burns (A♠, Q♠, J♠) are not from the broadcast; they are
    /// the first three cards of [`Cards::deck`] that the hand does not use,
    /// which is what [`HandHistory::stacked_deck`](crate::hand_history::HandHistory::stacked_deck)
    /// fills a burn with, so the recorded hand rebuilds this very deck.
    ///
    /// [`the_hand_table`](TestData::the_hand_table) rotates the hole cards for
    /// [`Table`], which deals from the seat after the button.
    #[must_use]
    pub fn the_hand_cards_dealable() -> Cards {
        cards!("T♠ 8♣ A♦ 5♦ 6♠ K♠ 4♦ 7♣ 2♥ 3♥ Q♣ 5♣ 6♥ J♦ 4♣ 2♣ A♠ 9♣ 6♦ 5♥ Q♠ 5♠ J♠ 8♠")
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
        Self::the_hand_players_with_stacks([1_000_000; 8])
    }

    /// The names of "The Hand"'s eight players, seat 0 first.
    const THE_HAND_NAMES: [&'static str; 8] = [
        "Doyle Brunson",
        "Eli Elezra",
        "Antonio Esfandiari",
        "Gus Hansen",
        "Daniel Negreanu",
        "Cory Zeidman",
        "Barry Greenstein",
        "Amnon Filippi",
    ];

    /// The stacks "The Hand" was played with, seat 0 first: Hansen's 287,100
    /// is the broadcast's (his river all in closes the 575,700 pot). The rest
    /// are ESTIMATES that no source publishes: Negreanu 387,100 (he had to
    /// cover Hansen) and 100,000 for each of the six who fold. They change no
    /// pot, no action and no result (`DEFECT_040`).
    const THE_HAND_BROADCAST_STACKS: [usize; 8] =
        [100_000, 100_000, 100_000, 287_100, 387_100, 100_000, 100_000, 100_000];

    fn the_hand_players_with_stacks(stacks: [usize; 8]) -> Vec<Seat> {
        Self::THE_HAND_NAMES
            .iter()
            .zip(stacks)
            .zip(1_u128..)
            .map(|((name, stack), id)| {
                Seat::new_with_cards(Self::fixture_player(id, name, stack), BoxedCards::blanks(2))
            })
            .collect()
    }

    /// The eight players of "The Hand", each already holding the two cards
    /// they held on the night.
    #[must_use]
    pub fn the_hand_seats() -> Vec<Seat> {
        vec![
            Seat::new_with_cards(Self::fixture_player(1, "Doyle Brunson", 1_000_000), boxed!("T♠ 2♥")),
            Seat::new_with_cards(Self::fixture_player(2, "Eli Elezra", 1_000_000), boxed!("8♣ 3♥")),
            Seat::new_with_cards(
                Self::fixture_player(3, "Antonio Esfandiari", 1_000_000),
                boxed!("A♦ Q♣"),
            ),
            Seat::new_with_cards(Self::fixture_player(4, "Gus Hansen", 1_000_000), boxed!("5♦ 5♣")),
            Seat::new_with_cards(Self::fixture_player(5, "Daniel Negreanu", 1_000_000), boxed!("6♠ 6♥")),
            Seat::new_with_cards(Self::fixture_player(6, "Cory Zeidman", 1_000_000), boxed!("K♠ J♦")),
            Seat::new_with_cards(Self::fixture_player(7, "Barry Greenstein", 1_000_000), boxed!("4♣ 4♦")),
            Seat::new_with_cards(Self::fixture_player(8, "Amnon Filippi", 1_000_000), boxed!("7♣ 2♣")),
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
        let primed = primed.rotated_for_plain_deal(3, 2);
        Table::nlh_primed_with_id(
            Seats::new(TestData::min_players()),
            &Cards::deck_primed(&primed),
            ForcedBets::new(50, 100),
            Uuid::nil(),
        )
    }

    /// "The Hand" fixture on the plain [`Table`] engine.
    #[must_use]
    pub fn the_hand_table() -> Table {
        let primed = TestData::the_hand_cards_dealable().rotated_for_plain_deal(8, 2);
        Table::nlh_primed_with_id(
            Seats::new(TestData::the_hand_players()),
            &Cards::deck_primed(&primed),
            ForcedBets::new(50, 100),
            Uuid::nil(),
        )
    }

    /// "The Hand" at the broadcast's stakes: $300/$600 with a $100 ante, the
    /// button on seat 0 (Brunson), Elezra in the small blind and Esfandiari in
    /// the big, dealt from the same stacked deck as
    /// [`the_hand_table`](TestData::the_hand_table).
    ///
    /// Hansen (seat 3) has the broadcast's 287,100. The other seven stacks
    /// are ESTIMATES, not read off the episode: Negreanu (seat 4) 387,100, the
    /// six who fold 100,000 each. Play it with
    /// [`the_hand_line`](TestData::the_hand_line).
    ///
    /// ```
    /// use pkcore::util::data::TestData;
    ///
    /// let table = TestData::the_hand_broadcast_table();
    /// assert_eq!(0, table.button);
    /// assert_eq!(100, table.forced.ante);
    /// assert_eq!(287_100, table.seats.get_seat(3).unwrap().player.chips);
    /// ```
    #[must_use]
    pub fn the_hand_broadcast_table() -> Table {
        let primed = TestData::the_hand_cards_dealable().rotated_for_plain_deal(8, 2);
        Table::nlh_primed_with_id(
            Seats::new(Self::the_hand_players_with_stacks(Self::THE_HAND_BROADCAST_STACKS)),
            &Cards::deck_primed(&primed),
            ForcedBets::new_with_ante(300, 600, 100),
            Uuid::nil(),
        )
    }

    /// The nineteen voluntary actions of "The Hand", in order, as
    /// `(street, seat, action)`, seats 0-based. `Bet(n)` and `Raise(n)` are
    /// to `n` for the street, as [`Table::apply_action`] takes them.
    ///
    /// | Street  | Action |
    /// |---------|--------|
    /// | Preflop | Hansen (3) raises to 2,100, Negreanu (4) to 5,000, seats 5, 6, 7, 0, 1, 2 fold, Hansen calls |
    /// | Flop    | Hansen checks, Negreanu bets 8,000, Hansen raises to 26,000, Negreanu calls |
    /// | Turn    | Hansen bets 24,000, Negreanu calls |
    /// | River   | Hansen checks, Negreanu bets 65,000, Hansen is all in, Negreanu calls |
    ///
    /// On [`the_hand_broadcast_table`](TestData::the_hand_broadcast_table) it
    /// ends with a 575,700 pot.
    ///
    /// ```
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::hand_history::ReplayStreet;
    /// use pkcore::util::data::TestData;
    ///
    /// let line = TestData::the_hand_line();
    /// assert_eq!(19, line.len());
    /// assert_eq!((ReplayStreet::Preflop, 3, PlayerAction::Raise(2_100)), line[0]);
    /// assert_eq!((ReplayStreet::River, 4, PlayerAction::Call), line[18]);
    /// ```
    #[must_use]
    pub fn the_hand_line() -> Vec<(ReplayStreet, u8, PlayerAction)> {
        use PlayerAction::{AllIn, Bet, Call, Check, Fold, Raise};
        use ReplayStreet::{Flop, Preflop, River, Turn};
        vec![
            (Preflop, 3, Raise(2_100)),
            (Preflop, 4, Raise(5_000)),
            (Preflop, 5, Fold),
            (Preflop, 6, Fold),
            (Preflop, 7, Fold),
            (Preflop, 0, Fold),
            (Preflop, 1, Fold),
            (Preflop, 2, Fold),
            (Preflop, 3, Call),
            (Flop, 3, Check),
            (Flop, 4, Bet(8_000)),
            (Flop, 3, Raise(26_000)),
            (Flop, 4, Call),
            (Turn, 3, Bet(24_000)),
            (Turn, 4, Call),
            (River, 3, Check),
            (River, 4, Bet(65_000)),
            (River, 3, AllIn),
            (River, 4, Call),
        ]
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
            Seat::new_with_cards(Self::fixture_player(9, "Rich Man", 10_000), boxed!("Q♦ Q♣")),
            Seat::new_with_cards(Self::fixture_player(10, "Poor Man", 5_000), boxed!("A♠ A♥")),
            Seat::new_with_cards(Self::fixture_player(11, "Average Person", 9_000), boxed!("4♣ 4♦")),
        ]);

        Table::nlh_primed_with_id(seats, cards, ForcedBets::new(50, 100), Uuid::nil())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__data_tests {
    use super::*;

    /// We want to make sure that our test data enforces the correct contract of the structs that
    /// we are validating with it.
    #[test]
    fn the_hand_cards__is_every_hole_card_and_the_board_once() {
        let expected: Cards = TestData::the_hand_seats()
            .iter()
            .flat_map(|seat| seat.cards.cards().iter().copied().collect::<Vec<_>>())
            .chain(cards!("9♣ 6♦ 5♥ 5♠ 8♠").iter().copied())
            .collect();

        let cards = TestData::the_hand_cards();
        assert_eq!(21, cards.len(), "{cards}");
        assert_eq!(expected.sort(), cards.sort());
    }

    #[test]
    fn shu_hup_alignment() {
        let hup = TestData::the_hand_as_hup_result();
        let wins = TestData::the_hand_as_wins();
        let (first_wins, first_ties) = wins.wins_for(Win::FIRST);
        let (second_wins, second_ties) = wins.wins_for(Win::SECOND);

        assert_eq!(hup.odds.wins as usize, first_wins - first_ties);
        assert_eq!(hup.odds.losses as usize, second_wins - second_ties);
    }

    /// `DEFECT_040` item 9: the burns are in the dealable list, so the plain
    /// `Table` deals the board the broadcast showed. Item 8: no card is dealt
    /// twice, and every seat holds what [`TestData::the_hand_seats`] says.
    #[test]
    fn the_hand_table__deals_the_broadcast_board() {
        let mut table = TestData::the_hand_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        table.deal_flop().unwrap();
        table.deal_turn().unwrap();
        table.deal_river().unwrap();

        assert_eq!("9♣ 6♦ 5♥ 5♠ 8♠", table.board.to_string());

        let mut dealt: Vec<Card> = table.board.iter().copied().collect();
        for (seat, expected) in table.seats.iter().zip(TestData::the_hand_seats()) {
            assert_eq!(
                expected.cards.sorted_display(),
                seat.cards.sorted_display(),
                "{}",
                seat.player.handle
            );
            dealt.extend(seat.cards.as_slice().iter().copied());
        }
        let unique: std::collections::HashSet<Card> = dealt.iter().copied().collect();
        assert_eq!(21, dealt.len());
        assert_eq!(dealt.len(), unique.len(), "a card was dealt twice: {dealt:?}");
    }

    /// The broadcast's numbers close: 2 × 287,100 + the six folders' 1,500 is
    /// 575,700, Hansen's all in is all he has, and Negreanu keeps the 100,000
    /// he did not need.
    #[test]
    fn the_hand_line__ends_at_575_700_for_hansen() {
        use crate::casino::session::PokerSession;
        use crate::hand_history::ReplayStreet;

        let mut session = PokerSession::new(TestData::the_hand_broadcast_table());
        let deck = session.table.deck.clone();
        session.start_hand_with_deck(deck).unwrap();

        let line = TestData::the_hand_line();
        assert_eq!(19, line.len());
        for (street, seat, action) in line {
            assert_eq!(Some(seat), session.next_actor().unwrap(), "{street} {seat} {action}");
            let board = match street {
                ReplayStreet::Preflop => 0,
                ReplayStreet::Flop => 3,
                ReplayStreet::Turn => 4,
                ReplayStreet::River => 5,
            };
            assert_eq!(board, session.table.board.len(), "{street} {seat} {action}");
            session.apply_action(seat, action).unwrap();
        }
        assert_eq!(None, session.next_actor().unwrap());
        assert_eq!(575_700, session.table.effective_pot());

        session.end_hand().unwrap();

        let chips = |seat: u8| session.table.seats.get_seat(seat).unwrap().player.chips;
        assert_eq!(575_700, chips(3), "Gus Hansen");
        assert_eq!(100_000, chips(4), "Daniel Negreanu");
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
