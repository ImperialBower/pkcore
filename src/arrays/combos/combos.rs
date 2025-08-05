use crate::PKError;
use crate::arrays::combos::combo::Combo;
use crate::util::Util;
use std::str::FromStr;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Combos(Vec<Combo>);

impl Combos {
    fn combo_range(s: &str) -> Result<Combos, PKError> {
        let mut iter = s.split('-');
        if iter.clone().count() == 2 {
            let start = iter.next().ok_or(PKError::InvalidRangeIndex)?.parse::<Combo>()?;
            let end = iter.next().ok_or(PKError::InvalidRangeIndex)?.parse::<Combo>()?;
            Ok(Combos(vec![start, end]))
        } else {
            Err(PKError::InvalidRangeIndex)
        }
    }
}

impl From<Vec<Combo>> for Combos {
    fn from(combos: Vec<Combo>) -> Self {
        if combos.is_empty() {
            Combos::default()
        } else {
            Combos(combos)
        }
    }
}

impl FromStr for Combos {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let index = Util::str_remove_spaces(s);

        let combos = index
            .split(',')
            .map(str::parse::<Combo>)
            .collect::<Result<Vec<Combo>, PKError>>()?;

        Ok(Combos::from(combos))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__ranges__combos_tests {
    use super::*;

    /// `JJ-22,AQs-ATs,KJs+,QJs,JTs,T9s,98s,87s,76s,65s,54s,AQo-ATo,KJo+`
    #[test]
    fn combo_range() {
        let range = "AQs-ATs";
        let expected = Combos(vec![Combo::COMBO_AQs, Combo::COMBO_ATs]);

        let actual = Combos::combo_range(range).unwrap();

        assert_eq!(expected, actual);
        assert!(Combos::combo_range("AQs-ATs-AAs").is_err());
        assert!(Combos::combo_range("AQs").is_err());
    }
}
