//! `perf` — the native and WASI runner for the pkcore performance harness.
//!
//! ```text
//! perf list                       # print the catalog
//! perf run                        # measure everything, write JSON
//! perf run eval.five.or_rank_bits # measure one workload
//! perf report                     # regenerate docs/perf/RESULTS.md
//! ```
//!
//! Options for `run`:
//!   --out DIR        results directory (default: docs/perf/results)
//!   --utc STAMP      ISO-8601 timestamp to record (default: 1970-01-01T00:00:00Z)
//!   --trials N       override the per-band trial count (N >= 1)
//!   --iters N        override the per-workload inner iteration count (N >= 1)
//!   --stdout         print JSON instead of writing a file
//!   --label NAME     tag the results file, so same-day runs do not collide
//!   --sweep          measure each parallel workload at 1, 4 and 8 rayon threads
//!
//! Options for `report`:
//!   --dir DIR        results directory to read (default: docs/perf/results)
//!   --out FILE       markdown file to write (default: docs/perf/RESULTS.md)
//!
//! Default paths are anchored at the repo root — the nearest ancestor of the
//! working directory that contains `docs/perf` — so running from `perf/`
//! writes to the same committed tree as running from the repo root.
//!
//! Exit code: non-zero if any argument is invalid, any file operation fails,
//! or any measured sample is not `Status::Ok`. The results file is still
//! written before a sample failure is reported, so a partially bad run is
//! inspectable.

use pkcore_perf::catalog::catalog;
use pkcore_perf::report::render;
use pkcore_perf::results::{Results, RunMeta};
use pkcore_perf::runner::{Sample, Status, default_trials, measure};
use pkcore_perf::sweep::sweep;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Default results directory, relative to the repo root (see [`anchored`]).
const DEFAULT_OUT: &str = "docs/perf/results";

/// Default report file, relative to the repo root (see [`anchored`]).
const DEFAULT_REPORT: &str = "docs/perf/RESULTS.md";

/// Default timestamp recorded when `--utc` is not given. Deliberately the
/// epoch, so an unstamped run is obviously unstamped rather than plausibly
/// recent.
const DEFAULT_UTC: &str = "1970-01-01T00:00:00Z";

const RUN_USAGE: &str =
    "usage: perf run [WORKLOAD] [--out DIR] [--utc STAMP] [--trials N] [--iters N] [--label NAME] [--stdout] [--sweep]";

const REPORT_USAGE: &str = "usage: perf report [--dir DIR] [--out FILE]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (command, rest) = split_args(&args);

    match command {
        "list" => {
            list();
            ExitCode::SUCCESS
        }
        "run" => run(rest),
        "report" => report(rest),
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: perf [list|run|report]");
            ExitCode::FAILURE
        }
    }
}

/// Splits parsed argv (already stripped of `argv[0]`) into the subcommand
/// name and its trailing arguments.
///
/// Bare invocation — an empty `args` — defaults `command` to `"run"` with an
/// empty tail. That default exists precisely so `perf` with no arguments
/// behaves like `perf run`; naively slicing `&args[1..]` to compute the tail
/// panics on a zero-length slice for exactly that case ("range start index 1
/// out of range for slice of length 0"), which is the one path the default
/// was supposed to support. `args.get(1..).unwrap_or(&[])` returns an empty
/// slice instead.
fn split_args(args: &[String]) -> (&str, &[String]) {
    let command = args.first().map_or("run", String::as_str);
    let rest = args.get(1..).unwrap_or(&[]);
    (command, rest)
}

fn list() {
    for workload in catalog() {
        let features = if workload.features.is_empty() {
            "pure-kernel".to_string()
        } else {
            workload.features.join(",")
        };
        println!(
            "{:<32} {:?}  iters={:<8} {features}",
            workload.name, workload.band, workload.inner_iters
        );
    }
}

/// Everything `perf run` accepts, fully validated.
#[derive(Debug, PartialEq, Eq)]
struct RunArgs {
    /// Workload name to run; `None` runs the whole catalog. Positional, and
    /// accepted anywhere among the flags — an earlier version only recognised
    /// it as the first token, so `perf run --trials 5 NAME` silently ran (and
    /// overwrote the results file for) the entire catalog.
    filter: Option<String>,
    /// `--out`; `None` means the anchored [`DEFAULT_OUT`].
    out_dir: Option<String>,
    utc: String,
    label: Option<String>,
    trials: Option<u32>,
    iters: Option<u32>,
    to_stdout: bool,
    do_sweep: bool,
}

/// Everything `perf report` accepts, fully validated.
#[derive(Debug, PartialEq, Eq)]
struct ReportArgs {
    /// `--dir`; `None` means the anchored [`DEFAULT_OUT`].
    dir: Option<String>,
    /// `--out`; `None` means the anchored [`DEFAULT_REPORT`].
    out: Option<String>,
}

/// Parses `perf run`'s arguments strictly: unknown options, missing values,
/// unparseable or zero counts, and duplicate workload names are all errors.
/// Silence was worse — a typo like `--trials 5o` used to fall back to the
/// band default without a word, and a misplaced workload name ran the whole
/// catalog.
fn parse_run_args(args: &[String]) -> Result<RunArgs, String> {
    let mut parsed = RunArgs {
        filter: None,
        out_dir: None,
        utc: DEFAULT_UTC.to_string(),
        label: None,
        trials: None,
        iters: None,
        to_stdout: false,
        do_sweep: false,
    };

    let mut tokens = args.iter();
    while let Some(token) = tokens.next() {
        match token.as_str() {
            "--out" => parsed.out_dir = Some(value_for("--out", tokens.next())?),
            "--utc" => parsed.utc = value_for("--utc", tokens.next())?,
            "--label" => parsed.label = Some(value_for("--label", tokens.next())?),
            "--trials" => parsed.trials = Some(count_for("--trials", tokens.next())?),
            "--iters" => parsed.iters = Some(count_for("--iters", tokens.next())?),
            "--stdout" => parsed.to_stdout = true,
            "--sweep" => parsed.do_sweep = true,
            flag if flag.starts_with("--") => return Err(format!("unknown option: {flag}")),
            name => {
                if let Some(existing) = &parsed.filter {
                    return Err(format!("two workload names given: {existing:?} and {name:?}"));
                }
                parsed.filter = Some(name.to_string());
            }
        }
    }

    Ok(parsed)
}

/// Parses `perf report`'s arguments with the same strictness as
/// [`parse_run_args`].
fn parse_report_args(args: &[String]) -> Result<ReportArgs, String> {
    let mut parsed = ReportArgs { dir: None, out: None };

    let mut tokens = args.iter();
    while let Some(token) = tokens.next() {
        match token.as_str() {
            "--dir" => parsed.dir = Some(value_for("--dir", tokens.next())?),
            "--out" => parsed.out = Some(value_for("--out", tokens.next())?),
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    Ok(parsed)
}

/// The value following a flag, or an error naming the flag that lacked one.
fn value_for(flag: &str, value: Option<&String>) -> Result<String, String> {
    value.cloned().ok_or_else(|| format!("{flag} needs a value"))
}

/// A count following a flag: a whole number of at least 1. Zero is rejected
/// because a zero-iteration run produces a checksum of 0 and no timings while
/// still reporting `Status::Ok`.
fn count_for(flag: &str, value: Option<&String>) -> Result<u32, String> {
    let raw = value_for(flag, value)?;
    let count: u32 = raw
        .parse()
        .map_err(|_| format!("{flag} needs a whole number, got {raw:?}"))?;
    if count == 0 {
        return Err(format!("{flag} must be at least 1"));
    }
    Ok(count)
}

/// The nearest ancestor of `start` (inclusive) containing `docs/perf` — the
/// marker this harness treats as the repo root.
fn find_repo_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join("docs").join("perf").is_dir())
        .map(Path::to_path_buf)
}

/// Anchors a default output path at the repo root, so `cargo run` from
/// inside `perf/` (as the repo docs suggest) and `make perf-native` from the
/// repo root write to the same committed tree. Falls back to the path as
/// given — cwd-relative, the old behaviour — when no `docs/perf` ancestor
/// exists, e.g. under a bare checkout of the perf crate alone.
fn anchored(default_tail: &str) -> String {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| find_repo_root(&cwd))
        .map_or_else(
            || default_tail.to_string(),
            |root| root.join(default_tail).to_string_lossy().into_owned(),
        )
}

/// The pkcore cargo features actually compiled into this binary.
///
/// This is what makes `RunMeta.features` — and so `docs/perf/RESULTS.md`'s
/// "Features:" line — trustworthy: it reflects `cfg!` at build time rather
/// than a value the caller has to remember to pass in. A hardcoded `vec![]`
/// here previously reported every run as `pure-kernel`, including runs of a
/// binary built with `--features "equity sim"`.
fn active_features() -> Vec<String> {
    let mut features = Vec::new();
    if cfg!(feature = "equity") {
        features.push("equity".to_string());
    }
    if cfg!(feature = "sim") {
        features.push("sim".to_string());
    }
    features
}

/// The names of every sample that is not [`Status::Ok`], in catalog order.
fn failed_samples(samples: &[Sample]) -> Vec<&str> {
    samples
        .iter()
        .filter(|s| s.status != Status::Ok)
        .map(|s| s.name.as_str())
        .collect()
}

/// Reports failed samples on stderr and converts them into the process exit
/// code, so `make perf-native` actually fails when a workload errors or goes
/// nondeterministic instead of leaving the regression for a human reading
/// the results table.
fn exit_code_for(samples: &[Sample]) -> ExitCode {
    let failed = failed_samples(samples);
    if failed.is_empty() {
        return ExitCode::SUCCESS;
    }
    eprintln!(
        "{} workload(s) did not measure cleanly: {}",
        failed.len(),
        failed.join(", ")
    );
    ExitCode::FAILURE
}

fn run(args: &[String]) -> ExitCode {
    let parsed = match parse_run_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{RUN_USAGE}");
            return ExitCode::FAILURE;
        }
    };

    let selected: Vec<_> = catalog()
        .into_iter()
        .filter(|w| parsed.filter.as_ref().is_none_or(|f| w.name == f.as_str()))
        .collect();

    if selected.is_empty() {
        eprintln!("no workload matched; try `perf list`");
        return ExitCode::FAILURE;
    }

    let mut samples: Vec<Sample> = Vec::with_capacity(selected.len());
    for workload in &selected {
        let (warmup, default_count) = default_trials(workload.band);
        let trials = parsed.trials.unwrap_or(default_count);
        let iters = parsed.iters.unwrap_or(workload.inner_iters);

        // Only parallel workloads sweep: a serial workload measures the same
        // thing at every pool size, so sweeping it triples the run time for
        // three identical rows.
        if parsed.do_sweep && workload.parallel {
            eprintln!("sweeping {} ({trials} trials x {iters})", workload.name);
            samples.extend(sweep(workload, warmup, trials, iters));
        } else {
            if parsed.do_sweep {
                eprintln!("not sweeping {} (serial workload)", workload.name);
            }
            eprintln!("measuring {} ({trials} trials x {iters})", workload.name);
            samples.push(measure(workload, warmup, trials, iters));
        }
    }

    let mut run = RunMeta::capture("native", active_features(), None, parsed.utc.clone());
    run.label = parsed.label.clone();

    let results = Results {
        schema: Results::SCHEMA,
        run,
        samples,
    };

    let json = match serde_json::to_string_pretty(&results) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("could not serialize results: {e}");
            return ExitCode::FAILURE;
        }
    };

    if parsed.to_stdout {
        println!("{json}");
        return exit_code_for(&results.samples);
    }

    let out_dir = parsed.out_dir.clone().unwrap_or_else(|| anchored(DEFAULT_OUT));
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("could not create {out_dir}: {e}");
        return ExitCode::FAILURE;
    }
    let path = format!("{out_dir}/{}", results.filename());
    if let Err(e) = std::fs::write(&path, &json) {
        eprintln!("could not write {path}: {e}");
        return ExitCode::FAILURE;
    }

    eprintln!("wrote {path}");
    summarize(&results);
    exit_code_for(&results.samples)
}

fn report(args: &[String]) -> ExitCode {
    let parsed = match parse_report_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{REPORT_USAGE}");
            return ExitCode::FAILURE;
        }
    };

    let dir = parsed.dir.unwrap_or_else(|| anchored(DEFAULT_OUT));
    let out = parsed.out.unwrap_or_else(|| anchored(DEFAULT_REPORT));

    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("could not read {dir}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut paths: Vec<std::path::PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();

    let mut runs = Vec::with_capacity(paths.len());
    for path in &paths {
        let raw = match std::fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(e) => {
                eprintln!("skipping {}: {e}", path.display());
                continue;
            }
        };
        match serde_json::from_str(&raw) {
            Ok(parsed) => runs.push(parsed),
            Err(e) => eprintln!("skipping {}: {e}", path.display()),
        }
    }

    if let Err(e) = std::fs::write(&out, render(&runs)) {
        eprintln!("could not write {out}: {e}");
        return ExitCode::FAILURE;
    }

    eprintln!("wrote {out} from {} run(s)", runs.len());
    ExitCode::SUCCESS
}

fn summarize(results: &Results) {
    for sample in &results.samples {
        match sample.ns_per_op {
            Some(stats) => println!(
                "{:<32} {:>4} {:>12.2} ns/op (min {:.2}, p95 {:.2}, MAD {:.2})",
                sample.name,
                sample
                    .rayon_threads
                    .map_or_else(|| "-".to_string(), |t| format!("{t}t")),
                stats.median,
                stats.min,
                stats.p95,
                stats.mad
            ),
            None => println!("{:<32} {:>4} {:?}", sample.name, "-", sample.status),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod perf__bin_tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    /// Regression test for the bare-invocation panic: `perf` with zero
    /// arguments used to crash with "range start index 1 out of range for
    /// slice of length 0" because `main` sliced `&args[1..]` on an empty
    /// `Vec`. Bare invocation must default to `run` with no trailing
    /// arguments, not panic.
    #[test]
    fn split_args_handles_bare_invocation_without_panicking() {
        let args: Vec<String> = Vec::new();
        let (command, rest) = split_args(&args);
        assert_eq!(command, "run");
        assert!(rest.is_empty(), "expected no trailing args, got {rest:?}");
    }

    #[test]
    fn split_args_slices_off_an_explicit_subcommand() {
        let args = args(&["report", "--dir", "x"]);
        let (command, rest) = split_args(&args);
        assert_eq!(command, "report");
        assert_eq!(rest, ["--dir".to_string(), "x".to_string()]);
    }

    #[test]
    fn split_args_handles_a_bare_subcommand_with_no_trailing_args() {
        let args = args(&["list"]);
        let (command, rest) = split_args(&args);
        assert_eq!(command, "list");
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_run_args_defaults_when_given_nothing() {
        let parsed = parse_run_args(&[]).expect("parses");
        assert_eq!(parsed.filter, None);
        assert_eq!(parsed.out_dir, None);
        assert_eq!(parsed.utc, DEFAULT_UTC);
        assert_eq!(parsed.label, None);
        assert_eq!(parsed.trials, None);
        assert_eq!(parsed.iters, None);
        assert!(!parsed.to_stdout);
        assert!(!parsed.do_sweep);
    }

    /// The regression this guards: the workload name was only recognised as
    /// the first token, so `perf run --trials 5 NAME` silently dropped the
    /// filter and ran — and overwrote the results file for — the whole
    /// catalog.
    #[test]
    fn parse_run_args_accepts_a_workload_name_after_flags() {
        let parsed = parse_run_args(&args(&["--trials", "5", "gto.cfr.iters"])).expect("parses");
        assert_eq!(parsed.filter, Some("gto.cfr.iters".to_string()));
        assert_eq!(parsed.trials, Some(5));
    }

    #[test]
    fn parse_run_args_accepts_a_workload_name_before_flags() {
        let parsed = parse_run_args(&args(&["gto.cfr.iters", "--stdout"])).expect("parses");
        assert_eq!(parsed.filter, Some("gto.cfr.iters".to_string()));
        assert!(parsed.to_stdout);
    }

    #[test]
    fn parse_run_args_rejects_two_workload_names() {
        let err = parse_run_args(&args(&["a.b.c", "d.e.f"])).expect_err("rejects");
        assert!(err.contains("two workload names"), "{err}");
    }

    #[test]
    fn parse_run_args_rejects_an_unknown_option() {
        let err = parse_run_args(&args(&["--tirals", "5"])).expect_err("rejects");
        assert!(err.contains("unknown option: --tirals"), "{err}");
    }

    /// `--trials 5o` used to silently fall back to the band default while
    /// the operator believed their count was honored.
    #[test]
    fn parse_run_args_rejects_an_unparseable_count() {
        let err = parse_run_args(&args(&["--trials", "5o"])).expect_err("rejects");
        assert!(err.contains("--trials needs a whole number"), "{err}");
    }

    /// `--iters 0` used to run every hot loop zero times and publish a
    /// legitimate-looking Ok sample with checksum 0 and no timings.
    #[test]
    fn parse_run_args_rejects_zero_counts() {
        let err = parse_run_args(&args(&["--iters", "0"])).expect_err("rejects");
        assert!(err.contains("--iters must be at least 1"), "{err}");
        let err = parse_run_args(&args(&["--trials", "0"])).expect_err("rejects");
        assert!(err.contains("--trials must be at least 1"), "{err}");
    }

    #[test]
    fn parse_run_args_rejects_a_flag_without_its_value() {
        let err = parse_run_args(&args(&["--label"])).expect_err("rejects");
        assert!(err.contains("--label needs a value"), "{err}");
    }

    #[test]
    fn parse_run_args_accepts_every_flag_together() {
        let parsed = parse_run_args(&args(&[
            "--out",
            "outdir",
            "--utc",
            "2026-08-14T00:00:00Z",
            "--label",
            "x",
            "--trials",
            "7",
            "--iters",
            "9",
            "--stdout",
            "--sweep",
            "eval.five.or_rank_bits",
        ]))
        .expect("parses");
        assert_eq!(parsed.out_dir, Some("outdir".to_string()));
        assert_eq!(parsed.utc, "2026-08-14T00:00:00Z");
        assert_eq!(parsed.label, Some("x".to_string()));
        assert_eq!(parsed.trials, Some(7));
        assert_eq!(parsed.iters, Some(9));
        assert!(parsed.to_stdout);
        assert!(parsed.do_sweep);
        assert_eq!(parsed.filter, Some("eval.five.or_rank_bits".to_string()));
    }

    #[test]
    fn parse_report_args_accepts_dir_and_out() {
        let parsed = parse_report_args(&args(&["--dir", "d", "--out", "o"])).expect("parses");
        assert_eq!(parsed.dir, Some("d".to_string()));
        assert_eq!(parsed.out, Some("o".to_string()));
    }

    #[test]
    fn parse_report_args_rejects_anything_else() {
        let err = parse_report_args(&args(&["--sweep"])).expect_err("rejects");
        assert!(err.contains("unknown argument: --sweep"), "{err}");
    }

    /// The repo root is the nearest ancestor containing `docs/perf` — found
    /// from the repo root itself, and from inside `perf/`, which is exactly
    /// the working directory the repo docs tell developers to use.
    #[test]
    fn find_repo_root_walks_up_to_the_docs_perf_marker() {
        let base = std::env::temp_dir().join(format!("perf_root_test_{}", std::process::id()));
        let marker = base.join("docs").join("perf");
        let nested = base.join("perf").join("deeper");
        std::fs::create_dir_all(&marker).expect("creates marker");
        std::fs::create_dir_all(&nested).expect("creates nested");

        assert_eq!(find_repo_root(&base), Some(base.clone()));
        assert_eq!(find_repo_root(&nested), Some(base.clone()));

        std::fs::remove_dir_all(&base).expect("cleans up");
    }

    #[test]
    fn find_repo_root_returns_none_without_a_marker() {
        let base = std::env::temp_dir().join(format!("perf_rootless_test_{}", std::process::id()));
        std::fs::create_dir_all(&base).expect("creates base");

        assert_eq!(find_repo_root(&base), None);

        std::fs::remove_dir_all(&base).expect("cleans up");
    }

    /// A run with an Error or Nondeterministic sample must not exit 0 —
    /// otherwise `make perf-native` succeeds and the regression is only
    /// caught if a human reads the results table.
    #[test]
    fn failed_samples_names_everything_that_is_not_ok() {
        use pkcore_perf::runner::measure;
        use pkcore_perf::workload::{Band, PerfError, Workload, counting_workload};

        let ok = measure(&counting_workload(), 0, 2, 10);
        let failing = Workload {
            name: "test.failing",
            band: Band::Nano,
            inner_iters: 10,
            features: &[],
            parallel: false,
            make: || Err(PerfError::Setup("no cards".to_string())),
        };
        let error = measure(&failing, 0, 2, 10);

        assert_eq!(failed_samples(std::slice::from_ref(&ok)), Vec::<&str>::new());
        assert_eq!(failed_samples(&[ok, error]), vec!["test.failing"]);
    }

    /// The regression this replaces: `RunMeta.features` hardcoded to
    /// `vec![]` regardless of which pkcore cargo features the binary was
    /// actually built with, so `docs/perf/RESULTS.md` reported every run —
    /// including ones built with `--features "equity sim"` — as
    /// `pure-kernel`. This assertion is only as strong as the feature set
    /// `cargo test` compiles this binary with, but it fails immediately
    /// under `cargo test --features "equity sim"` (part of `make
    /// perf-check`) if the hardcoded-empty-vec bug ever comes back.
    #[test]
    fn active_features_reflects_the_compiled_in_cargo_features() {
        let features = active_features();
        assert_eq!(features.contains(&"equity".to_string()), cfg!(feature = "equity"));
        assert_eq!(features.contains(&"sim".to_string()), cfg!(feature = "sim"));
    }
}
