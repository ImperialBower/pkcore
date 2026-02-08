use pkcore::analysis::store::nubibus::pluribus::Pluribus;

fn main() {
    for game in Pluribus::read_in_log("data/pluribus/raw/sample_game_30.log").unwrap() {
        println!("{}", game);
    }
}
