use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::arrays::matchups::masked::MASKED_UNIQUE;
use rusqlite::Connection;

/// `cargo run --example types_remaining`
fn main() {
    let conn = Connection::open("../../generated/hups.db").unwrap();

    let type_eight = HUPResult::remaining(&conn, MASKED_UNIQUE.clone());

    // for shu in type_eight.iter() {
    //     println!("{shu}");
    //     for shift in shu.shifts() {
    //         println!("...{shift}");
    //     }
    // }
    println!("{} out of {} remaining type 8", type_eight.len(), MASKED_UNIQUE.len());
}
