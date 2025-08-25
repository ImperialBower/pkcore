use pkcore::Shifty;
use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::arrays::matchups::masked::MASKED_UNIQUE_TYPE_EIGHT;
use rusqlite::Connection;

/// `cargo run --example type8`
fn main() {
    let conn = Connection::open("generated/hups_TEST.db").unwrap();

    let type_eight = HUPResult::remaining(&conn, MASKED_UNIQUE_TYPE_EIGHT.clone());

    for shu in type_eight.iter() {
        println!("{shu}");
        for shift in shu.shifts() {
            println!("...{shift}");
        }
    }
}
