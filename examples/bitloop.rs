use pkcore::card_number::CardNumber;
use strum::IntoEnumIterator;

fn main() {
    for ckk in CardNumber::iter() {
        println!("{:#032b} {:?} {:?}", ckk as u32, ckk, ckk as u32);
    }
}
