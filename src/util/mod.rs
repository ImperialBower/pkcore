pub mod data;
pub mod random_ordering;
pub mod terminal;
pub mod wincounter;
pub mod csv;

/// Blank struct that is home to misfit utility functions.
///
/// There is a whole school that argues against util functions and modules like
/// this. Obviously, I am not one of them.
pub struct Util;

impl Util {
    /// Forgiving percentage calculator. It will return zero if you try
    /// to divide by zero.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate_percentage(number: usize, total: usize) -> f32 {
        match total {
            0 => 0_f32,
            _ => (number as f32 * 100.0) / total as f32,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__tests {
    use super::*;

    #[test]
    fn percent() {
        let percentage = Util::calculate_percentage(48, 2_598_960);

        assert_eq!("0.00185%", format!("{:.5}%", percentage));
        assert_eq!(
            "0.00000%",
            format!("{:.5}%", Util::calculate_percentage(0, 0))
        );
    }

    #[test]
    fn percent__zero_numerator() {
        let percentage = Util::calculate_percentage(0, 2_598_960);

        assert_eq!("0.00000%", format!("{:.5}%", percentage));
    }

    #[test]
    fn percent__zero_denominator() {
        let percentage = Util::calculate_percentage(48, 0);

        assert_eq!("0.00000%", format!("{:.5}%", percentage));
    }
}
