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
//!   --trials N       override the per-band trial count
//!   --iters N        override the per-workload inner iteration count
//!   --stdout         print JSON instead of writing a file
//!   --label NAME     tag the results file, so same-day runs do not collide
//!   --sweep          measure each workload at 1, 4 and 8 rayon threads

use pkcore_perf::catalog::catalog;
use pkcore_perf::report::render;
use pkcore_perf::results::{Results, RunMeta};
use pkcore_perf::runner::{Sample, default_trials, measure};
use pkcore_perf::sweep::sweep;
use std::process::ExitCode;

const DEFAULT_OUT: &str = "docs/perf/results";

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

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let index = args.iter().position(|a| a == name)?;
    args.get(index + 1).map(String::as_str)
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

fn run(args: &[String]) -> ExitCode {
    let filter = args.first().filter(|a| !a.starts_with("--")).cloned();
    let out_dir = flag(args, "--out").unwrap_or(DEFAULT_OUT).to_string();
    let utc = flag(args, "--utc").unwrap_or("1970-01-01T00:00:00Z").to_string();
    let label = flag(args, "--label").map(str::to_string);
    let trials_override: Option<u32> = flag(args, "--trials").and_then(|v| v.parse().ok());
    let iters_override: Option<u32> = flag(args, "--iters").and_then(|v| v.parse().ok());
    let to_stdout = args.iter().any(|a| a == "--stdout");
    let do_sweep = args.iter().any(|a| a == "--sweep");

    let selected: Vec<_> = catalog()
        .into_iter()
        .filter(|w| filter.as_ref().is_none_or(|f| w.name == f.as_str()))
        .collect();

    if selected.is_empty() {
        eprintln!("no workload matched; try `perf list`");
        return ExitCode::FAILURE;
    }

    let mut samples: Vec<Sample> = Vec::with_capacity(selected.len());
    for workload in &selected {
        let (warmup, default_count) = default_trials(workload.band);
        let trials = trials_override.unwrap_or(default_count);
        let iters = iters_override.unwrap_or(workload.inner_iters);

        if do_sweep {
            eprintln!("sweeping {} ({trials} trials x {iters})", workload.name);
            samples.extend(sweep(workload, warmup, trials, iters));
        } else {
            eprintln!("measuring {} ({trials} trials x {iters})", workload.name);
            samples.push(measure(workload, warmup, trials, iters));
        }
    }

    let mut run = RunMeta::capture("native", active_features(), None, utc);
    run.label = label;

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

    if to_stdout {
        println!("{json}");
        return ExitCode::SUCCESS;
    }

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
    ExitCode::SUCCESS
}

fn report(args: &[String]) -> ExitCode {
    let dir = flag(args, "--dir").unwrap_or(DEFAULT_OUT).to_string();
    let out = flag(args, "--out").unwrap_or("docs/perf/RESULTS.md").to_string();

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
        let args = vec!["report".to_string(), "--dir".to_string(), "x".to_string()];
        let (command, rest) = split_args(&args);
        assert_eq!(command, "report");
        assert_eq!(rest, ["--dir".to_string(), "x".to_string()]);
    }

    #[test]
    fn split_args_handles_a_bare_subcommand_with_no_trailing_args() {
        let args = vec!["list".to_string()];
        let (command, rest) = split_args(&args);
        assert_eq!(command, "list");
        assert!(rest.is_empty());
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
