use pkcore::arrays::five::Five;
use pkcore::arrays::HandRanker;
use pkcore::arrays::two::Two;
use pkcore::deck::POKER_DECK;

fn main() {


}

fn hands() -> Vec<Five> {
    let combos = POKER_DECK.combinations(5);

    let hands: Vec<Five> = combos.map(|c| {
        Five::try_from(c).unwrap().sort()
    }).collect();

    hands
}

fn _twos() {
    let combos = POKER_DECK.combinations(2);

    let twos: Vec<Two> = combos.map(|c| Two::from(c)).collect();

    for combo in twos {
        println!("{}", combo);
    }
}
