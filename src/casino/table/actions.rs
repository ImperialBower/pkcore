//! Betting-action methods for [`Table`]: the universal [`Table::act`]
//! regulator and the player-action handlers it dispatches to.

use super::Seat;
use crate::PKError;
use crate::card::Card;
use crate::casino::action::TableAction;
use crate::casino::state::PlayerState;
use crate::casino::table::TableOf;
use crate::games::GameFamily;
use crate::games::GamePhase;
use crate::games::betting_structure::BettingStructure;
use crate::games::razz::california::California;
use crate::seal::card_seal::CardSeal;

impl<S: CardSeal> TableOf<S> {
    /// Universal action regulator: advances the table through whatever step is
    /// needed next.
    ///
    /// # Errors
    ///
    /// Propagates any error from the sub-action called.
    pub fn act(&mut self) -> Result<(), PKError>
    where
        S: CardSeal<Sealed = Card>,
    {
        match self.determine_betting_phase() {
            GamePhase::BettingPreFlop => {
                if !self.have_posted_blinds() {
                    self.act_forced_bets()?;
                }
                if !self.seats.are_dealt() {
                    self.deal_cards_to_seats()?;
                }
                if self.seats.is_betting_complete() {
                    self.bring_it_in()?;
                    self.deal_flop()?;
                }
                Ok(())
            }
            GamePhase::BettingFlop => {
                if self.seats.is_betting_complete() {
                    self.bring_it_in()?;
                    self.deal_turn()?;
                    self.seats.reset_state_in_hand();
                }
                Ok(())
            }
            GamePhase::BettingTurn => {
                if self.seats.is_betting_complete() {
                    self.bring_it_in()?;
                    self.deal_river()?;
                    self.seats.reset_state_in_hand();
                }
                Ok(())
            }
            GamePhase::BettingRiver => {
                if self.is_game_over() {
                    self.end_hand()?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Posts forced bets for the start of a hand.
    ///
    /// Dispatches on [`GameFamily`] (EPIC-32 Phase 2):
    /// - Hold'em / Omaha: posts SB + BB. Optional antes if `forced.ante > 0`.
    /// - Stud / Razz: posts antes for every active seat. The bring-in is
    ///   posted later by [`Self::act_bring_in`] after 3rd-street dealing.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if a posting seat cannot be found.
    pub fn act_forced_bets(&mut self) -> Result<(), PKError> {
        // Snapshot before any chips move so end_hand() can verify conservation.
        self.hand_chip_total = self.table_chip_count();
        match self.game.family() {
            GameFamily::StudHi | GameFamily::Razz => {
                self.act_antes()?;
            }
            _ => {
                if self.forced.ante > 0 {
                    self.act_antes()?;
                }
                self.act_forced_bet_small_blind()?;
                self.act_forced_bet_big_blind()?;
            }
        }
        self.phase = GamePhase::ForcedBets;
        Ok(())
    }

    /// Posts the ante for every non-empty seat with chips (EPIC-32 Phase 2).
    /// Used by stud-family hands at the start of every hand, and optionally
    /// by Hold'em/Omaha when `forced.ante > 0`.
    ///
    /// Antes are **dead money**: each ante goes straight into the pot rather
    /// than into `player.bet`. This matches standard rules — the ante does not
    /// count toward matching a bet (no caller gets ante credit) and the
    /// bring-in posts its full amount instead of only the difference above the
    /// ante. Chip conservation and the `pot == Σ chips_in_play` showdown
    /// invariant are preserved because the ante moves through `chips_in_play`
    /// rather than `bet`.
    ///
    /// # Errors
    ///
    /// This method does not currently return an error; the `Result` is kept
    /// for signature stability with the rest of the forced-bet API.
    pub fn act_antes(&mut self) -> Result<(), PKError> {
        let ante = self.forced.ante;
        if ante == 0 {
            return Ok(());
        }
        let count = self.seats.size();
        for idx in 0..count {
            let actual = self.seats.post_dead_ante(idx, ante);
            if actual > 0 {
                self.pot += actual;
                self.log(TableAction::BetAnteForced(idx, actual));
            }
        }
        Ok(())
    }

    /// Posts the stud bring-in (EPIC-32 Phase 4). Dispatches on
    /// `game.family()`:
    /// - `StudHi`: lowest 3rd-street upcard pays.
    /// - `Razz`: highest 3rd-street upcard pays (EPIC-33).
    /// - Other families: returns `PKError::InvalidAction`.
    ///
    /// Uses only the **first** upcard in dealing order per seat (the
    /// 3rd-street upcard). This matters during hand-history replay where
    /// all 7 cards may already be present in `seat.hand`: bring-in
    /// selection must consider only the card visible at 3rd street, not
    /// all four eventual upcards.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidAction` if called on a non-stud-family game.
    /// - `PKError::NotDealt` if no seat has a face-up 3rd-street card.
    /// - `PKError::InvalidSeatNumber` if the chosen seat can't be found.
    pub fn act_bring_in(&mut self) -> Result<(), PKError> {
        let highest = matches!(self.game.family(), GameFamily::Razz);
        let in_stud_family = matches!(self.game.family(), GameFamily::StudHi | GameFamily::Razz);
        if !in_stud_family {
            return Err(PKError::InvalidAction);
        }
        let seat_idx = self
            .third_street_extreme_upcard_seat(highest)
            .ok_or(PKError::NotDealt)?;
        let amount = self.forced.bring_in;
        let actual = self.seats.act_forced_bet(seat_idx, amount)?;
        self.bet = self.bet.max(amount);
        self.log(TableAction::StudBringInPost(seat_idx, actual));
        Ok(())
    }

    /// EPIC-32 Phase 12: returns the active seat with the extreme
    /// 3rd-street upcard — highest for Razz (`highest = true`, ace ranked
    /// low), lowest for Stud Hi (`highest = false`, ace ranked high).
    /// Considers only each seat's **first** up-tagged card in dealing
    /// order — i.e. the 3rd-street upcard. Used by [`Self::act_bring_in`] so
    /// that replay (which has all 7 cards present) picks the same
    /// bring-in seat as the live session (which had only one upcard
    /// per seat when bring-in was selected).
    pub(super) fn third_street_extreme_upcard_seat(&self, highest: bool) -> Option<u8> {
        // `highest` is the scan *direction* (does the extreme upcard bring in?);
        // the ace-high-vs-low rank order is an independent property owned by the
        // game family (audit P9j.5). Today Razz is the only family that both
        // scans highest and ranks the ace low, but keeping the axes separate
        // means a deuce-to-seven variant (highest, ace-high) stays expressible.
        let ace_low = self.game.family().ranks_ace_low();
        let mut best: Option<(u8, u8, u8)> = None;
        for (idx, seat) in self.seats.0.iter().enumerate() {
            if seat.is_empty() || !seat.is_in_hand() {
                continue;
            }
            let Ok(seat_idx) = u8::try_from(idx) else {
                continue;
            };
            // First up-tagged card in dealing order.
            let Some(hole_card) = seat.hand.iter().find(|hc| hc.is_up()) else {
                continue;
            };
            let card = hole_card.card();

            let rank_key = if ace_low {
                California::ace_low_rank(card.get_rank())
            } else {
                card.get_rank() as u8
            };

            let suit = card.get_suit() as u8;
            let candidate = (seat_idx, rank_key, suit);
            match best {
                None => best = Some(candidate),
                Some((_, br, bs)) => {
                    let better = if highest {
                        rank_key > br || (rank_key == br && suit > bs)
                    } else {
                        rank_key < br || (rank_key == br && suit < bs)
                    };
                    if better {
                        best = Some(candidate);
                    }
                }
            }
        }
        best.map(|(seat, _, _)| seat)
    }

    /// Posts the small blind.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat is not found.
    pub fn act_forced_bet_small_blind(&mut self) -> Result<(), PKError> {
        let sb = self.determine_small_blind();
        // TDA 2024 Rule 32 (DEFECT_013): under a dead button the seat that owes
        // the small blind may be vacant. A dead blind is not posted by anyone —
        // it is not passed on to the next live player, which is the whole
        // difference from the cash-game moving-button convention.
        let actual = if self.is_small_blind_dead() {
            0
        } else {
            self.seats.act_forced_bet(sb, self.forced.small_blind)?
        };
        // TDA 2024 Rule 54-B (DEFECT_012): remember what was owed but never
        // posted, so the pre-flop pot-limit ceiling can assume full blinds. A
        // dead blind contributes its whole amount here; a short all-in blind
        // contributes the part it could not cover.
        self.blind_shortfall = self
            .blind_shortfall
            .saturating_add(self.forced.small_blind.saturating_sub(actual));
        self.log(TableAction::ForcedBetSmallBlind(sb, actual));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(())
    }

    /// Posts the big blind.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat is not found.
    pub fn act_forced_bet_big_blind(&mut self) -> Result<(), PKError> {
        let bb = self.determine_big_blind();
        let actual = self.seats.act_forced_bet(bb, self.forced.big_blind)?;
        // TDA 2024 Rule 54-B (DEFECT_012). Note `self.bet` below is already
        // 54-B compliant: the bet to call is the *full* big blind even when the
        // post fell short. Only the pot term needed fixing.
        self.blind_shortfall = self
            .blind_shortfall
            .saturating_add(self.forced.big_blind.saturating_sub(actual));
        self.bet = self.forced.big_blind;
        self.log(TableAction::ForcedBetBigBlind(bb, actual));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(())
    }

    /// Folds the seat identified by `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    /// - `PKError::InvalidSeatNumber` if the seat is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// t.act_fold(utg).unwrap();
    /// assert_eq!(PlayerState::Fold, t.seats.get_seat(utg).unwrap().player.state);
    /// ```
    pub fn act_fold(&mut self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Fold);
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        let folded_chips = self.seats.act_fold(seat_number)?;
        self.record_voluntary_action(seat_number, ChipCommitment::NoChips);
        self.pot += folded_chips;
        self.log(TableAction::Fold(seat_number));
        self.log(TableAction::BringItIn(folded_chips));
        self.log(TableAction::PotSize(self.pot));
        self.player_mucks_cards(seat_number);
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(folded_chips)
    }

    /// The maximum legal raise-to `amount` for `seat` under the current betting
    /// structure — the single-sourced form of the 5-argument
    /// [`BettingStructure::max_raise`] incantation (audit P9j.1). Returns 0 for
    /// an unknown seat.
    ///
    /// This is the *ceiling* only; a full raise must also clear
    /// [`Self::min_raise_to`] and the per-street cap — see [`Self::validate_raise`].
    #[must_use]
    pub(super) fn max_raise_for(&self, seat_number: u8) -> usize {
        let Some(seat) = self.seats.get_seat(seat_number) else {
            return 0;
        };
        let stack = seat.player.total_chip_count();
        self.betting.max_raise(
            // TDA 2024 Rule 54-B (DEFECT_012): pre-flop this is the pot as if
            // full blinds had been posted.
            self.pot_limit_pot(),
            self.bet,
            seat.player.bet,
            stack,
            self.current_bet_tier(),
        )
    }

    /// Validates a would-be *non-all-in* raise-to `amount` for `seat`: the
    /// minimum increment, the per-street raise cap, and the structure ceiling.
    ///
    /// This is the single source of truth for raise legality, executed by
    /// [`Self::act_raise`] before it mutates and queried by [`Self::raise_bounds`]
    /// / [`Self::legal_actions`]. Because the advisory surface and the mutating
    /// surface call the *same* function, they cannot drift (audit P9b, P9j.1).
    /// The all-in bypass is intentionally *not* applied here — callers handle
    /// all-in separately.
    ///
    /// # Errors
    ///
    /// - `PKError::InsufficientIncrement` if `amount` is below `min_raise_to()`.
    /// - `PKError::RaiseCapReached` if the per-street cap is hit.
    /// - `PKError::ExceedsBettingCap` if `amount` exceeds the structure ceiling.
    fn validate_raise(&self, seat_number: u8, amount: usize) -> Result<(), PKError> {
        if amount < self.min_raise_to() {
            return Err(PKError::InsufficientIncrement);
        }
        if self.betting.cap_reached(self.raises_this_street) {
            return Err(PKError::RaiseCapReached);
        }
        if amount > self.max_raise_for(seat_number) {
            return Err(PKError::ExceedsBettingCap);
        }
        Ok(())
    }

    /// The legal raise-to range `[min, max]` for `seat`, or `None` when no
    /// voluntary (non-all-in) raise is legal right now — the cap is reached, or
    /// the stack cannot cover the minimum raise. In fixed-limit `min == max`
    /// (one legal amount). Derived entirely from `validate_raise` and
    /// `max_raise_for`, so it agrees with `act_raise` by construction.
    ///
    /// Used by [`Self::legal_actions`] to advertise `Raise(min)` and by the sim
    /// to clamp a decider's oversize raise deterministically. All-in-for-less is
    /// not a raise and is not represented here.
    #[must_use]
    pub fn raise_bounds(&self, seat_number: u8) -> Option<(usize, usize)> {
        // TDA 2024 Rule 47-A rights gate (DEFECT_010) — see `is_reopen_gated`.
        if self.is_reopen_gated(seat_number) {
            return None;
        }
        // validate_raise(min) folds in the remaining reasons a raise could be
        // illegal (cap reached, min above the structure ceiling because the
        // stack is short).
        let min = self.min_raise_to();
        if self.validate_raise(seat_number, min).is_err() {
            return None;
        }
        Some((min, self.max_raise_for(seat_number)))
    }

    /// TDA 2024 Rule 47-A — is this seat barred from raising because the
    /// betting was never re-opened for it?
    ///
    /// An all-in (or a run of them) totalling less than a full raise does not
    /// re-open the betting for a player who has already acted and is not now
    /// facing at least a full raise. Such a player may only call or fold.
    ///
    /// Three properties are worth reading off the implementation:
    ///
    /// - **Scoped to no-limit and pot-limit**, because Rule 47-A names only
    ///   those. Fixed-limit has its own half-a-bet rule and is left alone.
    /// - **Cumulative for free.** Measuring against the level this seat last
    ///   acted at — rather than against the last single all-in — means two
    ///   short all-ins that together make a full raise correctly *do* re-open.
    ///   That is 47-A's cumulative clause with no extra machinery.
    /// - **The big-blind option is safe.** `is_yet_to_act_or_blind`'s
    ///   `Blind(_)` arm means a big blind who has posted but not yet taken the
    ///   option counts as not having acted, so the gate cannot fire on them.
    ///
    /// This is the single implementation of the rule. [`Self::raise_bounds`]
    /// and [`TableSnapshot::from_table`](crate::bot::table_snapshot::TableSnapshot::from_table)
    /// both call it, so the engine's view and a bot's view cannot drift.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 50_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 400)),
    ///     Seat::new(Player::new_with_chips("C".to_string(), 50_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    ///
    /// let a = t.next_to_act();
    /// t.act_raise(a, 300).unwrap();          // A raises, increment 200
    /// let b = t.next_to_act();
    /// t.act_all_in(b).unwrap();              // B shoves 400 — only 100 more
    /// let c = t.next_to_act();
    /// t.act_call(c).unwrap();
    ///
    /// // A faces 100, short of the 200 full raise: call or fold only.
    /// assert!(t.is_reopen_gated(a));
    /// assert_eq!(None, t.raise_bounds(a));
    /// ```
    #[must_use]
    pub fn is_reopen_gated(&self, seat_number: u8) -> bool {
        if !matches!(self.betting, BettingStructure::NoLimit | BettingStructure::PotLimit) {
            return false;
        }
        let Some(seat) = self.seats.get_seat(seat_number) else {
            return false;
        };
        let has_acted = !seat.is_yet_to_act_or_blind();
        let facing = self.bet.saturating_sub(seat.bet_level_when_last_acted);
        has_acted && facing < self.min_raise()
    }

    /// Places a bet of `amount` for seat `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    /// - `PKError::InsufficientIncrement` if `amount` is below the minimum
    ///   opening bet and the player is not going all-in for less.
    /// - `PKError::InsufficientChips` if not enough chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// t.act_bet(utg, 200).unwrap();
    /// assert_eq!(200, t.bet);
    /// ```
    pub fn act_bet(&mut self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Bet(amount));
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        // Pre-validate BEFORE any mutation (mirror act_raise; audit P9d). An
        // opening bet is a raise-from-zero: it must clear the minimum and the
        // structure ceiling unless the player is betting their whole stack
        // (all-in for less). Without this guard `seats.act_bet` mutated the seat
        // and then `set_raise_increment` rejected, stranding a live Bet with
        // `table.bet` still 0 and the seat no longer next to act.
        if let Some(seat) = self.seats.get_seat(seat_number) {
            let would_be_all_in = amount >= seat.player.total_chip_count();
            if !would_be_all_in {
                self.validate_raise(seat_number, amount)?;
            }
        }
        let remaining = self.seats.act_bet(seat_number, amount)?;
        // DEFECT_007: the increment is the delta over whatever already stood,
        // matching `act_raise`. Identical to `amount` on the intended path (an
        // opening bet, where `self.bet` is 0). It differs only when a caller
        // sends `Bet` where `legal_actions` advertises `Raise` — the big-blind
        // option — and there the absolute amount inflated `raise_increment`,
        // silently doubling the next player's minimum re-raise.
        self.set_raise_increment(seat_number, amount.saturating_sub(self.bet));
        if self.bet > 0 {
            // Re-opening an already-matched bet is a raise however it is
            // spelled, so it counts toward the per-street cap.
            self.raises_this_street = self.raises_this_street.saturating_add(1);
        }
        self.bet = amount;
        self.record_voluntary_action(seat_number, ChipCommitment::Chips);
        self.log(TableAction::Bet(seat_number, amount));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(remaining)
    }

    /// Calls the current bet for seat `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// t.act_call(utg).unwrap();
    /// assert_eq!(PlayerState::Call(100), t.seats.get_seat(utg).unwrap().player.state);
    /// ```
    pub fn act_call(&mut self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Call(0));
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        let call_target = self.bet;
        let seat_bet = self.seats.get_seat(seat_number).map_or(0, |s| s.player.bet);
        let to_call = call_target.saturating_sub(seat_bet);
        let seat = self.seats.get_seat_mut(seat_number).ok_or(PKError::InvalidSeatNumber)?;
        let actual_added = if to_call == 0 {
            seat.player.act_check()?;
            0
        } else if seat.player.chips < to_call {
            // Caller cannot cover the full call target — go all-in for partial.
            // Side pots and uncalled-bet returns at showdown reconcile the difference
            // (see docs/defects/DEFECT_001_BUGFIX_short_blind_call_target.md).
            let total_bet = seat.player.act_all_in()?;
            total_bet.saturating_sub(seat_bet)
        } else {
            seat.player.act_call(call_target)?;
            to_call
        };
        self.record_voluntary_action(seat_number, ChipCommitment::Chips);
        self.log(TableAction::Call(seat_number, actual_added));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(actual_added)
    }

    /// Checks for seat `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// // Force everyone to 0 bet with no active blind by resetting state.
    /// // (doc-test only shows the API; actual game flow requires proper sequencing)
    /// let _ = t; // just verify it compiles
    /// ```
    pub fn act_check(&mut self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Check);
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        let remaining = self.seats.act_check(seat_number)?;
        self.record_voluntary_action(seat_number, ChipCommitment::NoChips);
        self.log(TableAction::Check(seat_number));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(remaining)
    }

    /// Raises to `amount` for seat `seat_number`.
    ///
    /// `amount` is the **total raise-to** value — the new table-level bet that all
    /// other players must match.  It must be at least `table.bet + table.min_raise()`
    /// unless the player is going all-in for less.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    /// - `PKError::InsufficientIncrement` if `amount` is below the minimum raise
    ///   and the player is not going all-in.
    /// - `PKError::InsufficientChips` if not enough chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// t.act_raise(utg, 300).unwrap();
    /// assert_eq!(PlayerState::Raise(300), t.seats.get_seat(utg).unwrap().player.state);
    ///
    /// // Under-minimum raise is rejected before any state changes.
    /// let utg2 = t.next_to_act();
    /// assert!(t.act_raise(utg2, 301).is_err()); // below min (300 + 100 = 400)
    /// // The seat is still the active player — no state was corrupted.
    /// assert_eq!(utg2, t.next_to_act());
    /// ```
    pub fn act_raise(&mut self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Raise(amount));
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        // Pre-validate the raise BEFORE any state is modified. Without this
        // guard, act_bet_internal deducts chips for an under-sized raise and
        // sets the seat to Raise(_); then the seat is corrupt and no longer
        // "next to act". Validation is delegated to `validate_raise` — the same
        // check `legal_actions`/`raise_bounds` query, so the advisory and
        // mutating surfaces cannot drift (audit P9b/P9j.1). All-in bypasses it
        // (a short stack can always shove for less; NoLimit's max_raise == stack,
        // so oversized amounts route through the all-in branch above anyway).
        if let Some(seat) = self.seats.get_seat(seat_number) {
            let would_be_all_in = amount >= seat.player.total_chip_count();
            if !would_be_all_in {
                self.validate_raise(seat_number, amount)?;
            }
        }
        let remaining = self.seats.act_raise(seat_number, amount)?;
        self.set_raise_increment(seat_number, amount.saturating_sub(self.bet));
        self.bet = amount;
        // EPIC-30 Phase 3: count this raise toward the per-street cap.
        // Saturating add so a misconfigured raise_cap can't panic via
        // overflow (the cap_reached guard above prevents this anyway).
        self.raises_this_street = self.raises_this_street.saturating_add(1);
        self.record_voluntary_action(seat_number, ChipCommitment::Chips);
        self.log(TableAction::Raise(seat_number, amount));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(remaining)
    }

    /// Commits seat `seat_number`'s whole stack, returning the chips **committed**.
    ///
    /// In No-Limit this is an unconditional all-in for the full stack. In a
    /// **capped** structure (Fixed-Limit / Pot-Limit) a deep stack has no true
    /// "all-in raise", so the shove is degraded to the largest legal action and
    /// the player is *not* left all-in:
    /// - a legal raise exists and the stack overflows its ceiling → raise to the
    ///   max (e.g. the pot-limit clamp), returning the amount committed;
    /// - no legal raise remains (the cap is reached, or the bet is already
    ///   matched) → a plain call;
    /// - the stack is smaller than the max legal raise → a genuine all-in for less.
    ///
    /// This keeps the `AllIn` that [`Self::legal_actions`] advertises always
    /// acceptable by [`Self::apply_action`] (audit P9b), and the return value is
    /// chips-committed on every path (audit P9e).
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// // No-Limit: a real all-in for the whole stack.
    /// t.act_all_in(utg).unwrap();
    /// assert!(t.seats.get_seat(utg).unwrap().player.is_all_in());
    /// ```
    pub fn act_all_in(&mut self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let available = self
                .seats
                .get_seat(seat_number)
                .map_or(0, |s| s.player.total_chip_count());
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::AllIn(available));
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }

        // Capped structures (Fixed-Limit, Pot-Limit) have no true "all-in raise"
        // for a deep stack: the most it can commit voluntarily is one legal
        // raise. Degrade a deep-stack shove to the largest legal action so the
        // AllIn that `legal_actions` advertises is always accepted (audit P9b).
        // NoLimit's max_raise == stack, so this whole branch is a no-op for NLHE.
        if !self.betting.is_no_limit() {
            let stack = self
                .seats
                .get_seat(seat_number)
                .map_or(0, |s| s.player.total_chip_count());
            match self.raise_bounds(seat_number) {
                // A legal raise exists and the stack overflows its ceiling: raise
                // to the max. Normalize the return to chips *committed*
                // (`act_raise` reports chips remaining), matching every other
                // all-in path (audit P9e).
                Some((_, max)) if stack > max => {
                    self.act_raise(seat_number, max)?;
                    let committed = self.seats.get_seat(seat_number).map_or(max, |s| s.player.bet);
                    return Ok(committed);
                }
                // Stack fits within the max raise: fall through to a true all-in.
                Some(_) => {}
                // No voluntary raise is legal (cap reached, or the bet is already
                // matched). A deep stack can then only call; a stack that cannot
                // cover the call is the genuine all-in-for-less handled below.
                None => {
                    if stack > self.to_call(seat_number) {
                        return self.act_call(seat_number);
                    }
                }
            }
        }

        let old_bet = self.bet;
        let amount = self.seats.act_all_in(seat_number)?;
        self.bet = self.bet.max(amount);

        // A shove that is at least a full raise re-opens the betting: record the
        // new increment so the next player's minimum re-raise is measured from it
        // (audit P9f). A sub-min all-in does NOT re-open — leave the increment
        // untouched so players who already acted may only call the extra (Part V).
        let raise_delta = self.bet.saturating_sub(old_bet);
        if raise_delta >= self.min_raise() {
            self.raise_increment = raise_delta;
            self.raises_this_street = self.raises_this_street.saturating_add(1);
        }

        self.record_voluntary_action(seat_number, ChipCommitment::Chips);
        self.log(TableAction::AllIn(seat_number, amount));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(amount)
    }

    /// Stores the raise increment (the delta over the previous bet) after a
    /// bet/raise has already been applied. A pure store now that both
    /// [`Self::act_bet`] and [`Self::act_raise`] pre-validate the amount against
    /// [`Self::min_raise_to`] before mutating (audit P9j.3). An all-in never
    /// updates the increment here: a sub-min all-in must not re-open the action,
    /// and a full-raise all-in is handled in [`Self::act_all_in`].
    fn set_raise_increment(&mut self, seat_number: u8, amount: usize) {
        let is_all_in = self.seats.get_seat(seat_number).is_some_and(Seat::is_all_in);
        if !is_all_in {
            self.raise_increment = amount;
        }
    }

    /// TDA 2024 Rule 36 — **substantial action** (`DEFECT_009`).
    ///
    /// > Substantial Action is either **A)** any 2 actions in turn, at least
    /// > one of which puts chips in the pot (i.e. any 2 actions except 2 checks
    /// > or 2 folds) or **B)** any combination of 3 actions in turn (check,
    /// > bet, raise, call, fold). **Posted blinds do not count towards SA.**
    ///
    /// SA is the point in a betting round past which an error stops being
    /// correctable — the boundary condition Rules 22, 34-A, 35-D, 52-A and
    /// 53-B all key off. It is deliberately kept separate from
    /// [`raises_this_street`](crate::casino::table::Table::raises_this_street), which answers a
    /// different question for a different consumer (the fixed-limit raise cap).
    ///
    /// Clause A is written here as "2 actions **and** at least one with chips"
    /// rather than the rule's own "any 2 actions except 2 checks or 2 folds".
    /// The two are equivalent — the only chipless actions are the check and the
    /// fold — and the positive form is directly checkable.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 10_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 10_000)),
    ///     Seat::new(Player::new_with_chips("C".to_string(), 10_000)),
    /// ]);
    /// let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// table.act_forced_bets().unwrap();
    /// table.deal_cards_to_seats().unwrap();
    ///
    /// // Posted blinds are not actions.
    /// assert!(!table.substantial_action());
    ///
    /// // One in-turn call is one action — short of both clauses.
    /// let utg = table.next_to_act();
    /// table.act_call(utg).unwrap();
    /// assert!(!table.substantial_action());
    ///
    /// // A second action, and one of the two committed chips: clause A.
    /// let sb = table.next_to_act();
    /// table.act_fold(sb).unwrap();
    /// assert!(table.substantial_action());
    /// ```
    #[must_use]
    pub fn substantial_action(&self) -> bool {
        self.actions_this_street >= 3 || (self.actions_this_street >= 2 && self.chip_actions_this_street >= 1)
    }

    /// Stamps the seat with the table-level `bet` as it stands **after** the
    /// voluntary action just applied (`DEFECT_010`, TDA 2024 Rule 47-A) and
    /// counts the action toward substantial action (`DEFECT_009`, TDA 2024
    /// Rule 36).
    ///
    /// Called from the six voluntary entry points only — `act_fold`,
    /// `act_check`, `act_bet`, `act_call`, `act_raise`, `act_all_in` — each
    /// *after* its turn guard has passed and after `self.bet` is final, so a
    /// rejected out-of-turn attempt never stamps anything and never counts.
    /// That single choke point is what keeps the two rules' definitions of "an
    /// action" from drifting apart.
    ///
    /// The forced-post paths (`act_forced_bets`, `act_antes`, `act_bring_in`)
    /// deliberately do **not** call this: a posted blind is not an action, so a
    /// big blind who has not yet exercised the option must not be treated as
    /// having acted, and Rule 36 excludes posted blinds in as many words. The
    /// stud bring-in is excluded on the same grounds — that is an
    /// interpretation, since Rule 36 names blinds only, but the bring-in is
    /// structurally a forced post and is treated as one throughout pkcore.
    ///
    /// Post-action rather than pre-action is the whole point for 47-A. A player
    /// who raises to 300 has to be measured against 300 when the action comes
    /// back to them, not against the 100 they faced before raising.
    fn record_voluntary_action(&mut self, seat_number: u8, chips: ChipCommitment) {
        let level = self.bet;
        if let Some(seat) = self.seats.get_seat_mut(seat_number) {
            seat.bet_level_when_last_acted = level;
        }
        self.actions_this_street = self.actions_this_street.saturating_add(1);
        if chips == ChipCommitment::Chips {
            self.chip_actions_this_street = self.chip_actions_this_street.saturating_add(1);
        }
    }
}

/// Whether a voluntary action put chips in the pot — the distinction TDA 2024
/// Rule 36 clause A turns on. A bet, call, raise or all-in commits chips; a
/// check or a fold does not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ChipCommitment {
    Chips,
    NoChips,
}
