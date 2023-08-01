use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use rusqlite::Connection;

/// I'm thinking that I want to turn this into a test.
///
/// `cargo run --example hup_check`
fn main() -> Result<(), rusqlite::Error> {
    let conn = Connection::open("generated/hups.db")?;

    let (v, hs) = HUPResult::db_count(&conn);
    match v == hs {
        true => println!("HUP Check passes! {v} unique entries"),
        false => {
            println!("HUP Check fails :-(");
            println!("unique: {hs}, in db: {v}");
        }
    }

    match conn.close() {
        Ok(_) => {}
        Err((_, e)) => {
            println!("ERROR CLOSING DB: {:?}", e)
        }
    };
    Ok(())
}
