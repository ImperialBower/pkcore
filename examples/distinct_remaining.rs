use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use rusqlite::Connection;

fn main() {
    let conn = Connection::open("generated/hups.db").unwrap();
    let distinct = HUPResult::distinct_remaining(&conn);

    for shu in distinct.clone() {
        println!("{shu}");
    }
    println!("{} remaining distinct", distinct.len());
    conn.close().unwrap();
}
