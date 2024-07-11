use mysql::prelude::*;
use mysql::*;
use pkcore::analysis::store::db::mysql::{HeadsUpRawResult, DB};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = DB::get_connection()?;

    let mut conn = DB::get_connection().unwrap();
    let hups = HeadsUpRawResult::all_as_hup_results(&mut conn).unwrap();

    for hup in hups {
        println!("{hup}");
    }

    Ok(())
}
