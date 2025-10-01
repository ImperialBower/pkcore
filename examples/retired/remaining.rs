use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::Masked;
use pkcore::{PKError, Shifty};
use rusqlite::Connection;

/// `cargo run --example remaining`
///
/// Run `cargo run --example types_remaining` to get a count of remaining type 8 hands to process.
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let conn = Connection::open("../../generated/hups.db").unwrap();
    let distinct = HUPResult::distinct_remaining(&conn);

    for masked in distinct.clone() {
        process_distinct(&conn, masked)?;
    }
    println!("{} remaining distinct", distinct.len());
    conn.close().unwrap();

    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}

fn process_distinct(conn: &Connection, masked: Masked) -> Result<(), PKError> {
    println!("Processing {masked}");

    if HUPResult::exists(conn, &masked.shu) {
        println!("{} exists!", masked.shu);
        Ok(())
    } else {
        println!("Calculating {}", masked.shu);
        let hupr = HUPResult::from(&masked.shu);
        println!("...inserting {hupr}");
        let _ = insert(&conn, &hupr);

        for shift in hupr.shifts() {
            println!("......shift {shift}");
            let _ = insert(&conn, &shift)?;
        }
        Ok(())
    }
}

fn insert(conn: &Connection, hup: &HUPResult) -> Result<(), PKError> {
    match HUPResult::insert(conn, hup) {
        Ok(_) => {
            println!("... inserted");
            Ok(())
        }
        Err(e) => {
            println!("Unable to insert {hup}");
            println!("Error: {:?}", e);
            Err(PKError::from(e))
        }
    }
}
