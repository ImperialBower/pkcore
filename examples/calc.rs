use clap::Parser;
use pkcore::analysis::player_wins::PlayerWins;
use pkcore::analysis::PlayOut;
use pkcore::arrays::HandRanker;
use pkcore::play::board::Board;
use pkcore::play::game::Game;
use pkcore::play::hole_cards::HoleCards;
use pkcore::util::data::TestData;
use pkcore::util::wincounter::results::Results;
use pkcore::{PKError, Pile};
use std::str::FromStr;

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
///
/// What about calling this hand The Fold?
/// RUST_LOG=trace cargo run --example calc -- -d  "5♠ 5♦ 9♠ 9♥ K♣ T♦" -b "5♣ 9♦ T♥ T♣ Q♦"
///
/// ## Step Three
///
/// Show me the winning percentages for each hand at the flop.
///
/// At this point I am starting to feel the strain on my system from my main method
/// trying to do too much. This is when I try to build code that will take the load
/// off and make things easier to maintain and build upon.
///
/// ## Step Four - Calc Structure
///
/// We're reaching the point in our code where the repl is doing to much...maintaining too
/// much state. Our `Game` struct was designed to simply hold all the cards that were needed
/// for the game.
///
/// For now, I want to get all the ducks in a row. Two things that I am missing:
/// * An ordered list of the possible hands at the flop.
/// * A collection of all types of possible hands for a player at the flop.
///
///
///
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let args = Args::parse();

    let game = Game::new(
        HoleCards::from_str(&*args.dealt)?,
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

    let mut pw = PlayerWins::default();

    pw.play_out_flop(&game.hands, game.board.flop);

    for (i, _) in game.hands.iter().enumerate() {
        let (wins, ties) = pw.wins.percentage_for_player(i);

        println!("Player #{} {:.2}% / {:.2}%", i + 1, wins, ties);
    }

    println!("{}", command(game));

    let results = Results::from_wins(&TestData::wins_the_hand(), 2);
    println!("{}", results);

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
