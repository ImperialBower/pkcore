use crate::PKError;
use crate::analysis::equity::{EquityReport, EquityRequest, Method, PlayerEquity, PlayerSpec};
use crate::analysis::gto::odds::WinLoseDraw;
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::play::hole_cards::HoleCards;
use std::fmt::Formatter;

/// Fixed RNG seed for the multi-way Monte Carlo path so preflop odds are
/// reproducible across runs and test threads.
const PREFLOP_SAMPLING_SEED: u64 = 0xDEA1_5EED;

/// Per-seat preflop odds for a whole table.
///
/// `DealEval::new` dispatches on seat count: a precomputed O(1) heads-up table
/// lookup for two seats, the multi-way equity engine for three to ten. Both
/// produce an [`EquityReport`]; `hands[i]` corresponds to `report.players[i]`.
#[derive(Clone, Debug)]
pub struct DealEval {
    pub hands: HoleCards,
    pub report: EquityReport,
}

impl DealEval {
    pub const HEADSUP_PREFLOP_COMBO_COUNT: usize = 1_712_304;

    /// Computes preflop odds for every seat.
    ///
    /// Dispatches on seat count: two seats use the precomputed, wasm-safe
    /// heads-up preflop table (`HUPResult`); three to ten use the multi-way
    /// equity engine (exact enumeration or seeded Monte Carlo).
    ///
    /// # Errors
    ///
    /// - [`PKError::NotEnoughHands`] for fewer than two seats.
    /// - [`PKError::TooManyHands`] for more than ten seats.
    /// - Propagates lookup / equity-engine errors (e.g. duplicate cards).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::play::stages::deal_eval::DealEval;
    /// use pkcore::play::hole_cards::HoleCards;
    /// use pkcore::arrays::two::Two;
    ///
    /// let hands = HoleCards::from(vec![Two::HAND_AS_AH, Two::HAND_KS_KH]);
    /// let eval = DealEval::new(hands).unwrap();
    /// assert_eq!(eval.report.players.len(), 2);
    /// ```
    pub fn new(hands: HoleCards) -> Result<DealEval, PKError> {
        let report = match hands.len() {
            0 | 1 => return Err(PKError::NotEnoughHands),
            2 => heads_up_report(&hands)?,
            _ => multiway_report(&hands)?,
        };
        Ok(DealEval { hands, report })
    }
}

/// Converts a heads-up win/lose/draw tally into a single seat's equity.
#[allow(clippy::cast_precision_loss)]
fn player_equity_from_wld(wld: WinLoseDraw) -> PlayerEquity {
    let total = wld.total().max(1) as f64;
    PlayerEquity {
        win: wld.wins as f64 / total,
        tie: wld.draws as f64 / total,
        equity: (wld.wins as f64 + wld.draws as f64 / 2.0) / total,
        wins: wld.wins,
        ties: wld.draws,
    }
}

/// Heads-up branch: O(1) embedded HUP lookup, mapped back to seat order.
///
/// The HUP table is oriented by higher/lower hand, never seat order, so the
/// odds are assigned to seats by checking which seat holds the higher hand.
fn heads_up_report(hands: &HoleCards) -> Result<EquityReport, PKError> {
    let a = *hands.get(0).ok_or(PKError::NotEnoughHands)?;
    let b = *hands.get(1).ok_or(PKError::NotEnoughHands)?;
    let shu = SortedHeadsUp::new(a, b);
    let hup = shu.hup_result()?;
    let higher = player_equity_from_wld(hup.odds);
    let lower = player_equity_from_wld(hup.flip_mode().odds);
    let players = if shu.is_higher(&a) {
        vec![higher, lower]
    } else {
        vec![lower, higher]
    };
    Ok(EquityReport {
        players,
        method: Method::Hup,
        samples: hup.odds.total(),
    })
}

/// Multi-way branch (3–10 seats): the equity engine, seeded for reproducibility.
fn multiway_report(hands: &HoleCards) -> Result<EquityReport, PKError> {
    let players: Vec<PlayerSpec> = hands.iter().map(|two| PlayerSpec::Exact(*two)).collect();
    let mut req = EquityRequest::new(players);
    req.opts.seed = Some(PREFLOP_SAMPLING_SEED);
    req.compute()
}

impl std::fmt::Display for DealEval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut v = Vec::new();
        v.push(format!(
            "Method: {:?} ({} samples)",
            self.report.method, self.report.samples
        ));
        for (i, hand) in self.hands.iter().enumerate() {
            if let Some(pe) = self.report.players.get(i) {
                v.push(format!(
                    "Player #{i}: {hand}  win {:.2}% / tie {:.2}% / equity {:.2}%",
                    pe.win * 100.0,
                    pe.tie * 100.0,
                    pe.equity * 100.0
                ));
            }
        }
        write!(f, "{}", v.join("\n"))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__stages__deal_eval_tests {
    use super::*;
    use crate::Pile;
    use crate::analysis::equity::Method;
    use crate::arrays::two::Two;
    use crate::util::data::TestData;
    use std::str::FromStr;

    fn hands(twos: Vec<Two>) -> HoleCards {
        HoleCards::from(twos)
    }

    #[test]
    fn new__heads_up_uses_hup() {
        let eval = DealEval::new(hands(vec![Two::HAND_AS_AH, Two::HAND_KS_KH])).unwrap();
        assert_eq!(eval.report.method, Method::Hup);
        assert_eq!(eval.report.players.len(), 2);
    }

    #[test]
    fn new__heads_up_favorite_equity() {
        // AA vs KK preflop is ~82% for the aces.
        let eval = DealEval::new(hands(vec![Two::HAND_AS_AH, Two::HAND_KS_KH])).unwrap();
        let aa = eval.report.players[0].equity;
        assert!(aa > 0.80 && aa < 0.84, "AA equity was {aa}");
    }

    #[test]
    fn new__heads_up_orientation_follows_seat_order() {
        // Seat 0 = KK, seat 1 = AA: the ~82% must land on seat 1, not seat 0.
        let eval = DealEval::new(hands(vec![Two::HAND_KS_KH, Two::HAND_AS_AH])).unwrap();
        assert!(
            eval.report.players[1].equity > 0.80,
            "AA (seat 1) should be the favorite"
        );
        assert!(
            eval.report.players[0].equity < 0.20,
            "KK (seat 0) should be the underdog"
        );
    }

    #[test]
    fn new__multiway_sums_to_one() {
        let eval = DealEval::new(hands(vec![Two::HAND_AS_AH, Two::HAND_KS_KH, Two::HAND_QS_QH])).unwrap();
        assert_eq!(eval.report.players.len(), 3);
        let sum: f64 = eval.report.players.iter().map(|p| p.equity).sum();
        assert!((sum - 1.0).abs() < 0.01, "equities summed to {sum}");
        // Aces are the clear favorite three-way.
        assert!(eval.report.players[0].equity > eval.report.players[1].equity);
        assert!(eval.report.players[0].equity > eval.report.players[2].equity);
    }

    #[test]
    fn new__multiway_is_deterministic() {
        let h = hands(vec![Two::HAND_AS_AH, Two::HAND_KS_KH, Two::HAND_QS_QH]);
        let a = DealEval::new(h.clone()).unwrap();
        let b = DealEval::new(h).unwrap();
        assert_eq!(a.report.players[0].equity, b.report.players[0].equity);
        assert_eq!(a.report.players[1].equity, b.report.players[1].equity);
    }

    #[test]
    fn new__too_few_hands_errors() {
        assert!(DealEval::new(hands(vec![Two::HAND_AS_AH])).is_err());
    }

    #[test]
    fn new__too_many_hands_errors() {
        // 11 hands (22 cards) exceeds the engine's 10-seat cap.
        let eleven = HoleCards::from_str("As Ah Ks Kh Qs Qh Js Jh Ts Th 9s 9h 8s 8h 7s 7h 6s 6h 5s 5h 4s 4h").unwrap();
        assert_eq!(eleven.len(), 11);
        assert!(DealEval::new(eleven).is_err());
    }

    #[test]
    fn iterations_heads_up() {
        let game = TestData::the_hand();

        let combos = game.hands.enumerate_remaining(5);

        assert_eq!(combos.count(), DealEval::HEADSUP_PREFLOP_COMBO_COUNT);
    }
}
