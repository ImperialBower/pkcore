//! The feature-free transition surface: [`Table::legal_actions`] and
//! [`Table::apply_action`].

use crate::PKError;
use crate::casino::table::TableOf;
use crate::seal::card_seal::CardSeal;

impl<S: CardSeal> TableOf<S> {
    /// Returns the [`PlayerAction`](crate::casino::action::PlayerAction)s that are
    /// legal for `seat_id` in the current betting state.
    ///
    /// This is the *advisory* half of the engine's transition surface (the
    /// dispatching half is [`Self::apply_action`]): it answers "what can this
    /// seat do now?" **without** mutating the table or trying-then-rolling-back an
    /// action. `Bet` and `Raise` are reported at their **minimum** legal size; any
    /// larger amount up to the structure's ceiling is also legal and is validated
    /// by [`Self::act_bet`] / [`Self::act_raise`] when applied.
    ///
    /// The raise checks mirror those in [`Self::act_raise`] exactly, so an action
    /// this method reports as legal will not then be rejected by the matching
    /// `act_*` method — that fidelity is the whole point of the surface, and is
    /// what lets betting-rule correctness be table-driven rather than probed.
    ///
    /// Returns an empty `Vec` for a seat with no decision: an unknown seat, or one
    /// that is all-in, folded, or busted — analogous to the empty action set at a
    /// terminal node in [`games::kuhn`](crate::games::kuhn).
    ///
    /// # Forced posts vs. voluntary betting
    ///
    /// This surface models *voluntary* betting only. Forced posts — blinds,
    /// antes, and the stud/razz 3rd-street bring-in — are posted by their own
    /// methods ([`Self::act_forced_bets`] / [`Self::act_bring_in`], driven by
    /// [`PokerSession`](crate::casino::session::PokerSession) at hand start), not
    /// chosen here, exactly as blinds are not a `PlayerAction`. Stud/razz
    /// voluntary betting *is* covered: once the bring-in is posted, the
    /// completer's `Raise(small_bet)` (completion) and the subsequent fixed-limit
    /// raises surface here like any other bet, because they flow through
    /// [`Self::to_call`] / [`Self::min_raise_to`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    ///
    /// // UTG faces the big blind: fold, call, raise, or shove — but never check.
    /// let utg = t.determine_utg();
    /// let actions = t.legal_actions(utg);
    /// assert!(actions.contains(&PlayerAction::Fold));
    /// assert!(actions.contains(&PlayerAction::Call));
    /// assert!(!actions.contains(&PlayerAction::Check));
    /// ```
    #[must_use]
    pub fn legal_actions(&self, seat_id: u8) -> Vec<crate::casino::action::PlayerAction> {
        use crate::casino::action::PlayerAction;

        let mut actions = Vec::new();

        let Some(seat) = self.seats.get_seat(seat_id) else {
            return actions;
        };
        // An all-in, folded, or busted seat has no decision to make. `is_in_hand`
        // is false once a seat has folded or been eliminated; an all-in seat is
        // still in the hand but has nothing left to decide.
        if !seat.player.is_in_hand() || seat.player.is_all_in() || seat.player.chips == 0 {
            return actions;
        }

        let to_call = self.to_call(seat_id);
        let stack = seat.player.total_chip_count();
        let min_bet = self.min_raise();
        // Single source of raise legality: the same `validate_raise` check
        // `act_raise` runs, so what we advertise here can never be rejected there
        // (audit P9b/P9j.1). `None` means no voluntary raise is legal.
        let raise_bounds = self.raise_bounds(seat_id);

        if to_call == 0 {
            // No live bet faces this seat, so it may check.
            actions.push(PlayerAction::Check);
            if self.bet == 0 {
                // Opening the betting is a Bet; the minimum open is `min_raise()`
                // (the big blind before any raise this street).
                if stack >= min_bet {
                    actions.push(PlayerAction::Bet(min_bet));
                }
            } else if let Some((min_raise_to, _)) = raise_bounds {
                // Big-blind option: the live bet is already matched, so re-opening
                // it is a Raise rather than a Bet.
                actions.push(PlayerAction::Raise(min_raise_to));
            }
        } else {
            // Facing a bet: fold and call are always available (`act_call`
            // converts a short stack into a partial all-in call).
            actions.push(PlayerAction::Fold);
            actions.push(PlayerAction::Call);
            if let Some((min_raise_to, _)) = raise_bounds {
                actions.push(PlayerAction::Raise(min_raise_to));
            }
        }

        // A short stack can always shove, even when a full min-raise is illegal;
        // a deep stack's shove degrades to the largest legal action inside
        // `act_all_in` (never an error), so AllIn is always accepted (audit P9b).
        if stack > 0 {
            actions.push(PlayerAction::AllIn);
        }

        actions
    }

    /// Apply a [`crate::casino::action::PlayerAction`] to the given seat.
    ///
    /// Translates the action enum variant to the corresponding `act_*` method.
    /// Returns `Err` if the action is illegal at this point in the hand (e.g.
    /// acting out of turn, invalid bet size).
    ///
    /// # Errors
    ///
    /// Propagates any [`PKError`] from the underlying `act_*` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// assert!(t.apply_action(utg, PlayerAction::Fold).is_ok());
    /// ```
    pub fn apply_action(&mut self, seat: u8, action: crate::casino::action::PlayerAction) -> Result<(), PKError> {
        use crate::casino::action::PlayerAction;
        match action {
            PlayerAction::Fold => {
                self.act_fold(seat)?;
            }
            PlayerAction::Check => {
                self.act_check(seat)?;
            }
            PlayerAction::Call => {
                // Degrade to check when the player already matches the current bet.
                if self.to_call(seat) == 0 {
                    self.act_check(seat)?;
                } else {
                    self.act_call(seat)?;
                }
            }
            PlayerAction::AllIn => {
                self.act_all_in(seat)?;
            }
            PlayerAction::Bet(n) => {
                self.act_bet(seat, n)?;
            }
            PlayerAction::Raise(n) => {
                self.act_raise(seat, n)?;
            }
        }
        Ok(())
    }
}

// ── Transition-surface tests (legal_actions / apply_action) ───────────────────
//
// P8: the audit's payoff — betting-rule correctness expressed as table-driven
// assertions instead of probe archaeology. Feature-free, like the surface itself.
#[cfg(test)]
#[allow(non_snake_case)]
mod transition_surface_tests {
    use crate::casino::action::PlayerAction;
    use crate::casino::game::ForcedBets;
    use crate::casino::state::PlayerState;
    use crate::casino::table::Table;
    use crate::casino::table::{Player, Seat, Seats};

    /// A 3-handed 50/100 NL table advanced to the first preflop decision (UTG
    /// facing the big blind).
    fn nlh_at_utg() -> Table {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("Alice".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Bob".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        t.act_forced_bets().expect("forced bets");
        t.deal_cards_to_seats().expect("deal");
        t
    }

    #[test]
    fn legal_actions__utg_facing_bb_is_fold_call_raise_allin_no_check() {
        let t = nlh_at_utg();
        let utg = t.next_to_act();
        let actions = t.legal_actions(utg);

        assert!(actions.contains(&PlayerAction::Fold));
        assert!(actions.contains(&PlayerAction::Call));
        assert!(actions.contains(&PlayerAction::Raise(t.min_raise_to())));
        assert!(actions.contains(&PlayerAction::AllIn));
        // Facing a live bet, checking is illegal, and there is no *opening* bet.
        assert!(!actions.contains(&PlayerAction::Check));
        assert!(!actions.iter().any(|a| matches!(a, PlayerAction::Bet(_))));
    }

    #[test]
    fn legal_actions__empty_for_folded_seat() {
        let mut t = nlh_at_utg();
        let utg = t.next_to_act();
        t.act_fold(utg).expect("fold");
        // A folded seat has no decision to make.
        assert!(t.legal_actions(utg).is_empty());
    }

    #[test]
    fn legal_actions__empty_for_unknown_seat() {
        let t = nlh_at_utg();
        assert!(t.legal_actions(99).is_empty());
    }

    #[test]
    fn apply_action__fold_advances_and_folds_the_seat() {
        let mut t = nlh_at_utg();
        let utg = t.next_to_act();
        t.apply_action(utg, PlayerAction::Fold).expect("apply fold");

        assert_eq!(PlayerState::Fold, t.seats.get_seat(utg).unwrap().player.state);
        assert_ne!(utg, t.next_to_act(), "action should have advanced to the next seat");
    }

    /// The crown-jewel invariant of the transition surface: an action reported
    /// as legal is never rejected when applied. Each action is applied to a fresh
    /// table so the mutations do not interfere.
    #[test]
    fn every_legal_action_is_accepted_by_apply_action() {
        let seat = nlh_at_utg().next_to_act();
        let actions = nlh_at_utg().legal_actions(seat);
        assert!(!actions.is_empty());

        for action in actions {
            let mut t = nlh_at_utg();
            assert!(
                t.apply_action(seat, action).is_ok(),
                "legal_actions reported {action:?} but apply_action rejected it"
            );
        }
    }

    /// Three seats limped to the big blind, who now has the option: `to_call`
    /// is 0 but a bet of one big blind already stands.
    fn nlh_at_big_blind_option() -> (Table, u8) {
        let mut t = nlh_at_utg();
        let utg = t.next_to_act();
        t.act_call(utg).expect("utg calls");
        let small_blind = t.next_to_act();
        t.act_call(small_blind).expect("sb completes");
        let big_blind = t.next_to_act();
        (t, big_blind)
    }

    /// `DEFECT_007`: `legal_actions` is explicit that re-opening an already
    /// matched bet is a `Raise`, never a `Bet`. This pins the reason — the two
    /// are not interchangeable, and a caller that sends the wrong one corrupts
    /// the betting ladder without being rejected.
    #[test]
    fn act_bet_over_a_standing_bet_matches_act_raise() {
        let (table, big_blind) = nlh_at_big_blind_option();
        assert_eq!(0, table.to_call(big_blind), "fixture: the option, not a call");
        assert_eq!(100, table.bet, "fixture: one big blind stands");

        let mut as_bet = table.clone();
        as_bet.apply_action(big_blind, PlayerAction::Bet(200)).expect("bet");

        let mut as_raise = table.clone();
        as_raise
            .apply_action(big_blind, PlayerAction::Raise(200))
            .expect("raise");

        assert_eq!(as_raise.bet, as_bet.bet, "both put 200 on the table");
        assert_eq!(
            as_raise.raise_increment, as_bet.raise_increment,
            "the increment is the delta over the standing bet (100), not the absolute amount"
        );
        assert_eq!(
            as_raise.min_raise(),
            as_bet.min_raise(),
            "an inflated increment doubles the next player's minimum re-raise"
        );
        assert_eq!(
            as_raise.raises_this_street, as_bet.raises_this_street,
            "re-opening a matched bet is a raise and must count toward the per-street cap"
        );
    }

    /// The opening-bet path is unchanged: with no bet standing the increment is
    /// the absolute amount, because the delta over zero is the amount.
    #[test]
    fn act_bet_opening_the_betting_records_the_full_amount_as_the_increment() {
        let mut t = nlh_at_utg();
        t.act_call(t.next_to_act()).expect("utg calls");
        t.act_call(t.next_to_act()).expect("sb completes");
        t.act_check(t.next_to_act()).expect("bb checks");
        t.bring_it_in().expect("sweep");
        t.deal_flop().expect("flop");
        t.seats.reset_state_in_hand();

        let actor = t.next_to_act();
        assert_eq!(0, t.bet, "fixture: the betting is open");
        t.apply_action(actor, PlayerAction::Bet(300)).expect("open for 300");
        assert_eq!(300, t.raise_increment);
        assert_eq!(300, t.bet);
    }

    // ── Stud/razz: voluntary betting after the forced bring-in ────────────────

    /// A 3-handed fixed-limit stud table (ante 2, bring-in 5, small bet 20, big
    /// bet 40) advanced past the forced bring-in to the first voluntary actor
    /// (the "completer"), mirroring `PokerSession::start_hand`'s setup order.
    fn stud_at_completer() -> Table {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("Alice".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Bob".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        let mut t = Table::stud_hi_from_seats(seats, 2, 5, 20, 40).unwrap();
        t.act_forced_bets().expect("antes");
        t.deal_stud_3rd_street().expect("deal 3rd");
        t.act_bring_in().expect("bring-in"); // forced post, like blinds
        t
    }

    #[test]
    fn legal_actions__stud_completer_can_fold_call_and_complete() {
        let t = stud_at_completer();
        let completer = t.next_to_act();
        let actions = t.legal_actions(completer);

        // Facing the partial bring-in: fold or call, and *complete* to the full
        // small bet — which surfaces as a Raise to `min_raise_to()` (== 20).
        assert!(actions.contains(&PlayerAction::Fold));
        assert!(actions.contains(&PlayerAction::Call));
        assert!(actions.contains(&PlayerAction::Raise(t.min_raise_to())));
        assert_eq!(20, t.min_raise_to(), "completion should target one small bet");
        assert!(actions.contains(&PlayerAction::AllIn));
        assert!(!actions.contains(&PlayerAction::Check));
    }

    /// Fidelity holds in the fixed-limit stud completion state too: every action
    /// `legal_actions` reports for the completer is accepted by `apply_action`.
    #[test]
    fn every_legal_action_is_accepted_by_apply_action__stud() {
        let completer = stud_at_completer().next_to_act();
        let actions = stud_at_completer().legal_actions(completer);
        assert!(!actions.is_empty());

        for action in actions {
            let mut t = stud_at_completer();
            assert!(
                t.apply_action(completer, action).is_ok(),
                "legal_actions reported {action:?} but apply_action rejected it in stud"
            );
        }
    }

    // P9b — fidelity at the fixed-limit raise cap. When no further raise is legal,
    // legal_actions must not offer Raise, but AllIn is still offered — and applying
    // it must NOT error. A deep-stacked "all-in" at the cap degrades to a call,
    // not a rerouted act_raise that trips RaiseCapReached (the pre-fix bug that
    // broke the crown-jewel invariant for capped structures).
    #[test]
    fn fixed_limit_all_in_at_cap_degrades_to_call_not_error() {
        let mut t = stud_at_completer();
        let completer = t.next_to_act();
        t.apply_action(completer, PlayerAction::Raise(20)).unwrap(); // complete to small bet
        t.raises_this_street = 99; // force the per-street raise cap

        let actor = t.next_to_act();
        let actions = t.legal_actions(actor);
        assert!(
            !actions.iter().any(|a| matches!(a, PlayerAction::Raise(_))),
            "no raise may be offered once the cap is reached"
        );
        assert!(
            actions.contains(&PlayerAction::AllIn),
            "all-in is still offered (fidelity requires it be accepted)"
        );

        assert!(
            t.apply_action(actor, PlayerAction::AllIn).is_ok(),
            "a deep-stack all-in at the FL cap must degrade to a call, not error"
        );
        assert!(
            !t.seats.get_seat(actor).unwrap().player.is_all_in(),
            "the deep stack called; it did not actually go all-in"
        );
    }
}
