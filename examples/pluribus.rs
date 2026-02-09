use pkcore::analysis::store::nubibus::pluribus::{Pluribus, PluribusEvent};
use std::io;
use pkcore::games::GamePhase;
use pkcore::prelude::Table;

/// `cargo run --example pluribus`
fn main() {
    for game in Pluribus::read_in_log("data/pluribus/raw/sample_game_30.log").unwrap() {
        println!("{}", game);

        let table = Table::try_from(&game).unwrap();
        println!("{table}");

        let actions = game.parse_all_rounds();

        for action in actions {
            println!("{action}");
            let seat_to_act = table.next_to_act();

            match action {
                PluribusEvent::Fold => {
                    table.act_fold(seat_to_act);
                }
                PluribusEvent::Call => {
                    table.act_call(seat_to_act);
                }
                PluribusEvent::Raise(amount) => {
                    table.act_bet(seat_to_act, amount);
                }
            }

            table.commentary_action_to();

            if table.is_game_over() {
                let hand_result = table.end_hand().unwrap();
                println!("{hand_result}");
            } else if table.is_betting_complete() {
                match table.determine_betting_phase() {
                    GamePhase::BettingPreFlop => {
                        table.deal_flop();
                        println!("Board: {}", table.board)
                    }
                    GamePhase::BettingFlop => {
                        table.deal_turn();
                        println!("Board: {}", table.board)
                    }
                    GamePhase::BettingTurn => {
                        table.deal_river();
                        println!("Board: {}", table.board)
                    }
                    GamePhase::BettingRiver => {
                        table.eval_river_display()
                    }
                    _ => {}
                }
            }

            // println!("\nPress Enter to continue..");
            // let _ = io::stdin().read_line(&mut String::new());
        }

        // println!("\nPress Enter to continue to the next game...");
        // let _ = io::stdin().read_line(&mut String::new());
    }
}
