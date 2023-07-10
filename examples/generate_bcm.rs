use pkcore::analysis::store::bcm::binary_card_map::BinaryCardMap;

/// cargo run --example generate_bcm
fn main() {
    BinaryCardMap::generate("generated/bcm.csv").expect("TODO: panic message");
}
