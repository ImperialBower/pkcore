use pkcore::PKError;
use pkcore::analysis::store::nubibus::pluribus::Pluribus;
use std::fs;
use std::path::Path;

/// `cargo run --example pluribus`
fn main() -> Result<(), PKError> {
    // let logs = vec![
    //     "data/pluribus/raw/sample_game_30.log",
    //     "data/pluribus/raw/sample_game_31.log",
    //     "data/pluribus/raw/sample_game_32.log",
    //     "data/pluribus/raw/sample_game_33.log",
    //     "data/pluribus/raw/sample_game_34.log",
    //     "data/pluribus/raw/sample_game_35.log",
    //     "data/pluribus/raw/sample_game_40.log",
    //     "data/pluribus/raw/sample_game_40b.log",
    //     "data/pluribus/raw/sample_game_41.log",
    //     "data/pluribus/raw/sample_game_41b.log",
    //     "data/pluribus/raw/sample_game_42.log",
    //     "data/pluribus/raw/sample_game_42b.log",
    // ];

    let logs = get_log_files()?;

    for log in logs {
        for plur in Pluribus::read_in_log(log.as_str())? {
            plur.play_hand()?;
        }
    }

    Ok(())
}

fn get_log_files() -> Result<Vec<String>, PKError> {
    let dir = Path::new("data/pluribus/raw/");
    let mut log_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "log" {
                    if let Some(path_str) = path.to_str() {
                        log_files.push(path_str.to_string());
                    }
                }
            }
        }
    }

    log_files.sort();
    Ok(log_files)
}
