use bitvec::prelude::*;
use pkcore::card::Card;
use pkcore::Pile;

fn main() {
    let card = Card::ACE_SPADES;
    let bard = card.bard();

    println!("Card: {card}");
    println!("bard: {bard}");
    println!("bard: {bard:b}");
    let binding = card.as_u32();
    let bits = binding.view_bits::<Msb0>();
    println!("{bits:b}");
    println!("{}", Card::ACE_SPADES.bit_string_guided());

    // let fouraces = Cards::from_str("AS AH AD AC").unwrap();
    // for suits = fouraces.iter().map(|c| c.as_u32() & Card::SUIT_FLAG_FILTER).collect()
}
