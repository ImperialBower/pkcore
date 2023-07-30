use clap::Parser;
use rusqlite::Connection;
use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'f', long)]
    from: String,

    #[clap(short = 't', long)]
    to: String,
}

/// `cargo run --example hups_import -- -f "data/washed_hups.csv" -t "data/hups.db"`
fn main() {
    let args = Args::parse();

    let from = &*args.from;
    let to = &*args.to;

    let hups = HUPResult::read_csv(from).unwrap();
    let conn = Connection::open(to).unwrap();
    HUPResult::create_table(&conn).unwrap();

    for h in hups.clone() {
        HUPResult::insert(&conn, &h).unwrap();
    }
    assert!(HUPResult::db_is_valid(&conn));
    conn.close().unwrap();
}