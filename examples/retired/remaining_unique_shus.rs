use pkcore::PKError;
use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use rusqlite::Connection;

/// `cargo run --example remaining_unique_shus`
fn main() -> Result<(), PKError> {
    let mut unique = SortedHeadsUp::unique()?;
    println!("{} total unique shus", unique.len());
    let conn = Connection::open("../../generated/hups.db").unwrap();

    if !HUPResult::db_is_valid(&conn) {
        return Err(PKError::SqlError);
    };

    let hups = HUPResult::select_all(&conn);
    println!("{} shus processed", hups.len());

    for hup in hups {
        let shu = hup.get_sorted_heads_up().ok_or(PKError::Fubar)?;
        if !unique.remove(&shu) {
            println!("Unable to remove {}", shu);
            return Err(PKError::Fubar);
        }
    }

    println!("{} unique shus remaining", unique.len());
    SortedHeadsUp::generate_csv("data/remaining_unique_shus.csv", unique).expect("Unable to generate CSV.");

    Ok(())
}
