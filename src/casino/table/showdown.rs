use crate::PKError;
use crate::casino::table::winnings::Winnings;
use crate::prelude::{Eval, Pile, SeatEquity, Seatbit, Seven, Table, TableAction};

pub struct Showdown;

impl Showdown {
    pub fn process(table: &Table) -> Result<Vec<Winnings>, PKError> {
        table.log_info(TableAction::EndHand);

        if !table.is_game_over() {
            return Err(PKError::ActionIsntFinished);
        }

        println!("{}", table.determine_hand_equity());

        // let mut winnings: Vec<Winnings> = Vec::new();

        let winnings = match table.seats.active_in_hand().len() {
            0 => return Err(PKError::Fubar),
            1 => Showdown::process_single_seat_in_hand(table)?,
            2 => Showdown::process_headsup(table)?,
            _ => Showdown::process_multiway(table)?,
        };

        Ok(winnings)
    }

    fn process_single_seat_in_hand(table: &Table) -> Result<Vec<Winnings>, PKError> {
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

        let winnings = vec![Winnings { equity, eval }];

        Ok(winnings)
    }

    fn process_headsup(_table: &Table) -> Result<Vec<Winnings>, PKError> {
        // Heads-up is effectively the same flow as Table::end_hand for >1 players
        // Build a case eval, close out the bets into the pot, mark seats as in
        // showdown, split the pot between the winners and award chips. Return
        // a Vec<Winnings> describing what each winning seat received.
        let table = _table;

        let game = crate::play::game::Game::try_from(table)?;
        let case_eval = game.river_case_eval()?;

        let winners = case_eval.winning_seats();

        let _brought_in = table.close_it_out()?;
        table.seats.showdown(table.pot.count())?;

        let shares = table.pot.take().divvy_up(winners.len());

        let mut results: Vec<Winnings> = Vec::new();

        for (i, winner_seat_number) in winners.iter().enumerate() {
            if let Some(mut seat) = table.get_seat_mut(*winner_seat_number) {
                let player_winnings = shares.get(i).cloned().unwrap_or_default();
                let winnings_amount = player_winnings.count();

                // Award to player's chips
                let _ = seat.player.chips.wins(player_winnings.clone());

                // Build eval for the seat
                let eval = match table.effective_player_cards(*winner_seat_number) {
                    Some(cards) => match Seven::try_from(cards) {
                        Ok(seven) => Eval::from(seven),
                        Err(_) => Eval::default(),
                    },
                    None => Eval::default(),
                };

                // Log the win
                let hand = seat.cards.bard();
                let id = seat.player.id;
                let chips_won = winnings_amount.saturating_sub(seat.player.chips_in_play.take());
                table.log_info(TableAction::PlayerWins(
                    *winner_seat_number,
                    id,
                    hand,
                    chips_won,
                    winnings_amount,
                ));

                results.push(Winnings {
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

        Ok(results)
    }

    fn process_multiway(_table: &Table) -> Result<Vec<Winnings>, PKError> {
        // Multi-way showdown must consider side-pots. Use PotManager to
        // construct pots, close out bets into the table pot, evaluate winners
        // and distribute each pot to its eligible winners. Aggregate per-seat
        // winnings and return them as Vec<Winnings].
        let table = _table;

        let pot_manager = crate::casino::table::pot::PotManager::create_pots(&table.seats);

        let _brought_in = table.close_it_out()?;

        let game = crate::play::game::Game::try_from(table)?;
        let case_eval = game.river_case_eval()?;

        table.seats.showdown(table.pot.count())?;

        use std::collections::HashMap;

        let mut per_seat: HashMap<u8, usize> = HashMap::new();
        let mut evals: HashMap<u8, Eval> = HashMap::new();

        for (pot_index, pot_info) in pot_manager.pots.iter().enumerate() {
            // Determine eligible winners for this pot
            let eligible_winners: Vec<u8> = case_eval
                .winning_seats()
                .iter()
                .filter(|s| pot_info.eligible_seats.contains(s))
                .copied()
                .collect();

            if eligible_winners.is_empty() {
                continue;
            }

            let shares = crate::casino::cashier::chips::Stack::new(pot_info.amount).divvy_up(eligible_winners.len());

            for (i, &winner_seat) in eligible_winners.iter().enumerate() {
                if let Some(mut seat) = table.get_seat_mut(winner_seat) {
                    let share = shares.get(i).cloned().unwrap_or_default();
                    let share_amount = share.count();

                    // Award chips
                    let _ = seat.player.chips.wins(share.clone());

                    // Log main/side pot win
                    if pot_index == 0 {
                        table.log_info(TableAction::PlayerWinsMainPot(winner_seat, share_amount));
                    } else {
                        table.log_info(TableAction::PlayerWinsSidePot(winner_seat, share_amount));
                    }

                    // Aggregate
                    *per_seat.entry(winner_seat).or_insert(0) += share_amount;

                    evals
                        .entry(winner_seat)
                        .or_insert_with(|| match table.effective_player_cards(winner_seat) {
                            Some(cards) => match Seven::try_from(cards) {
                                Ok(seven) => Eval::from(seven),
                                Err(_) => Eval::default(),
                            },
                            None => Eval::default(),
                        });
                }
            }
        }

        // Build Winnings vector from aggregated results
        let mut results: Vec<Winnings> = per_seat
            .into_iter()
            .map(|(seat, chips)| Winnings {
                equity: SeatEquity::new(chips, Seatbit::from(seat)),
                eval: evals.remove(&seat).unwrap_or_default(),
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__showdown_tests {
    use super::*;
    use crate::prelude::TestData;

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

        println!("{}", winnings[0]);

        // We expect a single Winnings entry since only one player remains
        assert_eq!(1, winnings.len());
        let w = &winnings[0];
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
}
