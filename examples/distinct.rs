use pkcore::arrays::matchups::sorted_heads_up::{SortedHeadsUp, SortedHeadsUpSuitBinary};
use pkcore::PKError;
use std::collections::HashSet;
use itertools::Itertools;

// A♦ A♣ - K♠ K♥
// A♠ A♣ - K♥ K♦
// A♥ A♦ - K♠ K♣
// A♠ A♥ - K♦ K♣
// A♠ A♦ - K♥ K♣
// A♥ A♣ - K♠ K♦
// 1111 - suited, suited, same suit
// 1112 - suited, off suit, sharing suit
// 1122 - suited, suited, different suits
// 1123 - suited, off suit, different suits
// 1223 - off suit, off suit, sharing one suit
// 1212 - off suit, off suit, sharing both suits
// 1234 - off suit, off suit, sharing no suits
fn main() -> Result<(), PKError> {
    // ♠ ♥ ♦ ♣

    let mut unique = SortedHeadsUp::unique()?;
    let mut type_one = HashSet::new();
    let mut t1_shusbs = HashSet::new();

    for shu in unique.clone().into_iter() {
        if shu.is_type_one() {
            unique.remove(&shu);
            type_one.insert(shu);
            t1_shusbs.insert(SortedHeadsUpSuitBinary::from(&shu));
        }
    }

    println!("{} type one hands with {} suit sigs", type_one.len(), t1_shusbs.len());
    for shusb in t1_shusbs.iter().sorted() {
        println!("{shusb}");
    }
    SortedHeadsUp::generate_csv("generated/unique_type_one.csv", type_one)
        .expect("TODO: panic message");

    let mut type_two = HashSet::new();
    let mut t2_shusbs = HashSet::new();

    for shu in unique.clone().into_iter() {
        if shu.is_type_two() {
            unique.remove(&shu);
            type_two.insert(shu);
            t2_shusbs.insert(SortedHeadsUpSuitBinary::from(&shu));
        }
    }
    println!("{} type two hands with {} suit sigs", type_two.len(), t2_shusbs.len());
    for shusb in t2_shusbs.iter().sorted() {
        println!("{shusb}");
    }
    SortedHeadsUp::generate_csv("generated/unique_type_two.csv", type_two)
        .expect("TODO: panic message");

    Ok(())
}

fn generate_type_one(_unique: &mut HashSet<SortedHeadsUp>) {

}
