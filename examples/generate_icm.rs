use pkcore::analysis::store::bcm::index_card_map::IndexCardMap;

/// RUST_LOG=trace cargo run --example generate_icm
fn main() {
    let now = std::time::Instant::now();
    env_logger::init();

    IndexCardMap::generate_csv("generated/icm.csv").expect("TODO: panic message");

    println!("Elapsed: {:.2?}", now.elapsed());
}
