use mysql::*;
use pkcore::analysis::store::db::mysql::{HeadsUpRawResult, DB};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = DB::get_connection()?;

    let hups = HeadsUpRawResult::all(&mut conn).unwrap();

    for hup in hups {
        println!("{hup}");
    }

    Ok(())
}
