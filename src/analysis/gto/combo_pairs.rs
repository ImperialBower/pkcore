use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::twos::Twos;
use crate::arrays::two::Two;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComboPairs(HashMap<Combo, Twos>);

impl ComboPairs {
    pub fn add(&mut self, combo: Combo, two: Two) {
        let twos = self.0.entry(combo).or_default();
        twos.insert(two);
    }

    #[must_use]
    pub fn combo(&self, combo: &Combo) -> Option<&Twos> {
        self.0.get(combo)
    }

    pub fn insert(&mut self, combo: Combo, twos: Twos) {
        self.0.insert(combo, twos);
    }

    #[must_use]
    pub fn hash_map(&self) -> &HashMap<Combo, Twos> {
        &self.0
    }
}

impl Display for ComboPairs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (combo, twos) in &self.0 {
            writeln!(f, "{combo}: {twos}")?;
        }
        Ok(())
    }
}

impl From<HashMap<Combo, Twos>> for ComboPairs {
    fn from(hash_map: HashMap<Combo, Twos>) -> Self {
        ComboPairs(hash_map)
    }
}
