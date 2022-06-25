use clap::Parser;
use pkcore::arrays::HandRanker;
use pkcore::play::board::Board;
use pkcore::play::game::Game;
use pkcore::play::hands::Hands;
use pkcore::{PKError, Pile};
use std::str::FromStr;
use pkcore::analysis::player_wins::PlayerWins;
use pkcore::analysis::PlayOut;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'd', long)]
    dealt: String,

    #[clap(short = 'b', long)]
    board: String,
}

/// The goal of calc isn't to run a full simulation of play at a holdem poker table. It's
/// to provide a quick tool that can calculate odds and outs for a specific combination of hands.
///
/// NOTE ON PERSPECTIVE (double dummy)
///
/// We are taking the all knowing view of play, granted to us by modern poker TV shows, pioneered
/// by [Henry Orenstein](https://www.usbets.com/remembering-poker-pioneer-henry-orenstein/).
///
/// ## Step One
///
/// We want to be able to take the cards dealt, and display them representing the hole cards
/// for each of the players.
///
/// ## Step Two
///
/// Show me who has the best hand at the flop
///
/// The hand:
/// `❯ cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"`
///
/// To add logging:
/// RUST_LOG=trace cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"
/// RUST_LOG=trace cargo run --example calc -- -d  "5♠ 5♦ 9♠ 9♥ K♣ T♦" -b "5♣ 9♦ T♥ T♣ Q♦"
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let args = Args::parse();

    let game = Game::new(
        Hands::from_str(&*args.dealt)?,
        Board::from_str(&*args.board)?,
    );

    println!("{}", game);

    // Phase 2.1
    for (i, hole_cards) in game.hands.iter().enumerate() {
        println!(
            "Player #{} {} - {}",
            i + 1,
            hole_cards,
            game.five_at_flop(i)?.hand_rank()
        );
    }

    let mut wins = PlayerWins::default();
    game.pof::<PlayerWins>(&mut wins);

    // game.play_out_flop();

    wins.play_out_flop(&game.hands, game.board.flop);

    println!("{}", command(game));

    println!("Elapsed: {:.2?}", now.elapsed());

    Ok(())
}

fn command(game: Game) -> String {
    format!(
        "cargo run --example calc -- -d  \"{}\" -b \"{}\"",
        game.hands.cards(),
        game.board.cards()
    )
}
