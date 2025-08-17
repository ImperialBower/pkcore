use clap::Parser;
use pkcore::PKError;
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::solver::Solver;
use pkcore::analysis::gto::twos::Twos;
use pkcore::arrays::two::Two;
use pkcore::play::board::Board;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'p', long)]
    player: String,

    #[clap(short = 'v', long)]
    villain: String,

    #[clap(short = 'b', long)]
    board: String,

    #[clap(short = 'n', long)]
    nuts: bool,
}

/// `cargo run --example gto -- -p "K♠ K♥" -v "66+,AJs+,KQs,AJo+,KQo" -b "J♦ T♣ A♥ K♣ 2♣" -n`
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let args = Args::parse();

    let solver = Solver::new(
        Two::from_str(&*args.player)?,
        Combos::from_str(&*args.villain)?,
        Board::from_str(&*args.board)?,
    );

    println!("{}", solver);

    let twos = Twos::from(solver.villain()).to_vec();

    println!();
    println!("ALL:");
    for (i, combo) in twos.into_iter().enumerate() {
        if i % 10 == 0 {
            println!();
        }
        print!(" {combo} ");
    }

    println!();

    let twos = solver.remaining().to_vec();
    println!();
    println!("BLOCKED:");

    for (i, combo) in twos.into_iter().enumerate() {
        if i % 10 == 0 {
            println!();
        }
        print!(" {combo} ");
    }

    println!();
    println!();

    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}
