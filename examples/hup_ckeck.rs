use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::{Connect, Sqlable};
use rusqlite::Connection;
use std::collections::HashSet;

fn main() {
    let hups = read_db();

    let conn = Connect::in_memory_connection().unwrap().connection;
    HUPResult::create_table(&conn).unwrap();

    // assert_eq!(hs.len(), hups.len());
}

fn read_db() -> Vec<HUPResult> {
    let conn = Connection::open("generated/hups.db").unwrap();
    let hups = HUPResult::select_all(&conn);
    conn.close().unwrap();
    hups
}
