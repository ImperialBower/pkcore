use mysql::*;
use pkcore::analysis::store::db::mysql::DbConnectOps;
use pkcore::analysis::store::db::mysql::{HeadsUpQuery, MySqlDB};
use pkcore::arrays::matchups::masked::Masked;
use pkcore::util::terminal::Terminal;

/// `cargo run --example insert_unique`
#[allow(unreachable_code)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut distinct = Masked::distinct_shus_from_csv_as_masked_vec();

    let mut conn = MySqlDB::get_connection()?;

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

        // let huq = HeadsUpQuery::from(masked);

        let huq = HeadsUpQuery::from(masked).query(conn);

        match huq {
            Ok(result) => {
                println!("{}", result);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}
