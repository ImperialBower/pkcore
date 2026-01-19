use pkcore::prelude::*;

/// `cargo run --example byx`
fn main() {
    let cards = cards!("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠");

    // let by =

    let byx = cards.by_x(2).unwrap();

    let _expected = cards!("A♠ 9♠ K♠ 8♠ Q♠ 7♠ J♠ 6♠ T♠ 5♠");

    println!("{byx}")
}
