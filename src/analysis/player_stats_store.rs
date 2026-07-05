//! On-disk persistence for [`StatsRegistry`](crate::analysis::player_stats::StatsRegistry).
//!
//! [`PlayerStatsStore`] abstracts the storage backend; [`YamlPlayerStatsStore`]
//! is the default implementation, writing one YAML file per player Uuid under
//! a configured directory.
//!
//! # Workflow
//!
//! Construct a registry from a store with
//! [`StatsRegistry::with_store`](crate::analysis::player_stats::StatsRegistry::with_store)
//! — the registry eagerly loads every existing player record on construction.
//! After ingesting hands during a session, call
//! [`StatsRegistry::flush`](crate::analysis::player_stats::StatsRegistry::flush)
//! (or simply drop the registry — `Drop` calls `flush` automatically) to write
//! the in-memory state back to disk.
//!
//! See EPIC-26 Phase 4 for design rationale.
//!
//! # Eager vs. lazy load
//!
//! The EPIC-26 design doc described a lazy-load model where
//! `StatsRegistry::get(&self)` would consult the store on cache miss.  This
//! implementation went eager instead: `with_store` reads every record at
//! construction, in-memory operations stay `&self` / `&mut self` exactly as
//! before, and `flush` writes everything out at the end. This matches the
//! typical "session-start load, session-end save" workflow and avoids
//! retrofitting interior mutability across the existing
//! [`get`](crate::analysis::player_stats::StatsRegistry::get) / `iter`
//! borrow-pattern (which Phase 3's
//! [`TableSnapshot::from_table_with_stats`](crate::bot::table_snapshot::TableSnapshot::from_table_with_stats)
//! depends on).
//!
//! # Examples
//!
//! ```no_run
//! use pkcore::analysis::player_stats::StatsRegistry;
//! use pkcore::analysis::player_stats_store::YamlPlayerStatsStore;
//!
//! let store = YamlPlayerStatsStore::new("generated/players").unwrap();
//! let mut registry = StatsRegistry::with_store(Box::new(store)).unwrap();
//! // ... ingest hands ...
//! registry.flush().unwrap();
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::PKError;
use crate::analysis::player_stats::PlayerStats;

/// Storage backend for per-player [`PlayerStats`].
///
/// Implementations must be `Send + Sync` so a [`StatsRegistry`] holding a
/// `Box<dyn PlayerStatsStore>` can be moved across threads (e.g. from a
/// `SimTable` running on a worker thread back to the main thread for
/// review). `Debug` is required so [`StatsRegistry`] can keep its
/// `#[derive(Debug)]`.
///
/// [`StatsRegistry`]: crate::analysis::player_stats::StatsRegistry
pub trait PlayerStatsStore: std::fmt::Debug + Send + Sync {
    /// Reads the stats for `id`, returning `Ok(None)` when no record exists.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidIO`] for filesystem errors or malformed
    /// stored data.
    fn load(&self, id: Uuid) -> Result<Option<PlayerStats>, PKError>;

    /// Reads every record the backend knows about.
    ///
    /// Used by [`StatsRegistry::with_store`](crate::analysis::player_stats::StatsRegistry::with_store)
    /// at session start to eagerly populate the in-memory cache.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidIO`] for filesystem errors or malformed
    /// stored data.
    fn load_all(&self) -> Result<HashMap<Uuid, PlayerStats>, PKError>;

    /// Persists the stats for `id`, overwriting any existing record.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidIO`] for filesystem errors or serialization
    /// failures.
    fn save(&self, id: Uuid, stats: &PlayerStats) -> Result<(), PKError>;

    /// Flushes any buffered writes.  The default impl is a no-op for
    /// backends that write through on every `save` (like
    /// [`YamlPlayerStatsStore`]).
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidIO`] when buffered data cannot be written.
    fn flush(&self) -> Result<(), PKError> {
        Ok(())
    }
}

/// One-YAML-file-per-player on-disk store.
///
/// Layout: `<dir>/<uuid>.yaml`. Filenames are the canonical hyphenated UUID
/// representation (`Uuid::to_string` / `Uuid::parse_str`).  Directory is
/// created on construction if it doesn't already exist.
///
/// # Examples
///
/// ```no_run
/// use pkcore::analysis::player_stats_store::YamlPlayerStatsStore;
/// let store = YamlPlayerStatsStore::new("generated/players").unwrap();
/// let _ = store; // pass to StatsRegistry::with_store
/// ```
#[derive(Debug)]
pub struct YamlPlayerStatsStore {
    dir: PathBuf,
}

impl YamlPlayerStatsStore {
    /// Creates a new store rooted at `dir`, creating the directory if
    /// necessary.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidIO`] if the directory cannot be created.
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Self, PKError> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir).map_err(|_| PKError::InvalidIO)?;
        Ok(Self { dir })
    }

    /// Returns the on-disk path for `id`'s record.
    fn path_for(&self, id: Uuid) -> PathBuf {
        self.dir.join(format!("{id}.yaml"))
    }
}

impl PlayerStatsStore for YamlPlayerStatsStore {
    fn load(&self, id: Uuid) -> Result<Option<PlayerStats>, PKError> {
        let path = self.path_for(id);
        if !path.exists() {
            return Ok(None);
        }
        let yaml = fs::read_to_string(&path).map_err(|_| PKError::InvalidIO)?;
        let stats = serde_yaml_bw::from_str::<PlayerStats>(&yaml).map_err(|_| PKError::InvalidIO)?;
        Ok(Some(stats))
    }

    fn load_all(&self) -> Result<HashMap<Uuid, PlayerStats>, PKError> {
        let mut out = HashMap::new();
        // An absent root dir is equivalent to "no records yet" — return empty.
        if !self.dir.exists() {
            return Ok(out);
        }
        let entries = fs::read_dir(&self.dir).map_err(|_| PKError::InvalidIO)?;
        for entry in entries {
            let entry = entry.map_err(|_| PKError::InvalidIO)?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            // Skip files whose stem isn't a UUID — they're not ours to read.
            let Ok(id) = Uuid::parse_str(stem) else {
                continue;
            };
            // II.10: skip-and-log a single bad file rather than failing the
            // whole directory. A crash mid-flush can leave one truncated YAML;
            // that must not brick *every* player's stats on the next load. The
            // offending file is logged (with its path) and left in place.
            let yaml = match fs::read_to_string(&path) {
                Ok(yaml) => yaml,
                Err(e) => {
                    log::warn!("player-stats: skipping unreadable {}: {e}", path.display());
                    continue;
                }
            };
            match serde_yaml_bw::from_str::<PlayerStats>(&yaml) {
                Ok(stats) => {
                    out.insert(id, stats);
                }
                Err(e) => {
                    log::warn!("player-stats: skipping malformed {}: {e}", path.display());
                }
            }
        }
        Ok(out)
    }

    fn save(&self, id: Uuid, stats: &PlayerStats) -> Result<(), PKError> {
        let yaml = serde_yaml_bw::to_string(stats).map_err(|_| PKError::InvalidIO)?;
        let path = self.path_for(id);
        // II.10: atomic write. Serialise to a sibling temp file, then rename it
        // over the target — an atomic operation within a directory on the
        // supported (unix) filesystems. A crash mid-write leaves either the
        // untouched previous file or the temp file, never a truncated target
        // that `load_all` would choke on. The `.yaml.tmp` extension keeps the
        // temp file out of `load_all`'s `.yaml`-only scan.
        let tmp = path.with_extension("yaml.tmp");
        fs::write(&tmp, yaml).map_err(|_| PKError::InvalidIO)?;
        fs::rename(&tmp, &path).map_err(|_| PKError::InvalidIO)?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__player_stats_store_tests {
    use super::*;
    use crate::analysis::player_stats::PlayerStats;
    use std::env;

    /// Returns a unique temp directory under `std::env::temp_dir`. Caller is
    /// responsible for cleanup; tests use `fs::remove_dir_all` at the end.
    fn unique_temp_dir(label: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("pkcore_player_stats_store_{label}_{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn sample_stats(hands: u64) -> PlayerStats {
        let mut s = PlayerStats::default();
        s.hands_dealt = hands;
        s.hands_voluntarily_played = hands / 4;
        s.went_to_showdown = hands / 8;
        s.won_at_showdown = hands / 16;
        s
    }

    #[test]
    fn new_creates_directory_if_missing() {
        let dir = env::temp_dir().join(format!("pkcore_pss_mkdir_{}", Uuid::new_v4()));
        assert!(!dir.exists());
        let _ = YamlPlayerStatsStore::new(&dir).expect("should create dir");
        assert!(dir.exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = unique_temp_dir("round_trip");
        let store = YamlPlayerStatsStore::new(&dir).unwrap();
        let id = Uuid::new_v4();
        let stats = sample_stats(40);
        store.save(id, &stats).expect("save");
        let loaded = store.load(id).expect("load").expect("present");
        assert_eq!(stats, loaded);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_returns_none_for_missing_uuid() {
        let dir = unique_temp_dir("missing");
        let store = YamlPlayerStatsStore::new(&dir).unwrap();
        let missing = Uuid::new_v4();
        assert!(store.load(missing).expect("load ok").is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_all_returns_every_saved_record() {
        let dir = unique_temp_dir("load_all");
        let store = YamlPlayerStatsStore::new(&dir).unwrap();
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();
        store.save(alice, &sample_stats(50)).unwrap();
        store.save(bob, &sample_stats(80)).unwrap();
        let all = store.load_all().expect("load_all");
        assert_eq!(2, all.len());
        assert_eq!(50, all[&alice].hands_dealt);
        assert_eq!(80, all[&bob].hands_dealt);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_all_skips_non_yaml_files() {
        // A stray `.txt` or a `.yaml` whose stem isn't a UUID must be ignored
        // — the store's contract is "every UUID-named YAML is mine; everything
        // else belongs to someone else."
        let dir = unique_temp_dir("non_yaml");
        let store = YamlPlayerStatsStore::new(&dir).unwrap();
        let id = Uuid::new_v4();
        store.save(id, &sample_stats(10)).unwrap();
        fs::write(dir.join("README.txt"), "not mine").unwrap();
        fs::write(dir.join("not-a-uuid.yaml"), "this: too").unwrap();
        let all = store.load_all().expect("load_all");
        assert_eq!(1, all.len());
        assert!(all.contains_key(&id));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_all_skips_corrupt_yaml_file() {
        // II.10: one truncated/malformed file (e.g. a crash mid-flush) must not
        // fail the whole directory — the good records still load.
        let dir = unique_temp_dir("corrupt");
        let store = YamlPlayerStatsStore::new(&dir).unwrap();
        let good = Uuid::new_v4();
        store.save(good, &sample_stats(70)).unwrap();
        // A UUID-named YAML that will not parse as PlayerStats.
        let bad = Uuid::new_v4();
        fs::write(dir.join(format!("{bad}.yaml")), "[unterminated").unwrap();
        let all = store.load_all().expect("load_all must not fail on one bad file");
        assert_eq!(1, all.len(), "only the good record loads");
        assert!(all.contains_key(&good));
        assert!(!all.contains_key(&bad));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_leaves_no_temp_file_behind() {
        // The atomic temp+rename must not leave a `.yaml.tmp` sibling.
        let dir = unique_temp_dir("atomic");
        let store = YamlPlayerStatsStore::new(&dir).unwrap();
        let id = Uuid::new_v4();
        store.save(id, &sample_stats(30)).unwrap();
        let has_tmp = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("tmp"));
        assert!(!has_tmp, "no .tmp file should remain after save");
        // The final file landed and round-trips.
        assert_eq!(30, store.load(id).unwrap().unwrap().hands_dealt);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_overwrites_existing_record() {
        let dir = unique_temp_dir("overwrite");
        let store = YamlPlayerStatsStore::new(&dir).unwrap();
        let id = Uuid::new_v4();
        store.save(id, &sample_stats(10)).unwrap();
        store.save(id, &sample_stats(50)).unwrap();
        let loaded = store.load(id).unwrap().unwrap();
        assert_eq!(50, loaded.hands_dealt);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn default_flush_is_noop() {
        let dir = unique_temp_dir("flush");
        let store = YamlPlayerStatsStore::new(&dir).unwrap();
        // The default `flush` impl returns Ok — YAML store writes through.
        assert!(store.flush().is_ok());
        fs::remove_dir_all(&dir).ok();
    }
}
