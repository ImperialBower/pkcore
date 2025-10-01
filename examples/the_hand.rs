use crate::basic::decks::standard52::Standard52;
use bint::BintCell;
use cardpack::prelude::*;
use pkcore::casino::table::Table;
use pkcore::casino::table::seat::Seat;
use pkcore::games::GamePhase;

/// cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠" HSP THE HAND Negreanu/Hansen
///     https://www.youtube.com/watch?v=vjM60lqRhPg
///     https://www.youtube.com/watch?v=fEEW06iX4n8
///
/// `cargo run --example the_hand`
fn main() {
    env_logger::init();

    let mut table = Table::default();
    table.phase = GamePhase::ForcedBets.into();
    table.seats = seats();
    table.dealer = BintCell::new(seats().len() as u8);

    // table.deal_hole_cards().unwrap();

    println!("{table}");
}

fn seats() -> Vec<Seat> {
    let doyle_brunson = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Doyle Brunson".to_string(), 1_000_000),
        cards: basic_cell!("T♠ 2♥"),
    };
    let eli_elezra = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Eli Elezra".to_string(), 1_000_000),
        cards: basic_cell!("8♠ 3♥"),
    };
    let antonio_esfandiari = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Antonio Esfandari".to_string(), 1_000_000),
        cards: basic_cell!("A♦ Q♣"),
    };
    let gus_hansen = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Gus Hansen".to_string(), 1_000_000),
        cards: basic_cell!("5♦ 5♣"),
    };
    let daniel_negreanu = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Daniel Negreanu".to_string(), 1_000_000),
        cards: basic_cell!("6♠ 6♥"),
    };
    let cory_zeidman = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Cory Zeidman".to_string(), 1_000_000),
        cards: basic_cell!("K♠ J♦"),
    };
    let barry_greenstein = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Barry Greenstein".to_string(), 1_000_000),
        cards: basic_cell!("4♣ 4♦"),
    };
    let amnon_filippi = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Amnon Filippi".to_string(), 1_000_000),
        cards: basic_cell!("7♣ 2♣"),
    };
    vec![
        doyle_brunson,
        eli_elezra,
        antonio_esfandiari,
        gus_hansen,
        daniel_negreanu,
        cory_zeidman,
        barry_greenstein,
        amnon_filippi,
    ]
}
