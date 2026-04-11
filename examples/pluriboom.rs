use pkcore::PKError;
use pkcore::analysis::nubibus::Pluribus;
use pkcore::prelude::Nubificus;
use rayon::prelude::*;

/// `cargo run --example pluriboom [concurrency]`
///
/// Runs all Pluribus game logs simultaneously across N threads.
/// Defaults to the number of logical CPUs when no argument is given.
///
/// # Examples
///
/// ```bash
/// cargo run --example pluriboom        # use all CPUs
/// cargo run --example pluriboom 1      # sequential baseline
/// cargo run --example pluriboom 8      # 8 threads
/// ```
fn main() -> Result<(), PKError> {
    let concurrency: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4));

    let logs = Nubificus::get_log_files("data/pluribus/raw/")?;

    let mut all_games: Vec<Pluribus> = Vec::new();
    for log in logs.iter() {
        for plur in Pluribus::read_in_log(log.as_str())? {
            all_games.push(plur);
        }
    }
    let total = all_games.len();

    println!("Running {total} games on {concurrency} threads...");

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build()
        .expect("failed to build thread pool");

    let start = std::time::Instant::now();

    let error_count = pool.install(|| {
        all_games
            .into_par_iter()
            .enumerate()
            .filter(
                |(idx, plur)| match Nubificus::try_from(plur).and_then(|n| n.play_hand()) {
                    Ok(_) => false,
                    Err(e) => {
                        eprintln!("Game #{idx} failed: {e}");
                        true
                    }
                },
            )
            .count()
    });

    let elapsed = start.elapsed();
    println!("Completed {total} games in {elapsed:.2?} — {error_count} errors");

    Ok(())
}
