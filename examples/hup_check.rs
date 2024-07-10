use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use rusqlite::Connection;

/// I'm thinking that I want to turn this into a test.
///
/// `cargo run --example hup_check`
fn main() -> Result<(), rusqlite::Error> {
    let conn = Connection::open("generated/hups.db")?;
    match HUPResult::check_db(&conn) {
        Ok(count) => println!("HUP Check passes! {count} unique entries"),
        Err(_) => println!("DB misaligned"),
    };
    conn.close().unwrap();
    Ok(())
}
