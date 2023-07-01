use pkcore::arrays::five::Five;
use pkcore::card::Card;
use pkcore::cards::Cards;

/// TODO: Filtering for specific hand types is very slow. There must be a better way??!!!
/// Hint, there is: `BitCard` is coming.
fn main() {
    let deck = Cards::deck().shuffle();

    let straight_flushes: Vec<Vec<Card>> = deck
        .combinations(5)
        .filter(|cards| Five::try_from(cards.clone()).unwrap().is_straight_flush())
        .collect();

    for v in straight_flushes {
        println!("{}", Cards::from(v).sort().to_string());
    }
}
