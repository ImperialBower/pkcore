use mysql::*;
use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::mysql::{DB, HeadsUpQuery};
use pkcore::arrays::matchups::masked::Masked;
use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;
use pkcore::util::terminal::Terminal;
use rand::seq::SliceRandom; // For the shuffle method
use rand::thread_rng; // For the random number generator

/// `cargo run --example insert_distinct`
#[allow(unreachable_code)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut distinct = distinct_shus_from_csv_as_masked_vec();
    let mut rng = thread_rng(); // Obtain a random number generator
    distinct.shuffle(&mut rng); // Shuffle the vector

    let mut conn = DB::get_connection()?;

    loop {
        read_input(&mut conn, &mut distinct);
    }

    Ok(())
}
fn read_input(conn: &mut PooledConn, distinct: &mut Vec<Masked>) {
    let mut x = 0usize;
    let i = Terminal::receive_usize("How many runs? ");
    println!("Processing {i} hands.");

    while x < i {
        let Some(masked) = distinct.pop() else {
            println!("None remaining.");
            return;
        };
        let huq = HeadsUpQuery::from(masked);

        if huq.exists(conn) {
            println!("{} exists!", masked.shu);
            continue;
        } else {
            println!("Calculating #{x} {}", masked.shu);
            let hupr = HUPResult::from(&masked.shu);
            if huq.exists(conn) {
                println!("{} exists after calc!", masked.shu);
                continue;
            }
            match huq.insert(conn, &hupr) {
                Ok(_) => {
                    println!("... inserted");
                }
                Err(e) => {
                    println!("Unable to insert {hupr}");
                    println!("Error: {:?}", e);
                }
            }
        }
        x = x + 1;
    }
}
