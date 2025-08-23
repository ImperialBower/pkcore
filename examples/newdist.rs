use pkcore::PKError;
use pkcore::arrays::matchups::sorted_heads_up::SORTED_HEADS_UP_UNIQUE_TYPE_EIGHT;

fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();

    let hups8 = SORTED_HEADS_UP_UNIQUE_TYPE_EIGHT.clone();

    println!("Elapsed: {:.2?}", now.elapsed());
    Ok(())
}
