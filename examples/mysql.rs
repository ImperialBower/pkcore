use mysql::*;
use pkcore::analysis::store::db::mysql::DbConnectOps;
use pkcore::analysis::store::db::mysql::MySqlDB;

/// `cargo run --example mysql`
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MySQL version: {:?}", MySqlDB::version_string().unwrap());

    Ok(())
}
