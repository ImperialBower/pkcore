use pkcore::analysis::store::bcm::binary_card_map::SevenFiveBCM;

/// RUST_LOG=trace cargo run --example generate_bcm
fn main() {
    let now = std::time::Instant::now();
    env_logger::init();

    SevenFiveBCM::generate_csv("generated/bcm.csv").expect("TODO: panic message");

    println!("Elapsed: {:.2?}", now.elapsed());
}
