use pkcore::analysis::store::nubibus::pluribus::Pluribus;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;

/// This code's origin is from
/// [Rust by Example](https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html).
fn main() {
    // File hosts.txt must exist in the current path
    if let Ok(lines) = read_lines("data/pluribus/raw/sample_game_30.log") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(ip) = line {
                match Pluribus::from_str(ip.as_str()) {
                    Ok(pl) => println!("{}", pl),
                    Err(_) => {}
                }
            }
        }
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
