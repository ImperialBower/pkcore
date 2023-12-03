use std::collections::HashMap;
use std::fs::File;
use std::sync::OnceLock;
use csv::Reader;
use pkcore::analysis::store::bcm::binary_card_map::{FiveBCM, SevenFiveBCM};
use pkcore::bard::Bard;

pub static BC_RANK: OnceLock<HashMap<Bard, FiveBCM>> = {
    let once_lock = OnceLock::new();
    let mut m = HashMap::new();
    let file = File::open(SevenFiveBCM::get_csv_filepath()).unwrap();
    let mut rdr = Reader::from_reader(file);

    for result in rdr.deserialize() {
        let bcm: SevenFiveBCM = result.unwrap();
        m.insert(bcm.bc, FiveBCM::from(bcm));
    }

    once_lock.set(m).expect("Should be possible to set the first time");
    once_lock
};

fn main() {
    assert!(BC_RANK.get().is_none());
}