use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::twos::Twos;
use crate::arrays::two::Two;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Write as _; // import without risk of name clashing

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComboPairs(HashMap<Combo, Twos>);

impl ComboPairs {
    pub fn add(&mut self, combo: Combo, two: Two) {
        // println!("Adding {two} to {combo}");
        let twos = self.0.entry(combo).or_default();
        twos.insert(two);
    }

    #[must_use]
    pub fn hash_map(&self) -> &HashMap<Combo, Twos> {
        &self.0
    }

    pub fn insert(&mut self, combo: Combo, twos: Twos) {
        self.0.insert(combo, twos);
    }

    #[must_use]
    pub fn key_vec(&self) -> Vec<Combo> {
        let mut v: Vec<Combo> = self.0.keys().copied().collect();
        v.sort();
        v.reverse();
        v
    }

    pub fn keys(&self) -> impl Iterator<Item = &Combo> {
        self.0.keys()
    }

    #[must_use]
    pub fn twos_for_combo(&self, combo: &Combo) -> Option<&Twos> {
        self.0.get(combo)
    }
}

impl Display for ComboPairs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for combo in self.key_vec() {
            match self.twos_for_combo(&combo) {
                // https://rust-lang.github.io/rust-clippy/master/index.html#format_push_string
                Some(twos) => {
                    let _ = write!(s, "{:>03}", combo.to_string());
                    let _ = write!(s, " {:>2} of {:>2}", twos.len(), combo.total_pairs());
                    let _ = writeln!(s, ": {twos}");
                }
                None => {
                    let _ = write!(s, "{:>03}:", combo.to_string());
                }
            }
        }
        write!(f, "{s}")
    }
}

impl From<HashMap<Combo, Twos>> for ComboPairs {
    fn from(hash_map: HashMap<Combo, Twos>) -> Self {
        ComboPairs(hash_map)
    }
}
