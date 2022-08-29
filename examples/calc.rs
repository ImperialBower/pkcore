use clap::Parser;
use pkcore::play::board::Board;
use pkcore::play::game::Game;
use pkcore::play::hole_cards::HoleCards;
use pkcore::play::stages::flop_eval::FlopEval;
use pkcore::{PKError, Pile};
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'd', long)]
    dealt: String,

    #[clap(short = 'b', long)]
    board: String,

    #[clap(short = 'n', long)]
    nuts: bool,
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
/// ## PHASE 3.1: Outs
///
/// Now that we have the win percentages displayed at the flop, we need to add the icing on the cake:
/// player outs. One of the clearest ways to display the meaning behind the odds is to show the
/// cards that the player behind on the hand would need in order to win.
///
/// Since our calc example is starting to take on a lot of business logic, this may be a good time
/// to do some refactoring and move it into dedicated structs.
///
/// Calculating win percentages and outs should be part of the same iteration through the possible
/// cases. I'm feeling the need to break this problem down with a spike in our example hear and
/// see where it leads us.
///
/// The structure that I am thinking to hold each of the player's outs is simple:
///
/// ```
/// #[derive(Clone, Debug, Default, Eq, PartialEq)]
/// pub struct Outs(Vec<Cards>);
/// ```
///
/// `cargo run --example calc -- -d "A♠ K♥ 8♦ 6♣" -b "A♣ 8♥ 7♥ 9♠ 5♠" -n`
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

    println!();
    let flop_eval = FlopEval::try_from(game.clone()).unwrap();
    println!("{}", flop_eval);

    if args.nuts {
        println!();
        println!("The Nuts @ Flop:");
        println!("{}", game.board.flop.evals());
    }

    game.turn_display_odds()?;

    // too slow
    // if args.nuts {
    //     game.display_evals_at_turn();
    // }

    game.river_display_results();

    println!();
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
