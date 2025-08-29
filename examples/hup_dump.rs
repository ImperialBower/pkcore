use clap::Parser;
use pkcore::analysis::store::db::hup::HUPResult;

/// `cargo run --example hup_dump -- -f "generated/hups_07_31_2025.db" -t "generated/current_hups.csv"`
/// `cargo run --example hup_dump -- -f "generated/hups.db" -t "generated/hups.csv"`
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'f', long)]
    from: String,

    #[clap(short = 't', long)]
    to: String,
}

fn main() {
    let args = Args::parse();

    let from = &*args.from;
    let to = &*args.to;

    let hups = HUPResult::read_db(from).unwrap();
    HUPResult::generate_csv_from_vector(to, &hups).unwrap();

    println!("{from} {to} - {} records", hups.len());
}
