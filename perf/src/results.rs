//! The on-disk results schema.
//!
//! Every number carries the context that makes it meaningful: target triple,
//! runtime, host CPU topology, compiler, pkcore version, active features, and
//! rayon pool size. Recording features per run is what stops numbers taken
//! under different feature sets from being silently compared.

use crate::runner::Sample;
use serde::{Deserialize, Serialize};

/// The machine a run was taken on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Host {
    /// CPU brand string, e.g. `"Apple M1"`. `"unknown"` where undetectable.
    pub cpu: String,
    /// Total logical cores.
    pub cores: usize,
    /// Performance cores, where the platform distinguishes them.
    pub p_cores: Option<usize>,
    /// Efficiency cores, where the platform distinguishes them.
    pub e_cores: Option<usize>,
}

impl Host {
    /// Detects host CPU facts.
    ///
    /// On macOS this shells out to `sysctl`; elsewhere the CPU brand is
    /// `"unknown"` and the P/E split is `None`. Never fails — undetectable
    /// facts become `unknown`/`None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore_perf::results::Host;
    ///
    /// let host = Host::detect();
    /// assert!(host.cores >= 1);
    /// ```
    #[must_use]
    pub fn detect() -> Host {
        let cores = std::thread::available_parallelism().map_or(1, |n| n.get());

        Host {
            cpu: Self::sysctl("machdep.cpu.brand_string").unwrap_or_else(|| "unknown".to_string()),
            cores,
            p_cores: Self::sysctl("hw.perflevel0.logicalcpu").and_then(|s| s.parse().ok()),
            e_cores: Self::sysctl("hw.perflevel1.logicalcpu").and_then(|s| s.parse().ok()),
        }
    }

    #[cfg(target_os = "macos")]
    fn sysctl(key: &str) -> Option<String> {
        let out = std::process::Command::new("sysctl").args(["-n", key]).output().ok()?;
        let text = String::from_utf8(out.stdout).ok()?.trim().to_string();
        if text.is_empty() { None } else { Some(text) }
    }

    #[cfg(not(target_os = "macos"))]
    fn sysctl(_key: &str) -> Option<String> {
        None
    }
}

/// Everything about a run except the measurements themselves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunMeta {
    /// ISO-8601 UTC timestamp, supplied by the caller.
    pub utc: String,
    /// Target triple, captured at build time.
    pub target: String,
    /// Execution environment: `"native"`, `"wasmtime"`, or `"browser"`.
    pub runtime: String,
    /// Host CPU facts.
    pub host: Host,
    /// Compiler version, captured at build time.
    pub rustc: String,
    /// pkcore version, supplied via the `PKCORE_VERSION` env var.
    pub pkcore: String,
    /// pkcore cargo features active in this build.
    pub features: Vec<String>,
    /// Rayon pool size, where the run configured one.
    pub rayon_threads: Option<usize>,
    /// Optional run label, e.g. `"post-fix"`. Distinguishes two runs taken on
    /// the same day for the same target, which would otherwise collide on
    /// filename and silently overwrite one another.
    #[serde(default)]
    pub label: Option<String>,
}

impl RunMeta {
    /// Captures run metadata.
    ///
    /// `utc` is passed in rather than read from a clock so callers control the
    /// format and tests stay deterministic. `pkcore` comes from the
    /// `PKCORE_VERSION` environment variable, which the Makefile sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore_perf::results::RunMeta;
    ///
    /// let meta = RunMeta::capture("native", vec![], None, "2026-07-30T00:00:00Z".into());
    /// assert_eq!(meta.runtime, "native");
    /// ```
    #[must_use]
    pub fn capture(runtime: &str, features: Vec<String>, rayon_threads: Option<usize>, utc: String) -> RunMeta {
        RunMeta {
            utc,
            target: crate::target_triple().to_string(),
            runtime: runtime.to_string(),
            host: Host::detect(),
            rustc: crate::rustc_version().to_string(),
            pkcore: std::env::var("PKCORE_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            features,
            rayon_threads,
            label: None,
        }
    }
}

/// A complete results file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Results {
    /// Schema version. Bump when the shape changes incompatibly.
    pub schema: u32,
    /// Run context.
    pub run: RunMeta,
    /// One entry per workload measured.
    pub samples: Vec<Sample>,
}

impl Results {
    /// Current schema version.
    pub const SCHEMA: u32 = 1;

    /// The conventional filename for this run: `<target>[-<label>]-<date>.json`.
    ///
    /// The time-of-day portion of `utc` is dropped so the name stays free of
    /// colons, which are not portable in filenames. Without a label, two runs
    /// on the same day for the same target collide — pass one whenever a run
    /// is meant to sit alongside another rather than replace it.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore_perf::results::{Results, RunMeta};
    ///
    /// let mut results = Results {
    ///     schema: Results::SCHEMA,
    ///     run: RunMeta::capture("native", vec![], None, "2026-07-31T18:04:11Z".into()),
    ///     samples: vec![],
    /// };
    /// results.run.label = Some("post-fix".into());
    /// assert!(results.filename().contains("post-fix"));
    /// ```
    #[must_use]
    pub fn filename(&self) -> String {
        let date = self.run.utc.split('T').next().unwrap_or("undated");
        match &self.run.label {
            Some(label) => format!("{}-{label}-{date}.json", self.run.target),
            None => format!("{}-{date}.json", self.run.target),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod perf__results_tests {
    use super::*;
    use crate::catalog::catalog;
    use crate::runner::measure;

    #[test]
    fn host_detect_reports_at_least_one_core() {
        let host = Host::detect();
        assert!(host.cores >= 1);
        assert!(!host.cpu.is_empty());
    }

    #[test]
    fn results_round_trip_through_json() {
        let samples = vec![measure(&catalog()[0], 1, 3, 100)];
        let results = Results {
            schema: Results::SCHEMA,
            run: RunMeta::capture("native", vec![], None, "2026-07-30T00:00:00Z".to_string()),
            samples,
        };

        let json = serde_json::to_string_pretty(&results).expect("serializes");
        let back: Results = serde_json::from_str(&json).expect("deserializes");

        assert_eq!(back.schema, 1);
        assert_eq!(back.samples.len(), 1);
        assert_eq!(back.run.runtime, "native");
    }

    #[test]
    fn filename_joins_target_and_date() {
        let results = Results {
            schema: Results::SCHEMA,
            run: RunMeta::capture("native", vec![], None, "2026-07-30T18:04:11Z".to_string()),
            samples: vec![],
        };
        let name = results.filename();
        assert!(name.ends_with("-2026-07-30.json"), "got {name}");
        assert!(!name.contains(':'), "colons are not portable in filenames");
    }

    #[test]
    fn capture_records_build_facts() {
        let meta = RunMeta::capture("native", vec!["equity".to_string()], Some(8), "x".to_string());
        assert_eq!(meta.target, crate::target_triple());
        assert!(meta.rustc.starts_with("rustc"));
        assert_eq!(meta.features, vec!["equity".to_string()]);
        assert_eq!(meta.rayon_threads, Some(8));
    }

    #[test]
    fn filename_includes_the_label_when_present() {
        let mut results = Results {
            schema: Results::SCHEMA,
            run: RunMeta::capture("native", vec![], None, "2026-07-31T18:04:11Z".to_string()),
            samples: vec![],
        };
        results.run.label = Some("post-fix".to_string());

        let name = results.filename();
        assert!(name.contains("post-fix"), "got {name}");
        assert!(name.ends_with("-2026-07-31.json"), "got {name}");
    }

    #[test]
    fn filename_omits_the_label_when_absent() {
        let results = Results {
            schema: Results::SCHEMA,
            run: RunMeta::capture("native", vec![], None, "2026-07-31T18:04:11Z".to_string()),
            samples: vec![],
        };
        assert_eq!(
            results.filename(),
            format!("{}-2026-07-31.json", crate::target_triple())
        );
    }
}
