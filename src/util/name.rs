use rnglib::{Language, RNG};

pub struct Name;

/// The Demonic-language name generator, or `None` if `rnglib` could not build
/// it. [`Name::generate`] falls back to [`Name::FALLBACK`] in that case rather
/// than panicking on first use.
pub static NAMER: std::sync::LazyLock<Option<RNG>> = std::sync::LazyLock::new(|| RNG::new(&Language::Demonic).ok());

impl Name {
    /// The handle used when the name generator is unavailable.
    pub const FALLBACK: &'static str = "Nameless Demon";

    #[must_use]
    pub fn generate() -> String {
        Self::generate_with(NAMER.as_ref())
    }

    /// Two generated words from `namer`, or [`Name::FALLBACK`] without one.
    #[must_use]
    pub fn generate_with(namer: Option<&RNG>) -> String {
        match namer {
            Some(rng) => format!("{} {}", rng.generate_name(), rng.generate_name()),
            None => Self::FALLBACK.to_string(),
        }
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

    #[test]
    fn generate_with__falls_back_when_the_namer_is_unavailable() {
        assert_eq!(Name::FALLBACK, Name::generate_with(None));
    }

    #[test]
    fn generate_with__uses_the_namer_when_available() {
        let name = Name::generate_with(NAMER.as_ref());
        assert_ne!(Name::FALLBACK, name);
        assert_eq!(2, name.split(' ').count());
    }
}
