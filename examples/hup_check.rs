use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::{Connect, Sqlable};
use pkcore::Shifty;
use rusqlite::Connection;
use std::collections::HashSet;
use std::io;
use std::io::Write;

/// I'm thinking that I want to turn this into a test.
///
/// `cargo run --example hup_check`
fn main() {
    let hups = HUPResult::read_csv("data/washed_hups.csv").unwrap();
    let hups_length = hups.len();
    let conn = Connect::in_memory_connection().unwrap().connection;
    HUPResult::create_table(&conn).unwrap();

    for hup in hups {
        println!("{hup}");

        process(&conn, &hup);
    }
    validate(&conn, hups_length);
    // HUPResult::generate_csv_from_vector("data/washed_hups.csv", &HUPResult::select_all(&conn))
    //     .unwrap();
    conn.close().unwrap();
}

fn validate(conn: &Connection, hups_length: usize) {
    let (is, should) = HUPResult::db_count(conn);
    match HUPResult::db_is_valid(conn) {
        true => println!("DB passes internal validation"),
        false => {
            println!("is: {is} - should be: {should}");
        }
    }
    match is == hups_length {
        true => println!("DB passes external validation"),
        false => {
            println!("is: {is} - source length: {hups_length}");
        }
    }
}

fn process(conn: &Connection, hup: &HUPResult) -> HashSet<HUPResult> {
    let shifts = hup.shifts();
    for shift in hup.shifts().clone() {
        match HUPResult::insert(&conn, &shift).unwrap() {
            false => println!("    {shift} already inserted!"),
            true => (),
        }
        match HUPResult::exists(&conn, &shift.get_sorted_heads_up().unwrap()) {
            true => println!("    {shift} in DB!"),
            false => println!("     {shift} missing!"),
        }
    }
    shifts
}

fn _next() {
    print!("...");
    let _ = io::stdout().flush();
    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("Failed to receive value");
}
