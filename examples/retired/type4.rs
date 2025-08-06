use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::arrays::matchups::masked::Masked;
use pkcore::Shifty;
use std::str::FromStr;

/// This is our first Masked shift defect case.
///
/// Q♥ 3♠,J♦ 2♦,
/// defect
///
/// Type 4 - Type1123
///
/// ```txt
/// Type1123
/// ```
///
/// `cargo run --example type4`
fn main() {
    // let case1 = Masked::from_str("A♠ A♥ K♠ Q♠").unwrap(); All match
    let case1 = Masked::from_str("A♠ 4♠ K♠ K♥").unwrap();

    case(&case1);
}

fn case(shift: &Masked) {
    println!("Auditing Case: {shift}");

    let baseline = HUPResult::from(&shift.shu);

    dump(&shift, &baseline);
    println!("=================");
    println!("=================");

    // let conn = Connection::open("generated/hups_07_31_2025.db").unwrap();

    for shift in shift.other_shifts() {
        process_case(&shift, &baseline);
    }
}

fn process_case(shift: &Masked, baseline: &HUPResult) {
    let case_hupr = HUPResult::from(&shift.shu);
    dump(&shift, &baseline);
    println!("=================");
    println!("=================");

    for shift in shift.other_shifts() {
        let hupr = HUPResult::from(&shift.shu);
        dump(&shift, &hupr);
        if baseline.matches(&case_hupr) {
            println!("...MATCH!");
        } else {
            println!("...NO MATCH!");
        }
    }
}

fn dump(masked: &Masked, hupr: &HUPResult) {
    println!("-----");
    println!("{masked}");
    println!("{hupr}");
}
