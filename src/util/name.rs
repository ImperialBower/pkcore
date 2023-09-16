use lazy_static::lazy_static;
use rnglib::{Language, RNG};

pub struct Name;

lazy_static! {
    pub static ref NAMER: RNG = RNG::new(&Language::Demonic).unwrap();
}

impl Name {
    #[must_use]
    pub fn generate() -> String {
        format!("{} {}", NAMER.generate_name(), NAMER.generate_name())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__name_tests {
    use super::*;

    #[test]
    fn generate() {
        assert!(!Name::generate().is_empty())
    }
}
