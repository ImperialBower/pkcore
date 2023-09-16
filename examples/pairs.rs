use pkcore::arrays::two::Two;

fn main() {
    for two in Two::pairs() {
        println!("{two}");
    }
}
