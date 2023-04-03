use pkcore::play::stages::deal_eval::DealEval;
use pkcore::util::data::TestData;
use pkcore::PKError;

fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();

    let game = TestData::the_hand();
    let _deal_eval = DealEval::new(game.hands);

    println!("as_written Elapsed: {:.2?}", now.elapsed());
    Ok(())
}
