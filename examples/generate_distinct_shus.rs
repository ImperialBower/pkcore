use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;

/// `cargo run --example generate_distinct_shus`
fn main() {
    let now = std::time::Instant::now();
    env_logger::init();

    let hs = SortedHeadsUp::distinct().unwrap();
    println!("{}", hs.len());
    SortedHeadsUp::generate_csv("generated/distinct_shu3.csv", hs).expect("TODO: panic message");

    println!("Elapsed: {:.2?}", now.elapsed());
}
