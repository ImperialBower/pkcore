use pkcore::cards::Cards;

fn main() {
    let deck = Cards::deck().shuffle();

    println!("{}", deck);
}
