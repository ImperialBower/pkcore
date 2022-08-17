use pkcore::analysis::the_nuts::TheNuts;
use pkcore::arrays::HandRanker;
use pkcore::play::game::Game;
use pkcore::util::data::TestData;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let game = TestData::the_hand();

    println!("TAKE 1: {:.2?}", take1(&game));
    println!("TAKE 2: {:.2?}", take2(&game));
}

fn take1(game: &Game) -> Duration {
    let now = std::time::Instant::now();

    let mut the_nuts = TheNuts::default();
    let board = game.flop_and_turn();

    for v in game.turn_remaining_board().combinations(3) {
        if let Ok(seven) = Game::flop_get_seven(board, &v) {
            the_nuts.push(seven.eval());
        }
    }

    the_nuts.sort_in_place();

    now.elapsed()
}

fn take2(game: &Game) -> Duration {
    let now = std::time::Instant::now();

    let mut the_nuts = TheNuts::default();
    let board = game.flop_and_turn();

    let (tx, rx) = mpsc::channel();

    for v in game.turn_remaining_board().combinations(3) {
        if let Ok(seven) = Game::flop_get_seven(board, &v) {
            let tx = tx.clone();
            thread::spawn(move || {
                tx.send(seven.eval()).unwrap();
            });
        }
    }

    drop(tx);

    for received in rx {
        the_nuts.push(received);
    }

    the_nuts.sort_in_place();

    now.elapsed()
}
