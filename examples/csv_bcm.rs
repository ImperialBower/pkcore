use pkcore::arrays::five::Five;
use pkcore::arrays::seven::Seven;
use pkcore::arrays::two::Two;
use pkcore::cards::Cards;
use pkcore::util::wincounter::heads_up::HeadsUp;
use pkcore::util::wincounter::win::Win;
use pkcore::util::wincounter::wins::Wins;
use pkcore::{PKError, Pile};
use std::io;
use std::io::Write;
use std::str::FromStr;

/// cargo run --example csv_bcm
/// A♠ A♥ A♦ A♣
fn main() {
    env_logger::init();
    loop {
        read_input();
    }
}

fn read_input() {
    let now = std::time::Instant::now();

    print!("hole cards> ");
    let _ = io::stdout().flush();
    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("Failed to receive value");

    let cards = Cards::from_str(input_text.as_str());

    match cards {
        Ok(c) => {
            if c.len() != 4 {
                println!("Enter 4 cards");
            } else {
                println!("{}", c);
                match work(c.clone()) {
                    Ok(hup) => {
                        println!("{}, {}", c, hup);
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
            }
        }
        Err(_) => println!("Invalid Cards"),
    }
    println!("Elapsed: {:.2?}", now.elapsed());
}

fn work(cards: Cards) -> Result<HeadsUp, PKError> {
    println!("{cards}");
    let hands = cards.as_twos()?;
    let hero = match hands.get(0) {
        None => return Err(PKError::Fubar),
        Some(t) => t,
    };
    let villain = match hands.get(1) {
        None => return Err(PKError::Fubar),
        Some(t) => t,
    };

    let wins = grind(*hero, *villain, cards.remaining());
    Ok(wins.results_heads_up())
    // println!("{}, {}", cards, results);
}

fn grind(hero: Two, villain: Two, remaining: Cards) -> Wins {
    let now = std::time::Instant::now();

    let mut wins = Wins::default();
    let combos = remaining.combinations(5);

    for combo in combos {
        let five = Five::try_from(combo).unwrap();

        let hero7 = Seven::from_case_at_deal(hero, five).unwrap().to_bard();
        let villain7 = Seven::from_case_at_deal(villain, five).unwrap().to_bard();

        let hero_rank = pkcore::analysis::store::bcm::binary_card_map::BC_RANK
            .get(&hero7)
            .unwrap();
        let villain_rank = pkcore::analysis::store::bcm::binary_card_map::BC_RANK
            .get(&villain7)
            .unwrap();

        if hero_rank.rank < villain_rank.rank {
            wins.add(Win::FIRST);
        } else if villain_rank.rank < hero_rank.rank {
            wins.add(Win::SECOND);
        } else {
            wins.add(Win::FIRST | Win::SECOND);
        }
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    wins
}
