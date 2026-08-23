//! Ported defect tests + the three-shells equivalence proof.
//! Each test names the pkcore artifact it ports.

use spike_kernel::kernel::*;
use spike_kernel::shells::{immutable_act_raise, TableCelledLite, TableMut};

/// NLHE 50/100, three seats with blinds already posted: seat 0 = button/UTG
/// (3-handed: button acts first preflop), seat 1 = SB(50), seat 2 = BB(100).
fn preflop(stacks: [usize; 3]) -> HandBetting {
    let mut s0 = SeatBetting::with_chips(stacks[0]);
    s0.state = PlayerState::YetToAct;
    let mut s1 = SeatBetting::with_chips(stacks[1]);
    s1.chips -= 50;
    s1.bet = 50;
    s1.chips_in_play = 50;
    s1.state = PlayerState::Blind(50);
    let mut s2 = SeatBetting::with_chips(stacks[2]);
    s2.chips -= 100;
    s2.bet = 100;
    s2.chips_in_play = 100;
    s2.state = PlayerState::Blind(100);
    HandBetting {
        seats: vec![s0, s1, s2],
        bet: 100,
        raise_increment: 0,
        pot: 0,
        raises_this_street: 0,
        actions_this_street: 0,
        chip_actions_this_street: 0,
        big_blind: 100,
        betting: Betting::NoLimit,
        utg: 0,
    }
}

// ── DEFECT_015: all-in for less than the current bet ───────────────────
// Repro from docs/defects/DEFECT_015_act_raise_all_in_underflow.md:
// blinds 50/100, BB holds 300 total, UTG raises to 400, then
// `act_raise(bb, 300)`. The unhardened sibling underflowed
// `amount - self.bet` here (panic in debug, wrapped increment in release).
#[test]
fn defect_015_all_in_for_less_does_not_underflow_or_reopen() {
    let state = preflop([50_000, 50_000, 300]);
    let step = act_raise(&state, 0, 400).expect("UTG raise to 400");
    assert_eq!(300, step.next.raise_increment); // 400 over the 100 blind
    let state = act_call(&step.next, 1).expect("SB calls 400").next;

    // BB shoves 300 total *through act_raise* — the DEFECT_015 path.
    // amount (300) < table bet (400): the unchecked subtraction's trap.
    let step = act_raise(&state, 2, 300).expect("all-in for less must be legal");
    let bb = step.next.seats[2];
    assert!(bb.is_all_in());
    assert_eq!(PlayerState::AllIn(300), bb.state);
    assert_eq!(0, bb.chips);
    // The sub-bet shove neither corrupts nor re-opens:
    assert_eq!(300, step.next.raise_increment); // untouched, not wrapped
    assert_eq!(300, step.next.min_raise()); // min_raise() stays sane
    assert_eq!(400, step.next.bet); // table bet not lowered
}

// ── DEFECT_007 lineage: under-minimum raise corrupts nothing ───────────
// Port of `table_act_raise__under_minimum_does_not_corrupt_state`
// (table.rs:3073) and the act_raise doc-test: rejection must happen
// BEFORE any state changes.
#[test]
fn under_minimum_raise_rejected_before_any_state_change() {
    let state = preflop([5_000, 5_000, 5_000]);
    let step = act_raise(&state, 0, 300).expect("open to 300");
    let before = step.next.clone();
    // min re-raise-to is 300 + 200 = 500; 301 and 150 are both illegal.
    assert_eq!(500, before.min_raise_to());
    let nta = before.next_to_act();
    assert_eq!(Err(KError::InsufficientIncrement), act_raise(&before, nta, 301).map(|_| ()));
    assert_eq!(Err(KError::InsufficientIncrement), act_raise(&before, nta, 150).map(|_| ()));
    // Purity makes "no corruption" trivial — but assert it in the shell
    // shape the original test used:
    let mut shell = TableMut::new(before.clone());
    assert!(shell.act_raise(nta, 301).is_err());
    assert_eq!(before, shell.state); // byte-for-byte untouched
    assert_eq!(nta, shell.state.next_to_act()); // still that seat's turn
}

// ── DEFECT_010 / TDA 47-A: the re-open gate ────────────────────────────
// Port of the `is_reopen_gated` doc-test (actions.rs:404): A raises 300,
// B shoves 400 (only 100 more, short of the 200 full raise), C calls;
// A may only call or fold.
#[test]
fn rule_47a_sub_min_all_in_does_not_reopen_for_prior_actor() {
    let state = preflop([50_000, 400, 50_000]);
    let a = state.next_to_act();
    assert_eq!(0, a);
    let state = act_raise(&state, 0, 300).expect("A raises to 300").next;
    assert_eq!(200, state.raise_increment);
    let state = act_all_in(&state, 1).expect("B shoves 400 total").next;
    // 400 - 300 = 100 < min_raise 200: increment must NOT move (Part V).
    assert_eq!(200, state.raise_increment);
    assert_eq!(400, state.bet);
    let state = act_call(&state, 2).expect("C calls 400").next;

    // Action returns to A, who faces 100 — short of a full raise.
    assert_eq!(0, state.next_to_act());
    assert!(state.is_reopen_gated(0));
    assert_eq!(None, state.raise_bounds(0));
    // And the mutating surface agrees with the advisory one:
    assert!(act_call(&state, 0).is_ok());
}

// ── Cumulative 47-A: two short shoves that sum to a full raise re-open ──
#[test]
fn rule_47a_cumulative_shoves_do_reopen() {
    // A raises to 300 (increment 200). B shoves 400. C shoves 620.
    // A last acted at level 300; now faces 320 >= min_raise → re-opened.
    let state = preflop([50_000, 400, 620]);
    let state = act_raise(&state, 0, 300).unwrap().next;
    let state = act_all_in(&state, 1).unwrap().next; // to 400: +100, no re-open
    assert_eq!(200, state.raise_increment);
    let state = act_all_in(&state, 2).unwrap().next; // to 620: +220 ≥ 200 → re-opens
    assert_eq!(220, state.raise_increment);
    assert!(!state.is_reopen_gated(0));
    assert!(state.raise_bounds(0).is_some());
}

// ── Advisory/mutating coherence (audit P9b) as a property ──────────────
// Any amount inside raise_bounds is accepted; the bounds' min minus one is
// rejected — because both surfaces call the same validate_raise.
#[test]
fn raise_bounds_and_act_raise_cannot_drift() {
    let state = preflop([5_000, 5_000, 5_000]);
    let state = act_raise(&state, 0, 300).unwrap().next;
    let seat = state.next_to_act();
    let (min, max) = state.raise_bounds(seat).expect("raise is legal");
    assert!(act_raise(&state, seat, min).is_ok());
    assert!(act_raise(&state, seat, max).is_ok()); // NL max = stack → all-in path
    assert_eq!(Err(KError::InsufficientIncrement), act_raise(&state, seat, min - 1).map(|_| ()));
}

// ── One kernel, three shells ───────────────────────────────────────────
// The same scripted sequence through the `&mut self` shell, the `&self`
// celled shell, and bare value semantics produces identical states and
// identical event streams. This is the anti-DEFECT_015 property: there is
// no second body to forget.
#[test]
fn three_shells_one_kernel_identical_outcomes() {
    let start = preflop([50_000, 50_000, 300]);

    // Immutable driver.
    let s1 = immutable_act_raise(&start, 0, 400).unwrap();
    let s2 = act_call(&s1.next, 1).unwrap();
    let s3 = act_raise(&s2.next, 2, 300).unwrap(); // DEFECT_015 shove
    let mut value_events = Vec::new();
    value_events.extend(s1.events.clone());
    value_events.extend(s2.events.clone());
    value_events.extend(s3.events.clone());
    let value_final = s3.next.clone();

    // &mut shell.
    let mut tm = TableMut::new(start.clone());
    tm.act_raise(0, 400).unwrap();
    tm.act_call(1).unwrap();
    tm.act_raise(2, 300).unwrap();

    // Celled shell (&self throughout).
    let tc = TableCelledLite::new(start);
    tc.act_raise(0, 400).unwrap();
    tc.act_call(1).unwrap();
    tc.act_raise(2, 300).unwrap();
    let (tc_state, tc_events) = tc.snapshot();

    assert_eq!(value_final, tm.state);
    assert_eq!(value_final, tc_state);
    assert_eq!(value_events, tm.event_log);
    assert_eq!(value_events, tc_events);
}

// ── Order guard ────────────────────────────────────────────────────────
#[test]
fn out_of_order_action_rejected() {
    let state = preflop([5_000, 5_000, 5_000]);
    assert_eq!(Err(KError::OutOfOrder { seat: 1 }), act_raise(&state, 1, 300).map(|_| ()));
    assert_eq!(Err(KError::OutOfOrder { seat: 2 }), act_call(&state, 2).map(|_| ()));
}

// ── DEFECT_022 shape: action restarts behind the aggressor, not UTG ────
// A(utg) raises 300, B calls, C(BB) re-raises to 900. Next to act must be
// A (clockwise of the aggressor C), not a re-scan from UTG handing action
// to a seat that already matched.
#[test]
fn defect_022_next_to_act_roots_at_last_aggressor() {
    let state = preflop([50_000, 50_000, 50_000]);
    let state = act_raise(&state, 0, 300).unwrap().next;
    let state = act_call(&state, 1).unwrap().next;
    let state = act_raise(&state, 2, 900).unwrap().next; // BB 3-bets
    assert_eq!(0, state.next_to_act()); // A, owing 600 — not B
    let state = act_call(&state, 0).unwrap().next;
    assert_eq!(1, state.next_to_act()); // then B
}
