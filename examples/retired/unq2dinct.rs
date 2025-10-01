use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::arrays::two::Two;
use pkcore::{PKError, Shifty};
use rusqlite::Connection;

// `cargo run --example unq2dinct`
fn main() -> Result<(), PKError> {
    let conn = match Connection::open("../../generated/hups.db") {
        Ok(c) => c,
        Err(_) => return Err(PKError::SqlError),
    };
    let hups = HUPResult::select_all(&conn);
    println!("{} shus processed", hups.len());

    let hero = Two::HAND_AS_AH;
    let villain = Two::HAND_AD_AC;

    let shu = SortedHeadsUp::new(hero, villain);

    let shifts = shu.shifts();

    for shift in shifts {
        println!("{shift}");
    }

    Ok(())
}
