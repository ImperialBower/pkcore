use pkcore::PKError;
use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::Masked;
use pkcore::arrays::matchups::shift::Shifter;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use rusqlite::Connection;

// `cargo run --example unq`
fn main() -> Result<(), PKError> {
    let conn = match Connection::open("generated/hups_TEST.db") {
        Ok(c) => c,
        Err(_) => return Err(PKError::SqlError),
    };
    let hups = HUPResult::select_all(&conn);
    println!("{} shus processed", hups.len());

    for hupr in hups.clone() {
        let masked = Shifter::from(hupr);
        println!("{hupr}");
        println!("{masked}");
    }

    println!("{} shus processed", hups.len());

    // let shu = SortedHeadsUp::new(Two::HAND_2S_2C, Two::HAND_2H_2D);
    // shifty(&shu, &conn);

    Ok(())
}

fn _shifty(shu: &SortedHeadsUp, conn: &Connection) {
    if !HUPResult::exists(conn, shu) {
        let masked = Masked::from(*shu);
        let _hup_result = HUPResult::select_from_shifts(conn, &masked);

        match HUPResult::select_from_shifts(conn, &masked) {
            Some(hupr) => {
                println!("{hupr}");

                match HUPResult::insert(&conn, &hupr) {
                    Ok(_) => {
                        let remaining = HUPResult::distinct_remaining(&conn).len();
                        println!("... inserted... {remaining} remaining");
                    }
                    Err(e) => {
                        println!("Unable to insert {hupr}");
                        println!("Error: {:?}", e);
                    }
                }
            }
            None => {
                println!("No such result.")
            }
        }
    } else {
        println!("{} already exists", shu);
    }
}
