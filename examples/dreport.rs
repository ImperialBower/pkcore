use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;
use pkcore::Shifty;

/// `cargo run --example dreport`
fn main() {
    let mut distinct = distinct_shus_from_csv_as_masked_vec();
    distinct.reverse();

    for masked in distinct {
        println!("{masked}");
        for shift in masked.other_shifts() {
            println!("...{shift}");
        }
    }
}
