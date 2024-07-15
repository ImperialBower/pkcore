use mysql::*;
use pkcore::analysis::store::db::mysql::DbConnectOps;
use pkcore::analysis::store::db::mysql::{HeadsUpRawResult, MySqlDB};

/// `cargo run --example pkdbdump`
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = MySqlDB::get_connection()?;

    let hups = HeadsUpRawResult::all(&mut conn).unwrap();

    for hup in hups {
        println!("{hup}");
    }

    Ok(())
}
