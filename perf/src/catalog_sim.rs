//! Bot self-play workloads — whole hands, end to end.
//!
//! This is the closest the catalog gets to a production workload: dealing,
//! betting rounds, rule-based bot decisions, and showdown, all of which sit on
//! top of the evaluator the nano band measures.
//!
//! The design doc that seeded this catalog assumed self-play needed the
//! `bot-profiles` and `hand-histories` pkcore features. Inspection (and a
//! throwaway `cargo run --example feature_probe --no-default-features` probe,
//! deleted after use) showed otherwise: `pkcore::bot::mod` declares `sim` and
//! `profile` as ungated modules, `BotProfile`'s constructors
//! (`gto`, `tight_aggressive`, ...) are unconditional, and every
//! `#[cfg(feature = "player-stats")]` gate inside `sim.rs` guards optional
//! stat-tracking the sweep never touches. Those two features add YAML
//! serialisation for saving/loading profiles, which self-play does not use.
//! This workload is pure kernel: `features: &[]`, no module-level feature
//! gate.
//!
//! The checksum folds `hands_played` and the per-seat net chip counts, which
//! are integers. A fixed seed makes the whole session replayable, so a
//! `Status::Nondeterministic` here means genuine scheduling non-determinism has
//! leaked into the simulation.

use crate::workload::{Band, HotFn, PerfError, Workload};
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::{Player, Seat, Seats, Table};

/// Seed for every self-play session, so runs are comparable across days.
const SEED: u64 = 42;

/// Starting stack per seat, in chips.
const STACK: usize = 10_000;

/// Six seats with distinct playing styles, so the sample exercises a range of
/// betting behaviour rather than six copies of one decision tree.
fn six_max_table() -> Result<(Table, Vec<(u8, BotProfile)>), PerfError> {
    let profiles = [
        ("gto", BotProfile::gto()),
        ("tag", BotProfile::tight_aggressive()),
        ("lag", BotProfile::loose_aggressive()),
        ("tp", BotProfile::tight_passive()),
        ("lp", BotProfile::loose_passive()),
        ("maniac", BotProfile::maniac()),
    ];

    let seats = Seats::new(
        profiles
            .iter()
            .map(|(name, _)| Seat::new(Player::new_with_chips((*name).to_string(), STACK)))
            .collect(),
    );

    let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));

    let bots = profiles
        .iter()
        .enumerate()
        .map(|(index, (_, profile))| {
            let seat = u8::try_from(index).map_err(|_| PerfError::Setup(format!("seat index {index} exceeds u8")))?;
            Ok((seat, profile.clone()))
        })
        .collect::<Result<Vec<(u8, BotProfile)>, PerfError>>()?;

    Ok((table, bots))
}

/// Sessions prebuilt at setup, outside the timed region. The default macro
/// protocol is 1 warmup + 5 trials, so 16 leaves headroom for a `--trials`
/// override; a run that exhausts the pool falls back to building in the
/// timed region (see the closure body).
const PREBUILT_SESSIONS: usize = 16;

/// Builds one fresh, identically-seeded session.
fn build_session() -> Result<SimTable, PerfError> {
    let (table, bots) = six_max_table()?;
    Ok(SimTable::with_rule_based(table, bots).with_seed(SEED))
}

/// `iters` is the number of hands per trial, not a repeat count — a macro-band
/// workload measures nanoseconds per hand.
///
/// Every trial needs a *fresh* session: a session carries the deal state
/// forward, so reusing one would give each trial different hands and a
/// different checksum — instant `Status::Nondeterministic`. An earlier
/// version therefore rebuilt the table inside the timed closure, timing
/// session construction along with the hands. The sessions are now prebuilt
/// at setup into a pool the closure pops from, so the timed region is hands
/// only for any run that stays within [`PREBUILT_SESSIONS`] calls.
fn make_selfplay_6max() -> Result<HotFn, PerfError> {
    let mut sessions = Vec::with_capacity(PREBUILT_SESSIONS);
    for _ in 0..PREBUILT_SESSIONS {
        sessions.push(build_session()?);
    }
    let pool = std::cell::RefCell::new(sessions);

    Ok(Box::new(move |iters: u32| {
        let prebuilt = pool
            .try_borrow_mut()
            .map_err(|_| PerfError::Run("session pool already borrowed; hot closure re-entered".to_string()))?
            .pop();
        let mut sim = match prebuilt {
            Some(session) => session,
            // Pool exhausted (a --trials override beyond PREBUILT_SESSIONS):
            // build in place. This puts one construction inside the timed
            // region for those extra trials — a small, visible bias, not a
            // failure.
            None => build_session()?,
        };

        let result = sim
            .run_n_hands(iters as usize)
            .map_err(|e| PerfError::Run(format!("run_n_hands({iters}) failed: {e:?}")))?;

        let mut acc = result.hands_played as u64;
        // Sort by seat so the fold is order-independent regardless of the
        // HashMap's iteration order.
        let mut nets: Vec<(u8, i64)> = result.net_chips.iter().map(|(k, v)| (*k, *v)).collect();
        nets.sort_unstable();
        for (seat, net) in nets {
            acc = acc.wrapping_add(u64::from(seat)).wrapping_add(net.unsigned_abs());
        }
        Ok(acc)
    }))
}

/// Every self-play workload.
///
/// # Examples
///
/// ```
/// use pkcore_perf::catalog_sim::sim_workloads;
///
/// assert_eq!(sim_workloads().len(), 1);
/// ```
#[must_use]
pub fn sim_workloads() -> Vec<Workload> {
    vec![Workload {
        name: "sim.selfplay.6max",
        band: Band::Macro,
        inner_iters: 200,
        features: &[],
        parallel: false,
        make: make_selfplay_6max,
    }]
}

#[cfg(test)]
#[allow(non_snake_case)]
mod perf__catalog_sim_tests {
    use super::*;
    use crate::runner::{Status, measure};
    use crate::workload::Band;

    #[test]
    fn selfplay_is_a_macro_workload() {
        let workloads = sim_workloads();
        assert_eq!(workloads.len(), 1);
        assert_eq!(workloads[0].name, "sim.selfplay.6max");
        assert_eq!(workloads[0].band, Band::Macro);
    }

    /// Seeded self-play must replay identically. If it does not, every figure
    /// this workload produces describes a different game each trial.
    #[test]
    fn selfplay_is_deterministic_under_a_fixed_seed() {
        let sample = measure(&sim_workloads().remove(0), 0, 2, 5);
        assert_eq!(sample.status, Status::Ok, "{:?}", sample.message);
        assert_ne!(sample.checksum, Some(0));
    }
}
