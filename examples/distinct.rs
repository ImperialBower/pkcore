use itertools::Itertools;
use pkcore::arrays::matchups::sorted_heads_up::{SortedHeadsUp, SortedHeadsUpSuitBinary};
use pkcore::PKError;
use std::collections::HashSet;

/// ♠ ♥ ♦ ♣
/// 1111 - suited, suited, same suit
/// 1112 - suited, off suit, sharing suit
/// 1122 - suited, suited, different suits
/// 1123 - suited, off suit, different suits
/// 1223 - off suit, off suit, sharing one suit
/// 1212 - off suit, off suit, sharing both suits
/// 1234 - off suit, off suit, sharing no suits
fn main() -> Result<(), PKError> {
    let mut unique = SortedHeadsUp::unique()?;

    let (type_one, t1_shusbs) = generate(&mut unique, SortedHeadsUp::is_type_one);

    info(&type_one, &t1_shusbs, "one");

    for shusb in t1_shusbs.iter().sorted() {
        println!("{shusb}");
    }
    SortedHeadsUp::generate_csv("generated/unique_type_one.csv", type_one)
        .expect("TODO: panic message");

    let (type_two, t2_shusbs) = generate(&mut unique, SortedHeadsUp::is_type_two);

    info(&type_two, &t2_shusbs, "two");

    for shusb in t2_shusbs.iter().sorted() {
        println!("{shusb}");
    }
    SortedHeadsUp::generate_csv("generated/unique_type_two.csv", type_two)
        .expect("TODO: panic message");

    Ok(())
}

/// Original:
///
/// ```
/// fn generate_type_one(
///     unique: &mut HashSet<SortedHeadsUp>,
/// ) -> (HashSet<SortedHeadsUp>, HashSet<SortedHeadsUpSuitBinary>) {
///     let type_one: HashSet<SortedHeadsUp> = unique
///         .clone()
///         .into_iter()
///         .filter(SortedHeadsUp::is_type_one)
///         .collect();
///     let shusb: HashSet<SortedHeadsUpSuitBinary> = type_one
///         .clone()
///         .into_iter()
///         .map(SortedHeadsUpSuitBinary::from)
///         .collect();
///     (type_one, shusb)
/// }
/// ```
fn generate(
    unique: &mut HashSet<SortedHeadsUp>, f: fn(&SortedHeadsUp) -> bool
) -> (HashSet<SortedHeadsUp>, HashSet<SortedHeadsUpSuitBinary>) {
    let type_one: HashSet<SortedHeadsUp> = unique
        .clone()
        .into_iter()
        .filter(f)
        .collect();
    let shusb: HashSet<SortedHeadsUpSuitBinary> = type_one
        .clone()
        .into_iter()
        .map(SortedHeadsUpSuitBinary::from)
        .collect();
    (type_one, shusb)
}

fn info(shus: &HashSet<SortedHeadsUp>, shusbs: &HashSet<SortedHeadsUpSuitBinary>, s: &str) {
    println!(
        "{} type {} hands with {} suit sigs",
        shus.len(),
        s,
        shusbs.len()
    );
}