use pkcore::analysis::case_eval::CaseEval;
use pkcore::analysis::case_evals::CaseEvals;
use pkcore::arrays::five::Five;
use pkcore::arrays::hole_cards::twos::Twos;
use pkcore::arrays::two::Two;

use pkcore::Pile;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;

const HERO: Two = Two::HAND_AS_4S;
const VILLAIN: Two = Two::HAND_KS_KH;
const MINION: Two = Two::HAND_8C_7C;

fn main() {
    pkcore();
}

fn pkcore() {
    let now = std::time::Instant::now();

    let _ = case_evals();

    println!("Elapsed: {:.2?}", now.elapsed());
}

fn combos() -> Vec<Five> {
    let hands = Twos::from([HERO, VILLAIN, MINION]);
    hands
        .combinations_remaining(5)
        .map(|f| match Five::try_from(f) {
            Ok(five) => five,
            Err(_) => Five::default(),
        })
        .collect::<Vec<Five>>()
}

fn case_evals() -> CaseEvals {
    CaseEvals::from(collect())
}

fn collect() -> Vec<CaseEval> {
    let hands = Twos::from([HERO, VILLAIN, MINION]);
    combos()
        .into_par_iter()
        .map(|v| hands.heavy_case_eval(Five::from(v)))
        .collect()
}
