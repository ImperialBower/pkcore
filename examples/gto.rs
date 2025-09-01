use clap::Parser;
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::vs::Versus;
use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::arrays::two::Two;
use pkcore::play::board::Board;
use pkcore::play::stages::flop_eval::FlopEval;
use pkcore::{GTO, PKError};
use rusqlite::Connection;
use std::str::FromStr;
use pkcore::analysis::gto::odds::WinLoseDraw;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'p', long)]
    player: String,

    #[clap(short = 'v', long)]
    villain: String,

    #[clap(short = 'b', long, required = false)]
    board: Option<Board>,

    #[clap(short = 'n', long)]
    nuts: bool,
}

/// `cargo run --example gto -- -p "K♠ K♥" -v "66+,AJs+,KQs,AJo+,KQo"`
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let args = Args::parse();

    let solver: Versus;

    if let Some(board) = args.board {
        solver = Versus::new_with_board(Two::from_str(&*args.player)?, Combos::from_str(&*args.villain)?, board);
    } else {
        solver = Versus::new(Two::from_str(&*args.player)?, Combos::from_str(&*args.villain)?);
    }

    println!("{}", solver);
    println!();
    println!("{}", solver.villain.combo_pairs());

    println!();

    println!("{}", solver.combo_pairs());
    println!();
    println!("¹⁄₁₆");

    println!();
    println!();

    let conn = Connection::open("generated/hups.db").unwrap();

    let hups = solver.hups_at_deal(&conn);

    for key in hups.keys() {
        println!("{}", hups.get(key).unwrap());
    }

    let results = Versus::combined_odds_at_deal(&hups.values().collect::<Vec<&HUPResult>>());
    println!();
    println!("{}", results);

    if solver.has_board() {
        // let games = solver.games_at_flop();
        // for game in &games {
        //     let fe = FlopEval::try_from(game.clone()).unwrap();
        //     println!("{fe}");
        //     println!(">>>{}", WinLoseDraw::from(fe));
        //
        // }
        println!("{}", solver.combined_odds_at_flop());

        // for game in &games {
        //     game.river_display_results();
        // }
    }

    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}
