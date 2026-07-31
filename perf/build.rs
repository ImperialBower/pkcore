//! Captures build-time facts the runner records in its results file.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=PERF_TARGET={target}");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let version = Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string());
    println!("cargo:rustc-env=PERF_RUSTC={version}");
}
