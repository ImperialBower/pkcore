use crate::arrays::matchups::masked::Masked;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;

pub const DISTINCT_SHUS_CSV_PATH: &str = "data/csv/shus/distinct_masked_shus.csv";

pub fn distinct_shus_from_csv_as_masked_vec() -> Vec<Masked> {
    let shus = SortedHeadsUp::read_csv(DISTINCT_SHUS_CSV_PATH).unwrap();
    let mut distinct = Masked::parse_as_vectors(&*shus);
    distinct.reverse();
    distinct
}