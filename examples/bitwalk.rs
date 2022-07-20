fn main() {
    let mut i: u16 = 0b0000_0000_0000_0101;

    for _ in 0..8 {
        let is_set = i & 1 == 1;
        println!("{:b} {}", i, is_set);
        i = i >> 1;
    }
}
