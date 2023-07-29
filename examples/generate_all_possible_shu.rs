use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;

fn main() {
    let now = std::time::Instant::now();
    env_logger::init();

    let hs = SortedHeadsUp::unique().unwrap();
    println!("{}", hs.len());
    SortedHeadsUp::generate_csv("generated/all_possible_shu.csv", hs).expect("TODO: panic message");

    println!("Elapsed: {:.2?}", now.elapsed());
}
