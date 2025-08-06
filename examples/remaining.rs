use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use rusqlite::Connection;

/// `cargo run --example remaining`
fn main() {
    let conn = Connection::open("generated/hups_TEST.db").unwrap();
    let distinct = HUPResult::unique_remaining(&conn);

    for shu in distinct.clone() {
        println!("{shu}");
    }
    println!("{} remaining distinct", distinct.len());
    conn.close().unwrap();
}
