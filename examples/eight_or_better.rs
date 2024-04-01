use std::collections::HashMap;
use pkcore::analysis::omaha::EightOrBetter;
use pkcore::arrays::five::Five;
use pkcore::deck::POKER_DECK;

fn main() {

    let combos = POKER_DECK.combinations(5);

    // let lows = combos.filter(EightOrBetter::is_eight_or_better).collect::<Vec<_>>();

    let mut all: Vec<Five> = Vec::new();
    let mut mappy: HashMap<u32, Five> = HashMap::new();

    POKER_DECK.combinations(5).for_each(|c| {
        let hand = Five::try_from(c).unwrap();
        match EightOrBetter::filter(hand) {
            Some(v) => {
                all.push(hand);
                if !mappy.contains_key(&v) {
                    mappy.insert(v, hand);
                }
            },
            None => ()
        }
    });

    let mut lows: Vec<Five> = mappy.values().cloned().collect();
    lows.sort();
    for low in lows {
        println!("{}", low);
    }



    // let lows: Vec<Five> = combos
    //     .filter(|c| match_value(Five::try_from(c)).is_ok())
    //     .filter(EightOrBetter::is_eight_or_better)
    //     .collect();
    //



}