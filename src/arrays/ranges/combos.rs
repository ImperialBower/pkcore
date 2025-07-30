use std::str::FromStr;
use crate::arrays::ranges::combo::Combo;
use crate::PKError;
use crate::util::Util;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Combos(Vec<Combo>);

impl Combos {

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
            .map(|c| c.parse::<Combo>())
            .collect::<Result<Vec<Combo>, PKError>>()?;

        Ok(Combos::from(combos))
    }
}