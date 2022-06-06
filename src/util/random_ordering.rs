extern crate core;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::cmp::Ordering;

/// From: <https://github.com/bluss/indexmap/issues/171#issuecomment-786429977>
/// TODO: Craft better randomizer
pub struct RandomOrdering(Ordering);

#[allow(clippy::from_over_into)]
impl Into<Ordering> for RandomOrdering {
    fn into(self) -> Ordering {
        self.0
    }
}

impl Distribution<RandomOrdering> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RandomOrdering {
        RandomOrdering(match rng.gen_range(0..2) {
            0 => Ordering::Less,
            1 => Ordering::Equal,
            _ => Ordering::Greater,
        })
    }
}
