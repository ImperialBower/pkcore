pub mod california;

pub struct Razz;

impl Razz {
    pub const WHEEL: u64 = 0b0_0000_0001_1111;
}

#[cfg(test)]
#[allow(non_snake_case)]
mod games__razz_tests {
    use super::*;
    use rstest::rstest;
}
