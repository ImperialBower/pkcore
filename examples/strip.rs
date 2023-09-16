use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use rusqlite::Connection;

fn main() {
    old_db();
}

fn old_db() {
    env_logger::init();

    let distinct = SortedHeadsUp::distinct().unwrap();

    let from = Connection::open("generated/clean_hups.db").unwrap();
    let to = Connection::open("generated/dhups.db").unwrap();
    HUPResult::create_table(&to).expect("TODO: panic message");

    for (i, d) in distinct.into_iter().enumerate() {
        match HUPResult::select(&from, &d) {
            None => {}
            Some(hr) => {
                println!("Inserting #{i} - {hr}");
                let _ = HUPResult::insert(&to, &hr);
            }
        }
    }

    from.close().unwrap();
    to.close().unwrap();
}
