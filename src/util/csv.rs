use crate::arrays::matchups::masked::Masked;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;

pub const DISTINCT_SHUS_CSV_PATH: &str = "data/csv/shus/distinct_masked_shus.csv";

/// # Panics
///
/// If the path isn't there
#[must_use]
pub fn distinct_shus_from_csv_as_masked_vec() -> Vec<Masked> {
    match SortedHeadsUp::read_csv(DISTINCT_SHUS_CSV_PATH) {
        Ok(shus) => {
            let mut distinct = Masked::parse_as_vectors(&shus);
            distinct.reverse();
            distinct
        }
        Err(_e) => {
            log::error!("Unable to read {DISTINCT_SHUS_CSV_PATH}");
            Vec::default()
        }
    }
}
