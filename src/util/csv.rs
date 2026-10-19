use crate::arrays::matchups::masked::Masked;

pub const DISTINCT_SHUS_CSV_PATH: &str = "data/csv/shus/distinct_masked_shus.csv";

/// Reads the distinct masked matchups; see [`Masked::distinct_from_csv`].
///
/// # Examples
///
/// ```
/// # #![allow(deprecated)]
/// use pkcore::arrays::matchups::masked::Masked;
/// use pkcore::util::csv::distinct_shus_from_csv_as_masked_vec;
///
/// assert_eq!(distinct_shus_from_csv_as_masked_vec(), Masked::distinct_from_csv());
/// ```
#[deprecated(since = "0.16.0", note = "use `Masked::distinct_from_csv`")]
#[must_use]
pub fn distinct_shus_from_csv_as_masked_vec() -> Vec<Masked> {
    Masked::distinct_from_csv()
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__csv_tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn deprecated_function_matches_the_method() {
        assert_eq!(distinct_shus_from_csv_as_masked_vec(), Masked::distinct_from_csv());
    }
}
