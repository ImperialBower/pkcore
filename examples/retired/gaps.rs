use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::Masked;
use rusqlite::Connection;
use std::collections::HashMap;

/// `cargo run --example gaps`
fn main() {
    let conn = Connection::open("../../generated/dhups.db").unwrap();
    let huprs = HUPResult::select_all(&conn);
    let mut mappy: HashMap<u64, Vec<HUPResult>> = HashMap::new();
    for hupr in huprs.clone() {
        mappy.insert(hupr.odds.wins, Vec::new());
    }

    for hupr in huprs {
        mappy.get_mut(&hupr.odds.wins).expect("REASON").push(hupr);
    }

    for m in mappy.keys() {
        if mappy.get(m).unwrap().len() > 1 {
            println!();
            for dupe in mappy.get(m).unwrap() {
                let masked = Masked::from(dupe);
                println!("{masked} {dupe}");
            }
        }
    }
}
