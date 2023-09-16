use pkcore::cards::Cards;
use pkcore::{PKError, SuitShift};

fn main() -> Result<(), PKError> {
    let mut cards = Cards::deck();
    dump(&cards);
    cards.shuffle_in_place();

    let five = cards.draw(5)?;

    dump(&five);

    let shift = five.shift_suit_up();
    dump(&shift);

    let mut five_iter = five.iter();
    println!("{}", five_iter.next().ok_or(PKError::NotEnoughCards)?);
    println!("{}", five_iter.next().ok_or(PKError::NotEnoughCards)?);
    println!("{}", five_iter.next().ok_or(PKError::NotEnoughCards)?);
    println!("{}", five_iter.next().ok_or(PKError::NotEnoughCards)?);
    println!("{}", five_iter.next().ok_or(PKError::NotEnoughCards)?);

    Ok(())
}

fn dump(cards: &Cards) {
    for card in cards.iter() {
        print!("{card} ");
    }
    println!();
}
