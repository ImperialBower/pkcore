use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::{Connect, Sqlable};
use rusqlite::Connection;
use std::collections::HashSet;

fn main() -> rusqlite::Result<(), (Connection, rusqlite::Error)> {
    let hups = HUPResult::read_db("generated/hups.db").unwrap();

    for hup in hups {
        println!("{hup}")
    }

    let conn = Connect::in_memory_connection().unwrap().connection;
    HUPResult::create_table(&conn).unwrap();

    conn.close()
}
