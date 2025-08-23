use std::borrow::Cow;
use std::str::Utf8Error;

pub mod csv;
pub mod data;
pub mod name;
pub mod random_ordering;
pub mod terminal;
pub mod wincounter;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Percentage {
    pub number: usize,
    pub total: usize,
}

impl Percentage {
    #[must_use]
    pub fn new(number: usize, total: usize) -> Self {
        Self { number, total }
    }

    #[must_use]
    pub fn calculate(&self) -> f32 {
        Util::calculate_percentage(self.number, self.total)
    }
}

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

    ///
    ///
    /// # Errors
    ///
    /// Returns `Utf8Error` if the `&str` is not valid UTF-8.
    pub fn percent_decode(s: &str) -> Result<String, Utf8Error> {
        Ok(percent_encoding::percent_decode_str(s).decode_utf8()?.to_string())
    }

    /// I need to study the ideas behind `Cow`.
    ///
    /// Usage:
    /// ```
    /// use pkcore::util::Util;
    /// assert_eq!(Util::replace_plus("A♠+J♦+6♥+6♣".into()), "A♠ J♦ 6♥ 6♣");
    /// ```
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn replace_plus(s: Cow<str>) -> String {
        s.replace('+', " ")
    }

    #[must_use]
    pub fn str_remove_spaces(s: &str) -> String {
        s.replace(' ', "")
    }

    #[must_use]
    pub fn str_splitter(s: &str, splitter: &str) -> Vec<String> {
        s.split(splitter).map(std::string::ToString::to_string).collect()
    }

    /// Code from [stackoverflow](https://stackoverflow.com/questions/57029974/how-to-split-string-into-chunks-in-rust-to-insert-spaces)
    #[must_use]
    pub fn str_len_splitter(s: &str, on: usize) -> String {
        s.chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i != 0 && i % on == 0 { Some(' ') } else { None }
                    .into_iter()
                    .chain(std::iter::once(c))
            })
            .collect::<String>()
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
        assert_eq!("0.00000%", format!("{:.5}%", Util::calculate_percentage(0, 0)));
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

    #[test]
    fn str_remove_spaces() {
        let index = "JJ-22, AQs -    ATs,KJs+";

        let expected = "JJ-22,AQs-ATs,KJs+".to_string();

        assert_eq!(Util::str_remove_spaces(index), expected);
    }
}
