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
///
/// 8580 type one hands with 4 suit sigs
/// 133848 type two hands with 24 suit sigs
/// 36504 type three hands with 12 suit sigs
fn main() -> Result<(), PKError> {
    let mut unique = SortedHeadsUp::unique()?;

    do_it(&mut unique, SortedHeadsUp::is_type_one, "one");
    do_it(&mut unique, SortedHeadsUp::is_type_two, "two");
    do_it(&mut unique, SortedHeadsUp::is_type_three, "three");

    Ok(())
}

fn do_it(unique: &mut HashSet<SortedHeadsUp>, f: fn(&SortedHeadsUp) -> bool, s: &str) {
    let (shus, shusbs) = generate(unique, f);
    info(&shus, &shusbs, s);
    for shusb in shusbs.iter().sorted() {
        println!("{shusb}");
    }

    SortedHeadsUp::generate_csv(format!("generated/unique_type_{}.csv", s).as_str(), shus)
        .expect("TODO: panic message");
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
    unique: &mut HashSet<SortedHeadsUp>,
    f: fn(&SortedHeadsUp) -> bool,
) -> (HashSet<SortedHeadsUp>, HashSet<SortedHeadsUpSuitBinary>) {
    let type_one: HashSet<SortedHeadsUp> = unique.clone().into_iter().filter(f).collect();
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
