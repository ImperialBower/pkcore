use rusqlite::Connection;
use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;

fn main() {
    let conn = Connection::open("data/hups.db").unwrap();
    HUPResult::create_table(&conn).unwrap();
}