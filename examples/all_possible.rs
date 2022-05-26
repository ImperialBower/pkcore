use pkcore::arrays::five::Five;
use pkcore::cards::Cards;

fn main() {
    let deck = Cards::deck();

    // let straight_flushes =
    //     deck.combinations(5).filter(Five::try_from).collect();

    for v in deck.combinations(5) {
        println!("{}", Cards::from(v).to_string());
    }
}