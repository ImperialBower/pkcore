use rnglib::{Language, RNG};

pub struct Name;

/// Random name generator using Demonic language.
///
/// # Panics
///
/// Panics at first use if `RNG::new(&Language::Demonic)` fails (i.e., the
/// underlying entropy source is unavailable at startup).
#[allow(clippy::unwrap_used)]
pub static NAMER: std::sync::LazyLock<RNG> = std::sync::LazyLock::new(|| RNG::new(&Language::Demonic).unwrap());

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
