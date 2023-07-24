use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;

fn main() {
    let now = std::time::Instant::now();
    env_logger::init();

    SortedHeadsUp::generate_csv("generated/all_possible_shu.csv").expect("TODO: panic message");

    println!("Elapsed: {:.2?}", now.elapsed());
}
