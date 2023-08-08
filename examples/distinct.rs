use itertools::Itertools;
use pkcore::arrays::matchups::sorted_heads_up::{SortedHeadsUp, SortedHeadsUpSuitBinary};
use pkcore::PKError;
use std::collections::HashSet;

/// ```txt
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
/// 158184 type four hands with 24 suit sigs
/// 316368 type five hands with 24 suit sigs
/// 73008 type six hands with 6 suit sigs
/// 85683 type 85683 hands with 6 suit sigs
/// ```
fn main() -> Result<(), PKError> {
    let mut unique = SortedHeadsUp::unique()?;

    let total = unique.len();

    let one = do_it(&mut unique, SortedHeadsUp::is_type_one, "one");
    let two = do_it(&mut unique, SortedHeadsUp::is_type_two, "two");
    let three = do_it(&mut unique, SortedHeadsUp::is_type_three, "three");
    let four = do_it(&mut unique, SortedHeadsUp::is_type_four, "four");
    let five = do_it(&mut unique, SortedHeadsUp::is_type_five, "five");
    let six = do_it(&mut unique, SortedHeadsUp::is_type_six, "six");
    let seven = do_it(&mut unique, SortedHeadsUp::is_type_seven, "seven");

    let sum =
        one.len() + two.len() + three.len() + four.len() + five.len() + six.len() + seven.len();
    assert_eq!(0, unique.len());
    assert_eq!(total, unique.len() + sum);
    check(&unique, &one);
    check(&unique, &two);
    check(&unique, &three);
    check(&unique, &four);
    check(&unique, &five);
    check(&unique, &six);

    // assert_eq!(total, one + two + three + four + five + six);
    SortedHeadsUp::generate_csv("generated/unique_type_remaining.csv", unique)
        .expect("TODO: panic message");

    Ok(())
}

fn check(unique: &HashSet<SortedHeadsUp>, types: &HashSet<SortedHeadsUp>) {
    for shu in types {
        if unique.contains(shu) {
            println!("{shu} not removed");
        }
    }
}

fn do_it(
    unique: &mut HashSet<SortedHeadsUp>,
    f: fn(&SortedHeadsUp) -> bool,
    s: &str,
) -> HashSet<SortedHeadsUp> {
    let (shus, shusbs) = generate(unique, f);
    info(&shus, &shusbs, s);
    for shusb in shusbs.iter().sorted() {
        println!("{shusb}");
    }

    SortedHeadsUp::generate_csv(
        format!("generated/unique_type_{}.csv", s).as_str(),
        shus.clone(),
    )
    .expect("TODO: panic message");

    shus
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
    let t: HashSet<SortedHeadsUp> = unique.clone().into_iter().filter(f).collect();
    let shusb: HashSet<SortedHeadsUpSuitBinary> = t
        .clone()
        .into_iter()
        .map(SortedHeadsUpSuitBinary::from)
        .collect();

    for shu in &t {
        unique.remove(&shu);
    }

    (t, shusb)
}

fn info(shus: &HashSet<SortedHeadsUp>, shusbs: &HashSet<SortedHeadsUpSuitBinary>, s: &str) {
    println!(
        "{} type {} hands with {} suit sigs",
        shus.len(),
        s,
        shusbs.len()
    );
}
