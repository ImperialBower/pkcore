use pkcore::analysis::omaha::EightOrBetter;
use pkcore::arrays::five::Five;
use pkcore::cards::Cards;
use pkcore::deck::POKER_DECK;
use std::collections::HashMap;

/// `cargo run --example eight_or_better`
fn main() {
    let mut mappy: HashMap<u8, Five> = HashMap::new();

    POKER_DECK.combinations(5).for_each(|c| {
        let cards = Cards::from(c);
        let bits = EightOrBetter::get_low_bits(&cards);
        if bits.count_ones() == 5 {
            match Five::try_from(cards) {
                Ok(five) => {
                    mappy.insert(bits, five);
                }
                Err(_) => {}
            }
        }
    });

    let mut keys = mappy.keys().cloned().collect::<Vec<u8>>();
    keys.sort();

    for (i, key) in keys.iter().enumerate() {
        println!("{i} - {key:0b} {key}: {}", mappy.get(&key).unwrap());
    }
}
