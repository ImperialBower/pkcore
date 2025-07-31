use rusqlite::Connection;
use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::PKError;

// `cargo run --example unq2dinct`
fn main() -> Result<(), PKError> {

    let conn = match Connection::open("generated/hups.db") {
        Ok(c) => c,
        Err(_) => return Err(PKError::SqlError),
    };
    let hups = HUPResult::select_all(&conn);
    println!("{} shus processed", hups.len());

    Ok(())
}