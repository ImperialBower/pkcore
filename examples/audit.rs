use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::Masked;
use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;
use pkcore::util::terminal::Terminal;
use rand::prelude::IndexedRandom;
use rusqlite::Connection;

/// `cargo run --example audit`
fn main() {
    env_logger::init();
    let distinct = distinct_shus_from_csv_as_masked_vec();

    let conn = Connection::open("generated/hups.db").unwrap();
    loop {
        read_input(&conn, &distinct);
    }
}

fn read_input(conn: &Connection, distinct: &Vec<Masked>) {
    let mut x = 0usize;
    let i = Terminal::receive_usize("How many audits? ");
    println!("Auditing {i} hands.");

    while x < i {
        let masked = random(&distinct).unwrap();
        match HUPResult::select(&conn, &masked.shu) {
            None => println!("{x} - Not in DB: {}", &masked.shu),
            Some(actual) => {
                println!("{x} - Auditing: {}", &masked.shu);
                let expected = HUPResult::from(&masked.shu);
                if expected == actual {
                    println!("   {} passes!", &masked);
                } else {
                    println!("   {} fails audit", &masked);
                    println!("      expected: {expected}");
                    println!("      actual: {actual}");
                    assert_eq!(expected, actual);
                }
            }
        };
        if HUPResult::exists(&conn, &masked.shu) {
        } else {
            println!("Not in DB: {}", &masked.shu);
        }
        x = x + 1;
    }
}

fn random(distinct: &Vec<Masked>) -> Option<&Masked> {
    let mut rng = rand::rng();
    distinct.choose(&mut rng)
}
