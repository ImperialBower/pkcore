//! TDA 2024 conformance harness.
//!
//! Table-driven assertions taken from the **Poker TDA 2024 Illustration Addendum**,
//! which publishes worked examples with their expected numbers. Those numbers are an
//! external authority: they are not derived from pkcore, so they cannot drift with it.
//!
//! Source of the parsed ruleset: the sibling `tda_parsed` repo —
//! `tda_2024.yaml` (all 71 rules) and `tda_2024_online.yaml` (the 50 that bind an
//! automated engine, each carrying an `implemented` / `evidence` / `gap` audit).
//!
//! # Layout
//!
//! Two groups, deliberately separated:
//!
//! 1. **Conformant** — rules pkcore already satisfies. These pin behaviour that is
//!    correct today so a future change has to argue with the TDA rather than with us.
//!    Five rules moved into this group by being fixed rather than by being right all
//!    along: Rule 47-A's re-open *rights* gate (`DEFECT_010`), Rule 36's
//!    substantial-action predicate (`DEFECT_009`), Rule 20's odd chip
//!    (`DEFECT_011`), Rule 54-B's short-blind pot calculation (`DEFECT_012`) and
//!    Rule 32's dead button (`DEFECT_013`). Each one's reproducing assertion is
//!    still here — it simply passes now.
//! 2. **Known defects** — `#[ignore]`d with their `DEFECT_008` finding id, asserting
//!    the TDA-correct answer and therefore failing by design, so CI stays green while
//!    the defect stays recorded in executable form:
//!
//!    ```text
//!    cargo test --test tda_conformance -- --include-ignored
//!    ```
//!
//!    **This group is currently empty.** All four reproducible `DEFECT_008` findings
//!    have been fixed and un-`ignore`d. That was the Gold Standard from
//!    `docs/epics/EPIC-00f_Coverage.md` applied to this audit: when it was written, a fix for
//!    any D8-N would have made no existing test fail. Only D8-6 is still open, and it
//!    is unreachable until a multi-table event model exists — no assertion can hold it.
//!
//! # Adaptations, stated up front
//!
//! Several addendum examples are written post-flop. Driving a table to the flop adds
//! setup that is irrelevant to the rule under test, so where the arithmetic is
//! provably identical pre-flop the example is reproduced pre-flop and the equivalence
//! is shown in the test's own comment. Where it is *not* identical the example is
//! reproduced faithfully or omitted — it is never silently rescaled.
//!
//! # Not covered here
//!
//! Rule 36 (substantial action) was the one finding this harness could not
//! hold — an absent predicate cannot be asserted against, so any test naming it failed
//! to *compile*. `DEFECT_009` added `Table::substantial_action`, and the eleven Rule 36
//! assertions now sit in the conformant group.
//!
//! Rule 21 (side pots split separately) is covered by `tests/split_pots.rs`, added for
//! `DEFECT_003`, and is not duplicated here.
//!
//! TDA rules used by permission of the Poker TDA, <http://www.pokertda.com>,
//! all rights reserved.

#[allow(nonstandard_style)]
mod tda_2024_conformance {
    use pkcore::bot::table_snapshot::TableSnapshot;
    use pkcore::prelude::*;

    /// A seated player with `chips`.
    fn seat(handle: &str, chips: usize) -> Seat {
        Seat::new(Player::new_with_chips(handle.to_string(), chips))
    }

    /// An unoccupied seat. `Seat::is_empty` keys off an empty handle
    /// (`src/casino/table/seat.rs:68`), so this models an eliminated player whose
    /// seat has not been re-filled.
    fn empty_seat() -> Seat {
        Seat::new(Player::new_with_chips(String::new(), 0))
    }

    /// No-limit hold'em table, blinds `(sb, bb)`, dealt and ready for the first
    /// voluntary action.
    fn nlhe(stacks: &[usize], sb: usize, bb: usize) -> Table {
        let seats = Seats::new(
            stacks
                .iter()
                .enumerate()
                .map(|(i, c)| seat(&format!("P{i}"), *c))
                .collect(),
        );
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(sb, bb));
        table.act_forced_bets().expect("forced bets should post");
        table.deal_cards_to_seats().expect("cards should deal");
        table
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Conformant — pkcore already satisfies these
    // ═══════════════════════════════════════════════════════════════════════

    /// **TDA Rule 43, Addendum Example 1.** The minimum re-raise is the largest prior
    /// *increment* on the street, not the largest prior *total*.
    ///
    /// > A opens with a bet of 600. B raises 1000 for total of 1600. C re-raises 2000
    /// > for total of 3600. […] D must re-raise at least 2000 more for a total of
    /// > **5600**. Note that D's minimum raise is not 3600 (C's total bet), but only
    /// > 2000, the additional raise action that C added.
    ///
    /// *Adaptation:* the addendum plays this post-flop, opening with a **bet** of 600.
    /// Reproduced pre-flop at 100/200, the opening action is a **raise** to 600 over
    /// the 200 big blind. The first increment therefore differs (400 here vs 600
    /// there), but every later increment is identical — 600→1600 is 1000 and
    /// 1600→3600 is 2000 in both readings — so D's minimum is the same 5600 that the
    /// addendum publishes, and 5600 is what this test asserts.
    #[test]
    fn rule_43_ex1_min_reraise_is_the_last_increment_not_the_total() {
        let mut table = nlhe(&[50_000; 5], 100, 200);

        let a = table.next_to_act();
        table.act_raise(a, 600).expect("open to 600");
        let b = table.next_to_act();
        table.act_raise(b, 1_600).expect("raise to 1600");
        let c = table.next_to_act();
        table.act_raise(c, 3_600).expect("re-raise to 3600");

        assert_eq!(
            5_600,
            table.min_raise_to(),
            "TDA 43 Addendum Ex 1: D's minimum is C's 2000 increment on top of 3600, \
             not a doubling of C's 3600 total"
        );
    }

    /// **TDA Rule 43, Addendum Example 2.** A short all-in that does not itself
    /// constitute a full raise leaves the minimum where it was.
    ///
    /// > Blinds 50-100. Pre-flop A is under the gun and goes all-in for a total of 150
    /// > (an increase in the bet of 50). […] The 100 is still the "largest bet or raise
    /// > of the current round", so if B wants to re-raise he must raise at least 100
    /// > for a total of **250**.
    ///
    /// Native pre-flop — no adaptation.
    #[test]
    fn rule_43_ex2_short_all_in_does_not_raise_the_minimum() {
        // 3-handed: seat 0 button, seat 1 SB, seat 2 BB — so seat 0 is UTG pre-flop.
        let mut table = nlhe(&[150, 50_000, 50_000], 50, 100);

        let utg = table.next_to_act();
        table.act_all_in(utg).expect("UTG shoves 150 total");

        assert_eq!(
            250,
            table.min_raise_to(),
            "TDA 43 Addendum Ex 2: the 50 increment is short of a full raise, so the \
             big blind's 100 remains the largest bet or raise of the round"
        );
    }

    /// **TDA Rule 47, Addendum Example 1.** When short all-ins do re-open the betting,
    /// the minimum raise is still *the last full valid bet or raise of the round* —
    /// not the accumulated total of the short shoves.
    ///
    /// ## Scenario Blinds 50-100
    ///
    /// - Post-flop
    ///   - A opens betting for the 100 minimum.
    ///   - B goes all in for a total of 125
    ///   - C calls the 125.
    ///   - D goes all in for 200 total
    ///   - E calls 200
    ///
    /// > Neither B's increment of 25 or D's increment of 75 is by itself a
    /// > full raise, but when added together they total a full raise.
    ///
    /// The published minimum re-raise for the re-opened player is **300**.
    ///
    /// *Adaptation:* played pre-flop at 50/100, where the posted big blind *is* the
    /// 100 opening bet the addendum describes. Increments are unchanged (25 then 75),
    /// so the resulting 300 is the addendum's own figure.
    ///
    /// Note this asserts *sizing* only. Whether the re-opened player may legally raise
    /// is `DEFECT_008` D8-2, asserted separately below.
    #[test]
    fn rule_47_ex1_min_reraise_after_cumulative_short_all_ins() {
        // seats: 0 button, 1 SB, 2 BB, 3 UTG, 4 — shove stacks on 3 and 0.
        let mut table = nlhe(&[200, 50_000, 50_000, 125, 50_000], 50, 100);

        let utg = table.next_to_act();
        table.act_all_in(utg).expect("UTG shoves 125 (increment 25)");
        let n = table.next_to_act();
        table.act_call(n).expect("call 125");
        let d = table.next_to_act();
        table.act_all_in(d).expect("shove 200 (increment 75)");
        let e = table.next_to_act();
        table.act_call(e).expect("call 200");

        assert_eq!(
            300,
            table.min_raise_to(),
            "TDA 47 Addendum Ex 1: the minimum stays the last full valid bet or raise \
             of the round (100) on top of the 200 now owed"
        );
    }

    /// **TDA Rule 48.** No raise cap in no-limit or pot-limit; fixed-limit caps at the
    /// house limit.
    ///
    /// > There is no cap on the number of raises in no-limit and pot-limit. In limit
    /// > play, there is a limit to raises […] the house limit applies.
    #[test]
    fn rule_48_raise_cap_applies_only_to_fixed_limit() {
        assert!(
            !BettingStructure::NoLimit.cap_reached(100),
            "TDA 48: no-limit is uncapped"
        );
        assert!(
            !BettingStructure::PotLimit.cap_reached(100),
            "TDA 48: pot-limit is uncapped"
        );

        let fl = BettingStructure::FixedLimit {
            small_bet: 100,
            big_bet: 200,
            raise_cap: 3,
        };
        assert!(!fl.cap_reached(2), "TDA 48: under the house limit");
        assert!(fl.cap_reached(3), "TDA 48: at the house limit");
    }

    /// **TDA Rule 54-C.** Post-flop, the pot-limit maximum is computed from the actual
    /// pot: `current_bet + pot + call_amount`, capped at the stack.
    ///
    /// This pins the formula itself, which is correct. The pre-flop exception in 54-B
    /// is `DEFECT_008` D8-3, asserted separately below.
    #[test]
    fn rule_54_c_pot_limit_maximum_uses_the_actual_pot_postflop() {
        let pl = BettingStructure::PotLimit;
        // pot 1000, current bet 100, nothing committed → call 100 → 100 + 1000 + 100.
        assert_eq!(
            1_200,
            pl.max_raise(1_000, 100, 0, 50_000, BetTier::Small),
            "TDA 54-C: pot-limit maximum is current_bet + pot + call_amount"
        );
        // The stack is a hard ceiling.
        assert_eq!(
            800,
            pl.max_raise(1_000, 100, 0, 800, BetTier::Small),
            "TDA 54-C: the maximum is capped at the player's stack"
        );
    }

    // ── Rule 47-A, the re-open rights gate (DEFECT_010, fixed) ─────────────
    //
    // Rule 47-A carries two obligations: how small a legal re-raise may be
    // (*sizing*, pinned by `rule_47_ex1_*` and `rule_43_ex2_*` above) and
    // whether a given player may raise at all (*rights*, pinned here).
    //
    // > An all-in wager (or cumulative multiple short all-ins) totaling less
    // > than a full bet or raise **will not reopen betting for players who
    // > have already acted and are not facing at least a full bet or raise**
    // > when the action returns to them.
    //
    // The rights half was absent until `DEFECT_010`. `Table::is_reopen_gated`
    // is now its single implementation; `Table::raise_bounds` and
    // `TableSnapshot` both consult it.

    /// The published case. A raises to 300 (increment 200), B shoves 400
    /// (increment 100 — short of a full raise), C calls. A now faces only 100
    /// more, so TDA permits call or fold and no raise may be offered.
    ///
    /// This is the assertion that reproduced `DEFECT_010`.
    #[test]
    fn rule_47_a_player_who_already_acted_may_not_reraise_a_short_all_in() {
        // seats: 0 button/UTG, 1 SB (shove stack), 2 BB.
        let mut table = nlhe(&[50_000, 400, 50_000], 50, 100);

        let a = table.next_to_act();
        table.act_raise(a, 300).expect("A raises to 300, increment 200");
        let b = table.next_to_act();
        table.act_all_in(b).expect("B shoves 400, increment 100 — short");
        let c = table.next_to_act();
        table.act_call(c).expect("C calls 400");

        assert_eq!(a, table.next_to_act(), "action returns to A");
        assert_eq!(
            200, table.raise_increment,
            "the short shove must not move the increment"
        );
        assert!(
            table.raise_bounds(a).is_none(),
            "TDA 47-A: A already acted and faces 100, short of the 200 full raise — \
             call or fold only, no raise may be offered"
        );
    }

    /// The gate must not over-fire. C has not acted at all when the short
    /// all-in reaches it, so 47-A does not restrict C — it may raise.
    #[test]
    fn player_who_has_not_acted_may_raise_a_short_all_in() {
        let mut table = nlhe(&[50_000, 400, 50_000], 50, 100);

        let a = table.next_to_act();
        table.act_raise(a, 300).expect("A raises to 300");
        let b = table.next_to_act();
        table.act_all_in(b).expect("B shoves 400 — short");

        let c = table.next_to_act();
        assert!(
            !table.is_reopen_gated(c),
            "TDA 47-A restricts only players who have already acted"
        );
        assert!(
            table.raise_bounds(c).is_some(),
            "C has not acted this street, so a raise stays available to it"
        );
    }

    /// A posted big blind has **not** acted: the option is still to come. The
    /// gate keys off `PlayerState`, whose `Blind(_)` arm exists for exactly
    /// this, so a short all-in must not strip the big blind of its option.
    #[test]
    fn big_blind_option_is_not_gated_by_a_short_all_in() {
        // seats: 0 button/UTG, 1 SB, 2 BB. UTG shoves 150 over the 100 blind —
        // an increment of 50, short of the 100 full raise.
        let mut table = nlhe(&[150, 50_000, 50_000], 50, 100);

        let utg = table.next_to_act();
        table.act_all_in(utg).expect("UTG shoves 150 — short");
        let sb = table.next_to_act();
        table.act_call(sb).expect("SB calls 150");

        let bb = table.next_to_act();
        assert!(
            !table.is_reopen_gated(bb),
            "TDA 47-A: a posted blind is not an action, so the big blind has not \
             acted and keeps its option"
        );
    }

    /// 47-A's cumulative clause. Two short all-ins that *together* make a full
    /// raise do re-open the betting for a player who already acted.
    ///
    /// This is the assertion that separates a correct fix from one written as
    /// "compare against the last all-in" — the comparison has to be against
    /// the level this seat last acted at.
    #[test]
    fn cumulative_short_all_ins_reopen_for_a_player_who_acted() {
        // A raises to 300 (increment 200). B shoves 400 (+100), C shoves 500
        // (+100). A now faces 200 in total, which is a full raise.
        let mut table = nlhe(&[50_000, 400, 500], 50, 100);

        let a = table.next_to_act();
        table.act_raise(a, 300).expect("A raises to 300, increment 200");
        let b = table.next_to_act();
        table.act_all_in(b).expect("B shoves 400 — short on its own");
        let c = table.next_to_act();
        table.act_all_in(c).expect("C shoves 500 — short on its own");

        assert_eq!(a, table.next_to_act(), "action returns to A");
        assert_eq!(500, table.bet, "the table bet is now 500");
        assert!(
            !table.is_reopen_gated(a),
            "TDA 47-A cumulative clause: A last acted at 300 and now faces 200, \
             a full raise, so the betting is re-opened for it"
        );
        assert!(
            table.raise_bounds(a).is_some(),
            "A may raise again once the shoves cumulatively make a full raise"
        );
    }

    /// A genuine full-raise all-in re-opens the betting for everyone, including
    /// players who already acted. The gate must lift.
    #[test]
    fn full_raise_all_in_reopens_for_a_player_who_acted() {
        // A raises to 300 (increment 200). B shoves 600 — an increment of 300,
        // comfortably a full raise. C calls.
        let mut table = nlhe(&[50_000, 600, 50_000], 50, 100);

        let a = table.next_to_act();
        table.act_raise(a, 300).expect("A raises to 300, increment 200");
        let b = table.next_to_act();
        table.act_all_in(b).expect("B shoves 600 — a full raise");
        let c = table.next_to_act();
        table.act_call(c).expect("C calls 600");

        assert_eq!(a, table.next_to_act(), "action returns to A");
        assert!(
            !table.is_reopen_gated(a),
            "TDA 47-A applies only to wagers short of a full raise"
        );
        assert!(
            table.raise_bounds(a).is_some(),
            "a full-raise all-in re-opens the betting for A"
        );
    }

    /// The gate is a per-street question. A seat restricted on one street is
    /// unrestricted on the next, because the recorded level dies with the
    /// street alongside `PlayerState`.
    #[test]
    fn reopen_gate_clears_at_the_street_boundary() {
        let mut table = nlhe(&[50_000, 400, 50_000], 50, 100);

        let a = table.next_to_act();
        table.act_raise(a, 300).expect("A raises to 300");
        let b = table.next_to_act();
        table.act_all_in(b).expect("B shoves 400 — short");
        let c = table.next_to_act();
        table.act_call(c).expect("C calls 400");
        assert!(table.is_reopen_gated(a), "A is restricted pre-flop");

        table.act_call(a).expect("A calls the extra 100, closing the street");
        table.bring_it_in().expect("street closes");

        assert!(
            !table.is_reopen_gated(a),
            "the restriction dies with the street it was created on"
        );
    }

    /// **Interpretation, not a quotation.** Rule 47-A names no-limit and
    /// pot-limit only. Fixed-limit has its own half-a-bet treatment, so the
    /// gate is deliberately scoped away from it; this test pins that choice so
    /// a later reading of the rules can find and challenge it.
    #[test]
    fn fixed_limit_is_not_gated_by_rule_47_a() {
        let seats = Seats::new(vec![seat("A", 50_000), seat("B", 250), seat("C", 50_000)]);
        // Blinds are derived: small_bet 100 → 50/100.
        let mut table = Table::limit_holdem_from_seats(seats, 100, 200, 4);
        table.act_forced_bets().expect("forced bets should post");
        table.deal_cards_to_seats().expect("cards should deal");

        let a = table.next_to_act();
        table.act_raise(a, 200).expect("A raises to 200");
        let b = table.next_to_act();
        table.act_all_in(b).expect("B shoves 250 — short of a full bet");

        let c = table.next_to_act();
        table.act_call(c).expect("C calls");

        assert!(
            !table.is_reopen_gated(a),
            "TDA 47-A is scoped to no-limit and pot-limit; fixed-limit is governed \
             by its own rule and its own raise cap"
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // TDA Rule 36 — Substantial Action (`DEFECT_009`)
    //
    // > Substantial Action is either A) any 2 actions in turn, at least one of
    // > which puts chips in the pot (i.e. any 2 actions except 2 checks or 2
    // > folds) or B) any combination of 3 actions in turn (check, bet, raise,
    // > call, fold). Posted blinds do not count towards SA.
    //
    // Eleven assertions: the seven that transcribe the rule's own text and its
    // stated counter-examples, then four that pin the mechanics the rule
    // implies — the two reset boundaries, the turn guard, and the bring-in
    // interpretation.
    // ───────────────────────────────────────────────────────────────────────

    /// Drives the three-handed pre-flop round to its close and deals the flop,
    /// leaving the table on a fresh street with the counters cleared. Seat 0 is
    /// the button and therefore under the gun; seat 1 acts first post-flop.
    fn three_handed_to_the_flop() -> Table {
        let mut table = nlhe(&[50_000; 3], 50, 100);
        let utg = table.next_to_act();
        table.act_call(utg).expect("UTG calls 100");
        let sb = table.next_to_act();
        table.act_call(sb).expect("SB completes to 100");
        let bb = table.next_to_act();
        table.act_check(bb).expect("BB checks the option");
        table.bring_it_in().expect("pre-flop closes");
        table.deal_flop().expect("flop should deal");
        table
    }

    /// **Rule 36, the exclusion clause.** "Posted blinds do not count towards
    /// SA." A hand where both blinds are up and nobody has voluntarily acted
    /// has a substantial action of zero.
    #[test]
    fn blinds_alone_are_not_substantial_action() {
        let table = nlhe(&[50_000; 3], 50, 100);

        assert!(
            !table.substantial_action(),
            "TDA 36: a posted blind is not an action, so two blinds are not SA"
        );
    }

    /// **Rule 36, clause A counter-example.** The rule names two checks as the
    /// case that is explicitly *not* SA.
    #[test]
    fn two_checks_are_not_substantial_action() {
        let mut table = three_handed_to_the_flop();

        let first = table.next_to_act();
        table.act_check(first).expect("first checks");
        let second = table.next_to_act();
        table.act_check(second).expect("second checks");

        assert!(
            !table.substantial_action(),
            "TDA 36 A: 2 actions, neither committing chips, is not SA"
        );
    }

    /// **Rule 36, clause A's other counter-example.** Two folds put no chips in
    /// the pot either.
    #[test]
    fn two_folds_are_not_substantial_action() {
        let mut table = nlhe(&[50_000; 4], 50, 100);

        let utg = table.next_to_act();
        table.act_fold(utg).expect("UTG folds");
        let next = table.next_to_act();
        table.act_fold(next).expect("button folds");

        assert!(
            !table.substantial_action(),
            "TDA 36 A: 2 folds commit no chips, so they are not SA"
        );
    }

    /// **Rule 36, clause A.** Two actions where one puts chips in the pot.
    #[test]
    fn check_then_bet_is_substantial_action() {
        let mut table = three_handed_to_the_flop();

        let first = table.next_to_act();
        table.act_check(first).expect("first checks");
        let second = table.next_to_act();
        table.act_bet(second, 200).expect("second bets 200");

        assert!(
            table.substantial_action(),
            "TDA 36 A: 2 actions, one of which committed chips, is SA"
        );
    }

    /// **Rule 36, clause A** reached without an opening bet — the chip action
    /// is a call, and the action before it is a fold. Clause A cares only that
    /// *one* of the two moved chips, not which one or that it opened betting.
    #[test]
    fn fold_then_call_is_substantial_action() {
        let mut table = nlhe(&[50_000; 4], 50, 100);

        let utg = table.next_to_act();
        table.act_fold(utg).expect("UTG folds");
        let next = table.next_to_act();
        table.act_call(next).expect("button calls 100");

        assert!(
            table.substantial_action(),
            "TDA 36 A: fold + call is 2 actions with one committing chips"
        );
    }

    /// **Rule 36, clause B.** Three actions of any kind, with no chips moved at
    /// all — clause A can never fire here, so this isolates clause B.
    #[test]
    fn three_checks_are_substantial_action() {
        let mut table = three_handed_to_the_flop();

        for _ in 0..3 {
            let seat = table.next_to_act();
            table.act_check(seat).expect("check around");
        }

        assert!(
            table.substantial_action(),
            "TDA 36 B: any 3 in-turn actions are SA, chips or no chips"
        );
    }

    /// **Rule 36, clause B** again, with the other chipless action. Three folds
    /// are SA even though two folds are not — the clauses differ only in count.
    #[test]
    fn three_folds_are_substantial_action() {
        let mut table = nlhe(&[50_000; 5], 50, 100);

        for _ in 0..3 {
            let seat = table.next_to_act();
            table.act_fold(seat).expect("fold around");
        }

        assert!(
            table.substantial_action(),
            "TDA 36 B: 3 folds are 3 in-turn actions, so SA even with no chips"
        );
    }

    /// SA is a property of the current betting round. Closing the street clears
    /// it, so a new street starts back at zero.
    #[test]
    fn substantial_action_resets_at_street_boundary() {
        let mut table = three_handed_to_the_flop();
        let first = table.next_to_act();
        table.act_check(first).expect("first checks");
        let second = table.next_to_act();
        table.act_bet(second, 200).expect("second bets 200");
        assert!(table.substantial_action(), "SA is reached on the flop");

        let third = table.next_to_act();
        table.act_call(third).expect("third calls");
        let closer = table.next_to_act();
        table.act_call(closer).expect("first calls, closing the street");
        table.bring_it_in().expect("flop closes");

        assert!(
            !table.substantial_action(),
            "SA is a per-street question; the street boundary clears it"
        );
    }

    /// The hand boundary clears it too, so the next hand cannot inherit the
    /// previous one's action count.
    #[test]
    fn substantial_action_resets_at_hand_boundary() {
        let mut table = three_handed_to_the_flop();
        let first = table.next_to_act();
        table.act_check(first).expect("first checks");
        let second = table.next_to_act();
        table.act_bet(second, 200).expect("second bets 200");
        assert!(table.substantial_action(), "SA is reached on the flop");

        table.reset();

        assert!(
            !table.substantial_action(),
            "a new hand starts with no substantial action"
        );
    }

    /// Rule 36 counts actions **in turn**. An action pkcore refuses because it
    /// is out of turn never happened, so it must not move either counter.
    #[test]
    fn rejected_out_of_turn_action_does_not_count() {
        let mut table = nlhe(&[50_000; 3], 50, 100);

        let utg = table.next_to_act();
        table.act_call(utg).expect("UTG calls — one in-turn chip action");

        let out_of_turn = table.next_to_act() + 1;
        assert!(
            table.act_fold(out_of_turn).is_err(),
            "the seat after the actor cannot act yet"
        );
        assert!(
            table.act_raise(out_of_turn, 300).is_err(),
            "nor can it raise out of turn"
        );

        assert!(
            !table.substantial_action(),
            "TDA 36: only actions in turn count, so one in-turn call is still \
             one action and short of both clauses"
        );
    }

    /// `DEFECT_018`: eight-handed stud is a legal, standard table size. It
    /// needs 56 individual cards from a 52-card deck, so 7th street is dealt
    /// as a single shared community card. The end-to-end proof: the hand must
    /// reach a real showdown and award chips, not merely avoid an error.
    #[test]
    fn stud_full_table_runs_to_showdown() {
        for razz in [false, true] {
            let seats = Seats::new((0..8).map(|i| seat(&format!("P{i}"), 10_000)).collect());
            let table = if razz {
                Table::razz_from_seats(seats, 2, 5, 20, 40).unwrap()
            } else {
                Table::stud_hi_from_seats(seats, 2, 5, 20, 40).unwrap()
            };
            let mut session = PokerSession::new(table);
            session.start_hand().expect("hand should start");

            // Everyone calls every street: the widest possible field reaches
            // 7th street, which is the condition that exhausts the deck.
            let mut winnings = None;
            for _ in 0..400 {
                match session.next_step() {
                    SessionStep::PlayerToAct(seat_number) => {
                        session
                            .apply_action(seat_number, PlayerAction::Call)
                            .expect("a call must be legal");
                    }
                    SessionStep::StreetAdvanced => {}
                    SessionStep::HandComplete => {
                        winnings = Some(session.end_hand().expect("showdown should resolve"));
                        break;
                    }
                    SessionStep::Failed(e) => {
                        panic!("eight-handed {} stalled: {e:?}", if razz { "razz" } else { "stud" })
                    }
                }
            }

            let winnings = winnings.expect("hand never completed");
            assert!(
                !winnings.vec().is_empty(),
                "eight-handed {} awarded nothing",
                if razz { "razz" } else { "stud" }
            );
        }
    }

    /// **Interpretation, not a quotation.** Rule 36 excludes "posted blinds"
    /// and says nothing about the stud bring-in. The bring-in is structurally a
    /// forced post and is treated as one throughout pkcore, so it is excluded
    /// on the same grounds. This test pins that reading so a later one can find
    /// and challenge it.
    #[test]
    fn stud_bring_in_is_not_substantial_action() {
        let seats = Seats::new(vec![seat("A", 10_000), seat("B", 10_000), seat("C", 10_000)]);
        // ante 2, bring-in 5, small bet 20, big bet 40.
        let mut table = Table::stud_hi_from_seats(seats, 2, 5, 20, 40).unwrap();
        table.act_forced_bets().expect("antes should post");
        table.deal_stud_3rd_street().expect("3rd street should deal");
        table.act_bring_in().expect("bring-in should post");

        assert!(
            !table.substantial_action(),
            "the bring-in is a forced post, so it is excluded exactly as a \
             posted blind is"
        );
    }

    /// **TDA Rule 54-C**, on a live table rather than on the bare structure.
    ///
    /// > Post-flop, all bets are calculated on the **actual** pot.
    ///
    /// The same short blind that inflates the pre-flop ceiling must not inflate
    /// the post-flop one. This is the over-correction guard: 54-B without a
    /// phase gate would leak the unposted 100 into every later street.
    #[test]
    fn rule_54_c_short_blind_does_not_inflate_the_postflop_pot_limit_maximum() {
        let seats = Seats::new(vec![
            seat("Button", 50_000),
            seat("SmallBlind", 50_000),
            seat("ShortBigBlind", 100),
        ]);
        let mut table = Table::plo_from_seats(seats, (100, 200));
        table.act_forced_bets().expect("forced bets should post");
        table.deal_cards_to_seats().expect("cards should deal");

        let utg = table.next_to_act();
        table.act_call(utg).expect("button calls 200");
        let sb = table.next_to_act();
        table.act_call(sb).expect("small blind completes to 200");

        table.bring_it_in().expect("pre-flop closes");
        table.deal_flop().expect("flop should deal");

        // 200 + 200 from the live players, 100 from the short all-in blind.
        assert_eq!(500, table.effective_pot(), "the real pot after the flop");
        assert_eq!(
            500,
            table.pot_limit_pot(),
            "TDA 54-C: post-flop the notional-blind substitution is over — the \
             unposted 100 must not follow the hand onto later streets"
        );

        let first = table.next_to_act();
        let (_, max) = table.raise_bounds(first).expect("a pot bet must be available");
        assert_eq!(
            500, max,
            "TDA 54-C: the maximum post-flop bet is the actual pot, not the \
             pre-flop notional one"
        );
    }

    /// The bots must see the same ceiling the engine enforces. `DEFECT_010`
    /// found `TableSnapshot` re-deriving raise legality instead of delegating,
    /// and the two disagreed the moment the engine learned a new rule. Rule
    /// 54-B is exactly that shape again, so it is pinned rather than assumed.
    #[test]
    fn bot_snapshot_agrees_with_the_table_on_the_54_b_maximum() {
        let seats = Seats::new(vec![
            seat("Button", 50_000),
            seat("SmallBlind", 50_000),
            seat("ShortBigBlind", 100),
        ]);
        let mut table = Table::plo_from_seats(seats, (100, 200));
        table.act_forced_bets().expect("forced bets should post");
        table.deal_cards_to_seats().expect("cards should deal");

        let utg = table.next_to_act();
        let (_, table_max) = table.raise_bounds(utg).expect("a raise must be available");
        let snapshot = TableSnapshot::from_table(&table, utg);

        assert_eq!(
            table_max,
            snapshot.max_raise_to(),
            "the advisory surface the bots read must agree with the engine that \
             enforces it"
        );
        assert_eq!(700, snapshot.max_raise_to(), "and both must be the TDA answer");
    }

    // ───────────────────────────────────────────────────────────────────────
    // TDA Rule 20 — the odd chip (`DEFECT_011`)
    // ───────────────────────────────────────────────────────────────────────

    /// Checks the street around for every seat still owed action.
    fn check_around(table: &mut Table) {
        while !table.seats.is_betting_complete() {
            let seat = table.next_to_act();
            table.act_check(seat).expect("check around");
        }
    }

    /// TDA 20-A ordering, as an independent reference implementation: among tied
    /// winners, the odd chip goes to the first seat to the **left of the button**,
    /// wrapping. Deliberately written from the rule text rather than by calling
    /// pkcore, so it cannot drift with the implementation it checks.
    fn tda_20a_first_seat_left_of_button(winners: &[u8], button: u8, seat_count: u8) -> u8 {
        (1..=seat_count)
            .map(|step| (button + step) % seat_count)
            .find(|s| winners.contains(s))
            .expect("at least one winner must be seated")
    }

    /// **TDA Rule 20-A**, driven through a real showdown.
    ///
    /// > Board games with 2 or more high or low hands: the odd chip goes to the
    /// > **first seat left of the button**.
    ///
    /// Eight seats, button on **7**, blinds 25/50. Seats 2 and 5 call and everyone
    /// else folds, so the pot is 25 + 50 + 50 + 50 = **175** — odd on purpose. The
    /// board is aces full of kings and neither live hand improves on it, so the two
    /// play the board and tie exactly.
    ///
    /// 175 splits as 87 and 88. TDA puts the extra chip on the first tied seat left
    /// of the button, which with the button on 7 is seat **2**.
    ///
    /// *Setup note:* the board is stacked into the deck rather than drawn at random,
    /// because a tie has to be certain for the assertion to mean anything. Burn cards
    /// are included in the stack so the real dealing path runs unmodified.
    #[test]
    fn rule_20_a_odd_chip_goes_to_the_first_seat_left_of_the_button() {
        const SEAT_COUNT: u8 = 8;
        let winners = [2u8, 5u8];

        let seats = Seats::new((0..SEAT_COUNT).map(|i| seat(&format!("P{i}"), 5_000)).collect());
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(25, 50));
        table.button = 7;
        table.act_forced_bets().expect("forced bets should post");

        // Hole cards for every seat; the two winners hold cards that cannot beat or
        // separate the board.
        table
            .inject_hole_cards(&[
                (0, "2♠ 3♠"),
                (1, "4♠ 5♠"),
                (2, "7♦ 8♦"),
                (3, "6♠ 7♠"),
                (4, "8♠ 9♠"),
                (5, "9♣ T♣"),
                (6, "T♠ J♠"),
                (7, "Q♠ J♥"),
            ])
            .expect("hole cards should inject");

        // burn, flop, burn, turn, burn, river.
        table.deck = Cards::from_str("2♣ A♠ A♥ A♦ 3♣ K♠ 4♣ K♥").expect("stacked deck");

        // Button 7 of 8 → SB 0, BB 1, UTG 2. Order: 2 3 4 5 6 7 0 1.
        table.act_call(2).expect("seat 2 calls 50");
        table.act_fold(3).expect("seat 3 folds");
        table.act_fold(4).expect("seat 4 folds");
        table.act_call(5).expect("seat 5 calls 50");
        table.act_fold(6).expect("seat 6 folds");
        table.act_fold(7).expect("seat 7 folds");
        table.act_fold(0).expect("small blind folds");
        table.act_fold(1).expect("big blind folds the option");

        table.bring_it_in().expect("pre-flop closes");
        table.deal_flop().expect("flop");
        check_around(&mut table);
        table.bring_it_in().expect("flop closes");
        table.deal_turn().expect("turn");
        check_around(&mut table);
        table.bring_it_in().expect("turn closes");
        table.deal_river().expect("river");
        check_around(&mut table);

        let before: Vec<usize> = winners
            .iter()
            .map(|&s| table.seats.get_seat(s).expect("seat").player.chips)
            .collect();
        table.end_hand().expect("hand should end");
        let gained: Vec<usize> = winners
            .iter()
            .enumerate()
            .map(|(i, &s)| table.seats.get_seat(s).expect("seat").player.chips - before[i])
            .collect();

        assert_eq!(
            vec![88, 87],
            gained,
            "the 175 pot splits 88/87, and TDA 20-A puts the odd chip on seat 2"
        );

        let odd_index = gained.iter().position(|&g| g == 88).expect("one odd chip");
        assert_eq!(
            tda_20a_first_seat_left_of_button(&winners, table.button, SEAT_COUNT),
            winners[odd_index],
            "TDA 20-A: with the button on seat 7, the odd chip belongs to the first \
             tied winner to its left"
        );
    }

    /// **TDA Rule 20-B.** Stud and razz have no button clause — the cards decide.
    ///
    /// > Stud, razz, and if 2 or more high or low hands in stud/8: the odd chip goes
    /// > to the **high card by suit** in the player's 5-card winning hand.
    ///
    /// Both seats make the same broadway straight, so they tie. They are separated
    /// only by the suit of the ace, and spades beats hearts in the bridge ranking the
    /// TDA uses. The pairing is then swapped between the two seats: if the ordering
    /// were reading seat numbers rather than cards, the answer would not move.
    #[test]
    fn rule_20_b_stud_odd_chip_goes_to_the_high_card_by_suit() {
        let stud = |seat_one: &str, seat_two: &str| {
            let seats = Seats::new(vec![seat("A", 10_000), seat("B", 10_000), seat("C", 10_000)]);
            let mut table = Table::stud_hi_from_seats(seats, 2, 5, 20, 40).unwrap();
            table
                .inject_hole_cards(&[(1, seat_one), (2, seat_two)])
                .expect("hole cards should inject");
            table
        };

        // Seat 1 holds the ace of spades, seat 2 the ace of hearts.
        let table = stud("A♠ K♦ Q♦ J♦ T♦ 2♣ 3♣", "A♥ K♣ Q♣ J♣ T♣ 2♦ 3♦");
        assert_eq!(
            vec![1, 2],
            table.tda_odd_chip_order(&[1, 2]),
            "TDA 20-B: the ace of spades outranks the ace of hearts"
        );

        // Same two hands, swapped between the seats. The answer swaps with them.
        let table = stud("A♥ K♣ Q♣ J♣ T♣ 2♦ 3♦", "A♠ K♦ Q♦ J♦ T♦ 2♣ 3♣");
        assert_eq!(
            vec![2, 1],
            table.tda_odd_chip_order(&[1, 2]),
            "TDA 20-B reads the cards, not the seat index — the button is not \
             consulted in stud at all"
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // TDA Rule 32 — the dead button (`DEFECT_013`)
    //
    // > Tournament play will use a dead button.
    //
    // One sentence, and the 2024 ruleset never spells out the mechanics. What
    // it does do is name a **dead SB** in Rule 54-B's worked example ("dead SB,
    // BB posts 200") and never once name a dead BB. That asymmetry is the whole
    // design: the small blind position may be vacant and go unposted, the big
    // blind is always posted by a live player.
    // ───────────────────────────────────────────────────────────────────────

    /// Six seats, seats 1 and 2 eliminated, live players on 0, 3, 4 and 5. The
    /// button has advanced onto seat 1 — a vacated seat, which is what makes it
    /// a *dead* button.
    fn dead_button_table() -> Table {
        let seats = Seats::new(vec![
            seat("P0", 50_000),
            empty_seat(), // eliminated
            empty_seat(), // eliminated
            seat("P3", 50_000),
            seat("P4", 50_000),
            seat("P5", 50_000),
        ]);
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(100, 200));
        table.button = 1;
        table
    }

    /// **TDA Rule 32.** Blinds are assigned by **position**. The small blind
    /// position (seat 2) is empty, so it goes unposted and the big blind sits on
    /// seat 3 — not on seat 4, which is where walking past both empties lands.
    #[test]
    fn rule_32_dead_button_assigns_blinds_by_position_not_occupancy() {
        let table = dead_button_table();

        assert_eq!(
            2,
            table.determine_small_blind(),
            "TDA 32: the small blind is owed by the seat one to the button's \
             left, whether or not anybody is sitting there"
        );
        assert_eq!(
            3,
            table.determine_big_blind(),
            "TDA 32: with the small blind dead on seat 2, the big blind is the \
             first live player from seat 3 on"
        );
    }

    /// A dead small blind is **not posted**. Only the big blind reaches the pot.
    #[test]
    fn rule_32_dead_small_blind_is_not_posted() {
        let mut table = dead_button_table();
        assert!(table.is_small_blind_dead(), "seat 2 is vacant");

        table.act_forced_bets().expect("forced bets should post");

        assert_eq!(
            200,
            table.effective_pot(),
            "TDA 32: one big blind and nothing else — the small blind is dead"
        );
        assert_eq!(200, table.bet, "the bet to call is still a full big blind");
    }

    /// **TDA Rule 54-B, Example 1** — reachable at last.
    ///
    /// > Ex 1: PLO, 100-200 blinds, **dead SB**, BB posts 200. […] the
    /// > pot-limit bet for first player to act is **700**.
    ///
    /// `DEFECT_012` fixed the pot-limit calculation using Example 2 (a *short*
    /// blind), because Example 1's dead blind could not occur without the dead
    /// button. It can now, and the same 700 falls out.
    #[test]
    fn rule_54_b_ex1_dead_small_blind_does_not_shrink_the_pot_limit_maximum() {
        let seats = Seats::new(vec![
            seat("P0", 50_000),
            empty_seat(),
            empty_seat(),
            seat("P3", 50_000),
            seat("P4", 50_000),
            seat("P5", 50_000),
        ]);
        let mut table = Table::plo_from_seats(seats, (100, 200));
        table.button = 1;
        table.act_forced_bets().expect("forced bets should post");
        table.deal_cards_to_seats().expect("cards should deal");

        assert_eq!(200, table.effective_pot(), "only the big blind reached the pot");
        assert_eq!(100, table.blind_shortfall, "the whole small blind went unposted");
        assert_eq!(300, table.pot_limit_pot(), "TDA 54-B assumes full blinds");

        let utg = table.next_to_act();
        let (_, max) = table.raise_bounds(utg).expect("a pot-limit raise must be available");
        assert_eq!(
            700, max,
            "TDA 54-B Ex 1: a dead small blind must not shrink the maximum"
        );
    }

    /// Action order follows the blinds. With the small blind dead, first action
    /// pre-flop is the seat after the big blind.
    #[test]
    fn rule_32_utg_follows_the_big_blind_when_the_small_blind_is_dead() {
        let mut table = dead_button_table();
        table.act_forced_bets().expect("forced bets should post");
        table.deal_cards_to_seats().expect("cards should deal");

        assert_eq!(4, table.determine_utg(), "the seat after the big blind on 3");
        assert_eq!(4, table.next_to_act(), "and that is who acts first");
    }

    /// The guard against over-correction: on a full ring the dead button and
    /// the cash-game convention agree, and nothing about the common case moves.
    #[test]
    fn rule_32_full_ring_is_unchanged() {
        let mut table = nlhe(&[50_000; 6], 100, 200);
        table.button = 0;

        assert_eq!(1, table.determine_small_blind());
        assert_eq!(2, table.determine_big_blind());
        assert_eq!(3, table.determine_utg());
        assert!(!table.is_small_blind_dead());
    }

    /// **TDA Rule 54-B** — `DEFECT_008` D8-3.
    ///
    /// > Pre-flop **a dead or short all-in blind will not affect pot calculation. All
    /// > pre-flop pot and re-pot bets will assume full blinds were posted.** […] Ex 2:
    /// > SB posts 100, BB short posts 100. […] the pot-limit bet for first player to
    /// > act is **700**.
    ///
    /// `BettingStructure::max_raise` (`src/games/betting_structure.rs:170`) uses the
    /// pot it is handed, and no caller substitutes notional full blinds pre-flop, so a
    /// short big blind shrinks the maximum legal bet.
    #[test]
    fn rule_54_b_short_blind_must_not_shrink_the_preflop_pot_limit_maximum() {
        // PLO 100/200, seat 2 is the big blind with only 100 chips — a short post.
        let seats = Seats::new(vec![
            seat("Button", 50_000),
            seat("SmallBlind", 50_000),
            seat("ShortBigBlind", 100),
        ]);
        let mut table = Table::plo_from_seats(seats, (100, 200));
        table.act_forced_bets().expect("forced bets should post");
        table.deal_cards_to_seats().expect("cards should deal");

        let utg = table.next_to_act();
        let (_, max) = table
            .raise_bounds(utg)
            .expect("a pot-limit raise must be available to the first player to act");

        assert_eq!(
            700, max,
            "TDA 54-B Ex 2: the pot-limit maximum assumes full blinds were posted \
             (100 + 200 = 300 pot, 200 current bet, 200 to call), so it is 700 \
             regardless of the big blind being short"
        );
    }
}
