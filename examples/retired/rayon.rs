use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::{PKError, Shifty};
use rayon::prelude::*;
use rusqlite::Connection;

fn main() -> Result<(), PKError> {
    let shus = SortedHeadsUp::distinct()?;

    shus.into_par_iter().for_each(|shu| {
        process(&shu);
    });

    Ok(())
}

fn process(shu: &SortedHeadsUp) {
    println!("{shu}");

    let conn = Connection::open("../../generated/hups_07_31_2025.db").unwrap();

    if !HUPResult::exists(&conn, &shu) {
        let hup = HUPResult::from(shu);
        for shift in hup.shifts() {
            HUPResult::insert(&conn, &hup).unwrap();
            println!(">>>>> {shift} shift inserted!");
        }
    }

    conn.close().unwrap();
}
