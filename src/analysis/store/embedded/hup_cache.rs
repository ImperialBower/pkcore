use crate::analysis::gto::odds::WinLoseDraw;
use std::collections::HashMap;
use std::sync::LazyLock;

#[cfg(target_arch = "wasm32")]
static HUPS_BIN: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/hups.bin"));

pub static HUP_CACHE: LazyLock<HashMap<(u64, u64), WinLoseDraw>> = LazyLock::new(|| {
    #[cfg(target_arch = "wasm32")]
    {
        match postcard::from_bytes::<Vec<(u64, u64, u64, u64, u64)>>(HUPS_BIN) {
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
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        HashMap::new()
    }
});

/// Look up precomputed heads-up preflop odds from the embedded cache.
///
/// Returns `None` if the matchup is not found (e.g., on native builds where the cache is empty).
pub fn lookup_odds(higher: u64, lower: u64) -> Option<WinLoseDraw> {
    HUP_CACHE.get(&(higher, lower)).copied()
}
