use indexmap::set::IntoIter;
use itertools::{Combinations, Itertools};
use log::debug;
use pkcore::analysis::the_nuts::TheNuts;
use pkcore::arrays::seven::Seven;
use pkcore::arrays::two::Two;
use pkcore::arrays::HandRanker;
use pkcore::card::Card;
use pkcore::play::board::Board;
use pkcore::util::wincounter::win::Win;
use pkcore::util::wincounter::wins::Wins;
use pkcore::util::wincounter::PlayerFlag;
use pkcore::{PKError, Pile};
use rayon::prelude::*;
use std::sync::mpsc;

/// We're going to move our current round of headbanging to this example file.
/// I need a place to spike out this shit, and so here it is. There has to be
/// cleaner ways of doing this.
///
/// The first thing that we'll need to do is get some map
///
/// Let's try something different. I'm thinking there must be a way to leverage
/// [Rayon's](https://github.com/rayon-rs/rayon) superfoo.
///
/// RUST_LOG=debug cargo run --example faceoff
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TwoBy2 {
    pub first: Two,
    pub second: Two,
}

impl TwoBy2 {
    pub const PREFLOP_COMBO_COUNT: usize = 1_712_304;
    pub const DEFAULT_WORKER_COUNT: usize = 10;

    fn combinations(&self) -> Combinations<IntoIter<Card>> {
        self.remaining().combinations(5)
    }

    #[allow(clippy::comparison_chain)]
    fn win_for_board(&self, board: &Board) -> PlayerFlag {
        let (first_value, _) =
            Seven::from_case_and_board(&self.first, board).hand_rank_value_and_hand();
        let (second_value, _) =
            Seven::from_case_and_board(&self.second, board).hand_rank_value_and_hand();

        if first_value == second_value {
            Win::FIRST | Win::SECOND
        } else if first_value < second_value {
            Win::FIRST
        } else {
            Win::SECOND
        }
    }

    fn to_wins(&self) -> Result<Wins, PKError> {
        let mut wins = Wins::default();
        let remaining = self.remaining();

        debug!("Hands: {}", self.cards());
        debug!("Remaining: {remaining}");
        let combos = remaining.combinations(5);
        let chunks =
            combos.chunks((TwoBy2::PREFLOP_COMBO_COUNT / TwoBy2::DEFAULT_WORKER_COUNT).max(5));
        let (sender, receiver) = mpsc::channel();

        for chunk in &chunks {
            for combo in chunk {
                let sender = sender.clone();

                let board = Board::try_from(combo)?;
                debug!("{}", board);

                // let win = self.win_for_board(&board);

                sender
                    .send(self.win_for_board(&board))
                    .expect("TODO: panic message");
            }
        }

        drop(sender);

        for received in receiver {
            wins.add(received);
        }

        Ok(wins)
    }
}

impl Pile for TwoBy2 {
    fn clean(&self) -> Self {
        TwoBy2::default()
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        vec![
            self.first.first(),
            self.first.second(),
            self.second.first(),
            self.second.second(),
        ]
    }
}

pub const HAND: TwoBy2 = TwoBy2 {
    first: Two::HAND_JC_4H,
    second: Two::HAND_8C_7C,
};

fn main() {
    env_logger::init();

    // let bloop = HAND.combinations().into_iter().map(|b| )

    // HAND.combinations().par_iter()

    // let actual_wins = HAND.to_wins().unwrap();

    // println!("{:?}", actual_wins);
}
