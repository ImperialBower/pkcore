use clap::Parser;
use pkcore::{PKError, GTO};
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::twos::Twos;
use pkcore::analysis::gto::vs::Versus;
use pkcore::arrays::two::Two;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'p', long)]
    player: String,

    #[clap(short = 'v', long)]
    villain: String,

    #[clap(short = 'n', long)]
    nuts: bool,
}

/// `cargo run --example gto -- -p "K♠ K♥" -v "66+,AJs+,KQs,AJo+,KQo"`
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let args = Args::parse();

    let solver = Versus::new(Two::from_str(&*args.player)?, Combos::from_str(&*args.villain)?);

    println!("{}", solver);

    println!("{}", solver.villain.combo_pairs());

    // let twos = Twos::from(solver.villain()).to_vec();
    //
    // println!();
    // println!("ALL:");
    // for (i, combo) in twos.into_iter().enumerate() {
    //     if i % 10 == 0 {
    //         println!();
    //     }
    //     print!(" {combo} ");
    // }

    println!();
    println!();

    let combo_pairs = solver.combo_pairs();
    println!("{combo_pairs}");
    println!();
    println!("¹⁄₁₆");


    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}
