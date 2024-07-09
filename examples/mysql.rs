use mysql::prelude::*;
use mysql::*;
use pkcore::analysis::store::db::mysql::DB;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = DB::get_connection()?;

    let row: Option<String> = conn.query_first("SELECT VERSION()")?;
    println!("MySQL version: {:?}", row.unwrap());

    Ok(())
}

// let selected_id = 1; // Example parameter for the WHERE clause
// let query = conn.exec_map(
// "SELECT column_name FROM table_name WHERE id = ?",
// (selected_id,),
// |(column_name,)| column_name,
// )?;
//
// // Process the results
// for row in query {
// println!("{:?}", row);
// }