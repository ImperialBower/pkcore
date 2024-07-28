use std::collections::HashSet;
use std::error::Error;
use rusqlite::Connection;
use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::mysql::{DbConnectOps, HeadsUpRawResult, MySqlDB};
use pkcore::analysis::store::db::sqlite::Sqlitable;

fn main() {
    env_logger::init();

}

fn get_connection() -> Connection {
    let conn = Connection::open("generated/hups.db").unwrap();
    HUPResult::create_table_unless_exists(&conn).expect("TODO: panic message");
    conn
}

fn get_distinct_hups() -> Result<HashSet<HeadsUpRawResult>, Box<dyn Error>> {
    let mut conn = MySqlDB::get_connection()?;
    Ok(HeadsUpRawResult::all_as_hashset(&mut conn)?)
}