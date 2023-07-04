use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;
use wincounter::{Win, Wins};

lazy_static! {
    static ref BC_RANK: HashMap<BinaryCard, SimpleBinaryCardMap> = {
        let mut m = HashMap::new();
        let file_path = "logs/bcm.csv";
        let file = File::open(file_path).unwrap();
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let bcm: BinaryCardMap = result.unwrap();
            m.insert(bcm.bc, SimpleBinaryCardMap::from(bcm));
        }
        m
    };
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

    let cards = PokerCards::try_from(input_text);

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

fn work(cards: PokerCards) {
    let hands = cards.try_into_twos().unwrap();
    let hero = hands.get(0).unwrap();
    let villain = hands.get(1).unwrap();

    let wins = grind(*hero, *villain, cards.remaining());
    let results = wins.results_heads_up();
    println!("{}, {}", cards, results);
}

fn grind(hero: Two, villain: Two, remaining: PlayingCards) -> Wins {
    let now = std::time::Instant::now();

    let mut wins = Wins::default();
    let combos = remaining.combinations(5);

    for combo in combos {
        let board = FiveCard::from(PlayingCards::from(combo).to_five_array().unwrap());
        let five = Five::from(board.to_arr());

        let hero7 = BinaryCard::from_seven(Seven::new(hero, five));
        let villain7 = BinaryCard::from_seven(Seven::new(villain, five));

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
