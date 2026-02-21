use std::str::FromStr;
use pkcore::PKError;
use pkcore::prelude::{Nubificus, Terminal};

/// `cargo run --example pluripop`
fn main() -> Result<(), PKError> {
    let plur = "STATE:193:r225fcffc/ccc/ccc/ccc:2s7d|7c9c|KcQs|2dQh|9h9s|Ac8d/5cKhJh/As/7s:-50|-225|500|0|-225|0:Eddie|MrOrange|Bill|MrBlue|Pluribus|MrPink";
    let mut nubi = Nubificus::from_str(plur)?;

    println!("{}", nubi.pluribus);

    loop {
        Terminal::pause("boop> ")?;
        println!();
        nubi.boop()?;
        println!("{}", nubi.table.get_game_state());

        if nubi.table.is_game_over() {
            println!("Game over!");
            break;
        }
    }

    Ok(())
}

// STATE:29:fffr275fc/cc/cr725r1850f:Tc4h|5c6d|3cTs|9hKc|2c8h|Ks7s/7c3hJs/6h:-50|775|0|0|0|-725:Pluribus|MrBlue|MrBlonde|MrWhite|MrPink|MrBrown
// Eli|Antonio|Gus|Daniel|Cory|Barry|Amnon|Doyle
