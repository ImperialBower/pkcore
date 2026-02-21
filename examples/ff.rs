use pkcore::PKError;
use pkcore::analysis::nubibus::Nubificus;
use pkcore::play::stages::flop_eval::FlopEval;
use pkcore::prelude::Game;
use std::str::FromStr;

/// `cargo run --example ff`
fn main() -> Result<(), PKError> {
    // let plur = "STATE:193:r225fcffc/ccc/ccc/ccc:2s7d|7c9c|KcQs|2dQh|9h9s|Ac8d/5cKhJh/As/7s:-50|-225|500|0|-225|0:Eddie|MrOrange|Bill|MrBlue|Pluribus|MrPink";
    // let plur = "STATE:29:fffr275fc/cc/cr725r1850f:Tc4h|5c6d|3cTs|9hKc|2c8h|Ks7s/7c3hJs/6h:-50|775|0|0|0|-725:Pluribus|MrBlue|MrBlonde|MrWhite|MrPink|MrBrown";
    let plur = "STATE:193:r225fcffc/ccc/ccc/ccc:2s7d|7c9c|KcQs|2dQh|9h9s|Ac8d/5cKhJh/As/7s:-50|-225|500|0|-225|0:Eddie|MrOrange|Bill|MrBlue|Pluribus|MrPink";

    let mut nubi = Nubificus::from_str(plur)?;

    println!("{}", nubi.table);

    nubi.ff(6, true)?;

    nubi.table.eval_flop_display();

    let game = Game::try_from(nubi.table)?;
    let fe = FlopEval::try_from(game.clone())?;

    // println!("{game}");
    println!("{fe}");

    // fe

    Ok(())
}
