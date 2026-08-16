//! The [`BotDecider`] trait and concrete implementations.
//!
//! [`BotDecider`] is the core decision-making abstraction for poker bots.
//! Given a [`BotProfile`] and a read-only [`TableSnapshot`], it returns a
//! [`PlayerAction`].
//!
//! The same trait will be used by both the local simulation ([`SimTable`])
//! and the gRPC agent clients in Phase 4 of the ROADMAP.  Decision logic
//! lives in pkcore; transport lives in pkdealer.
//!
//! # Implementations
//!
//! - [`RuleBasedDecider`] — probabilistic decisions driven by the
//!   `aggression_factor` and `preferred_bet_sizes` fields of a
//!   [`BotProfile`].  No hand-strength analysis is performed.
//! - [`JokerDecider`] — randomly adopts one of the standard reference
//!   profiles at the start of each hand, then plays it faithfully for the
//!   whole hand using [`RuleBasedDecider`] logic.
//!
//! [`SimTable`]: crate::bot::sim::SimTable

use std::fmt;
use std::sync::Mutex;

use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::six::Six;
use crate::bot::betting_strategy::BettingStrategy;
use crate::bot::decision_config::{EquityMode, RangeMode};
use crate::bot::player_action::PlayerAction;
use crate::bot::profile::BotProfile;
use crate::bot::range_strategy::RangeStrategy;
use crate::bot::table_snapshot::TableSnapshot;
use crate::games::betting_structure::{BetTier, BettingStructure};

// ── BotDecider trait ──────────────────────────────────────────────────────────

/// Decision-making strategy for a poker bot.
///
/// Implement this trait to define how a bot selects an action given its
/// profile and the current table state.
///
/// # Object safety
///
/// `BotDecider` is object-safe and requires `Send + Sync` so that
/// `Box<dyn BotDecider>` can be stored in [`SimTable`] and moved across
/// thread boundaries.
///
/// # Examples
///
/// ```
/// use pkcore::bot::decider::{BotDecider, RuleBasedDecider};
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::table_snapshot::TableSnapshot;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
///
/// let seats = Seats::new(vec![
///     Seat::new(Player::new_with_chips("Alice".to_string(), 1_000)),
///     Seat::new(Player::new_with_chips("Bob".to_string(), 1_000)),
/// ]);
/// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let snap = TableSnapshot::from_table(&table, 0);
/// let profile = BotProfile::gto();
/// let action = RuleBasedDecider.decide(&profile, &snap);
/// let _ = action; // any valid action
/// ```
///
/// [`SimTable`]: crate::bot::sim::SimTable
pub trait BotDecider: Send + Sync {
    /// Called once at the start of each new hand, before any actions are taken.
    ///
    /// The default implementation is a no-op.  Override to perform per-hand
    /// setup — for example, [`JokerDecider`] uses this hook to randomly select
    /// a new playing style for the upcoming hand.
    fn on_new_hand(&self) {}

    /// Seeded variant of [`Self::on_new_hand`].
    ///
    /// The default implementation ignores `rng` and delegates to
    /// [`Self::on_new_hand`].  Override when per-hand state needs to be
    /// deterministic under a seeded [`crate::bot::sim::SimTable`].
    fn on_new_hand_with_rng(&self, _rng: &mut dyn rand::RngCore) {
        self.on_new_hand();
    }

    /// Choose a [`PlayerAction`] for the given `profile` and table `state`.
    fn decide(&self, profile: &BotProfile, state: &TableSnapshot) -> PlayerAction;

    /// Seeded variant of [`Self::decide`].
    ///
    /// The default implementation ignores `rng` and delegates to
    /// [`Self::decide`] (which typically uses a thread-local RNG).
    /// Override to make decisions reproducible under a seeded
    /// [`crate::bot::sim::SimTable`]. The three shipped deciders
    /// ([`RuleBasedDecider`], [`JokerDecider`],
    /// [`crate::bot::exploitative_decider::ExploitativeDecider`]) all
    /// override this so seeded sim runs are fully deterministic.
    fn decide_seeded(&self, profile: &BotProfile, state: &TableSnapshot, _rng: &mut dyn rand::RngCore) -> PlayerAction {
        self.decide(profile, state)
    }
}

// ── RuleBasedDecider ──────────────────────────────────────────────────────────

/// A probabilistic, profile-driven [`BotDecider`].
///
/// Decisions are derived from:
/// - `BotProfile.betting_strategy.aggression_factor` — controls the
///   fold / call / bet / raise split.
/// - `BotProfile.betting_strategy.preferred_bet_sizes` — sizes for bets
///   and raises as pot fractions.
/// - `BotProfile.betting_strategy.bluff_frequency` — postflop bluff rate
///   when the bot would otherwise check.
/// - `BotProfile.betting_strategy.check_raise_frequency` — raise rate when
///   the bot has checked this street and now faces a bet.
/// - `BotProfile.range_strategy.postflop_cbet_frequency` — overrides
///   `aggression_factor` for flop bets (continuation-bet frequency).
///
/// A thread-local RNG is used internally; no mutable state is needed.
///
/// This type is the library-level promotion of the `decide()` free function
/// in `examples/bot_selfplay.rs`.
///
/// # Examples
///
/// ```
/// use pkcore::bot::decider::{BotDecider, RuleBasedDecider};
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::table_snapshot::TableSnapshot;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
///
/// let seats = Seats::new(vec![
///     Seat::new(Player::new_with_chips("X".to_string(), 500)),
///     Seat::new(Player::new_with_chips("Y".to_string(), 500)),
/// ]);
/// let table = Table::nlh_from_seats(seats, ForcedBets::new(5, 10));
/// let snap = TableSnapshot::from_table(&table, 0);
/// let profile = BotProfile::tight_passive();
/// let action = RuleBasedDecider.decide(&profile, &snap);
/// let _ = action;
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RuleBasedDecider;

impl BotDecider for RuleBasedDecider {
    fn decide(&self, profile: &BotProfile, state: &TableSnapshot) -> PlayerAction {
        RuleBasedDecider::decide_with_rng(profile, state, &mut rand::rng())
    }

    fn decide_seeded(&self, profile: &BotProfile, state: &TableSnapshot, rng: &mut dyn rand::RngCore) -> PlayerAction {
        RuleBasedDecider::decide_with_rng(profile, state, rng)
    }
}

impl RuleBasedDecider {
    /// Core decision logic parameterised over any [`rand::Rng`].
    ///
    /// The public [`BotDecider::decide`] method calls this with the
    /// thread-local RNG.  Tests call it directly with a seeded
    /// [`rand::rngs::SmallRng`] for fully deterministic results.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn decide_with_rng<R: rand::Rng + ?Sized>(
        profile: &BotProfile,
        state: &TableSnapshot,
        rng: &mut R,
    ) -> PlayerAction {
        let chips = state.my_chips;

        if chips == 0 {
            return PlayerAction::Check;
        }

        // Exploit knob: when enabled and an opponent-stats registry is attached,
        // adjust the profile from aggregate opponent tendencies before deciding.
        // A no-op when the knob is Off or no registry is present, so opponent
        // identity never leaks into the decision.
        #[cfg(feature = "player-stats")]
        let exploit_adjusted = exploit_profile(profile, state);
        #[cfg(feature = "player-stats")]
        let profile: &BotProfile = exploit_adjusted.as_deref().unwrap_or(profile);

        // Resolve position-aware strategy; falls back to flat betting_strategy when
        // no Playbook entry exists for this table size / position.
        let strategy = state.position().map_or(&profile.betting_strategy, |pos| {
            profile.betting_for(state.seat_count, pos)
        });

        let aggr = strategy.aggression_for_phase(state.phase).as_f64();
        let roll: f64 = rng.random();

        // Check-raise: we checked earlier this street and now face a bet.
        if state.to_call > 0 && state.checked_this_street {
            let cr_rate = strategy.check_raise_frequency.as_f64();
            if roll < cr_rate {
                // DEFECT_007: `None` means the stack cannot cover a minimum
                // raise. Fall through to the equity logic below rather than
                // shoving — a check-raise the player cannot afford is not an
                // all-in commitment decision.
                if let Some(raise_to) = sized_raise_to(state, strategy, rng)
                    && raise_to > state.current_bet
                {
                    return PlayerAction::Raise(raise_to);
                }
            }
        }

        // When hole cards are available, use equity-based decisions.
        if let Some(equity) = hand_equity(profile, state, rng) {
            if state.to_call > 0 {
                if state.to_call >= chips {
                    return if equity > 0.5 {
                        PlayerAction::AllIn
                    } else {
                        PlayerAction::Fold
                    };
                }

                #[allow(clippy::cast_precision_loss)]
                let pot_odds = state.to_call as f64 / (state.pot + state.to_call) as f64;

                // pot_odds knob: `discipline` scales the call threshold. 1.0 is
                // the strict break-even call (historical behavior); 0.0 ignores
                // pot odds entirely (looser, weaker).
                let call_threshold = pot_odds * profile.decision.pot_odds.discipline;

                if equity > pot_odds * 2.0 {
                    // Strong hand: raise with probability proportional to aggression so
                    // that two bots with strong hands don't raise each other indefinitely.
                    let raise_roll: f64 = rng.random();
                    if raise_roll < aggr.max(0.5) {
                        // DEFECT_007: with a strong hand and a stack too short
                        // for a legal raise, all-in *is* the raise — the whole
                        // stack is less than one minimum increment anyway.
                        match sized_raise_to(state, strategy, rng) {
                            Some(raise_to) if raise_to > state.current_bet => {
                                return PlayerAction::Raise(raise_to);
                            }
                            Some(_) => {}
                            None => return PlayerAction::AllIn,
                        }
                    }
                    PlayerAction::Call
                } else if equity > call_threshold {
                    PlayerAction::Call
                } else {
                    // Weak hand: bluff-raise or fold.
                    let bluff_roll: f64 = rng.random();
                    // DEFECT_007: a bluff that cannot be sized legally folds.
                    // Converting it to an all-in shove would materially change
                    // bot behaviour and re-open the chip-concentration dynamics
                    // recorded in DEFECT_002.
                    if bluff_roll < strategy.bluff_frequency.as_f64()
                        && let Some(raise_to) = sized_raise_to(state, strategy, rng)
                        && raise_to > state.current_bet
                    {
                        return PlayerAction::Raise(raise_to);
                    }
                    PlayerAction::Fold
                }
            } else {
                // No outstanding bet: value-bet or bluff.
                //
                // DEFECT_007: `sized_bet_amount` returns `None` when the stack
                // cannot cover a legal opening bet. A value bet becomes an
                // all-in (the intent was to commit chips); a bluff checks,
                // for the same DEFECT_002 reason a bluff-raise does not shove.
                let value_threshold = strategy.effective_value_threshold();
                if equity > value_threshold {
                    sized_bet_amount(state, strategy, rng).map_or(PlayerAction::AllIn, |n| voluntary_open(state, n))
                } else if !state.phase.is_preflop() {
                    let bluff_roll: f64 = rng.random();
                    if bluff_roll < strategy.bluff_frequency.as_f64() {
                        sized_bet_amount(state, strategy, rng).map_or(PlayerAction::Check, |n| voluntary_open(state, n))
                    } else {
                        PlayerAction::Check
                    }
                } else {
                    PlayerAction::Check
                }
            }
        } else {
            // Fallback: aggression-factor-based logic when hole cards are unknown.
            if state.to_call > 0 {
                if state.to_call >= chips {
                    return if roll < aggr * 0.6 {
                        PlayerAction::AllIn
                    } else {
                        PlayerAction::Fold
                    };
                }

                // DEFECT_007: no legal raise → fall through to the call/fold
                // decision below. Hole cards are unknown on this path, so there
                // is no read strong enough to justify shoving instead.
                if roll < aggr * 0.25
                    && let Some(raise_to) = sized_raise_to(state, strategy, rng)
                    && raise_to > state.current_bet
                {
                    return PlayerAction::Raise(raise_to);
                }

                if roll < aggr {
                    PlayerAction::Call
                } else {
                    PlayerAction::Fold
                }
            } else {
                // On the flop, postflop_cbet_frequency overrides the flat aggression factor.
                let bet_threshold = if state.phase.is_flop() {
                    profile.range_strategy.postflop_cbet_frequency.as_f64()
                } else {
                    aggr
                };

                // DEFECT_007: hole cards are unknown here, so an unsizeable bet
                // checks rather than shoving on no information.
                if roll < bet_threshold {
                    sized_bet_amount(state, strategy, rng).map_or(PlayerAction::Check, |n| voluntary_open(state, n))
                } else if !state.phase.is_preflop() {
                    let bluff_rate = strategy.bluff_frequency.as_f64();
                    let roll_bluff: f64 = rng.random();
                    if roll_bluff < bluff_rate {
                        sized_bet_amount(state, strategy, rng).map_or(PlayerAction::Check, |n| voluntary_open(state, n))
                    } else {
                        PlayerAction::Check
                    }
                } else {
                    PlayerAction::Check
                }
            }
        }
    }
}

// ── JokerDecider ─────────────────────────────────────────────────────────────

/// A [`BotDecider`] that randomly adopts one of the standard reference
/// profiles at the start of each hand.
///
/// On every call to [`BotDecider::on_new_hand`] the joker rolls a new profile
/// from [`BotProfile::default_profiles`] and uses it for the remainder of that
/// hand.  All actual action decisions are delegated to [`RuleBasedDecider`],
/// so the joker's in-hand behaviour is indistinguishable from the chosen
/// profile — only the style changes between hands.
///
/// Pair this decider with [`BotProfile::joker()`] in a [`crate::bot::sim::SimTable`]:
///
/// ```
/// use pkcore::bot::decider::{BotDecider, JokerDecider};
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::sim::SimTable;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
///
/// let seats = Seats::new(vec![
///     Seat::new(Player::new_with_chips("joker".to_string(), 5_000)),
///     Seat::new(Player::new_with_chips("gto".to_string(), 5_000)),
/// ]);
/// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)> = vec![
///     (0, BotProfile::joker(), Box::new(JokerDecider::new())),
///     (1, BotProfile::gto(),   Box::new(pkcore::bot::decider::RuleBasedDecider)),
/// ];
/// let mut sim = SimTable::new(table, bots);
/// let result = sim.run_n_hands(5).unwrap();
/// assert!(result.hands_played > 0);
/// ```
pub struct JokerDecider {
    active: Mutex<BotProfile>,
}

impl JokerDecider {
    /// Creates a new `JokerDecider` with a randomly selected initial profile.
    ///
    /// The active profile is replaced at the start of each hand via
    /// [`BotDecider::on_new_hand`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::decider::JokerDecider;
    ///
    /// let decider = JokerDecider::new();
    /// let _ = decider; // ready to use in SimTable
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_rng(&mut rand::rng())
    }

    /// Like [`Self::new`] but uses the supplied RNG to pick the initial
    /// profile.  Use this when constructing a `JokerDecider` that will run
    /// inside a seeded [`crate::bot::sim::SimTable`] and you want the very
    /// first hand to be reproducible.  Subsequent hands re-roll via
    /// [`BotDecider::on_new_hand_with_rng`] using the sim's RNG.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::decider::JokerDecider;
    /// use rand::SeedableRng;
    /// use rand::rngs::SmallRng;
    ///
    /// let mut rng = SmallRng::seed_from_u64(42);
    /// let decider = JokerDecider::new_with_rng(&mut rng);
    /// let _ = decider;
    /// ```
    #[must_use]
    pub fn new_with_rng<R: rand::Rng + ?Sized>(rng: &mut R) -> Self {
        let profiles = BotProfile::default_profiles();
        let idx = rng.random_range(0..profiles.len());
        Self {
            active: Mutex::new(profiles[idx].clone()),
        }
    }
}

impl Default for JokerDecider {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for JokerDecider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .active
            .lock()
            .map_or_else(|_| "poisoned".to_string(), |g| g.name.clone());
        f.debug_struct("JokerDecider").field("active", &name).finish()
    }
}

impl BotDecider for JokerDecider {
    /// Randomly picks a new profile from [`BotProfile::default_profiles`] for
    /// the upcoming hand.
    fn on_new_hand(&self) {
        self.on_new_hand_with_rng(&mut rand::rng());
    }

    /// Seeded variant — picks the next per-hand profile using the supplied RNG.
    fn on_new_hand_with_rng(&self, rng: &mut dyn rand::RngCore) {
        use rand::Rng as _;
        let profiles = BotProfile::default_profiles();
        let idx = rng.random_range(0..profiles.len());
        if let Ok(mut guard) = self.active.lock() {
            *guard = profiles[idx].clone();
        }
    }

    /// Delegates to [`RuleBasedDecider`] using the profile chosen at hand start,
    /// ignoring the `_profile` argument which is just the placeholder
    /// [`BotProfile::joker()`] stored in the seat entry.
    fn decide(&self, _profile: &BotProfile, state: &TableSnapshot) -> PlayerAction {
        let active = self
            .active
            .lock()
            .map_or_else(|e| e.into_inner().clone(), |g| g.clone());
        RuleBasedDecider.decide(&active, state)
    }

    fn decide_seeded(&self, _profile: &BotProfile, state: &TableSnapshot, rng: &mut dyn rand::RngCore) -> PlayerAction {
        let active = self
            .active
            .lock()
            .map_or_else(|e| e.into_inner().clone(), |g| g.clone());
        RuleBasedDecider.decide_seeded(&active, state, rng)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Computes a normalized equity proxy for the bot's hand.
///
/// Returns `None` when hole cards have not been dealt yet, which triggers the
/// aggression-factor fallback path in `decide_with_rng`.
///
/// **Preflop:** rolls against the combo's `:f` frequency weight from the
/// `open_raise` range. A hand at weight `0.7` plays as `1.0` equity 70% of
/// the time and `0.0` the other 30%, producing a mixed strategy.
///
/// **Postflop:** evaluates the best 5-of-N hand from the combined hole cards
/// and board, then normalises the `hand_rank_value` to `[0.0, 1.0]` where
/// `1.0` is a royal flush and `0.0` is 7-high nothing.
fn hand_equity<R: rand::Rng + ?Sized>(profile: &BotProfile, state: &TableSnapshot, rng: &mut R) -> Option<f64> {
    if state.hole_cards.is_empty() {
        return None;
    }
    if state.phase.is_preflop() {
        // Ranges knob: position-aware playbook lookup or the flat range.
        let freq = preflop_open_frequency(profile, state);
        return Some(if rng.random::<f64>() < freq { 1.0 } else { 0.0 });
    }
    let total = state.hole_cards.len() + state.board.len();
    // EPIC-32 Phase 8: partial-hand heuristic for Stud-family mid-hand
    // (3rd / 4th street) where total is 3 or 4. Returns a coarse strength
    // bucket so the bot doesn't fall through to aggression-only logic.
    // Gated on `stud_street_index().is_some()` so NLHE/FLHE/PLO are
    // unaffected.
    if state.phase.stud_street_index().is_some() && matches!(total, 3 | 4) {
        return Some(stud_partial_equity(state));
    }
    // Equity knob: real multi-way engine (fast / exact) or the hand-rank proxy.
    // Fast/Exact fall back to the proxy when the real engine can't answer (no
    // active villain, non-NLHE hole size, or the `equity` feature is disabled).
    match profile.decision.equity {
        EquityMode::Off => proxy_equity(state),
        EquityMode::Fast { samples } => real_equity(state, u64::from(samples), rng).or_else(|| proxy_equity(state)),
        EquityMode::Exact => real_equity(state, EXACT_EQUITY_SAMPLES, rng).or_else(|| proxy_equity(state)),
    }
}

/// Sample budget used for `EquityMode::Exact`.
///
/// With unknown (`Random`) villains the engine cannot enumerate exactly, so
/// "exact" is realised as a high-budget seeded Monte Carlo that approaches the
/// true multi-way equity. See the EPIC-36 corrigendum.
const EXACT_EQUITY_SAMPLES: u64 = 100_000;

/// Postflop hand-rank proxy: normalise the best 5-of-N `hand_rank_value` to
/// `[0.0, 1.0]` where `1.0` is a royal flush and `0.0` is 7-high nothing. This
/// is the pre-EPIC-36 postflop equity.
fn proxy_equity(state: &TableSnapshot) -> Option<f64> {
    let combined = format!("{} {}", state.hole_cards, state.board);
    let hrv = match state.hole_cards.len() + state.board.len() {
        5 => combined.parse::<Five>().ok().map(|h| h.hand_rank_value()),
        6 => combined.parse::<Six>().ok().map(|h| h.hand_rank_value()),
        7 => combined.parse::<Seven>().ok().map(|h| h.hand_rank_value()),
        _ => None,
    }?;
    Some(1.0 - f64::from(hrv) / 7462.0)
}

/// Resolves the preflop open-raise frequency for the hero's hole cards.
///
/// With `ranges = position_aware` and a playbook entry for the current seat
/// count and position, the frequency comes from the position-aware range;
/// otherwise it falls back to the flat `range_strategy.open_raise`. The
/// position-aware range is reconstructed into a range string and evaluated
/// through the proven [`RangeStrategy::open_raise_frequency`] so that
/// plus-notation expansion (`QQ+`) and mixed-frequency suffixes (`JJ:0.95`)
/// are handled identically to the flat path.
fn preflop_open_frequency(profile: &BotProfile, state: &TableSnapshot) -> f64 {
    if matches!(profile.decision.ranges, RangeMode::PositionAware)
        && let Some(pos) = state.position()
        && let Some(range) = profile.range_for(state.seat_count, pos, "open_raise")
    {
        let range_str = range
            .combos()
            .iter()
            .map(|cw| {
                if (cw.frequency - 1.0).abs() < f64::EPSILON {
                    cw.range.clone()
                } else {
                    format!("{}:{}", cw.range, cw.frequency)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        return RangeStrategy::new(range_str, "", "", 0).open_raise_frequency(&state.hole_cards);
    }
    profile.range_strategy.open_raise_frequency(&state.hole_cards)
}

/// Real multi-way equity for the hero via the [`crate::analysis::equity`]
/// engine: hero as `Exact`, each active villain as `Random`. Returns `None`
/// (so the caller falls back to the proxy) when the hero is not a 2-card NLHE
/// hand, no active villain remains, or the seat count is out of range.
#[cfg(feature = "equity")]
fn real_equity<R: rand::Rng + ?Sized>(state: &TableSnapshot, max_samples: u64, rng: &mut R) -> Option<f64> {
    use crate::analysis::equity::{EquityOptions, EquityRequest, PlayerSpec};
    use crate::arrays::two::Two;
    use crate::play::board::Board;
    use std::str::FromStr;

    if state.hole_cards.len() != 2 {
        return None;
    }
    let hero = Two::from_str(&state.hole_cards.to_string()).ok()?;
    let board = Board::try_from(state.board.clone()).ok()?;
    let villains = state
        .stacks
        .iter()
        .filter(|s| s.is_active && s.seat != state.seat)
        .count();
    let total = villains + 1;
    if !(2..=10).contains(&total) {
        return None;
    }
    let mut players = Vec::with_capacity(total);
    players.push(PlayerSpec::Exact(hero));
    players.extend(std::iter::repeat_with(|| PlayerSpec::Random).take(villains));
    let opts = EquityOptions {
        max_samples,
        seed: Some(rng.random::<u64>()),
        ..EquityOptions::default()
    };
    let report = EquityRequest { players, board, opts }.compute().ok()?;
    report.players.first().map(|p| p.equity)
}

/// Feature-off stub: without the `equity` feature the real engine is absent,
/// so the equity knob transparently falls back to the proxy.
#[cfg(not(feature = "equity"))]
fn real_equity<R: rand::Rng + ?Sized>(_state: &TableSnapshot, _max_samples: u64, _rng: &mut R) -> Option<f64> {
    None
}

/// Exploit knob: returns an opponent-adjusted profile when the knob is enabled
/// **and** an opponent-stats registry is attached, otherwise `None`.
///
/// `Light` uses the default sample gates (adjusts only once opponents are
/// well-sampled); `Heavy` lowers the gates so it adjusts sooner and more
/// readily. The adjustment reads only aggregate opponent tendencies via
/// [`crate::bot::exploit::adjust_profile`] — never opponent identity — so the
/// knob is safe on any run path. The `DecisionConfig` is carried through the
/// clone, so the other knobs are preserved on the adjusted profile.
#[cfg(feature = "player-stats")]
fn exploit_profile(profile: &BotProfile, state: &TableSnapshot) -> Option<Box<BotProfile>> {
    use crate::bot::decision_config::ExploitMode;
    use crate::bot::exploit::{ExploitConfig, adjust_profile};

    let config = match profile.decision.exploit {
        ExploitMode::Off => return None,
        ExploitMode::Light => ExploitConfig::default(),
        ExploitMode::Heavy => ExploitConfig {
            min_hands_light: 15,
            min_hands_heavy: 25,
            ..ExploitConfig::default()
        },
    };
    state.opponent_stats?;
    Some(Box::new(adjust_profile(profile, state, &config)))
}

/// EPIC-32 Phase 8: discrete partial-hand strength bucket for Stud
/// mid-hand (3rd / 4th street). Returns a value in `[0.0, 1.0]`. Not a
/// real Monte Carlo equity — coarse "pair / trips / high cards"
/// classification. v1.1 polish item.
fn stud_partial_equity(state: &TableSnapshot) -> f64 {
    use std::collections::HashMap;
    use std::str::FromStr;
    // Parse the bot's hole cards into Card values.
    let cards_str = state.hole_cards.to_string();
    let cards: Vec<crate::card::Card> = cards_str
        .split_whitespace()
        .filter_map(|tok| crate::card::Card::from_str(tok).ok())
        .collect();
    if cards.is_empty() {
        return 0.25;
    }
    let mut rank_count: HashMap<u8, u8> = HashMap::new();
    for c in &cards {
        *rank_count.entry(c.get_rank() as u8).or_insert(0) += 1;
    }
    let max_count = rank_count.values().copied().max().unwrap_or(0);
    let pair_count = rank_count.values().filter(|&&v| v == 2).count();
    match (cards.len(), max_count) {
        (3, 3) => 0.90,                    // trips on 3rd street — premium
        (3, 2) => 0.65,                    // pair on 3rd
        (4, 4) => 0.98,                    // quads on 4th — virtual lock
        (4, 3) => 0.85,                    // trips on 4th
        (4, 2) if pair_count >= 2 => 0.75, // two pair on 4th
        (4, 2) => 0.55,                    // single pair on 4th
        _ => {
            // No pair: rank by highest card present. Aces ≈ 0.45,
            // 2-rank ≈ 0.25. Linear interpolation keeps the value tame.
            let top = cards.iter().map(|c| c.get_rank() as u8).max().unwrap_or(2);
            // top is 2..=14
            let t = f64::from(top.saturating_sub(2)) / 12.0; // 0.0..=1.0
            0.20 + 0.25 * t
        }
    }
}

/// Returns a random `(numerator, denominator)` pair from `strategy.preferred_bet_sizes`,
/// falling back to half-pot `(1, 2)` when the list is empty.
fn pick_bet_size<R: rand::Rng + ?Sized>(strategy: &BettingStrategy, rng: &mut R) -> (usize, usize) {
    let sizes = &strategy.preferred_bet_sizes;
    if sizes.is_empty() {
        return (1, 2);
    }
    let (n, d) = sizes[rng.random_range(0..sizes.len())].as_fraction();
    (n as usize, d as usize)
}

/// EPIC-30 Phase 6: returns the tier increment (`small_bet` or `big_bet`)
/// for the current street if the table runs a Fixed-Limit structure.
/// `None` for No-Limit and Pot-Limit, indicating the caller should fall
/// back to pot-fraction sizing.
fn fixed_limit_increment(state: &TableSnapshot) -> Option<usize> {
    match state.betting_structure {
        BettingStructure::FixedLimit { small_bet, big_bet, .. } => Some(match state.bet_tier {
            BetTier::Small => small_bet,
            BetTier::Big => big_bet,
        }),
        _ => None,
    }
}

/// EPIC-30 Phase 6: computes a raise target ("raise to N total") that's
/// legal under the current betting structure, or `None` when **no voluntary
/// raise of any size is legal** — the stack cannot reach the minimum.
///
/// `DEFECT_007`: the previous version clamped with `.min(state.my_chips)`.
/// That is a unit error as well as a clamp-order error. `my_chips` is the
/// stack *behind*; a raise-to is measured against `current_bet`, which
/// includes chips this player already committed this street. The ceiling is
/// therefore [`TableSnapshot::max_raise_to`], and when it falls below
/// [`TableSnapshot::min_raise_to`] the correct answer is `None`, not a
/// clamped-down amount the engine rejects with
/// [`PKError::InsufficientIncrement`](crate::errors::PKError::InsufficientIncrement).
///
/// Returning `Option` rather than substituting silently is deliberate: it
/// forces each call site to state what it does instead, because the right
/// substitute differs (all-in for a value raise, fold or call for a bluff).
fn sized_raise_to<R: rand::Rng + ?Sized>(
    state: &TableSnapshot,
    strategy: &BettingStrategy,
    rng: &mut R,
) -> Option<usize> {
    let (floor, ceiling) = state.raise_bounds()?;
    // Fixed-Limit has exactly one legal raise-to, so there is nothing to size.
    if fixed_limit_increment(state).is_some() {
        return Some(floor);
    }
    let (n, d) = pick_bet_size(strategy, rng);
    Some(
        state
            .current_bet
            .saturating_add(state.pot.saturating_mul(n) / d)
            .clamp(floor, ceiling),
    )
}

/// EPIC-30 Phase 6: computes a bet amount (no current bet to call) that's
/// legal under the current betting structure, or `None` when no legal
/// voluntary bet exists.
///
/// `DEFECT_007`: an opening bet is a raise-from-zero — `Table::act_bet`
/// validates it through the very same `validate_raise` an actual raise goes
/// through — so it shares the window and the same infeasibility case. The
/// old `.max(big_blind).min(my_chips)` pair could produce an amount below
/// the minimum for the same reason `sized_raise_to` could.
/// Wraps a voluntary opening amount in the variant the engine advertises.
///
/// `DEFECT_007`: `to_call == 0` does not mean the betting is open. On the
/// big-blind option a bet already stands and this seat has merely matched it,
/// so re-opening is a **`Raise`**, not a **`Bet`** —
/// [`Table::legal_actions`](crate::casino::table::Table::legal_actions) says so
/// in as many words.
///
/// `Table::act_bet` accepts either, which is why this never surfaced as a
/// rejection, but the two are not interchangeable: a `Bet` records its
/// *absolute* amount as the raise increment rather than the delta over the
/// standing bet (doubling the next player's minimum re-raise), does not count
/// toward the per-street raise cap, and writes the wrong event to the log.
fn voluntary_open(state: &TableSnapshot, amount: usize) -> PlayerAction {
    if state.current_bet == 0 {
        PlayerAction::Bet(amount)
    } else {
        PlayerAction::Raise(amount)
    }
}

fn sized_bet_amount<R: rand::Rng + ?Sized>(
    state: &TableSnapshot,
    strategy: &BettingStrategy,
    rng: &mut R,
) -> Option<usize> {
    let (floor, ceiling) = state.raise_bounds()?;
    if fixed_limit_increment(state).is_some() {
        return Some(floor);
    }
    let (n, d) = pick_bet_size(strategy, rng);
    Some((state.pot.saturating_mul(n) / d).clamp(floor, ceiling))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__decider_tests {
    use super::*;
    use crate::casino::game::ForcedBets;
    use crate::casino::table::{Player, Seat, Seats, Table};
    use crate::games::GamePhase;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn make_snapshot(seat: u8) -> TableSnapshot<'static> {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        TableSnapshot::from_table(&table, seat)
    }

    /// `DEFECT_007` fixture: `short` starts a heads-up 400/800 hand with
    /// `short_stack` chips, posts the big blind, and faces a raise to
    /// `raise_to`. Returns the snapshot from the short stack's perspective.
    fn short_stack_facing_a_raise(short_stack: usize, raise_to: usize) -> TableSnapshot<'static> {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("Deep".to_string(), 100_000)),
            Seat::new(Player::new_with_chips("Short".to_string(), short_stack)),
        ]);
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(400, 800));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        table.act_raise(table.determine_utg(), raise_to).unwrap();
        let actor = table.next_to_act();
        TableSnapshot::from_table(&table, actor)
    }

    /// The exact defect: the floor was applied, then `.min(my_chips)` pulled the
    /// result straight back below it because `my_chips` excludes the posted blind.
    #[test]
    fn sized_raise_to_never_returns_below_the_minimum() {
        let snap = short_stack_facing_a_raise(2_400, 1_600);
        let mut rng = SmallRng::seed_from_u64(7);
        let strategy = BotProfile::loose_aggressive().betting_strategy.clone();

        let raise_to = sized_raise_to(&snap, &strategy, &mut rng).expect("a legal raise exists");
        assert!(
            raise_to >= snap.min_raise_to(),
            "raise_to {raise_to} is below the minimum {}",
            snap.min_raise_to()
        );
    }

    /// The ceiling must be the whole stack (behind + already committed), not the
    /// chips behind. Capping at `my_chips` under-shoves by the posted blind.
    #[test]
    fn sized_raise_to_never_exceeds_the_whole_stack() {
        let snap = short_stack_facing_a_raise(2_400, 1_600);
        let mut rng = SmallRng::seed_from_u64(7);
        let strategy = BotProfile::loose_aggressive().betting_strategy.clone();

        let raise_to = sized_raise_to(&snap, &strategy, &mut rng).expect("a legal raise exists");
        assert!(
            raise_to <= snap.my_total_chips(),
            "raise_to {raise_to} exceeds the stack"
        );
        assert!(
            raise_to > snap.my_chips,
            "a raise-to of {raise_to} should exceed the chips behind ({}) — \
             the posted blind counts toward it",
            snap.my_chips
        );
    }

    /// When the whole stack cannot reach the minimum, no raise of any size is
    /// legal and the sizing function must say so rather than clamp into it.
    #[test]
    fn sized_raise_to_is_none_when_no_legal_raise_exists() {
        let snap = short_stack_facing_a_raise(2_000, 1_600);
        let mut rng = SmallRng::seed_from_u64(7);
        let strategy = BotProfile::loose_aggressive().betting_strategy.clone();

        assert_eq!(None, snap.raise_bounds(), "fixture should have no legal raise");
        assert_eq!(None, sized_raise_to(&snap, &strategy, &mut rng));
    }

    /// The `sized_bet_amount` counterpart: an opening bet is validated against
    /// the same minimum, so a sub-minimum bet is equally illegal.
    #[test]
    fn sized_bet_amount_never_returns_below_the_minimum() {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 100_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 100_000)),
        ]);
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(400, 800));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let snap = TableSnapshot::from_table(&table, table.next_to_act());
        let mut rng = SmallRng::seed_from_u64(3);
        let strategy = BotProfile::tight_passive().betting_strategy.clone();

        let bet = sized_bet_amount(&snap, &strategy, &mut rng).expect("a legal bet exists");
        assert!(bet >= snap.min_raise_to(), "bet {bet} is below the minimum");
    }

    /// The property the four call sites exist to uphold: sweep the stack across
    /// the feasibility boundary for every shipped profile and assert that no
    /// `Raise` or `Bet` is ever illegal.
    #[test]
    fn decide_never_returns_a_raise_the_engine_would_reject() {
        for profile in BotProfile::default_profiles() {
            for short_stack in (900..=4_000).step_by(100) {
                for seed in 0..8u64 {
                    let snap = short_stack_facing_a_raise(short_stack, 1_600);
                    let mut rng = SmallRng::seed_from_u64(seed);
                    let action = RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng);
                    match action {
                        PlayerAction::Raise(n) | PlayerAction::Bet(n) => {
                            let (min, max) = snap.raise_bounds().unwrap_or_else(|| {
                                panic!(
                                    "{} returned {action:?} with stack {short_stack} \
                                     when no legal raise exists",
                                    profile.name
                                )
                            });
                            assert!(
                                n >= min && n <= max,
                                "{} returned {action:?} outside the legal window \
                                 [{min}, {max}] with stack {short_stack}",
                                profile.name
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Three seats limped to the big blind, who now has the option: `to_call`
    /// is 0 but a bet of one big blind already stands.
    fn big_blind_option() -> (Table, u8) {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("C".to_string(), 10_000)),
        ]);
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_call(utg).unwrap();
        let small_blind = table.next_to_act();
        table.act_call(small_blind).unwrap();
        let big_blind = table.next_to_act();
        (table, big_blind)
    }

    /// `DEFECT_007` (third instance): `to_call == 0` does not mean the betting is
    /// open. On the big-blind option a bet already stands, so re-opening it is a
    /// `Raise`; `Table::legal_actions` advertises exactly that and never a `Bet`.
    /// The two are not interchangeable — see
    /// `act_bet_over_a_standing_bet_matches_act_raise` in the table's own tests.
    #[test]
    fn decide_re_opens_with_a_raise_not_a_bet_on_the_big_blind_option() {
        let (table, big_blind) = big_blind_option();
        assert_eq!(0, table.to_call(big_blind), "fixture: the option, not a call");
        assert!(table.bet > 0, "fixture: a bet already stands");

        let snap = TableSnapshot::from_table(&table, big_blind);
        for profile in BotProfile::default_profiles() {
            for seed in 0..40u64 {
                let mut rng = SmallRng::seed_from_u64(seed);
                let action = RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng);
                assert!(
                    !matches!(action, PlayerAction::Bet(_)),
                    "{} returned {action:?} on the big-blind option; \
                     the engine advertises {:?}",
                    profile.name,
                    table.legal_actions(big_blind)
                );
            }
        }
    }

    /// The opening-bet path is unaffected: with no bet standing, `Bet` is still
    /// the right variant and `Raise` would be wrong.
    #[test]
    fn decide_opens_with_a_bet_when_no_bet_stands() {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 10_000)),
        ]);
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        table.act_call(table.determine_utg()).unwrap();
        table.act_check(table.next_to_act()).unwrap();
        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.seats.reset_state_in_hand();

        let actor = table.next_to_act();
        assert_eq!(0, table.bet, "fixture: the betting is open on the flop");

        let snap = TableSnapshot::from_table(&table, actor);
        let mut saw_a_bet = false;
        for profile in BotProfile::default_profiles() {
            for seed in 0..40u64 {
                let mut rng = SmallRng::seed_from_u64(seed);
                let action = RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng);
                assert!(
                    !matches!(action, PlayerAction::Raise(_)),
                    "{} returned {action:?} with no bet standing",
                    profile.name
                );
                saw_a_bet |= matches!(action, PlayerAction::Bet(_));
            }
        }
        assert!(saw_a_bet, "fixture should produce at least one opening bet");
    }

    #[test]
    fn rule_based_decider_returns_action() {
        let snap = make_snapshot(0);
        let profile = BotProfile::gto();
        let action = RuleBasedDecider.decide(&profile, &snap);
        let _ = action; // any valid action is acceptable
    }

    #[test]
    fn rule_based_decider_zero_chips_checks() {
        let mut snap = make_snapshot(0);
        snap.my_chips = 0;
        let profile = BotProfile::gto();
        let action = RuleBasedDecider.decide(&profile, &snap);
        assert_eq!(PlayerAction::Check, action);
    }

    #[test]
    fn rule_based_decider_all_reference_profiles() {
        for profile in [
            BotProfile::gto(),
            BotProfile::tight_passive(),
            BotProfile::loose_aggressive(),
        ] {
            let snap = make_snapshot(0);
            let _ = RuleBasedDecider.decide(&profile, &snap);
        }
    }

    #[test]
    fn rule_based_decider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RuleBasedDecider>();
    }

    #[test]
    fn bot_decider_as_trait_object() {
        let decider: Box<dyn BotDecider> = Box::new(RuleBasedDecider);
        let snap = make_snapshot(0);
        let profile = BotProfile::gto();
        let _ = decider.decide(&profile, &snap);
    }

    #[test]
    fn joker_decider_returns_action() {
        let decider = JokerDecider::new();
        let snap = make_snapshot(0);
        let profile = BotProfile::joker();
        let action = decider.decide(&profile, &snap);
        let _ = action;
    }

    #[test]
    fn joker_decider_on_new_hand_changes_profile() {
        // Run on_new_hand many times and collect the names of the chosen profiles.
        // With 8 possible profiles and enough trials, we expect to see at least 2 distinct names.
        let decider = JokerDecider::new();
        let snap = make_snapshot(0);
        let joker_profile = BotProfile::joker();

        let mut names = std::collections::HashSet::new();
        for _ in 0..50 {
            decider.on_new_hand();
            let active = decider.active.lock().unwrap();
            names.insert(active.name.clone());
        }
        // Statistically certain to see more than one distinct profile across 50 rolls.
        assert!(
            names.len() > 1,
            "expected variety after 50 new-hand rolls, got: {names:?}"
        );
        // Sanity-check that decide still works after multiple on_new_hand calls.
        let _ = decider.decide(&joker_profile, &snap);
    }

    #[test]
    fn joker_decider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<JokerDecider>();
    }

    #[test]
    fn joker_decider_as_trait_object() {
        let decider: Box<dyn BotDecider> = Box::new(JokerDecider::new());
        let snap = make_snapshot(0);
        let profile = BotProfile::joker();
        let _ = decider.decide(&profile, &snap);
    }

    #[test]
    fn joker_decider_debug() {
        let decider = JokerDecider::new();
        let s = format!("{decider:?}");
        assert!(s.contains("JokerDecider"), "debug output should mention JokerDecider");
    }

    // ── Helpers for bluff / c-bet / check-raise tests ────────────────────────

    /// Build a minimal [`BotProfile`] with explicit frequency values for testing.
    fn make_profile(aggression: u8, bluff: u8, check_raise: u8, cbet: u8) -> BotProfile {
        use crate::analysis::gto::solver_config::BetSize;
        use crate::bot::betting_strategy::BettingStrategy;
        use crate::bot::profile::PlayStyle;
        use crate::bot::range_strategy::RangeStrategy;
        BotProfile::new(
            "test".to_string(),
            "test profile".to_string(),
            PlayStyle::new("test"),
            RangeStrategy::new("AA", "AA", "KK", cbet),
            BettingStrategy::new(aggression, bluff, check_raise, vec![BetSize::half_pot()]),
        )
    }

    /// Run `n` decisions using a seeded RNG and count each outcome variant.
    /// Returns `(bets, checks, raises, calls, folds, all_ins)`.
    fn count_with_seed(
        profile: &BotProfile,
        snap: &TableSnapshot,
        seed: u64,
        n: usize,
    ) -> (usize, usize, usize, usize, usize, usize) {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let mut rng = SmallRng::seed_from_u64(seed);
        let (mut bets, mut checks, mut raises, mut calls, mut folds, mut all_ins) = (0, 0, 0, 0, 0, 0);
        for _ in 0..n {
            match RuleBasedDecider::decide_with_rng(profile, snap, &mut rng) {
                PlayerAction::Bet(_) => bets += 1,
                PlayerAction::Check => checks += 1,
                PlayerAction::Raise(_) => raises += 1,
                PlayerAction::Call => calls += 1,
                PlayerAction::Fold => folds += 1,
                PlayerAction::AllIn => all_ins += 1,
            }
        }
        (bets, checks, raises, calls, folds, all_ins)
    }

    // ── 100 % / 0 % boundary tests ───────────────────────────────────────────

    /// 100 % c-bet frequency on the flop always produces a Bet.
    #[test]
    fn cbet_100_always_bets_on_flop() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let profile = make_profile(0, 0, 0, 100);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingFlop;
        snap.to_call = 0;
        snap.pot = 200;

        let mut rng = SmallRng::seed_from_u64(99);
        for _ in 0..50 {
            assert!(
                matches!(
                    RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                    PlayerAction::Bet(_)
                ),
                "100% c-bet must always Bet"
            );
        }
    }

    /// 0 % c-bet on the flop with 0 % bluff always produces a Check.
    #[test]
    fn cbet_0_and_bluff_0_always_checks_on_flop() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let profile = make_profile(0, 0, 0, 0);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingFlop;
        snap.to_call = 0;

        let mut rng = SmallRng::seed_from_u64(7);
        for _ in 0..50 {
            assert_eq!(
                PlayerAction::Check,
                RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                "0% cbet + 0% bluff must always Check on flop"
            );
        }
    }

    /// 100 % bluff frequency (with 0 % aggression/cbet) always bets postflop.
    #[test]
    fn bluff_100_always_bets_postflop() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // Turn — not flop (cbet path) and not preflop (bluff suppressed)
        let profile = make_profile(0, 100, 0, 0);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingTurn;
        snap.to_call = 0;
        snap.pot = 200;

        let mut rng = SmallRng::seed_from_u64(13);
        for _ in 0..50 {
            assert!(
                matches!(
                    RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                    PlayerAction::Bet(_)
                ),
                "100% bluff on turn must always Bet"
            );
        }
    }

    /// bluff_frequency is never applied preflop — always Check with 0 % aggression.
    #[test]
    fn bluff_never_fires_preflop() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let profile = make_profile(0, 100, 0, 0);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingPreFlop;
        snap.to_call = 0;

        let mut rng = SmallRng::seed_from_u64(21);
        for _ in 0..50 {
            assert_eq!(
                PlayerAction::Check,
                RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                "bluff must not fire preflop"
            );
        }
    }

    /// 100 % check-raise frequency always raises when checked_this_street and facing a bet.
    #[test]
    fn check_raise_100_always_raises() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let profile = make_profile(0, 0, 100, 50);
        let mut snap = make_snapshot(0);
        snap.to_call = 100;
        snap.current_bet = 100;
        snap.min_raise = 100;
        snap.pot = 300;
        snap.checked_this_street = true;

        let mut rng = SmallRng::seed_from_u64(5);
        for _ in 0..50 {
            assert!(
                matches!(
                    RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                    PlayerAction::Raise(_)
                ),
                "100% check-raise must always Raise when checked_this_street"
            );
        }
    }

    /// 0 % check-raise never raises via the check-raise path (falls through to
    /// call/fold based on aggression_factor).
    #[test]
    fn check_raise_0_never_check_raises() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // 100% aggression so we get Calls, not Folds — just verifying no check-raise Raises
        let profile = make_profile(100, 0, 0, 50);
        let mut snap = make_snapshot(0);
        snap.to_call = 100;
        snap.current_bet = 100;
        snap.min_raise = 100;
        snap.pot = 300;
        snap.checked_this_street = true;

        let mut rng = SmallRng::seed_from_u64(3);
        for _ in 0..50 {
            // With aggression=100 and check_raise=0 the raise path (aggr*0.25) may
            // still fire — that's fine. The check-raise-specific path should not
            // dominate; outcomes should be Call or Raise (from the normal raise path),
            // never a check-raise exclusive of the normal flow.
            let action = RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng);
            assert!(
                matches!(action, PlayerAction::Call | PlayerAction::Raise(_)),
                "with aggression=100 and to_call set, expected Call or Raise, got {action:?}"
            );
        }
    }

    // ── Intermediate-probability statistical tests ────────────────────────────
    //
    // These use a fixed seed (deterministic) and run N=1000 trials.
    // Bounds are intentionally wide (±25 pp) to avoid flakiness while still
    // catching the case where the field has no effect at all.

    /// C-bet frequency 50 % on the flop bets roughly half the time.
    #[test]
    fn cbet_50_bets_approximately_half() {
        let profile = make_profile(0, 0, 0, 50);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingFlop;
        snap.to_call = 0;
        snap.pot = 200;

        let (bets, checks, ..) = count_with_seed(&profile, &snap, 42, 1_000);
        assert_eq!(bets + checks, 1_000, "only Bet or Check expected");
        // Expect 50 % ± 25 pp  →  250..=750
        assert!(
            (250..=750).contains(&bets),
            "c-bet 50%: expected ~500 bets out of 1000, got {bets}"
        );
    }

    /// Bluff frequency 30 % bets roughly 30 % of the time on the turn
    /// when aggression and cbet are both 0.
    #[test]
    fn bluff_30_bets_approximately_30_percent() {
        let profile = make_profile(0, 30, 0, 0);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingTurn;
        snap.to_call = 0;
        snap.pot = 200;

        let (bets, checks, ..) = count_with_seed(&profile, &snap, 17, 1_000);
        assert_eq!(bets + checks, 1_000, "only Bet or Check expected");
        // Expect 30 % ± 15 pp  →  150..=450
        assert!(
            (150..=450).contains(&bets),
            "bluff 30%: expected ~300 bets out of 1000, got {bets}"
        );
    }

    /// Check-raise frequency 40 % raises roughly 40 % of the time when
    /// checked_this_street is true and facing a bet.
    #[test]
    fn check_raise_40_raises_approximately_40_percent() {
        // aggression 0 so raises only come from the check-raise path
        let profile = make_profile(0, 0, 40, 0);
        let mut snap = make_snapshot(0);
        snap.to_call = 100;
        snap.current_bet = 100;
        snap.min_raise = 100;
        snap.pot = 300;
        snap.checked_this_street = true;

        let (_, _, raises, _, folds, _) = count_with_seed(&profile, &snap, 88, 1_000);
        assert_eq!(raises + folds, 1_000, "only Raise or Fold expected with aggression=0");
        // Expect 40 % ± 20 pp  →  200..=600
        assert!(
            (200..=600).contains(&raises),
            "check-raise 40%: expected ~400 raises out of 1000, got {raises}"
        );
    }

    // ── Position-aware routing tests ─────────────────────────────────────────

    /// GTO profile has a Playbook giving BTN higher aggression than the flat default.
    /// With 100 % BTN aggression from the Playbook (verified via betting_for), the
    /// decider should bet on every turn when seat == button and seat_count == 6.
    #[test]
    fn rule_based_decider_uses_playbook_aggression_for_btn() {
        use crate::casino::position::Position;

        let profile = BotProfile::gto();

        // Verify the Playbook actually gives BTN different aggression than the flat default.
        let btn_aggr = profile.betting_for(6, Position::BTN).aggression_factor;
        let flat_aggr = profile.betting_strategy.aggression_factor;
        assert!(
            btn_aggr > flat_aggr,
            "GTO BTN aggression ({btn_aggr}) should exceed flat ({flat_aggr})"
        );

        // Build a snapshot where seat_count == 6, dealer_button == Some(0), seat == 0 (BTN).
        let mut snap = make_snapshot(0);
        snap.seat_count = 6;
        snap.dealer_button = Some(0);
        snap.phase = crate::games::GamePhase::BettingTurn;
        snap.to_call = 0;
        snap.pot = 200;

        let (bets_btn, checks_btn, ..) = count_with_seed(&profile, &snap, 55, 1_000);

        // Same profile but with dealer_button = None → falls back to flat aggression.
        snap.dealer_button = None;
        let (bets_flat, checks_flat, ..) = count_with_seed(&profile, &snap, 55, 1_000);

        // BTN path should produce more bets than the flat fallback.
        assert!(
            bets_btn > bets_flat,
            "BTN bets ({bets_btn}) should exceed flat bets ({bets_flat}) given higher BTN aggression"
        );
        let _ = (checks_btn, checks_flat);
    }

    // ── StreetAggression routing tests ───────────────────────────────────────

    /// 100% preflop street aggression always bets preflop.
    #[test]
    fn street_aggression_100_preflop_always_bets() {
        use crate::bot::betting_strategy::{Percentage, StreetAggression};
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        let mut profile = make_profile(0, 0, 0, 0);
        profile.betting_strategy.street_aggression = Some(StreetAggression {
            preflop: Percentage::new(100),
            flop: None,
            turn: None,
            river: None,
        });

        let mut snap = make_snapshot(0);
        snap.phase = crate::games::GamePhase::BettingPreFlop;
        snap.to_call = 0;
        snap.pot = 200;

        let mut rng = SmallRng::seed_from_u64(7);
        for _ in 0..50 {
            assert!(
                matches!(
                    RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                    PlayerAction::Bet(_)
                ),
                "100% preflop street aggression must always Bet preflop"
            );
        }
    }

    // ── HandStrengthDecisions tests ──────────────────────────────────────────

    /// Helper: build a snapshot with specific hole cards and board.
    fn make_snapshot_with_cards(
        hole: &str,
        board: &str,
        to_call: usize,
        pot: usize,
        phase: crate::games::GamePhase,
    ) -> TableSnapshot<'static> {
        use crate::cards::Cards;
        use std::str::FromStr;
        let mut snap = make_snapshot(0);
        snap.hole_cards = Cards::from_str(hole).unwrap();
        snap.board = if board.is_empty() {
            Cards::default()
        } else {
            Cards::from_str(board).unwrap()
        };
        snap.to_call = to_call;
        snap.current_bet = to_call;
        snap.min_raise = 100;
        snap.pot = pot;
        snap.phase = phase;
        snap
    }

    /// AA preflop facing a bet: equity(1.0) > pot_odds * 2 → always Raise or Call.
    #[test]
    fn calls_with_equity_above_pot_odds() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // open_raise = "AA" so AA is in range → equity = 1.0
        let profile = make_profile(50, 10, 0, 50);
        // pot_odds = 100 / (300 + 100) = 0.25; equity(1.0) > 0.5 → raise
        let snap = make_snapshot_with_cards("A♠ A♥", "", 100, 300, crate::games::GamePhase::BettingPreFlop);
        let mut rng = SmallRng::seed_from_u64(42);
        for _ in 0..20 {
            let action = RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng);
            assert!(
                matches!(action, PlayerAction::Raise(_) | PlayerAction::Call),
                "AA vs pot_odds=0.25 should Raise or Call, got {action:?}"
            );
        }
    }

    /// 72o preflop facing a bet with bluff_frequency=0: equity(0.0) < pot_odds → always Fold.
    #[test]
    fn folds_below_pot_odds_no_bluff() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // open_raise = "AA" so 72o is NOT in range → equity = 0.0
        let profile = make_profile(50, 0, 0, 50);
        // pot_odds = 100/400 = 0.25; equity(0.0) < pot_odds → fold
        let snap = make_snapshot_with_cards("7♠ 2♦", "", 100, 300, crate::games::GamePhase::BettingPreFlop);
        let mut rng = SmallRng::seed_from_u64(7);
        for _ in 0..50 {
            assert_eq!(
                PlayerAction::Fold,
                RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                "72o vs pot_odds=0.25 with bluff=0 must Fold"
            );
        }
    }

    /// 72o on K-Q-J flop with bluff_frequency=100 and no outstanding bet: always Bet.
    #[test]
    fn bluffs_despite_weak_hand() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // open_raise = "AA", 72o is out of range; postflop equity of 72o on KQJ ≈ 0.06
        let profile = make_profile(0, 100, 0, 0);
        let snap = make_snapshot_with_cards("7♠ 2♦", "K♠ Q♦ J♣", 0, 200, crate::games::GamePhase::BettingFlop);
        let mut rng = SmallRng::seed_from_u64(13);
        for _ in 0..50 {
            assert!(
                matches!(
                    RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                    PlayerAction::Bet(_)
                ),
                "bluff_freq=100 with weak hand must always Bet"
            );
        }
    }

    /// The raise gate must be probabilistic: with AA preflop (equity=1.0) and
    /// pot_odds=0.25, the bot enters the strong-hand branch but should sometimes
    /// Raise and sometimes Call across different RNG seeds.
    ///
    /// This test catches the regression where two bots with strong hands escalate
    /// indefinitely because the raise branch was unconditional.
    #[test]
    fn raise_gate_is_probabilistic_not_deterministic() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // GTO open_raise includes AA → preflop equity = 1.0
        // pot_odds = 100 / (300+100) = 0.25; equity (1.0) > pot_odds*2 (0.5) → raise branch
        let profile = BotProfile::gto();
        let snap = make_snapshot_with_cards("A♠ A♥", "", 100, 300, crate::games::GamePhase::BettingPreFlop);

        let mut saw_raise = false;
        let mut saw_call = false;

        for seed in 0u64..200 {
            let mut rng = SmallRng::seed_from_u64(seed);
            match RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng) {
                PlayerAction::Raise(_) => saw_raise = true,
                PlayerAction::Call => saw_call = true,
                _ => {}
            }
            if saw_raise && saw_call {
                break;
            }
        }

        assert!(saw_raise, "raise gate must sometimes raise when equity is strong");
        assert!(
            saw_call,
            "raise gate must sometimes call to prevent bot escalation loops"
        );
    }

    /// 0% river street aggression always checks on the river with no outstanding bet.
    #[test]
    fn street_aggression_0_river_always_checks() {
        use crate::bot::betting_strategy::{Percentage, StreetAggression};
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        // Use aggression_factor=100 so only the river override brings it to zero.
        let mut profile = make_profile(100, 0, 0, 0);
        profile.betting_strategy.street_aggression = Some(StreetAggression {
            preflop: None,
            flop: None,
            turn: None,
            river: Percentage::new(0),
        });

        let mut snap = make_snapshot(0);
        snap.phase = crate::games::GamePhase::BettingRiver;
        snap.to_call = 0;

        let mut rng = SmallRng::seed_from_u64(13);
        for _ in 0..50 {
            assert_eq!(
                PlayerAction::Check,
                RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                "0% river street aggression must always Check on river"
            );
        }
    }

    // ── EPIC-26 Phase 3 regression: deciders ignore opponent_stats ──────────

    /// Tripwire for EPIC-26 Phase 3's "non-behavior-changing" contract.
    ///
    /// Future exploitative deciders are expected to *read* `opponent_stats`,
    /// at which point this test should be removed or updated. As long as the
    /// shipped `RuleBasedDecider` ignores it, two snapshots identical except
    /// for the registry borrow must produce the same action under the same
    /// RNG seed.
    #[cfg(feature = "player-stats")]
    #[test]
    fn rule_based_decider_ignores_opponent_stats() {
        use crate::analysis::player_stats::StatsRegistry;
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let registry = StatsRegistry::new();
        let profile = BotProfile::tight_passive();

        let snap_no_stats = TableSnapshot::from_table(&table, 0);
        let snap_with_stats = TableSnapshot::from_table_with_stats(&table, 0, &registry);

        // 64 trials each with a fresh seed pair — independent RNGs reseeded
        // identically per trial. Locks in determinism over a sweep, not just
        // a single arbitrary seed.
        for seed in 0u64..64 {
            let mut rng_a = SmallRng::seed_from_u64(seed);
            let mut rng_b = SmallRng::seed_from_u64(seed);
            let action_a = RuleBasedDecider::decide_with_rng(&profile, &snap_no_stats, &mut rng_a);
            let action_b = RuleBasedDecider::decide_with_rng(&profile, &snap_with_stats, &mut rng_b);
            assert_eq!(
                action_a, action_b,
                "seed {seed}: decider must produce identical actions with vs without registry"
            );
        }
    }

    // ── EPIC-36 Phase 2: graded decision-capability knobs ────────────────────

    use crate::bot::decision_config::RangeMode;

    /// pot_odds discipline scales the call threshold. With `discipline = 1.0`
    /// (default) a weak made hand below break-even folds; with `discipline = 0.0`
    /// pot odds are ignored and the same hand calls.
    #[test]
    fn pot_odds_discipline_zero_calls_where_strict_folds() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // 72o on KQJ: proxy equity ≈ 0.06; pot_odds = 100/400 = 0.25. bluff = 0.
        let snap = make_snapshot_with_cards("7♠ 2♦", "K♠ Q♦ J♣", 100, 300, crate::games::GamePhase::BettingFlop);

        let mut strict = make_profile(50, 0, 0, 50);
        strict.decision.pot_odds.discipline = 1.0;
        let mut loose = make_profile(50, 0, 0, 50);
        loose.decision.pot_odds.discipline = 0.0;

        for seed in 0u64..40 {
            let mut rng_s = SmallRng::seed_from_u64(seed);
            assert_eq!(
                PlayerAction::Fold,
                RuleBasedDecider::decide_with_rng(&strict, &snap, &mut rng_s),
                "discipline 1.0: equity 0.06 < pot_odds 0.25 with bluff 0 must Fold"
            );
            let mut rng_l = SmallRng::seed_from_u64(seed);
            assert_eq!(
                PlayerAction::Call,
                RuleBasedDecider::decide_with_rng(&loose, &snap, &mut rng_l),
                "discipline 0.0: pot odds ignored, equity 0.06 > 0 must Call"
            );
        }
    }

    /// The equity knob replaces the hand-rank proxy with the real multi-way
    /// engine. An overpair's proxy strength (absolute rank vs a random full
    /// hand) understates its true equity vs a single opponent's unknown hand.
    #[cfg(feature = "equity")]
    #[test]
    fn equity_exact_exceeds_proxy_for_overpair() {
        use crate::bot::decision_config::EquityMode;
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let snap = make_snapshot_with_cards("A♠ A♥", "2♦ 7♣ 9♠", 0, 200, crate::games::GamePhase::BettingFlop);

        let mut off = make_profile(50, 0, 0, 50);
        off.decision.equity = EquityMode::Off;
        let mut exact = make_profile(50, 0, 0, 50);
        exact.decision.equity = EquityMode::Exact;

        let mut rng = SmallRng::seed_from_u64(42);
        let proxy = hand_equity(&off, &snap, &mut rng).expect("proxy equity");
        let mut rng2 = SmallRng::seed_from_u64(42);
        let real = hand_equity(&exact, &snap, &mut rng2).expect("real equity");

        assert!(proxy < 0.65, "overpair proxy should understate: {proxy}");
        assert!(real > 0.75, "real overpair equity vs 1 villain should be high: {real}");
        assert!(real > proxy, "equity knob must raise the estimate: {real} !> {proxy}");
    }

    /// Position-aware ranges must consult the playbook, producing a different
    /// open frequency than the flat range for at least one hand.
    #[test]
    fn position_aware_ranges_differ_from_flat() {
        use crate::cards::Cards;
        use std::str::FromStr;
        // gto ships a 6-max playbook; seat 0 on the button in 6-max.
        let mut snap = make_snapshot(0);
        snap.seat_count = 6;
        snap.dealer_button = Some(0);
        snap.logical_seat = Some(0);
        snap.phase = crate::games::GamePhase::BettingPreFlop;

        let mut flat = BotProfile::gto();
        flat.decision.ranges = RangeMode::Flat;
        let mut pa = BotProfile::gto();
        pa.decision.ranges = RangeMode::PositionAware;

        let hands = [
            "A♠ 5♠",
            "K♠ 9♠",
            "Q♠ 8♠",
            "J♠ 7♠",
            "7♠ 6♠",
            "5♠ 4♠",
            "T♠ 8♠",
            "9♦ 8♦",
            "A♦ 2♦",
            "K♥ T♠",
        ];
        let mut differs = false;
        for h in hands {
            snap.hole_cards = Cards::from_str(h).unwrap();
            let f = preflop_open_frequency(&flat, &snap);
            let p = preflop_open_frequency(&pa, &snap);
            if (f - p).abs() > 1e-9 {
                differs = true;
                break;
            }
        }
        assert!(differs, "position-aware ranges must differ from flat for some hand");
    }

    /// A profile with no playbook falls back to the flat range even when
    /// `ranges = position_aware`, so its open frequencies are unchanged.
    #[test]
    fn position_aware_without_playbook_matches_flat() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let mut snap = make_snapshot(0);
        snap.seat_count = 6;
        snap.dealer_button = Some(0);
        snap.logical_seat = Some(0);
        snap.phase = crate::games::GamePhase::BettingPreFlop;

        // maniac() has no playbook.
        let flat = BotProfile::maniac();
        let mut pa = BotProfile::maniac();
        pa.decision.ranges = RangeMode::PositionAware;

        for h in ["A♠ A♥", "7♠ 2♦", "K♠ Q♦"] {
            snap.hole_cards = Cards::from_str(h).unwrap();
            let f = preflop_open_frequency(&flat, &snap);
            let p = preflop_open_frequency(&pa, &snap);
            assert!(
                (f - p).abs() < 1e-9,
                "no playbook: position_aware must match flat for {h}"
            );
        }
    }

    #[cfg(feature = "player-stats")]
    #[test]
    fn exploit_off_returns_none_and_heavy_engages_with_stats() {
        use crate::analysis::player_stats::StatsRegistry;
        use crate::bot::decision_config::ExploitMode;
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let registry = StatsRegistry::new();
        let snap = TableSnapshot::from_table_with_stats(&table, 0, &registry);

        let mut p = BotProfile::tight_passive();
        assert!(exploit_profile(&p, &snap).is_none(), "exploit Off must not adjust");
        p.decision.exploit = ExploitMode::Heavy;
        assert!(
            exploit_profile(&p, &snap).is_some(),
            "exploit Heavy with a registry attached must engage the adjust path"
        );
    }

    #[cfg(feature = "player-stats")]
    #[test]
    fn exploit_heavy_without_stats_returns_none() {
        use crate::bot::decision_config::ExploitMode;
        let snap = make_snapshot(0); // no opponent_stats attached
        let mut p = BotProfile::tight_passive();
        p.decision.exploit = ExploitMode::Heavy;
        assert!(
            exploit_profile(&p, &snap).is_none(),
            "exploit knob must no-op when no stats registry is attached"
        );
    }
}
