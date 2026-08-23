//! The pure betting kernel — SPIKE.
//!
//! Faithful extraction of the card-free betting logic that today exists twice:
//! `pkcore/src/casino/table/actions.rs` (`&mut self`) and
//! `pkcore/src/casino/table_celled.rs` (`&self` + cells). Every function here
//! is a **pure transition**: `(&HandBetting, inputs) -> Result<Step, KError>`,
//! where `Step` carries the successor state and the emitted events. No
//! interior mutability, no I/O, no cards.
//!
//! Source lines cited against branch `EPIC-79b` @ `9367380`.
//!
//! Faithfulness notes (deviations are named, nothing is silent):
//! - `BettingStructure` is ported for NoLimit in full; FixedLimit carries only
//!   the raise cap needed by `validate_raise`. PotLimit / bet tiers / stud
//!   completion are production port work, not spike work.
//! - `utg` is an input to the state rather than derived from button + phase:
//!   `first_to_act_this_street` (table.rs:849) is street/variant dispatch, a
//!   separate concern from betting legality.
//! - Events mirror the `TableAction` variants the extracted paths emit.

/// Errors, mirroring the `PKError` variants the extracted paths return.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KError {
    OutOfOrder { seat: u8 },
    InsufficientIncrement,
    RaiseCapReached,
    ExceedsBettingCap,
    InsufficientChips,
    InvalidAction,
    InvalidTableAction,
    InvalidSeatNumber,
}

/// Mirror of `PlayerState` (`src/casino/state.rs:164`), betting-relevant subset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerState {
    Ready,
    YetToAct,
    Check,
    Blind(usize),
    Bet(usize),
    Call(usize),
    Raise(usize),
    ReRaise(usize),
    AllIn(usize),
    Fold,
    Out,
}

impl PlayerState {
    /// `src/casino/state.rs:194`.
    pub fn amount(&self) -> usize {
        match *self {
            PlayerState::Blind(a)
            | PlayerState::Bet(a)
            | PlayerState::Call(a)
            | PlayerState::Raise(a)
            | PlayerState::ReRaise(a)
            | PlayerState::AllIn(a) => a,
            _ => 0,
        }
    }
    pub fn is_active(&self) -> bool {
        !matches!(self, PlayerState::Fold | PlayerState::Out)
    }
    pub fn is_in_hand(&self) -> bool {
        self.is_active() && !matches!(self, PlayerState::Ready)
    }
    pub fn is_blind(&self) -> bool {
        matches!(self, PlayerState::Blind(_))
    }
    pub fn is_check(&self) -> bool {
        matches!(self, PlayerState::Check)
    }
    /// `state.rs:378` — the Rule 47-A "has not really acted yet" predicate.
    pub fn is_yet_to_act_or_blind(&self) -> bool {
        matches!(self, PlayerState::YetToAct | PlayerState::Blind(_))
    }
}

/// One seat's betting state — the card-free slice of
/// `Player` (`src/casino/table/player.rs:28`) + `Seat.bet_level_when_last_acted`
/// (`src/casino/table/seat.rs:46`, DEFECT_010).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SeatBetting {
    /// Remaining stack (chips not yet committed this round).
    pub chips: usize,
    /// Chips committed to the current betting round.
    pub bet: usize,
    /// Cumulative chips committed across all rounds of the current hand.
    pub chips_in_play: usize,
    pub state: PlayerState,
    /// Table-level `bet` immediately after this seat last voluntarily acted
    /// (TDA 2024 Rule 47-A, DEFECT_010).
    pub bet_level_when_last_acted: usize,
    /// Empty-seat marker (spike stand-in for `Seat::is_empty`).
    pub occupied: bool,
}

impl SeatBetting {
    pub fn with_chips(chips: usize) -> Self {
        SeatBetting {
            chips,
            bet: 0,
            chips_in_play: 0,
            state: PlayerState::YetToAct,
            bet_level_when_last_acted: 0,
            occupied: true,
        }
    }
    /// `player.rs` `total_chip_count`: stack + current-round bet.
    pub fn total_chip_count(&self) -> usize {
        self.chips + self.bet
    }
    /// `player.rs` `is_all_in` heuristic: felted with chips committed.
    pub fn is_all_in(&self) -> bool {
        (self.chips == 0 && self.bet > 0) || matches!(self.state, PlayerState::AllIn(_))
    }
    pub fn is_in_hand(&self) -> bool {
        self.occupied && self.state.is_in_hand()
    }
}

/// Betting structure — NoLimit ported in full; FixedLimit only as far as the
/// per-street raise cap that `validate_raise` (actions.rs:345) consults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Betting {
    NoLimit,
    FixedLimitLite { raise_cap: u8 },
}

impl Betting {
    pub fn is_no_limit(&self) -> bool {
        matches!(self, Betting::NoLimit)
    }
    /// `BettingStructure::cap_reached` as consulted by actions.rs:350.
    pub fn cap_reached(&self, raises_this_street: u8) -> bool {
        match *self {
            Betting::NoLimit => false,
            Betting::FixedLimitLite { raise_cap } => raises_this_street >= raise_cap,
        }
    }
    /// `BettingStructure::max_raise`, NoLimit arm: the ceiling is the seat's
    /// whole stack — max raise-to == total_chip_count (actions.rs:314 path).
    pub fn max_raise(&self, _bet: usize, seat_bet: usize, stack: usize) -> usize {
        match *self {
            Betting::NoLimit => stack, // stack already == total_chip_count via caller
            Betting::FixedLimitLite { .. } => seat_bet + stack, // not exercised by spike tests
        }
    }
}

/// The hand's betting state — the card-free slice of `TableOf<S>`
/// (`src/casino/table.rs:83`): `bet`, `raise_increment`, `pot`,
/// `raises_this_street`, `actions_this_street`, `chip_actions_this_street`,
/// plus the seats. A plain value. `Clone + Eq` derive cleanly — nothing here
/// is generic and nothing is a cell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandBetting {
    pub seats: Vec<SeatBetting>,
    pub bet: usize,
    pub raise_increment: usize,
    pub pot: usize,
    pub raises_this_street: u8,
    pub actions_this_street: u8,
    pub chip_actions_this_street: u8,
    pub big_blind: usize,
    pub betting: Betting,
    /// First-to-act for the current street. Derived by variant/phase dispatch
    /// outside the kernel (`first_to_act_this_street`, table.rs:849).
    pub utg: u8,
}

/// Events, mirroring the `TableAction` variants the extracted paths log.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    Raise(u8, usize),
    Call(u8, usize),
    AllIn(u8, usize),
    Check(u8),
    ActionTo(u8),
    InvalidPlayerAction(u8),
}

/// A successful transition: the successor state plus what it logged.
/// Value-in/value-out — this *is* the immutable table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Step {
    pub next: HandBetting,
    pub events: Vec<Event>,
    /// The acting seat's remaining chips (the `Ok(usize)` the originals return).
    pub returned: usize,
}

// ───────────────────────── read-only queries ─────────────────────────
// Extracted verbatim in behavior; each cites its origin. These are the
// advisory surface — `raise_bounds`/`legal_actions` — and they call the SAME
// validators the transitions call, preserving audit P9b/P9j.1 by construction.

impl HandBetting {
    /// `Seats::current_bet` (seats.rs:130-ish): max seat bet this street.
    pub fn current_bet(&self) -> usize {
        self.seats.iter().map(|s| s.bet).max().unwrap_or(0)
    }

    /// `Seats::has_everyone_bet` (seats.rs).
    pub fn has_everyone_bet(&self) -> bool {
        !self
            .seats
            .iter()
            .any(|s| s.occupied && s.is_in_hand() && s.state.is_yet_to_act_or_blind())
    }

    /// `Seats::last_aggressor` (seats.rs:270-ish) — the DEFECT_022 fix.
    pub fn last_aggressor(&self) -> Option<u8> {
        let current_bet = self.current_bet();
        if current_bet == 0 {
            return None;
        }
        for (idx, seat) in self.seats.iter().enumerate() {
            if !seat.occupied || seat.bet != current_bet {
                continue;
            }
            if matches!(
                seat.state,
                PlayerState::Blind(_)
                    | PlayerState::Bet(_)
                    | PlayerState::Raise(_)
                    | PlayerState::ReRaise(_)
                    | PlayerState::AllIn(_)
            ) {
                return u8::try_from(idx).ok();
            }
        }
        None
    }

    /// `Seats::next_to_act` (seats.rs:305) — scan clockwise of the last
    /// aggressor, falling back to `utg` (DEFECT_022), then
    /// `Table::next_to_act` (table.rs:821) unwraps to utg.
    pub fn next_to_act(&self) -> u8 {
        self.next_to_act_inner().unwrap_or(self.utg)
    }

    fn next_to_act_inner(&self) -> Option<u8> {
        let size = self.seats.len();
        if size == 0 {
            return None;
        }
        let current_bet = self.current_bet();
        let everyone_has_bet = self.has_everyone_bet();
        let start = match self.last_aggressor() {
            Some(aggressor) => (aggressor as usize + 1) % size,
            None => self.utg as usize,
        };
        for step in 0..size {
            let idx = (start + step) % size;
            let seat = &self.seats[idx];
            if !seat.occupied || !seat.is_in_hand() || seat.is_all_in() {
                continue;
            }
            if seat.state.is_blind() {
                return u8::try_from(idx).ok();
            }
            if matches!(seat.state, PlayerState::YetToAct) {
                return u8::try_from(idx).ok();
            }
            if seat.state.is_check() && current_bet == 0 {
                continue;
            }
            if seat.state.is_in_hand() && everyone_has_bet && seat.bet < current_bet {
                return u8::try_from(idx).ok();
            }
        }
        None
    }

    /// `Table::min_raise` (table.rs:1248), NoLimit arm: `raise_increment` if
    /// non-zero, else the big blind.
    pub fn min_raise(&self) -> usize {
        if self.raise_increment != 0 {
            self.raise_increment
        } else {
            self.big_blind
        }
    }

    /// `Table::min_raise_to` (table.rs:1295), hold'em path (no stud
    /// completion in the spike): `bet + min_raise()`.
    pub fn min_raise_to(&self) -> usize {
        self.bet + self.min_raise()
    }

    /// `Table::to_call` via `Seats::to_call` (seats.rs:117).
    pub fn to_call(&self, seat: u8) -> usize {
        let highest = self.current_bet();
        self.seats
            .get(seat as usize)
            .map(|s| highest.saturating_sub(s.bet))
            .unwrap_or(0)
    }

    /// `Table::max_raise_for` (actions.rs:314).
    pub fn max_raise_for(&self, seat: u8) -> usize {
        let Some(s) = self.seats.get(seat as usize) else {
            return 0;
        };
        self.betting.max_raise(self.bet, s.bet, s.total_chip_count())
    }

    /// `Table::validate_raise` (actions.rs:345) — the single source of raise
    /// legality, executed by `act_raise` before mutating and queried by
    /// `raise_bounds`, so the advisory and mutating surfaces cannot drift
    /// (audit P9b/P9j.1). Preserved as-is.
    pub fn validate_raise(&self, seat: u8, amount: usize) -> Result<(), KError> {
        if amount < self.min_raise_to() {
            return Err(KError::InsufficientIncrement);
        }
        if self.betting.cap_reached(self.raises_this_street) {
            return Err(KError::RaiseCapReached);
        }
        if amount > self.max_raise_for(seat) {
            return Err(KError::ExceedsBettingCap);
        }
        Ok(())
    }

    /// `Table::is_reopen_gated` (actions.rs:436) — TDA 2024 Rule 47-A
    /// (DEFECT_010), including the cumulative clause and the big-blind-option
    /// safety, both of which fall out of measuring against
    /// `bet_level_when_last_acted`.
    pub fn is_reopen_gated(&self, seat: u8) -> bool {
        if !self.betting.is_no_limit() {
            // Spike carries NL + PL under one arm; FixedLimitLite is excluded
            // exactly as the original excludes FixedLimit.
            return false;
        }
        let Some(s) = self.seats.get(seat as usize) else {
            return false;
        };
        let has_acted = !s.state.is_yet_to_act_or_blind();
        let facing = self.bet.saturating_sub(s.bet_level_when_last_acted);
        has_acted && facing < self.min_raise()
    }

    /// `Table::raise_bounds` (actions.rs:368) — verbatim structure.
    pub fn raise_bounds(&self, seat: u8) -> Option<(usize, usize)> {
        if self.is_reopen_gated(seat) {
            return None;
        }
        let min = self.min_raise_to();
        if self.validate_raise(seat, min).is_err() {
            return None;
        }
        Some((min, self.max_raise_for(seat)))
    }
}

// ───────────────────────── pure transitions ─────────────────────────

/// `Player::act_bet_internal` (`src/casino/table/player.rs`) — verbatim
/// behavior, on a value.
fn seat_bet_internal(seat: &SeatBetting, bet_type: PlayerState) -> Result<SeatBetting, KError> {
    if bet_type.amount() == 0 {
        return Err(KError::InvalidAction);
    }
    if bet_type.amount() > seat.total_chip_count() {
        return Err(KError::InsufficientChips);
    }
    if !seat.state.is_active() {
        return Err(KError::InvalidTableAction);
    }
    let additional_bet = bet_type.amount().saturating_sub(seat.bet);
    if additional_bet == 0 {
        return Err(KError::InsufficientChips);
    }
    if seat.chips < additional_bet {
        return Err(KError::InsufficientChips);
    }
    let mut next = *seat;
    next.chips -= additional_bet;
    next.bet += additional_bet;
    next.chips_in_play += additional_bet;
    if next.is_all_in() {
        next.state = PlayerState::AllIn(next.bet);
    } else {
        if matches!(bet_type, PlayerState::AllIn(_)) {
            return Err(KError::InvalidTableAction);
        }
        next.state = bet_type;
    }
    Ok(next)
}

/// `record_voluntary_action` (actions.rs:860) folded into the successor.
fn record_voluntary(next: &mut HandBetting, seat: u8, chips_committed: bool) {
    let level = next.bet;
    if let Some(s) = next.seats.get_mut(seat as usize) {
        s.bet_level_when_last_acted = level;
    }
    next.actions_this_street = next.actions_this_street.saturating_add(1);
    if chips_committed {
        next.chip_actions_this_street = next.chip_actions_this_street.saturating_add(1);
    }
}

/// `Table::act_raise` (`src/casino/table/actions.rs:638`) as a pure
/// transition. The DEFECT_007 pre-validation guard and the DEFECT_015
/// saturating increment are both preserved — and now exist ONCE.
pub fn act_raise(state: &HandBetting, seat: u8, amount: usize) -> Result<Step, KError> {
    if seat != state.next_to_act() {
        return Err(KError::OutOfOrder { seat });
    }
    // DEFECT_007 guard: pre-validate BEFORE any state is modified, via the
    // same `validate_raise` the advisory surface queries. All-in bypasses
    // (a short stack can always shove for less).
    if let Some(s) = state.seats.get(seat as usize) {
        let would_be_all_in = amount >= s.total_chip_count();
        if !would_be_all_in {
            state.validate_raise(seat, amount)?;
        }
    }
    let mut next = state.clone();
    let s = next.seats.get_mut(seat as usize).ok_or(KError::InvalidSeatNumber)?;
    let updated = seat_bet_internal(s, PlayerState::Raise(amount))?;
    let remaining = updated.chips;
    let updated_is_all_in = updated.is_all_in();
    *s = updated;
    // `set_raise_increment` (actions.rs:786): a pure store, skipped for an
    // all-in seat; DEFECT_015: the delta is saturating, because an
    // all-in-for-less makes `amount < state.bet` and unchecked subtraction
    // underflows — the exact defect that lived on in the unhardened sibling.
    if !updated_is_all_in {
        next.raise_increment = amount.saturating_sub(state.bet);
    }
    next.bet = state.bet.max(amount);
    next.raises_this_street = next.raises_this_street.saturating_add(1);
    record_voluntary(&mut next, seat, true);
    let action_to = next.next_to_act();
    Ok(Step {
        events: vec![Event::Raise(seat, amount), Event::ActionTo(action_to)],
        returned: remaining,
        next,
    })
}

/// `Table::act_call` (actions.rs:535) as a pure transition, including the
/// short-stack all-in-for-partial branch (DEFECT_001 lineage).
pub fn act_call(state: &HandBetting, seat: u8) -> Result<Step, KError> {
    if seat != state.next_to_act() {
        return Err(KError::OutOfOrder { seat });
    }
    let call_target = state.bet;
    let seat_bet = state.seats.get(seat as usize).map(|s| s.bet).unwrap_or(0);
    let to_call = call_target.saturating_sub(seat_bet);
    let mut next = state.clone();
    let s = next.seats.get_mut(seat as usize).ok_or(KError::InvalidSeatNumber)?;
    let (updated, _actual_added, event): (SeatBetting, usize, Event) = if to_call == 0 {
        let mut u = *s;
        if !u.state.is_active() {
            return Err(KError::InvalidTableAction);
        }
        u.state = PlayerState::Check;
        (u, 0, Event::Check(seat))
    } else if s.chips < to_call {
        // Short of the call target: all-in for partial.
        let u = seat_bet_internal(s, PlayerState::AllIn(s.total_chip_count()))?;
        let added = u.bet.saturating_sub(seat_bet);
        (u, added, Event::Call(seat, added))
    } else {
        let u = seat_bet_internal(s, PlayerState::Call(call_target))?;
        (u, to_call, Event::Call(seat, to_call))
    };
    let remaining = updated.chips;
    *s = updated;
    record_voluntary(&mut next, seat, true);
    let action_to = next.next_to_act();
    Ok(Step {
        events: vec![event, Event::ActionTo(action_to)],
        returned: remaining,
        next,
    })
}

/// `Table::act_all_in` (actions.rs:713), NoLimit path, as a pure transition —
/// including the Part V rule: a sub-min shove does NOT re-open the betting
/// (increment untouched), a full-raise shove does (audit P9f).
pub fn act_all_in(state: &HandBetting, seat: u8) -> Result<Step, KError> {
    if seat != state.next_to_act() {
        return Err(KError::OutOfOrder { seat });
    }
    // NL only in the spike: the capped-structure degrade ladder
    // (actions.rs:722-751) is production port work.
    let old_bet = state.bet;
    let mut next = state.clone();
    let s = next.seats.get_mut(seat as usize).ok_or(KError::InvalidSeatNumber)?;
    let updated = seat_bet_internal(s, PlayerState::AllIn(s.total_chip_count()))?;
    let amount = updated.bet;
    let remaining = updated.chips;
    *s = updated;
    next.bet = old_bet.max(amount);
    let raise_delta = next.bet.saturating_sub(old_bet);
    if raise_delta >= state.min_raise() {
        next.raise_increment = raise_delta;
        next.raises_this_street = next.raises_this_street.saturating_add(1);
    }
    record_voluntary(&mut next, seat, true);
    let action_to = next.next_to_act();
    Ok(Step {
        events: vec![Event::AllIn(seat, amount), Event::ActionTo(action_to)],
        returned: remaining,
        next,
    })
}
