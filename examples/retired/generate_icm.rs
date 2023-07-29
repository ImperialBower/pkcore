use pkcore::analysis::store::bcm::index_card_map::IndexCardMap;

/// RUST_LOG=trace cargo run --example generate_icm
///
/// This took 93791.61 seconds (26 hours)  to run on my Intel Linux box, and created a file that is
/// 8.8 GBs.
fn main() {
    let now = std::time::Instant::now();
    env_logger::init();

    IndexCardMap::generate_csv("generated/icm.csv").expect("TODO: panic message");

    println!("Elapsed: {:.2?}", now.elapsed());
}
