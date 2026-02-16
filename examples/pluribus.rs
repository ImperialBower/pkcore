use pkcore::PKError;
use pkcore::analysis::nubibus::Pluribus;
use std::fs;
use std::path::Path;

/// `cargo run --example pluribus`
fn main() -> Result<(), PKError> {
    let logs = get_log_files()?;

    let mut game_num = 0;
    for log in logs.iter() {
        for plur in Pluribus::read_in_log(log.as_str())? {
            println!();
            println!("------------------------------------------------------------------------------");
            println!("Game #{game_num}");
            println!("------------------------------------------------------------------------------");
            plur.play_hand()?;
            game_num += 1;
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
