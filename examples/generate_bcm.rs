use pkcore::analysis::store::bcm::binary_card_map::SevenFiveBCM;

/// Creates a pregenerated file that makes 7 cards to the best five cards and to the score NLH
/// Cactus Kev score for the hand. The file generated is a little under 5GB in size.
///
/// While it takes a over 10 minutes to load, once it's in memory, doing combo calculations is
/// much faster.
///
/// RUST_LOG=trace cargo run --example generate_bcm
fn main() {
    let now = std::time::Instant::now();
    env_logger::init();

    SevenFiveBCM::generate_csv("generated/bcm.csv").expect("TODO: panic message");

    println!("Elapsed: {:.2?}", now.elapsed());
}
