use clap::Parser;
use pkcore::cards::Cards;
use std::str::FromStr;

/// ```
/// ❯ cargo run --example repl -- -c "AS KS QS JS TS"
///     Finished dev [unoptimized + debuginfo] target(s) in 0.04s
///      Running `target/debug/examples/repl -c 'AS KS QS JS TS'`
/// A♠ K♠ Q♠ J♠ T♠
/// Elapsed: 325.18µs
/// ```
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'c', long)]
    card: String,
}
fn main() {
    let now = std::time::Instant::now();

    let args = Args::parse();

    // https://stackoverflow.com/a/23977218/1245251
    println!("{}", Cards::from_str(&*args.card).unwrap());

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
