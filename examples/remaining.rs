use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use rusqlite::Connection;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::PKError;

/// `cargo run --example remaining`
fn main() -> Result<(), PKError>  {
    let now = std::time::Instant::now();
    env_logger::init();

    let conn = Connection::open("generated/hups_add8.db").unwrap();
    let distinct = HUPResult::distinct_remaining(&conn);

    for masked in distinct.clone() {
        println!("Processing {masked}");

        if HUPResult::exists(&conn, &masked.shu) {
            println!("{} exists!", masked.shu);
        } else {
            println!("Calculating {}", masked.shu);
            let hupr = HUPResult::from(&masked.shu);
            match insert(&conn, &hupr) {
                Ok(_) => {

                }
                Err(_) => {}
            }
        }


    }
    println!("{} remaining distinct", distinct.len());
    conn.close().unwrap();

    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}

fn insert(conn: &Connection, hup: &HUPResult) -> Result<(), PKError> {
    match HUPResult::insert(conn, hup) {
        Ok(_) => {
            println!("... inserted");
            Ok(())
        }
        Err(e) => {
            println!("Unable to insert {hup}");
            println!("Error: {:?}", e);
            Err(PKError::from(e))
        }
    }
}
