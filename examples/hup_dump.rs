use clap::Parser;

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

    let hups = read_db("generated/hups.db");

    println!("{from} {to}");
}
