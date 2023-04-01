use pkcore::analysis::case_eval::CaseEval;
use pkcore::analysis::case_evals::CaseEvals;
use pkcore::analysis::eval::Eval;
use pkcore::arrays::seven::Seven;
use pkcore::arrays::three::Three;
use pkcore::arrays::two::Two;
use pkcore::play::hole_cards::HoleCards;
use pkcore::util::wincounter::results::Results;
use pkcore::util::wincounter::wins::Wins;
use pkcore::{PKError, Pile};
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

fn main() -> Result<(), PKError> {
    env_logger::init();

    as_written().expect("Oops!");

    println!("==========================================");

    concurrent().expect("Oops!");

    Ok(())
}

fn as_written() -> Result<(), PKError> {
    let now = std::time::Instant::now();

    // Hands that have been dealt to the players.
    let daniel = Two::HAND_6S_6H;
    let gus = Two::HAND_5D_5C;
    let hands = HoleCards::from(vec![daniel, gus]);

    // The Flop
    //
    // Cards dealt on the flop.
    let flop = Three::from_str("9♣ 6♦ 5♥")?;

    // Instantiate the struct to hold the `CaseEvals`.
    let mut case_evals = CaseEvals::default();

    // Utility class to help display win results.
    let _results = Results::default();

    //\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\
    // Collect the CaseEvals
    // Iterate through every combination of cards not yet dealt.
    let combos = hands.combinations_after(2, &flop.cards());
    println!("as_written Elapsed: {:.2?}", now.elapsed());
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

    //\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\
    // Convert the CaseEvals into wins
    // from FlopEval line 214
    let mut wins = Wins::default();
    for case_eval in case_evals.iter() {
        wins.add(case_eval.win_count());
    }

    //\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\
    // Convert wins into results
    // from FlopEval line 215
    let results = Results::from_wins(&wins, hands.len());

    println!("as_written");
    println!("{results}");

    println!("as_written Elapsed: {:.2?}", now.elapsed());
    Ok(())
}

fn concurrent() -> Result<(), PKError> {
    let now = std::time::Instant::now();

    // Hands that have been dealt to the players.
    let daniel = Two::HAND_6S_6H;
    let gus = Two::HAND_5D_5C;
    let hands = HoleCards::from(vec![daniel, gus]);

    // The Flop
    //
    // Cards dealt on the flop.
    let flop = Three::from_str("9♣ 6♦ 5♥")?;

    // Instantiate the struct to hold the `CaseEvals`.
    let mut case_evals = CaseEvals::default();

    //\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\
    // Collect the CaseEvals
    // Iterate through every combination of cards not yet dealt.
    let combos = hands.combinations_after(2, &flop.cards());
    println!("concurrent Elapsed: {:.2?}", now.elapsed());

    let (tx, rx) = mpsc::channel();

    for v in combos {
        let tx = tx.clone();
        let my_hands = hands.clone();
        thread::spawn(move || {
            let case = Two::from(v);

            let mut case_eval = CaseEval::default();
            for player in my_hands.iter() {
                let seven = Seven::from_case_at_flop(*player, flop, case).unwrap();
                let eval = Eval::from(seven);
                case_eval.push(eval);
            }
            tx.send(case_eval).unwrap();
        });

        // case_evals.push(case_eval);
    }
    drop(tx);

    for received in rx {
        case_evals.push(received);
    }

    //\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\
    // Convert the CaseEvals into wins
    // from FlopEval line 214
    let mut wins = Wins::default();
    for case_eval in case_evals.iter() {
        wins.add(case_eval.win_count());
    }

    //\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\//\\
    // Convert wins into results
    // from FlopEval line 215
    let results = Results::from_wins(&wins, hands.len());

    println!("concurrent");
    println!("{results}");

    println!("concurrent Elapsed: {:.2?}", now.elapsed());
    Ok(())
}

// FlopEval::try_from(game.clone()) -> CaseEvals::from_holdem_at_flop(board, &hands);
