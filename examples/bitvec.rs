use pkcore::card::Card;
use pkcore::Pile;

fn main() {
    let card = Card::ACE_SPADES;
    let bard = card.bard();

    println!("Card: {card}");
    println!("bard: {bard}");
    println!("bard: {bard:b}");
}
