use crate::analysis::case_eval::CaseEval;
use crate::analysis::case_evals::CaseEvals;
use crate::analysis::eval::Eval;
use crate::analysis::store::bcm::binary_card_map::BC_RANK_HASHMAP;
use crate::analysis::the_nuts::TheNuts;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::cards::Cards;
use crate::{PKError, Pile};
use rayon::iter::ParallelIterator;
use std::fmt::Formatter;
use std::str::FromStr;
use std::sync::mpsc;
use std::{fmt, thread};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Twos([Two; 9]);

impl Twos {
    /// This type of code scares me, because I can't simply test drive it. Perhaps
    /// I can? What about creating a smaller subset bcm file?
    /// TODO: do that
    ///
    /// # Errors
    ///
    /// If `BC_RANK_HASHMAP` is incapable of parsing the cards passed in.
    pub fn bcm_case_eval(&self, case: Five) -> Result<CaseEval, PKError> {
        let mut case_eval = CaseEval::default();

        for player in &self.vec() {
            if let Ok(seven) = Seven::from_case_at_deal(*player, case) {
                let bard = seven.bard();
                let bcm = BC_RANK_HASHMAP.get(&bard).ok_or(PKError::Incomplete)?;
                case_eval.push(Eval::try_from(*bcm)?);
            }
        }

        Ok(case_eval)
    }

    fn process_case(twos: Twos, v: Vec<Card>) -> Result<CaseEval, PKError> {
        let case = Five::try_from(v)?;
        let mut case_eval = CaseEval::default();

        for player in twos.vec() {
            if let Ok(seven) = Seven::from_case_at_deal(player, case) {
                let bard = seven.bard();
                let bcm = BC_RANK_HASHMAP.get(&bard).ok_or(PKError::Incomplete)?;
                case_eval.push(Eval::try_from(*bcm)?);
            }
        }

        Ok(case_eval)
    }

    /// # Errors
    ///
    /// `PKError` if unable to convert the five `Cards`.
    ///
    /// # Panics
    ///
    /// If unable to process case
    pub fn bcm_mpsc_case_evals(&self) -> Result<CaseEvals, PKError> {
        let mut case_evals = CaseEvals::default();
        let twos = *self;
        let (tx, rx) = mpsc::channel();

        for v in self.combinations_remaining(5) {
            let tx = tx.clone();
            thread::spawn(move || {
                tx.send(Twos::process_case(twos, v).unwrap())
                    .expect("TODO: panic message");
            });
        }

        drop(tx);

        for received in rx {
            case_evals.push(received);
        }

        Ok(case_evals)
    }

    /// # Errors
    ///
    /// `PKError` if unable to convert the five `Cards`.
    // pub fn bcm_case_evals(&self) -> Result<CaseEvals, PKError> {
    //     self.combinations_remaining(5)
    //         .map(|v| {
    //             let five = Five::try_from(v).unwrap();
    //             self.bcm_case_eval(five).unwrap()
    //         })
    //         .collect()
    // }

    /// # Errors
    ///
    /// `PKError` if unable to convert the five `Cards`.
    // pub fn bcm_rayon_case_evals(&self) -> Result<CaseEvals, PKError> {
    //     let remaining = self.remaining();
    //     let v = remaining.par_combinations_remaining(5)
    //         .map(|v| {
    //             let five = Five::try_from(v).unwrap();
    //             self.bcm_case_eval(five)
    //         })
    //         .collect::<Vec<CaseEval>>();
    //     Ok(CaseEvals::from(v))
    //     //
    //     // let bridge = self.par_combinations_remaining(5);
    //     // bridge       //     .map(|v| {
    //     //         let five = Five::try_from(v)?;
    //     //         self.bcm_case_eval(five)
    //     //     })
    //     //
    //     // let v = self.par_combinations_remaining(5)
    //     //     .map(|v| {
    //     //         let five = Five::try_from(v)?;
    //     //         self.bcm_case_eval(five)
    //     //     })
    //     //     .collect::<Vec<CaseEval>>();
    //     // Ok(CaseEvals::from(v))
    // }

    /// # Panics
    ///
    /// Should not be possible. Fingers crossed
    pub fn bcm_rayon_case_evals(&self) -> Result<CaseEvals, PKError> {
        let v: Vec<CaseEval> = self
            .par_combinations_remaining(5)
            .map(|v| {
                let five = Five::try_from(v).unwrap();
                self.bcm_case_eval(five).unwrap()
            })
            .collect();
        Ok(CaseEvals::from(v))
    }

    #[must_use]
    pub fn heavy_case_eval(&self, case: Five) -> CaseEval {
        let mut case_eval = CaseEval::default();

        for player in &self.vec() {
            if let Ok(seven) = Seven::from_case_at_deal(*player, case) {
                let eval = Eval::from(seven);
                case_eval.push(eval);
            }
        }

        case_eval
    }

    #[must_use]
    pub fn array(&self) -> [Two; 9] {
        self.0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vec().is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.vec().len()
    }

    #[must_use]
    pub fn vec(&self) -> Vec<Two> {
        let mut v = Vec::new();
        for two in self.0 {
            if two.is_dealt() {
                v.push(two);
            }
        }
        v
    }
}

impl fmt::Display for Twos {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = self
            .vec()
            .iter()
            .map(Two::to_string)
            .collect::<Vec<String>>()
            .join(", ");

        write!(f, "{s}")
    }
}

impl From<[Two; 9]> for Twos {
    fn from(value: [Two; 9]) -> Self {
        Twos(value)
    }
}

impl From<[Two; 8]> for Twos {
    fn from(v: [Two; 8]) -> Self {
        Twos([
            v[0],
            v[1],
            v[2],
            v[3],
            v[4],
            v[5],
            v[6],
            v[7],
            Two::default(),
        ])
    }
}

impl From<[Two; 7]> for Twos {
    fn from(v: [Two; 7]) -> Self {
        Twos([
            v[0],
            v[1],
            v[2],
            v[3],
            v[4],
            v[5],
            v[6],
            Two::default(),
            Two::default(),
        ])
    }
}

impl From<[Two; 6]> for Twos {
    fn from(v: [Two; 6]) -> Self {
        Twos([
            v[0],
            v[1],
            v[2],
            v[3],
            v[4],
            v[5],
            Two::default(),
            Two::default(),
            Two::default(),
        ])
    }
}

impl From<[Two; 5]> for Twos {
    fn from(v: [Two; 5]) -> Self {
        Twos([
            v[0],
            v[1],
            v[2],
            v[3],
            v[4],
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
        ])
    }
}

impl From<[Two; 4]> for Twos {
    fn from(v: [Two; 4]) -> Self {
        Twos([
            v[0],
            v[1],
            v[2],
            v[3],
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
        ])
    }
}

impl From<[Two; 3]> for Twos {
    fn from(v: [Two; 3]) -> Self {
        Twos([
            v[0],
            v[1],
            v[2],
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
        ])
    }
}

impl From<[Two; 2]> for Twos {
    fn from(v: [Two; 2]) -> Self {
        Twos([
            v[0],
            v[1],
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
        ])
    }
}

impl From<Vec<Two>> for Twos {
    fn from(v: Vec<Two>) -> Self {
        let mut twos = Twos::default();
        for (i, two) in v.iter().enumerate() {
            twos.0[i] = *two;
        }
        twos
    }
}

impl FromStr for Twos {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Twos::try_from(Cards::from_str(s)?)
    }
}

impl Pile for Twos {
    fn clean(&self) -> Self {
        Twos::default()
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        let mut v = Vec::new();
        for two in self.vec() {
            v.push(two.first());
            v.push(two.second());
        }
        v
    }
}

impl TryFrom<Cards> for Twos {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.clone().as_twos() {
            Ok(t) => Ok(Twos::from(t)),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<&Cards> for Twos {
    type Error = PKError;

    fn try_from(cards: &Cards) -> Result<Self, Self::Error> {
        Twos::try_from(cards.clone())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__hole_cards__twos_tests {
    use super::*;
    use std::str::FromStr;

    const HERO: Two = Two::HAND_AS_4S;
    const VILLAIN: Two = Two::HAND_KS_KH;
    const MINION: Two = Two::HAND_8C_7C;

    #[test]
    fn is_empty() {
        assert!(Twos::default().is_empty());
        assert!(!Twos::from([HERO, VILLAIN, MINION]).is_empty());
    }

    #[test]
    fn len() {
        assert_eq!(3, Twos::from([HERO, VILLAIN, MINION]).len())
    }

    #[test]
    fn vec() {
        let twos = Twos::from([HERO, VILLAIN, MINION]);

        assert_eq!(3, twos.vec().len());
    }

    // A♠ 4♠ K♠ K♥ 8♣ 7♣
    #[test]
    fn display() {
        assert_eq!(
            "A♠ 4♠, K♠ K♥, 8♣ 7♣",
            Twos::from([HERO, VILLAIN, MINION]).to_string()
        );
    }

    #[test]
    fn from_str() {
        let expected = [
            HERO,
            VILLAIN,
            MINION,
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
        ];

        let twos = Twos::from_str("AS 4S KS KH 8C 7C").unwrap();

        assert_eq!(expected, twos.array());
        assert_eq!(vec![HERO, VILLAIN, MINION], twos.vec());
    }

    #[test]
    fn try_from__cards() {
        let cards = Cards::from_str("AS 4S KS KH 8C 7C").unwrap();
        let expected = [
            HERO,
            VILLAIN,
            MINION,
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
            Two::default(),
        ];

        let twos = Twos::try_from(cards).unwrap();

        assert_eq!(expected, twos.array());
        assert_eq!(vec![HERO, VILLAIN, MINION], twos.vec());
    }
}
