use csv::Reader;
use lazy_static::lazy_static;
use pkcore::analysis::hand_rank::HandRankValue;
use pkcore::arrays::five::Five;
use pkcore::arrays::seven::Seven;
use pkcore::arrays::two::Two;
use pkcore::bard::Bard;
use pkcore::cards::Cards;
use pkcore::Pile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;
use std::str::FromStr;
use wincounter::{Win, Wins};

lazy_static! {
    static ref BC_RANK: HashMap<Bard, SimpleBinaryCardMap> = {
        let mut m = HashMap::new();
        let file_path = "generated/bcm.csv";
        let file = File::open(file_path).unwrap();
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let bcm: BinaryCardMap = result.unwrap();
            // m.insert(bcm.bc, SimpleBinaryCardMap::from(bcm));
        }
        m
    };
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SimpleBinaryCardMap {
    pub bc: Bard,
    pub rank: HandRankValue,
}

impl SimpleBinaryCardMap {
    #[must_use]
    pub fn new(bc: Bard, rank: HandRankValue) -> SimpleBinaryCardMap {
        SimpleBinaryCardMap { bc, rank }
    }
}

impl From<BinaryCardMap> for SimpleBinaryCardMap {
    fn from(bcm: BinaryCardMap) -> Self {
        SimpleBinaryCardMap::new(bcm.best, bcm.rank)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct BinaryCardMap {
    pub bc: Bard,
    pub best: Bard,
    pub rank: HandRankValue,
}

/// cargo run --example bcrepl
fn main() {
    loop {
        read_input();
    }
}

fn read_input() {
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
                work(c);
            }
        }
        Err(_) => println!("Invalid Cards"),
    }
}

fn work(cards: Cards) {
    let hands = cards.as_twos().unwrap();
    let hero = hands.get(0).unwrap();
    let villain = hands.get(1).unwrap();

    let wins = grind(*hero, *villain, cards.remaining());
    let results = wins.results_heads_up();
    println!("{}, {}", cards, results);
}

fn grind(hero: Two, villain: Two, remaining: Cards) -> Wins {
    let now = std::time::Instant::now();

    let mut wins = Wins::default();
    let combos = remaining.combinations(5);

    for combo in combos {
        let five = Five::try_from(combo).unwrap();

        let hero7 = Seven::from_case_at_deal(hero, five).unwrap().to_bard();
        let villain7 = Seven::from_case_at_deal(villain, five).unwrap().to_bard();

        let hero_rank = BC_RANK.get(&hero7).unwrap();
        let villain_rank = BC_RANK.get(&villain7).unwrap();

        if hero_rank.rank < villain_rank.rank {
            wins.add_win(Win::FIRST);
        } else if villain_rank.rank < hero_rank.rank {
            wins.add_win(Win::SECOND);
        } else {
            wins.add_win(Win::FIRST | Win::SECOND);
        }
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    wins
}
