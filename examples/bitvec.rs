use bitvec::prelude::*;
use pkcore::card::Card;
use pkcore::Pile;

// pub struct SuitMask {
//     pub higher: BitArray::<[u16; 0, 4], Msb0>,
//     pub lower: BitArray::<[u16; 0, 4], Msb0>,
// }

fn main() {
    // let ba = BitArray::new()

    let bits = bits![u16, Msb0; 0; 4];
    // let b2 = BitSlice::;

    println!("{bits}");

    let card = Card::ACE_SPADES;
    let bard = card.bard();

    println!("Card: {card}");
    println!("bard: {bard}");
    println!("bard: {bard:b}");
    let mut binding = card.as_u32();
    let bits = binding.view_bits_mut::<Msb0>();
    bits.reverse();

    println!("{bits:b}");

    bits.reverse();
    println!("{bits:b}");

    println!("{}", bits);

    // println!("{:b}", rev.clone());
    println!("{}", Card::ACE_SPADES.bit_string_guided());

    // let fouraces = Cards::from_str("AS AH AD AC").unwrap();
    // for suits = fouraces.iter().map(|c| c.as_u32() & Card::SUIT_FLAG_FILTER).collect()
}
