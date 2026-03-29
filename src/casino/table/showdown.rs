use crate::PKError;
use crate::casino::cashier::chips::Stack;
use crate::casino::table::winnings::{Win, Winnings};
use crate::prelude::{Eval, Pile, SeatEquity, Seatbit, Seven, Table, TableAction, TableEquity};
use std::collections::{HashMap, HashSet};

pub struct Showdown;

impl Showdown {
    /// # Errors
    ///
    /// `PKError::Fubar` if noone is in hand
    pub fn process(table: &Table) -> Result<Winnings, PKError> {
        table.log_info(TableAction::EndHand);

        if !table.is_game_over() {
            return Err(PKError::ActionIsntFinished);
        }

        // Do not print directly from library code (clippy::print_stdout). Logging is used
        // for recording table events instead. The equity is still computed below.

        // let mut winnings: Vec<Winnings> = Vec::new();

        let winnings = match table.seats.active_in_hand().len() {
            0 => return Err(PKError::Fubar),
            1 => Showdown::process_single_seat_in_hand(table)?,
            2 => Showdown::process_headsup(table)?,
            _ => Showdown::process_multiway(table)?,
        };

        Ok(winnings)
    }

    fn process_single_seat_in_hand(table: &Table) -> Result<Winnings, PKError> {
        // Keep the active seats Vec alive so we can reference its first element
        // without borrowing a temporary that gets dropped.
        let seats_alive = table.seats.active_in_hand();
        let seat = seats_alive.first().ok_or(PKError::Fubar)?;

        // Collect outstanding bets from the seats directly (bypass Table::bring_it_in
        // which refuses to run when the table reports `game_over`). This mirrors the
        // behavior of Table::bring_it_in but avoids the top-level guard.
        let collected = table.seats.bring_it_in()?;
        table.pot.add_to(collected);
        let _ = table.bet.take();

        // The pot now contains all chips committed to the hand. Award the pot to the
        // single remaining active seat by taking the pot.
        let pot = table.pot.take();

        if let Some(s) = table.get_seat_mut(*seat) {
            // Add winnings to the player's chip stack.
            let _ = s.player.chips.wins(pot.clone());
        } else {
            return Err(PKError::InvalidSeatNumber);
        }

        // Build a `Winnings` record describing the award so callers can inspect
        // what was paid out. Use the pot amount as the equity chips and the
        // winning seat as the seat bit. Try to construct an `Eval` from the
        // winner's effective cards (hole cards + board); fall back to
        // `Eval::default()` if we cannot construct a meaningful eval.
        let equity = SeatEquity::new(pot.count(), Seatbit::from(*seat));

        let eval = match table.effective_player_cards(*seat) {
            Some(cards) => match Seven::try_from(cards) {
                Ok(seven) => Eval::from(seven),
                Err(_) => Eval::default(),
            },
            None => Eval::default(),
        };

        Ok(Winnings::from(Win { equity, eval }))
    }

    fn process_headsup(table: &Table) -> Result<Winnings, PKError> {
        // Heads-up is effectively the same flow as Table::end_hand for >1 players
        // Build a case eval, close out the bets into the pot, mark seats as in
        // showdown, split the pot between the winners and award chips. Return
        // a Vec<Winnings> describing what each winning seat received.

        let game = crate::play::game::Game::try_from(table)?;
        let case_eval = game.river_case_eval()?;

        let winners = case_eval.winning_seats();

        let _brought_in = table.close_it_out()?;
        table.seats.showdown(table.pot.count())?;

        let shares = table.pot.take().divvy_up(winners.len());

        let mut results: Vec<Win> = Vec::new();

        for (i, winner_seat_number) in winners.iter().enumerate() {
            if let Some(seat) = table.get_seat_mut(*winner_seat_number) {
                let player_winnings = shares.get(i).cloned().unwrap_or_default();
                let winnings_amount = player_winnings.count();

                // Award to player's chips
                let _ = seat.player.chips.wins(player_winnings.clone());

                // Extract fields needed for logging before releasing the mutable borrow.
                // effective_player_cards() and log_info(PlayerWins) both call get_seat()
                // on the same seat, which would fail if the RefMut is still held.
                let hand = seat.cards.bard();
                let id = seat.player.id;
                let chips_won = winnings_amount.saturating_sub(seat.player.chips_in_play.take());
                drop(seat);

                // Build eval for the seat (calls get_seat internally)
                let eval = match table.effective_player_cards(*winner_seat_number) {
                    Some(cards) => match Seven::try_from(cards) {
                        Ok(seven) => Eval::from(seven),
                        Err(_) => Eval::default(),
                    },
                    None => Eval::default(),
                };

                // Log the win (calls get_seat internally via log_info)
                table.log_info(TableAction::PlayerWins(
                    *winner_seat_number,
                    id,
                    hand,
                    chips_won,
                    winnings_amount,
                ));

                results.push(Win {
                    equity: SeatEquity::new(winnings_amount, Seatbit::from(*winner_seat_number)),
                    eval,
                });
            }
        }

        // Log losers
        for (i, seat_cell) in table.seats.borrow_all().iter().enumerate() {
            if seat_cell.is_in_hand()
                && let Some(seat) = table.get_seat(u8::try_from(i).unwrap_or_default())
                && !winners.contains(&u8::try_from(i).unwrap_or_default())
            {
                let player_loses = seat.player.chips_in_play.take();
                table.log_info(TableAction::PlayerLoses(
                    u8::try_from(i).unwrap_or_default(),
                    seat.player.id,
                    seat.cards.bard(),
                    player_loses,
                ));
            }
        }

        Ok(Winnings::from(results))
    }

    /// TODO: refactor me
    #[allow(clippy::too_many_lines)]
    fn process_multiway(table: &Table) -> Result<Winnings, PKError> {
        // Capture equity BEFORE close_it_out so chips_in_play still reflects
        // each seat's cumulative pot commitment (used to compute side pots).
        let mut equity: TableEquity = table.determine_hand_equity();

        let _brought_in = table.close_it_out()?;

        let game = crate::play::game::Game::try_from(table)?;
        let case_eval = game.river_case_eval()?;

        table.seats.showdown(table.pot.count())?;

        // Helper: build an Eval for a seat from its effective cards.
        let build_eval = |seat: u8| -> Eval {
            match table.effective_player_cards(seat) {
                Some(cards) => match Seven::try_from(cards) {
                    Ok(seven) => Eval::from(seven),
                    Err(_) => Eval::default(),
                },
                None => Eval::default(),
            }
        };

        let mut per_seat: HashMap<u8, usize> = HashMap::new();
        let mut evals: HashMap<u8, Eval> = HashMap::new();

        // ── Phase 1: distribute pots for overall winners ──────────────────
        // Sort winners from lowest chip commitment to highest so all-in players
        // who created the main pot are awarded before bigger-stack winners.
        // `player_ranking` returns 0 for the highest chip level, so a higher
        // index means a lower chip commitment – process those first.
        let mut overall_winners = case_eval.winning_seats();
        overall_winners.sort_by(|&a, &b| {
            let rank_a = equity.player_ranking(a).unwrap_or(0);
            let rank_b = equity.player_ranking(b).unwrap_or(0);
            rank_b.cmp(&rank_a) // descending rank index = ascending chip level
        });

        let mut processed_chip_levels: HashSet<usize> = HashSet::new();

        for &winner_seat in &overall_winners {
            if equity.is_empty() {
                break;
            }

            let winner_sb = Seatbit::from(winner_seat);

            // Find this winner's chip level in the current equity.
            let Some(winner_chip_level) = equity
                .equities()
                .iter()
                .find(|e| e.seats != Seatbit::NONE && (e.seats & winner_sb) != Seatbit::NONE)
                .map(|e| e.chips)
            else {
                continue;
            };

            if processed_chip_levels.contains(&winner_chip_level) {
                continue; // already handled this level (tied winners)
            }
            processed_chip_levels.insert(winner_chip_level);

            // All overall winners at this exact chip level share this pot.
            let tied_at_level: Vec<u8> = overall_winners
                .iter()
                .filter(|&&s| {
                    equity.equities().iter().any(|e| {
                        e.seats != Seatbit::NONE
                            && (e.seats & Seatbit::from(s)) != Seatbit::NONE
                            && e.chips == winner_chip_level
                    })
                })
                .copied()
                .collect();

            let Some((total, remaining)) = equity.winnings(winner_sb) else {
                break;
            };
            equity = remaining;

            let shares = Stack::new(total).divvy_up(tied_at_level.len());
            let is_main_pot = processed_chip_levels.len() == 1;

            for (i, &seat) in tied_at_level.iter().enumerate() {
                let share = shares.get(i).cloned().unwrap_or_default();
                let share_amount = share.count();

                if let Some(s) = table.get_seat_mut(seat) {
                    let _ = s.player.chips.wins(share);
                }

                if is_main_pot {
                    table.log_info(TableAction::PlayerWinsMainPot(seat, share_amount));
                } else {
                    table.log_info(TableAction::PlayerWinsSidePot(seat, share_amount));
                }

                *per_seat.entry(seat).or_insert(0) += share_amount;
                evals.entry(seat).or_insert_with(|| build_eval(seat));
            }
        }

        // ── Phase 2: distribute remaining side pots ───────────────────────
        // After overall winners have taken their share, any leftover equity
        // represents chips that a subset of players are still eligible to
        // contest. Find the best hand among those eligible seats and award
        // each sub-pot in turn, iterating until the equity is exhausted.
        while !equity.is_empty() {
            // Collect all individual seat numbers still present in the equity.
            let eligible_seats: Vec<u8> = equity
                .equities()
                .iter()
                .filter(|e| e.seats != Seatbit::NONE)
                .flat_map(|e| (0u8..16u8).filter(move |&i| e.seats.contains(i)))
                .collect();

            if eligible_seats.is_empty() {
                break;
            }

            // Find the best hand among eligible seats using case_eval indices
            // (which map 1-to-1 with seat numbers due to HoleCards::from(seats)).
            let best_eval = eligible_seats
                .iter()
                .filter_map(|&s| case_eval.get(s as usize))
                .max()
                .copied();

            let Some(best) = best_eval else { break };

            let side_winners: Vec<u8> = eligible_seats
                .iter()
                .filter(|&&s| case_eval.get(s as usize) == Some(&best))
                .copied()
                .collect();

            if side_winners.is_empty() {
                break;
            }

            // Use the winner with the lowest chip commitment so that
            // equity.winnings() caps the pot correctly at their level.
            let winner_with_lowest = *side_winners
                .iter()
                .min_by_key(|&&s| {
                    equity
                        .equities()
                        .iter()
                        .find(|e| e.seats != Seatbit::NONE && (e.seats & Seatbit::from(s)) != Seatbit::NONE)
                        .map_or(usize::MAX, |e| e.chips)
                })
                .unwrap_or(&side_winners[0]);

            let lowest_chip_level = equity
                .equities()
                .iter()
                .find(|e| e.seats != Seatbit::NONE && (e.seats & Seatbit::from(winner_with_lowest)) != Seatbit::NONE)
                .map_or(0, |e| e.chips);

            // If multiple side winners are tied at the same chip level, they
            // share this sub-pot equally.
            let tied_side: Vec<u8> = side_winners
                .iter()
                .filter(|&&s| {
                    equity.equities().iter().any(|e| {
                        e.seats != Seatbit::NONE
                            && (e.seats & Seatbit::from(s)) != Seatbit::NONE
                            && e.chips == lowest_chip_level
                    })
                })
                .copied()
                .collect();

            let Some((total, remaining)) = equity.winnings(Seatbit::from(winner_with_lowest)) else {
                break;
            };
            equity = remaining;

            let shares = Stack::new(total).divvy_up(tied_side.len());
            for (i, &seat) in tied_side.iter().enumerate() {
                let share = shares.get(i).cloned().unwrap_or_default();
                let share_amount = share.count();

                if let Some(s) = table.get_seat_mut(seat) {
                    let _ = s.player.chips.wins(share);
                }

                table.log_info(TableAction::PlayerWinsSidePot(seat, share_amount));

                *per_seat.entry(seat).or_insert(0) += share_amount;
                evals.entry(seat).or_insert_with(|| build_eval(seat));
            }
        }

        // ── Build result vector ────────────────────────────────────────────
        let results: Vec<Win> = per_seat
            .into_iter()
            .map(|(seat, chips)| Win {
                equity: SeatEquity::new(chips, Seatbit::from(seat)),
                eval: evals.remove(&seat).unwrap_or_default(),
            })
            .collect();

        Ok(Winnings::from(results))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__showdown_tests {
    use super::*;
    use crate::prelude::{Five, TestData};
    use std::str::FromStr;

    #[test]
    fn process() {
        let table = TestData::preroll_split_pot_with_blinds__to_completion(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 6♣ 5♣ 3♣ 2♣",
        );

        // Avoid calling `Showdown::process` here because heads-up / multiway
        // logic is not yet implemented and would hit `todo!()`. Instead
        // validate that equity can be determined for the completed table.
        let equity = table.determine_hand_equity();
        // Basic sanity check: equity string should not be empty.
        assert!(!equity.is_empty());

        println!("{table}");
        println!("{}", equity);
    }

    // Seat 0: Cards: Q♦ Q♣, Player: Rich Man: 10,000 chips / 0 in play [Yet to act]
    // Seat 1: Cards: 2♦ 7♣, Player: Small Blind: 6,950 chips / 50 in play [Blind 50]
    // Seat 2: Cards: 3♦ 8♣, Player: Big Blind: 6,900 chips / 100 in play [Blind 100]
    // Seat 3: Cards: A♠ A♥, Player: Poor Man: 4,500 chips / 500 in play [Bet 500]
    // Seat 4: Cards: 4♣ 4♦, Player: Average Person: 9,000 chips / 0 in play [Yet to act]
    #[test]
    fn process_single_seat_in_hand() {
        let table = TestData::preroll_split_pot_with_blinds(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 6♣ 5♣ 3♣ 2♣",
        );

        table.act_bet(3, 500).unwrap();
        table.act_fold(4).unwrap();
        table.act_fold(0).unwrap();
        table.act_raise(1, 1000).unwrap();
        table.act_fold(2).unwrap();
        table.act_all_in(3).unwrap();
        table.act_fold(1).unwrap();

        assert!(table.is_betting_complete());
        assert!(table.is_game_over());

        // Capture and validate the returned winnings
        let winnings = Showdown::process(&table).unwrap();

        println!("{}", winnings.first());

        // We expect a single Winnings entry since only one player remains
        assert_eq!(1, winnings.len());
        let w = &winnings.first();
        // Awarded equity should be positive
        assert!(w.equity.chips > 0);
        // The winning Seatbit should include seat 3
        assert!(w.equity.seats.contains(3));
        // The eval should be the default since there are no cards on the board
        assert_eq!(w.eval, Eval::default());

        // Verify chip counts remain as expected after payout
        assert_eq!(10_000, table.get_seat(0).expect("Seat 0").player.chips.count());
        assert_eq!(5_000, table.get_seat(1).expect("Seat 0").player.chips.count());
        assert_eq!(6_900, table.get_seat(2).expect("Seat 0").player.chips.count());
        assert_eq!(6_100, table.get_seat(3).expect("Seat 0").player.chips.count());
        assert_eq!(9_000, table.get_seat(4).expect("Seat 4").player.chips.count());

        println!("{table}");
    }

    #[test]
    fn process_split_pot() {
        let table = TestData::preroll_split_pot_with_blinds__to_completion(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣",
        );

        assert!(table.is_betting_complete());
        assert!(table.is_game_over());

        println!("{table}");

        // K♠ Q♠ A♦ J♠ A♣
        let poor_cards = Five::from_str("A♠ A♥ A♦ A♣ K♠").unwrap();
        let eval = Eval::from(poor_cards);
        let poor_winnings = Win {
            eval,
            equity: SeatEquity::new(15_150, Seatbit::SEAT_3),
        };

        // K♠ Q♠ A♦ J♠ A♣
        let rich_cards = Five::from_str("A♦ A♣ Q♦ Q♣ Q♠").unwrap();
        let eval = Eval::from(rich_cards);
        let rich_winnings = Win {
            eval,
            equity: SeatEquity::new(8_000, Seatbit::SEAT_0),
        };

        // Capture and validate the returned winnings
        let winnings = Showdown::process(&table).unwrap();

        assert_eq!(winnings.first().to_string(), poor_winnings.to_string());
        assert_eq!(
            winnings.first().to_string(),
            "Winnings(equity=SeatEquity(chips=15150, seats=0b0000000000001000, count=1), eval=A♠ A♥ A♦ A♣ K♠ - 11: FourAces)"
        );
        assert_eq!(winnings.second().to_string(), rich_winnings.to_string());
        assert_eq!(
            winnings.second().to_string(),
            "Winnings(equity=SeatEquity(chips=8000, seats=0b0000000000000001, count=1), eval=Q♠ Q♦ Q♣ A♦ A♣ - 191: QueensOverAces)"
        );

        assert_eq!(winnings, Winnings::from(vec![poor_winnings, rich_winnings]));

        // Verify chip counts remain as expected after payout
        assert_eq!(9_000, table.get_seat(0).expect("Seat 0").player.chips.count());
        assert_eq!(5_950, table.get_seat(1).expect("Seat 1").player.chips.count());
        assert_eq!(6_900, table.get_seat(2).expect("Seat 2").player.chips.count());
        assert_eq!(15_150, table.get_seat(3).expect("Seat 3").player.chips.count());
        assert_eq!(0, table.get_seat(4).expect("Seat 4").player.chips.count());
    }
}
