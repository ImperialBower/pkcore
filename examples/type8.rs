use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::arrays::matchups::masked::MASKED_UNIQUE_TYPE_EIGHT;
use rusqlite::Connection;

fn main() {
    let conn = Connection::open("generated/hups_TEST.db").unwrap();

    let type_eight = HUPResult::remaining(&conn, MASKED_UNIQUE_TYPE_EIGHT.clone());

    for shu in type_eight.iter() {
        println!("{shu}");
    }
}
