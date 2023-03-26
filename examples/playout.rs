use indexmap::IndexSet;
use pkcore::arrays::three::Three;
use pkcore::arrays::two::Two;
use pkcore::card::Card;
use pkcore::{Pile, PKError};
use std::str::FromStr;
use pkcore::analysis::case_eval::CaseEval;
use pkcore::analysis::case_evals::CaseEvals;
use pkcore::analysis::eval::Eval;
use pkcore::arrays::seven::Seven;
use pkcore::play::hole_cards::HoleCards;
use pkcore::util::wincounter::PlayerFlag;
use pkcore::util::wincounter::results::Results;

fn main() -> Result<(), PKError> {

    env_logger::init();

    as_written();

    Ok(())
}

fn as_written() -> Result<(), PKError> {
    let now = std::time::Instant::now();

    /// Hands that have been dealt to the players.
    let daniel = Two::HAND_6S_6H;
    let gus = Two::HAND_5D_5C;
    let hands = HoleCards::from(vec![daniel, gus]);

    /// # The Flop
    ///
    /// Cards dealt on the flop.
    let flop = Three::from_str("9♣ 6♦ 5♥")?;

    /// Instantiate the struct to hold the `CaseEvals`.
    let mut case_evals = CaseEvals::default();

    /// Instantiate the vector that will hold the binary win data for each case.
    let mut wins: Vec<PlayerFlag> = Vec::new();

    /// Utility class to help display win results.
    let mut results = Results::default();

    /// Iterate through every combination of cards not yet dealt.
    let combos = hands.combinations_after(2, &flop.cards());
    println!("Elapsed: {:.2?}", now.elapsed());
    for v in combos {

        let case = Two::from(v);

        let mut case_eval = CaseEval::default();
        for player in hands.iter() {
            let seven = Seven::from_case_at_flop(*player, flop, case)?;
            let eval = Eval::from(seven);
            case_eval.push(eval);
        }
        case_evals.push(case_eval);
    }

    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}