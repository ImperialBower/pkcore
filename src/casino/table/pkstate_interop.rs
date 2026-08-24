//! `pkstate` interop for [`Table`] — snapshot a live table as a
//! [`pkstate::PKState`] so it can be persisted or handed to another process.
//!
//! Ported from `TableCelled` by EPIC-83.

use super::Table;
use crate::bard::Bard;
use crate::casino::action::TableAction;
use pkstate::act::Action;

impl From<&Table> for pkstate::PKState {
    /// Converts a [`Table`] snapshot into a [`pkstate::PKState`].
    ///
    /// Players are taken from the seats in order. The event log is walked once and
    /// split into [`pkstate::act::Round`]s whenever a street-dealing action is seen
    /// (`DealtFlop`, `DealtTurn`, `DealtRiver`). Every `Dealt`, `Check`, `Bet`,
    /// `Call`, `Raise`, `AllIn`, `Fold`, `PlayerWins`, and `PlayerLoses` action is
    /// mapped to its corresponding [`pkstate::act::Action`] variant.
    #[allow(clippy::too_many_lines)]
    fn from(table: &Table) -> Self {
        // ── players ──────────────────────────────────────────────────────────
        let players: Vec<pkstate::seat::Seat> = table
            .seats
            .iter()
            .map(|seat| pkstate::seat::Seat {
                id: Some(seat.player.id.to_string()),
                name: seat.player.handle.clone(),
                stack: seat.player.chips,
            })
            .collect();

        // ── forced bets ───────────────────────────────────────────────────────
        let forced_bets = pkstate::game::ForcedBets::new(table.forced.small_blind, table.forced.big_blind);

        // ── board ─────────────────────────────────────────────────────────────
        let board_str = table.board.to_string();
        let board: Option<cardpack::prelude::BasicPile> = if board_str.trim().is_empty() {
            None
        } else {
            board_str
                .parse::<cardpack::prelude::Pile<cardpack::prelude::Standard52>>()
                .ok()
                .map(|p| p.into_basic_pile())
        };

        // ── rounds (walk the event log) ───────────────────────────────────────
        let mut rounds: Vec<pkstate::act::Round> = Vec::new();
        let mut current: Vec<Action> = Vec::new();

        // The celled `TableLog::entries()` handed back owned actions; copying
        // here keeps the match arms binding by value exactly as they did.
        for action in table.event_log.iter().copied() {
            match action {
                // ── street boundaries: push the current round and start a new one ──
                TableAction::DealtFlop(bard) | TableAction::DealtTurn(bard) | TableAction::DealtRiver(bard) => {
                    if !current.is_empty() {
                        rounds.push(pkstate::act::Round(std::mem::take(&mut current)));
                    }
                    if let Some(pile) = bard.to_pile() {
                        current.push(Action::DealCommon(pile));
                    }
                }

                // ── hole cards ────────────────────────────────────────────────────
                TableAction::Dealt(seat, bard) | TableAction::ForceDealt(seat, bard) => {
                    if let Some(a) = dealt_action(seat, bard) {
                        current.push(a);
                    }
                }

                // ── player actions ────────────────────────────────────────────────
                TableAction::Check(seat) => {
                    if let Some(a) = match seat {
                        0 => Some(Action::P0Check),
                        1 => Some(Action::P1Check),
                        2 => Some(Action::P2Check),
                        3 => Some(Action::P3Check),
                        4 => Some(Action::P4Check),
                        5 => Some(Action::P5Check),
                        6 => Some(Action::P6Check),
                        7 => Some(Action::P7Check),
                        8 => Some(Action::P8Check),
                        9 => Some(Action::P9Check),
                        10 => Some(Action::P10Check),
                        11 => Some(Action::P11Check),
                        _ => None,
                    } {
                        current.push(a);
                    }
                }
                TableAction::Bet(seat, amount)
                | TableAction::Call(seat, amount)
                | TableAction::Raise(seat, amount)
                | TableAction::AllIn(seat, amount)
                | TableAction::ForcedBetSmallBlind(seat, amount)
                | TableAction::ForcedBetBigBlind(seat, amount) => {
                    if let Some(a) = match seat {
                        0 => Some(Action::P0CBR(amount)),
                        1 => Some(Action::P1CBR(amount)),
                        2 => Some(Action::P2CBR(amount)),
                        3 => Some(Action::P3CBR(amount)),
                        4 => Some(Action::P4CBR(amount)),
                        5 => Some(Action::P5CBR(amount)),
                        6 => Some(Action::P6CBR(amount)),
                        7 => Some(Action::P7CBR(amount)),
                        8 => Some(Action::P8CBR(amount)),
                        9 => Some(Action::P9CBR(amount)),
                        10 => Some(Action::P10CBR(amount)),
                        11 => Some(Action::P11CBR(amount)),
                        _ => None,
                    } {
                        current.push(a);
                    }
                }
                TableAction::Fold(seat) => {
                    if let Some(a) = match seat {
                        0 => Some(Action::P0Fold),
                        1 => Some(Action::P1Fold),
                        2 => Some(Action::P2Fold),
                        3 => Some(Action::P3Fold),
                        4 => Some(Action::P4Fold),
                        5 => Some(Action::P5Fold),
                        6 => Some(Action::P6Fold),
                        7 => Some(Action::P7Fold),
                        8 => Some(Action::P8Fold),
                        9 => Some(Action::P9Fold),
                        10 => Some(Action::P10Fold),
                        11 => Some(Action::P11Fold),
                        _ => None,
                    } {
                        current.push(a);
                    }
                }

                // ── results ───────────────────────────────────────────────────────
                TableAction::PlayerWins(seat, _, _, amount, _)
                | TableAction::PlayerWinsMainPot(seat, amount)
                | TableAction::PlayerWinsSidePot(seat, amount) => {
                    if let Some(a) = match seat {
                        0 => Some(Action::P0Wins(amount)),
                        1 => Some(Action::P1Wins(amount)),
                        2 => Some(Action::P2Wins(amount)),
                        3 => Some(Action::P3Wins(amount)),
                        4 => Some(Action::P4Wins(amount)),
                        5 => Some(Action::P5Wins(amount)),
                        6 => Some(Action::P6Wins(amount)),
                        7 => Some(Action::P7Wins(amount)),
                        8 => Some(Action::P8Wins(amount)),
                        9 => Some(Action::P9Wins(amount)),
                        10 => Some(Action::P10Wins(amount)),
                        11 => Some(Action::P11Wins(amount)),
                        _ => None,
                    } {
                        current.push(a);
                    }
                }
                TableAction::PlayerLoses(seat, _, _, amount)
                | TableAction::PlayerLosesMainPot(seat, amount)
                | TableAction::PlayerLosesSidePot(seat, amount) => {
                    if let Some(a) = match seat {
                        0 => Some(Action::P0Loses(amount)),
                        1 => Some(Action::P1Loses(amount)),
                        2 => Some(Action::P2Loses(amount)),
                        3 => Some(Action::P3Loses(amount)),
                        4 => Some(Action::P4Loses(amount)),
                        5 => Some(Action::P5Loses(amount)),
                        6 => Some(Action::P6Loses(amount)),
                        7 => Some(Action::P7Loses(amount)),
                        8 => Some(Action::P8Loses(amount)),
                        9 => Some(Action::P9Loses(amount)),
                        10 => Some(Action::P10Loses(amount)),
                        11 => Some(Action::P11Loses(amount)),
                        _ => None,
                    } {
                        current.push(a);
                    }
                }

                _ => {}
            }
        }

        if !current.is_empty() {
            rounds.push(pkstate::act::Round(current));
        }

        pkstate::PKState {
            id: Some(table.id.to_string()),
            datetime: None,
            game: pkstate::game::GameType::NoLimitHoldem,
            button: table.button as usize,
            forced_bets,
            board,
            players,
            rounds,
        }
    }
}

impl From<Table> for pkstate::PKState {
    fn from(table: Table) -> Self {
        pkstate::PKState::from(&table)
    }
}

/// Maps a seat's dealt hole cards onto the matching `PN Dealt` action.
///
/// `pkstate` names each seat's actions with a distinct variant rather than
/// carrying a seat index, so this is a lookup table, not a calculation.
/// Returns `None` past seat 11, which `pkstate` does not model.
fn dealt_action(seat: u8, bard: Bard) -> Option<Action> {
    let pile = bard.to_pile()?;
    match seat {
        0 => Some(Action::P0Dealt(pile)),
        1 => Some(Action::P1Dealt(pile)),
        2 => Some(Action::P2Dealt(pile)),
        3 => Some(Action::P3Dealt(pile)),
        4 => Some(Action::P4Dealt(pile)),
        5 => Some(Action::P5Dealt(pile)),
        6 => Some(Action::P6Dealt(pile)),
        7 => Some(Action::P7Dealt(pile)),
        8 => Some(Action::P8Dealt(pile)),
        9 => Some(Action::P9Dealt(pile)),
        10 => Some(Action::P10Dealt(pile)),
        11 => Some(Action::P11Dealt(pile)),
        _ => None,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__pkstate_interop_tests {
    use super::*;
    use crate::casino::game::ForcedBets;
    use crate::casino::table::{Player, Seat, Seats};

    fn played_table() -> Table {
        let mut table = Table::nlh_from_seats(
            Seats::new(vec![
                Seat::new(Player::new_with_chips("Ann".to_string(), 1_000)),
                Seat::new(Player::new_with_chips("Bo".to_string(), 1_000)),
            ]),
            ForcedBets::new(50, 100),
        );
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        table.deal_flop().unwrap();
        table
    }

    #[test]
    fn pkstate_carries_every_seat_in_order() {
        let table = played_table();

        let state = pkstate::PKState::from(&table);

        assert_eq!(2, state.players.len());
        assert_eq!("Ann", state.players[0].name);
        assert_eq!("Bo", state.players[1].name);
    }

    #[test]
    fn pkstate_reports_each_seats_remaining_stack() {
        let table = played_table();

        let state = pkstate::PKState::from(&table);

        for (index, seat) in table.seats.iter().enumerate() {
            assert_eq!(seat.player.chips, state.players[index].stack);
        }
    }

    #[test]
    fn pkstate_carries_the_table_identity_and_button() {
        let table = played_table();

        let state = pkstate::PKState::from(&table);

        assert_eq!(Some(table.id.to_string()), state.id);
        assert_eq!(table.button as usize, state.button);
        assert_eq!(
            pkstate::game::ForcedBets::new(50, 100),
            state.forced_bets,
            "blinds carry across unchanged"
        );
    }

    #[test]
    fn pkstate_carries_the_board_once_the_flop_is_out() {
        let table = played_table();

        let state = pkstate::PKState::from(&table);

        assert!(state.board.is_some(), "the flop should have come across");
    }

    #[test]
    fn pkstate_leaves_the_board_empty_before_the_flop() {
        let table = Table::default();

        let state = pkstate::PKState::from(&table);

        assert!(state.board.is_none());
    }

    #[test]
    fn pkstate_splits_the_event_log_into_rounds() {
        let table = played_table();

        let state = pkstate::PKState::from(&table);

        assert!(
            !state.rounds.is_empty(),
            "posting blinds and dealing should produce at least one round"
        );
    }

    #[test]
    fn pkstate_from_an_owned_table_matches_the_borrowed_form() {
        let table = played_table();

        let borrowed = pkstate::PKState::from(&table);
        let owned = pkstate::PKState::from(table);

        assert_eq!(borrowed.id, owned.id);
        assert_eq!(borrowed.players.len(), owned.players.len());
    }

    #[test]
    fn dealt_action_gives_up_past_the_seats_pkstate_models() {
        assert_eq!(None, dealt_action(12, Bard::default()));
    }
}
