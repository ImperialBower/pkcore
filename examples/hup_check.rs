use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;

fn main() {
    let hups = HUPResult::read_csv("generated/hups.csv").unwrap();
    for hup in hups {
        println!("{hup}");
    }
}
