//! Persistent disk cache for solved GTO spots.
//!
//! [`SolverCache`] stores [`SolverResult`] values on disk, keyed by a
//! deterministic hash of [`SolverConfig`]. A spot solved once is loaded from
//! disk on subsequent runs instead of being re-solved.
//!
//! # Cache Key
//!
//! [`cache_key`] produces a `u64` from the config's hero range, villain range,
//! board, bet sizings, effective stack, and pot. [`crate::analysis::gto::combos::Combos`] is backed by a
//! `HashSet`, so its contents are sorted before hashing to guarantee the same
//! key regardless of iteration order.
//!
//! The key uses [`std::collections::hash_map::DefaultHasher`], which is stable
//! within a single Rust build but not guaranteed stable across compiler
//! versions. A key mismatch causes a cache miss and a fresh solve — never a
//! correctness error.
//!
//! # File Layout
//!
//! Each entry is stored as `{key:016x}.bin` — a 16-character hex filename
//! containing a bincode-serialized [`SolverResult`]. Files are independent so
//! the cache is safe for concurrent reads.
//!
//! # Examples
//!
//! ```no_run
//! use std::str::FromStr;
//! use pkcore::analysis::gto::combos::Combos;
//! use pkcore::analysis::gto::solver::Solver;
//! use pkcore::analysis::gto::solver_cache::SolverCache;
//! use pkcore::analysis::gto::solver_config::SolverConfig;
//! use pkcore::play::board::Board;
//!
//! let config = SolverConfig::new(
//!     Combos::from_str("AA,KK").unwrap_or_default(),
//!     Combos::from_str("QQ,JJ").unwrap_or_default(),
//!     Board::from_str("Ah Kd 5c 2s 7h").unwrap_or_default(),
//!     1_000, 200,
//! );
//!
//! let cache = SolverCache::new("/tmp/pkcore_cache").unwrap();
//! let result = match cache.get(&config) {
//!     Some(r) => r,
//!     None => {
//!         let r = Solver::new(config.clone()).solve();
//!         cache.put(&config, &r).unwrap();
//!         r
//!     }
//! };
//! println!("exploitability: {:.4}", result.exploitability);
//! ```

use crate::analysis::gto::solver::{SolverError, SolverResult};
use crate::analysis::gto::solver_config::SolverConfig;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

// ── cache_key ─────────────────────────────────────────────────────────────────

/// Computes a deterministic `u64` cache key for a [`SolverConfig`].
///
/// Key inputs (in hash order):
/// - Hero range — sorted `Vec<Combo>` (removes `HashSet` non-determinism)
/// - Villain range — sorted `Vec<Combo>`
/// - Board cards — flop (3 cards), turn, river, in field order
/// - Bet sizings — each street's sizes sorted independently
/// - Effective stack
/// - Pot
///
/// Uses [`DefaultHasher`] — stable within a build, suitable for a local disk
/// cache. A key mismatch across builds is a cache miss, not a correctness
/// error.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use pkcore::analysis::gto::combos::Combos;
/// use pkcore::analysis::gto::solver_cache::cache_key;
/// use pkcore::analysis::gto::solver_config::SolverConfig;
/// use pkcore::play::board::Board;
///
/// let config = SolverConfig::new(
///     Combos::from_str("AA,KK").unwrap_or_default(),
///     Combos::from_str("QQ,JJ").unwrap_or_default(),
///     Board::from_str("Ah Kd 5c 2s 7h").unwrap_or_default(),
///     1_000, 200,
/// );
/// let k1 = cache_key(&config);
/// let k2 = cache_key(&config);
/// assert_eq!(k1, k2);
/// ```
#[must_use]
pub fn cache_key(config: &SolverConfig) -> u64 {
    let mut hasher = DefaultHasher::new();

    // Ranges — sort before hashing to neutralise HashSet iteration order.
    let mut hero: Vec<_> = config.hero_range.iter().copied().collect();
    hero.sort();
    hero.hash(&mut hasher);

    let mut villain: Vec<_> = config.villain_range.iter().copied().collect();
    villain.sort();
    villain.hash(&mut hasher);

    // Board — hash in fixed field order (flop1, flop2, flop3, turn, river).
    config.board.flop.first().hash(&mut hasher);
    config.board.flop.second().hash(&mut hasher);
    config.board.flop.third().hash(&mut hasher);
    config.board.turn.hash(&mut hasher);
    config.board.river.hash(&mut hasher);

    // Bet sizings — sort each street so {half, pot} and {pot, half} are equal.
    let mut flop_sizes = config.bet_sizings.flop.clone();
    flop_sizes.sort();
    flop_sizes.hash(&mut hasher);

    let mut turn_sizes = config.bet_sizings.turn.clone();
    turn_sizes.sort();
    turn_sizes.hash(&mut hasher);

    let mut river_sizes = config.bet_sizings.river.clone();
    river_sizes.sort();
    river_sizes.hash(&mut hasher);

    config.effective_stack.hash(&mut hasher);
    config.pot.hash(&mut hasher);

    hasher.finish()
}

// ── SolverCache ───────────────────────────────────────────────────────────────

/// A directory-backed cache of solved GTO spots.
///
/// Each entry is stored as `{key:016x}.bin` inside `dir`. Files are
/// independent bincode-serialized [`SolverResult`] values.
///
/// # Examples
///
/// ```no_run
/// use pkcore::analysis::gto::solver_cache::SolverCache;
///
/// let cache = SolverCache::new("/tmp/pkcore_cache").unwrap();
/// assert!(cache.is_empty());
/// ```
pub struct SolverCache {
    dir: PathBuf,
}

impl SolverCache {
    /// Opens a cache backed by `dir`, creating the directory if it does not
    /// exist.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::Io`] if the directory cannot be created.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pkcore::analysis::gto::solver_cache::SolverCache;
    ///
    /// let cache = SolverCache::new("/tmp/pkcore_cache").unwrap();
    /// ```
    pub fn new(dir: impl Into<PathBuf>) -> Result<Self, SolverError> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    /// Returns a cached result for `config`, or `None` on a cache miss or any
    /// I/O / deserialization error.
    ///
    /// Errors are silenced so the caller can always fall through to a fresh
    /// solve without special-casing partial writes or corrupt files.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::str::FromStr;
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver_cache::SolverCache;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(), Board::default(), 500, 100,
    /// );
    /// let cache = SolverCache::new("/tmp/pkcore_cache").unwrap();
    /// assert!(cache.get(&config).is_none()); // cold cache
    /// ```
    #[must_use]
    pub fn get(&self, config: &SolverConfig) -> Option<SolverResult> {
        let path = self.entry_path(config);
        let bytes = fs::read(path).ok()?;
        SolverResult::from_binary_bytes(&bytes).ok()
    }

    /// Stores `result` under the key derived from `config`.
    ///
    /// Overwrites any existing entry for the same key.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError`] on I/O or serialization failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::str::FromStr;
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver::Solver;
    /// use pkcore::analysis::gto::solver_cache::SolverCache;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(), Board::default(), 500, 100,
    /// ).with_max_iterations(5);
    /// let result = Solver::new(config.clone()).solve();
    /// let cache = SolverCache::new("/tmp/pkcore_cache").unwrap();
    /// cache.put(&config, &result).unwrap();
    /// assert!(cache.contains(&config));
    /// ```
    pub fn put(&self, config: &SolverConfig, result: &SolverResult) -> Result<(), SolverError> {
        let bytes = result.to_binary_bytes()?;
        fs::write(self.entry_path(config), bytes)?;
        Ok(())
    }

    /// Returns `true` if a cached entry exists for `config`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver_cache::SolverCache;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(), Board::default(), 500, 100,
    /// );
    /// let cache = SolverCache::new("/tmp/pkcore_cache").unwrap();
    /// assert!(!cache.contains(&config));
    /// ```
    #[must_use]
    pub fn contains(&self, config: &SolverConfig) -> bool {
        self.entry_path(config).exists()
    }

    /// Returns the number of cached entries (`.bin` files) in the cache
    /// directory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pkcore::analysis::gto::solver_cache::SolverCache;
    ///
    /// let cache = SolverCache::new("/tmp/pkcore_cache").unwrap();
    /// assert_eq!(cache.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.bin_files().count()
    }

    /// Returns `true` if the cache contains no entries.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pkcore::analysis::gto::solver_cache::SolverCache;
    ///
    /// let cache = SolverCache::new("/tmp/pkcore_cache").unwrap();
    /// assert!(cache.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Deletes all `.bin` files in the cache directory.
    ///
    /// The directory itself is preserved. Returns the first I/O error
    /// encountered, if any; entries before the failing one are already deleted.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::Io`] if any file cannot be deleted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pkcore::analysis::gto::solver_cache::SolverCache;
    ///
    /// let cache = SolverCache::new("/tmp/pkcore_cache").unwrap();
    /// cache.clear().unwrap();
    /// assert!(cache.is_empty());
    /// ```
    pub fn clear(&self) -> Result<(), SolverError> {
        for path in self.bin_files().collect::<Vec<_>>() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Returns the full path for the entry file of `config`.
    fn entry_path(&self, config: &SolverConfig) -> PathBuf {
        self.dir.join(format!("{:016x}.bin", cache_key(config)))
    }

    /// Iterates over all `.bin` file paths in the cache directory.
    fn bin_files(&self) -> impl Iterator<Item = PathBuf> {
        fs::read_dir(&self.dir)
            .into_iter()
            .flatten()
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("bin"))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::gto::combos::Combos;
    use crate::analysis::gto::solver::Solver;
    use crate::analysis::gto::solver_config::{BetSize, BetSizings, SolverConfig};
    use crate::play::board::Board;
    use std::str::FromStr;

    fn river_config() -> SolverConfig {
        SolverConfig::new(
            Combos::from_str("AA,KK").unwrap_or_default(),
            Combos::from_str("QQ,JJ").unwrap_or_default(),
            Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
            1_000,
            200,
        )
        .with_max_iterations(3)
    }

    fn temp_cache() -> SolverCache {
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};

        // Unique per call: mix process id, thread id, and subsecond clock so
        // parallel tests each get an isolated directory.
        let mut h = DefaultHasher::new();
        std::process::id().hash(&mut h);
        std::thread::current().id().hash(&mut h);
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
            .hash(&mut h);
        let dir = std::env::temp_dir().join(format!("pkcore_cache_test_{:016x}", h.finish()));
        SolverCache::new(&dir).unwrap()
    }

    // ── cache_key ─────────────────────────────────────────────────────────────

    #[test]
    fn test_cache_key_same_config_is_deterministic() {
        let config = river_config();
        assert_eq!(cache_key(&config), cache_key(&config));
    }

    #[test]
    fn test_cache_key_different_hero_range_differs() {
        let config1 = river_config();
        let config2 = SolverConfig::new(
            Combos::from_str("QQ,JJ").unwrap_or_default(),
            Combos::from_str("QQ,JJ").unwrap_or_default(),
            Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
            1_000,
            200,
        );
        assert_ne!(cache_key(&config1), cache_key(&config2));
    }

    #[test]
    fn test_cache_key_different_villain_range_differs() {
        let config1 = river_config();
        let config2 = SolverConfig::new(
            Combos::from_str("AA,KK").unwrap_or_default(),
            Combos::from_str("AA,KK").unwrap_or_default(),
            Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
            1_000,
            200,
        );
        assert_ne!(cache_key(&config1), cache_key(&config2));
    }

    #[test]
    fn test_cache_key_different_board_differs() {
        let config1 = river_config();
        let config2 = SolverConfig::new(
            Combos::from_str("AA,KK").unwrap_or_default(),
            Combos::from_str("QQ,JJ").unwrap_or_default(),
            Board::from_str("Ah Kd 5c 2s 7h").unwrap_or_default(),
            1_000,
            200,
        );
        assert_ne!(cache_key(&config1), cache_key(&config2));
    }

    #[test]
    fn test_cache_key_different_sizings_differs() {
        let config1 = river_config();
        let config2 = river_config().with_bet_sizings(BetSizings::uniform(vec![BetSize::pot()]));
        assert_ne!(cache_key(&config1), cache_key(&config2));
    }

    #[test]
    fn test_cache_key_different_stack_differs() {
        let config1 = river_config();
        let config2 = SolverConfig::new(
            Combos::from_str("AA,KK").unwrap_or_default(),
            Combos::from_str("QQ,JJ").unwrap_or_default(),
            Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
            2_000,
            200,
        );
        assert_ne!(cache_key(&config1), cache_key(&config2));
    }

    #[test]
    fn test_cache_key_different_pot_differs() {
        let config1 = river_config();
        let config2 = SolverConfig::new(
            Combos::from_str("AA,KK").unwrap_or_default(),
            Combos::from_str("QQ,JJ").unwrap_or_default(),
            Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
            1_000,
            400,
        );
        assert_ne!(cache_key(&config1), cache_key(&config2));
    }

    // ── SolverCache ───────────────────────────────────────────────────────────

    #[test]
    fn test_solver_cache_miss_returns_none() {
        let cache = temp_cache();
        assert!(cache.get(&river_config()).is_none());
    }

    #[test]
    fn test_solver_cache_is_empty_on_fresh_dir() {
        let cache = temp_cache();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_solver_cache_contains_false_before_put() {
        let cache = temp_cache();
        assert!(!cache.contains(&river_config()));
    }

    #[test]
    fn test_solver_cache_put_then_contains() {
        let cache = temp_cache();
        let config = river_config();
        let result = Solver::new(config.clone()).solve();
        cache.put(&config, &result).unwrap();
        assert!(cache.contains(&config));
    }

    #[test]
    fn test_solver_cache_put_then_get_round_trips() {
        let cache = temp_cache();
        let config = river_config();
        let result = Solver::new(config.clone()).solve();
        cache.put(&config, &result).unwrap();
        let loaded = cache.get(&config).expect("should be a cache hit");
        assert_eq!(loaded.iterations, result.iterations);
        assert!((loaded.exploitability - result.exploitability).abs() < 1e-9);
    }

    #[test]
    fn test_solver_cache_len_increments_on_put() {
        let cache = temp_cache();
        let config = river_config();
        let result = Solver::new(config.clone()).solve();
        assert_eq!(cache.len(), 0);
        cache.put(&config, &result).unwrap();
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_solver_cache_clear_removes_entries() {
        let cache = temp_cache();
        let config = river_config();
        let result = Solver::new(config.clone()).solve();
        cache.put(&config, &result).unwrap();
        assert!(!cache.is_empty());
        cache.clear().unwrap();
        assert!(cache.is_empty());
        assert!(!cache.contains(&config));
    }

    #[test]
    fn test_solver_cache_put_overwrites_existing() {
        let cache = temp_cache();
        let config = river_config();
        let r1 = Solver::new(config.clone()).solve();
        cache.put(&config, &r1).unwrap();
        let r2 = Solver::new(config.clone()).solve();
        cache.put(&config, &r2).unwrap();
        // Still only one entry — overwrite, not append.
        assert_eq!(cache.len(), 1);
        let loaded = cache.get(&config).unwrap();
        assert_eq!(loaded.iterations, r2.iterations);
    }
}
