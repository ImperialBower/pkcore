use pkcore::analysis::store::bcm::binary_card_map::BinaryCardMap;

fn main() {
    BinaryCardMap::generate("generated/bcm.csv").expect("TODO: panic message");
}