use pkcore::arrays::two::Two;
use pkcore::deck::POKER_DECK;

fn main() {
    let combos = POKER_DECK.combinations(2);

    let twos: Vec<Two> = combos.map(|c| Two::from(c)).collect();

    for combo in twos {
        println!("{}", combo);
    }
}
