use pkcore::analysis::omaha::EightOrBetter;
use pkcore::arrays::five::Five;
use pkcore::deck::POKER_DECK;

fn main() {

    let combos = POKER_DECK.combinations(5);

    // let lows = combos.filter(EightOrBetter::is_eight_or_better).collect::<Vec<_>>();

    let lows: Vec<Five> = combos
        .filter(|c| Five::try_from(c))
        .filter(EightOrBetter::is_eight_or_better)
        .collect();

    // POKER_DECK.combinations(5).for_each(|c| {
    //     let hand = Five::try_from(c).unwrap();
    //     println!("{hand}");
    // });
}