// `docs/KERNEL_PURITY_AUDIT.md` §3 (hup-charts caveat), fix 2: gate the 15.8 MB chart by `cfg`, not by
// trusting the linker to drop an unreferenced static.
#[cfg(feature = "hup-charts")]
pub mod hup_cache;
