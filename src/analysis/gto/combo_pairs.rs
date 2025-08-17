use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::twos::Twos;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComboPairs(HashMap<Combo, Twos>);

impl ComboPairs {
    #[must_use]
    pub fn count(&self) -> usize {
        self.0.len()
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
