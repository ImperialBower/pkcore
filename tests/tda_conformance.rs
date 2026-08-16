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
//! 2. **Known defects** — every one is `#[ignore]`d with its `DEFECT_008` finding id.
//!    They assert the TDA-correct answer and therefore **fail today, by design**. CI
//!    stays green; run them on demand:
//!
//!    ```text
//!    cargo test --test tda_conformance -- --include-ignored
//!    ```
//!
//!    Un-`ignore` each one as its finding is fixed. That is the Gold Standard from
//!    `docs/EPIC-00f_Coverage.md` applied to this audit: right now a fix for any
//!    D8-N would make no existing test fail, which is what these close.
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
//! - **Rule 21 (side pots split separately)** — already covered by `tests/split_pots.rs`,
//!   added for `DEFECT_003`. Not duplicated.
//! - **Rule 36 (substantial action)** — `DEFECT_008` D8-5. No test is possible: the
//!   predicate does not exist, so any assertion referencing it fails to compile. The
//!   absence is verifiable only as `rg -c 'substantial_action' src/` returning zero.
//!   This is the one finding this harness cannot hold.
//!
//! TDA rules used by permission of the Poker TDA, <http://www.pokertda.com>,
//! all rights reserved.

#[allow(nonstandard_style)]
mod tda_2024_conformance {
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
    /// > Blinds 50-100. Post-flop, A opens betting for the 100 minimum. B goes all in
    /// > for a total of 125. C calls the 125. D goes all in for 200 total and E calls
    /// > 200. […] neither B's increment of 25 or D's increment of 75 is by itself a
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

    // ═══════════════════════════════════════════════════════════════════════
    // Known defects — DEFECT_008. These assert the TDA answer and fail today.
    // ═══════════════════════════════════════════════════════════════════════

    /// TDA 20-A ordering, as a reference implementation: among tied winners, the odd
    /// chip goes to the first seat to the **left of the button**, wrapping.
    fn tda_20a_first_seat_left_of_button(winners: &[u8], button: u8, seat_count: u8) -> u8 {
        (1..=seat_count)
            .map(|step| (button + step) % seat_count)
            .find(|s| winners.contains(s))
            .expect("at least one winner must be seated")
    }

    /// **TDA Rule 20-A** — `DEFECT_008` D8-1.
    ///
    /// > Board games with 2 or more high or low hands: the odd chip goes to the
    /// > **first seat left of the button**.
    ///
    /// `Stack::divvy_up` (`src/casino/cashier/chips.rs:95`) awards the remainder to the
    /// *last* indices of the winners vector, and `CaseEval::winning_seats`
    /// (`src/analysis/case_eval.rs:231`) yields seats in ascending order. Composed,
    /// the odd chip always lands on the highest-numbered winning seat, with no
    /// reference to the button at all.
    ///
    /// With the button on seat 7 and tied winners on seats 2 and 5, TDA awards the odd
    /// chip to seat 2; pkcore awards it to seat 5.
    #[test]
    #[ignore = "DEFECT_008 D8-1: odd chip goes to the highest-numbered winning seat, not first-left-of-button"]
    fn rule_20_a_odd_chip_goes_to_the_first_seat_left_of_the_button() {
        const SEAT_COUNT: u8 = 8;
        let winners: Vec<u8> = vec![2, 5]; // ascending, as winning_seats() returns
        let button: u8 = 7;

        let shares = Stack::new(101).divvy_up(winners.len());
        let odd_index = shares
            .iter()
            .position(|s| s.count() == 51)
            .expect("one share carries the odd chip");
        let awarded_to = winners[odd_index];

        assert_eq!(
            tda_20a_first_seat_left_of_button(&winners, button, SEAT_COUNT),
            awarded_to,
            "TDA 20-A: with the button on seat {button}, the odd chip belongs to the \
             first tied winner to its left"
        );
    }

    /// **TDA Rule 47-A** — `DEFECT_008` D8-2.
    ///
    /// > An all-in wager […] totaling less than a full bet or raise **will not reopen
    /// > betting for players who have already acted and are not facing at least a full
    /// > bet or raise** when the action returns to them.
    ///
    /// Sizing is handled (see `rule_47_ex1_*` above). The *gate* is not:
    /// `Table::raise_bounds` (`src/casino/table/actions.rs:337`) consults only the
    /// raise cap and the stack, and no per-seat has-acted state is tracked, so a
    /// player who already acted may still re-raise a sub-minimum increment.
    ///
    /// Here A raises to 300 (increment 200), B shoves 400 (increment 100 — short of a
    /// full raise), C calls. A now faces 100 more, which is not a full raise, so TDA
    /// permits only call or fold.
    #[test]
    #[ignore = "DEFECT_008 D8-2: no re-open gate; a player who already acted can re-raise a short all-in"]
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
    #[ignore = "DEFECT_008 D8-3: pot-limit pre-flop max uses the actual pot; a short blind shrinks it"]
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

    /// **TDA Rule 32** — `DEFECT_008` D8-4.
    ///
    /// > Tournament play will use a dead button.
    ///
    /// Under a dead button the blinds are assigned by **position**: a small blind
    /// position that is empty is simply not posted, and the big blind sits one seat
    /// further on. `Table::determine_small_blind` / `determine_big_blind`
    /// (`src/casino/table.rs:479`, `:518`) instead walk to the next *occupied* seat,
    /// which is the moving-button / live-blind convention used in cash games.
    ///
    /// Button on seat 1 with seats 1 and 2 eliminated: TDA puts the big blind on
    /// seat 3 (small blind dead on seat 2). pkcore walks past both empties and returns
    /// seat 4.
    #[test]
    #[ignore = "DEFECT_008 D8-4: dead button not implemented; blinds skip to the next occupied seat"]
    fn rule_32_dead_button_assigns_blinds_by_position_not_occupancy() {
        let seats = Seats::new(vec![
            seat("P0", 50_000),
            empty_seat(), // eliminated
            empty_seat(), // eliminated
            seat("P3", 50_000),
            seat("P4", 50_000),
            seat("P5", 50_000),
        ]);
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(100, 200));
        table.button = 1; // button advanced onto a vacated seat — a dead button

        assert_eq!(
            3,
            table.determine_big_blind(),
            "TDA 32: with the button dead on seat 1, the small blind position (seat 2) \
             is also empty and goes unposted, putting the big blind on seat 3"
        );
    }
}
