use pkcore::analysis::store::bcm::binary_card_map::SevenFiveBCM;

/// Creates a pregenerated zstd-compressed binary file mapping 7-card combinations to the best
/// five-card hand and its Cactus Kev score. The file is typically ~300–600 MB, compared to the
/// ~5 GB CSV equivalent.
///
/// RUST_LOG=trace cargo run --example generate_bcm
fn main() {
    let now = std::time::Instant::now();
    env_logger::init();

    SevenFiveBCM::generate_bin("generated/bcm.zst").expect("Failed to generate bcm binary");

    println!("Elapsed: {:.2?}", now.elapsed());
}
