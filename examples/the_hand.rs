use bint::BintCell;
use pkcore::cards_cell::CardsCell;
use pkcore::casino::table::Table;
use pkcore::casino::table::seat::Seat;
use pkcore::games::GamePhase;
use std::str::FromStr;

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
        cards: CardsCell::from_str("T♠ 2♥").unwrap(),
    };
    let eli_elezra = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Eli Elezra".to_string(), 1_000_000),
        cards: CardsCell::from_str("8♠ 3♥").unwrap(),
    };
    let antonio_esfandiari = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Antonio Esfandari".to_string(), 1_000_000),
        cards: CardsCell::from_str("A♦ Q♣").unwrap(),
    };
    let gus_hansen = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Gus Hansen".to_string(), 1_000_000),
        cards: CardsCell::from_str("5♦ 5♣").unwrap(),
    };
    let daniel_negreanu = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Daniel Negreanu".to_string(), 1_000_000),
        cards: CardsCell::from_str("6♠ 6♥").unwrap(),
    };
    let cory_zeidman = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Cory Zeidman".to_string(), 1_000_000),
        cards: CardsCell::from_str("K♠ J♦").unwrap(),
    };
    let barry_greenstein = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Barry Greenstein".to_string(), 1_000_000),
        cards: CardsCell::from_str("4♣ 4♦").unwrap(),
    };
    let amnon_filippi = Seat {
        player: pkcore::casino::player::Player::new_with_chips("Amnon Filippi".to_string(), 1_000_000),
        cards: CardsCell::from_str("7♣ 2♣").unwrap(),
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
