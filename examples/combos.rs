use pkcore::arrays::five::Five;
use pkcore::arrays::five::hands::DISTINCT_HANDS;
use pkcore::arrays::HandRanker;
use pkcore::arrays::two::Two;
use pkcore::deck::POKER_DECK;

fn main() {
    for hand in DISTINCT_HANDS.iter() {
        println!("{hand}");
    }
}



fn _twos() {
    let combos = POKER_DECK.combinations(2);

    let twos: Vec<Two> = combos.map(|c| Two::from(c)).collect();

    for combo in twos {
        println!("{}", combo);
    }
}
