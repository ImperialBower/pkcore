use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::PKError;
use std::collections::HashSet;

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

    for shu in unique.clone().into_iter() {
        if shu.is_type_one() {
            unique.remove(&shu);
            type_one.insert(shu);
        }
    }

    println!("{} type one hands", type_one.len());
    SortedHeadsUp::generate_csv("generated/unique_type_one.csv", type_one)
        .expect("TODO: panic message");

    let mut type_two = HashSet::new();

    for shu in unique.clone().into_iter() {
        if shu.is_type_two() {
            unique.remove(&shu);
            type_two.insert(shu);
        }
    }
    println!("{} type two hands", type_two.len());
    SortedHeadsUp::generate_csv("generated/unique_type_two.csv", type_two)
        .expect("TODO: panic message");

    Ok(())
}
