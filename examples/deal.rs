use pkcore::PKError;
use pkcore::play::stages::deal_eval::DealEval;
use pkcore::util::data::TestData;

fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();

    let game = TestData::the_hand();
    let deal_eval = DealEval::new(game.hands);

    println!("as_written Elapsed: {:.2?}", now.elapsed());
    Ok(())
}