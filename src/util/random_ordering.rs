extern crate core;
use rand::prelude::*;
use std::cmp::Ordering;

/// Randomly samples one of the three [`Ordering`] variants.
///
/// This is used for tie-breaking paths that want a small, explicit 3-way
/// distribution instead of a coin flip.
///
/// # Examples
///
/// ```
/// use rand::distr::{Distribution, StandardUniform};
/// use rand::{rngs::StdRng, SeedableRng};
/// use pkcore::util::random_ordering::RandomOrdering;
///
/// let mut rng = StdRng::seed_from_u64(7);
/// let sample: RandomOrdering = StandardUniform.sample(&mut rng);
/// let ordering: std::cmp::Ordering = sample.into();
/// let _ = ordering;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RandomOrdering(Ordering);

#[allow(clippy::from_over_into)]
impl Into<Ordering> for RandomOrdering {
    fn into(self) -> Ordering {
        self.0
    }
}

impl Distribution<RandomOrdering> for rand::distr::StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RandomOrdering {
        RandomOrdering(match rng.random_range(0..3) {
            0 => Ordering::Less,
            1 => Ordering::Equal,
            _ => Ordering::Greater,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::distr::{Distribution, StandardUniform};
    use rand::rngs::StdRng;

    #[test]
    fn sample_covers_all_orderings() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut saw_less = false;
        let mut saw_equal = false;
        let mut saw_greater = false;

        for _ in 0..256 {
            let ordering: RandomOrdering = StandardUniform.sample(&mut rng);
            match ordering.into() {
                Ordering::Less => saw_less = true,
                Ordering::Equal => saw_equal = true,
                Ordering::Greater => saw_greater = true,
            }
        }

        assert!(saw_less && saw_equal && saw_greater);
    }

    #[test]
    fn into_returns_inner_ordering() {
        assert_eq!(Ordering::Less, RandomOrdering(Ordering::Less).into());
    }
}
