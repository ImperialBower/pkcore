use crate::analysis::gto::odds::WinLoseDraw;
use std::collections::HashMap;
use std::sync::LazyLock;

static HUPS_BIN: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/hups.bin"));

pub static HUP_CACHE: LazyLock<HashMap<(u64, u64), WinLoseDraw>> =
    LazyLock::new(
        || match postcard::from_bytes::<Vec<(u64, u64, u64, u64, u64)>>(HUPS_BIN) {
            Ok(records) => records
                .into_iter()
                .map(|(h, l, w, lo, d)| {
                    (
                        (h, l),
                        WinLoseDraw {
                            wins: w,
                            losses: lo,
                            draws: d,
                        },
                    )
                })
                .collect(),
            Err(e) => {
                log::error!("Failed to deserialize embedded HUP data: {e}");
                HashMap::new()
            }
        },
    );

/// The embedded heads-up preflop odds chart.
///
/// A unit struct that groups this module's functions; it holds no state. The
/// data lives in [`HUP_CACHE`].
///
/// # Domain
///
/// - **Role:** cohesive mechanism
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HupCache;

impl HupCache {
    /// Look up precomputed heads-up preflop odds from the embedded cache.
    ///
    /// `higher` and `lower` are the two hands as `Bard` bits, in
    /// `SortedHeadsUp` order. Returns `None` if the matchup is not found in the
    /// cache.
    ///
    /// # Domain
    ///
    /// - **Invariants:**
    ///   - `None` when the matchup is absent or the embedded data failed to decode
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::store::embedded::hup_cache::{HUP_CACHE, HupCache};
    ///
    /// let (&(higher, lower), &odds) = HUP_CACHE.iter().next().unwrap();
    /// assert_eq!(HupCache::lookup_odds(higher, lower), Some(odds));
    /// assert_eq!(HupCache::lookup_odds(0, 0), None);
    /// ```
    #[must_use]
    pub fn lookup_odds(higher: u64, lower: u64) -> Option<WinLoseDraw> {
        HUP_CACHE.get(&(higher, lower)).copied()
    }
}

/// Look up precomputed heads-up preflop odds; see [`HupCache::lookup_odds`].
///
/// # Examples
///
/// ```
/// # #![allow(deprecated)]
/// use pkcore::analysis::store::embedded::hup_cache::{HupCache, lookup_odds};
///
/// assert_eq!(lookup_odds(0, 0), HupCache::lookup_odds(0, 0));
/// ```
#[deprecated(since = "0.16.0", note = "use `HupCache::lookup_odds`")]
#[must_use]
pub fn lookup_odds(higher: u64, lower: u64) -> Option<WinLoseDraw> {
    HupCache::lookup_odds(higher, lower)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__store__embedded__hup_cache_tests {
    use super::*;

    #[test]
    fn hup_cache_is_a_stateless_unit_struct() {
        assert_eq!(HupCache, HupCache::default());
    }

    #[test]
    fn lookup_odds_finds_an_embedded_matchup() {
        let (&(higher, lower), &odds) = HUP_CACHE.iter().next().unwrap();

        assert_eq!(HupCache::lookup_odds(higher, lower), Some(odds));
    }

    #[test]
    fn lookup_odds_is_none_for_an_absent_matchup() {
        assert_eq!(HupCache::lookup_odds(0, 0), None);
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_lookup_odds_matches_the_method() {
        let (&(higher, lower), _) = HUP_CACHE.iter().next().unwrap();

        assert_eq!(lookup_odds(higher, lower), HupCache::lookup_odds(higher, lower));
        assert_eq!(lookup_odds(0, 0), HupCache::lookup_odds(0, 0));
    }
}
