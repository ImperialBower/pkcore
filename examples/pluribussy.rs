use pkcore::PKError;
use pkcore::analysis::nubibus::Pluribus;
use pkcore::prelude::Nubificus;

/// `cargo run --example pluribus`
fn main() -> Result<(), PKError> {
    let logs = Nubificus::get_log_files("data/pluribus/raw/")?;

    let mut game_num = 0;
    for log in logs.iter() {
        for plur in Pluribus::read_in_log(log.as_str())? {
            println!();
            println!("------------------------------------------------------------------------------");
            println!("Game #{game_num}");
            println!("------------------------------------------------------------------------------");
            Nubificus::try_from(&plur)?.play_hand()?;
            game_num += 1;
        }
    }

    Ok(())
}
