use pkcore::arrays::five::Five;
use pkcore::arrays::seven::Seven;
use pkcore::arrays::two::Two;
use pkcore::cards::Cards;
use pkcore::util::terminal::Terminal;
use pkcore::util::wincounter::heads_up::HeadsUp;
use pkcore::util::wincounter::win::Win;
use pkcore::util::wincounter::wins::Wins;
use pkcore::{PKError, Pile};

/// OK, this makes me sad. My new shiny pkcore library takes over twice as long to run a single calc
///
/// ```txt
/// ❯ cargo run --example bcrepl
/// ...
/// hole cards> A♠ A♥ 6♦ 6♣
/// Elapsed: 8.27s
// A♠ A♥ 6♦ 6♣, 79.66% (1363968), 20.05% (343394), 0.29% (4942)
/// ```
///
/// ```
/// pkcore❯ cargo run --example bcrepl
/// ...
/// hole cards> A♠ A♥ 6♦ 6♣
/// Elapsed: 22.00s
/// A♠ A♥ 6♦ 6♣, 79.66% (1363968), 20.05% (343394), 0.29% (4942)
/// ```
///
/// This is going to need some investigation.
///
/// `cargo run --example bcrepl_old`
/// `A♠ A♥ A♦ A♣`
fn main() {
    env_logger::init();
    loop {
        read_input();
    }
}

fn read_input() {
    let now = std::time::Instant::now();

    match Terminal::receive_x_cards("hole cards> ", 4) {
        Ok(cards) => match work(cards.clone()) {
            Ok(hup) => println!("{}, {}", cards, hup),
            Err(e) => println!("{:?}", e),
        },
        Err(e) => {
            println!("{:?}", e);
        }
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
}

fn grind(hero: Two, villain: Two, remaining: Cards) -> Wins {
    let now = std::time::Instant::now();

    let mut wins = Wins::default();
    let combos = remaining.combinations(5);

    for combo in combos {
        let five = Five::try_from(combo).unwrap();

        let hero7 = Seven::from_case_at_deal(hero, five).unwrap().bard();
        let villain7 = Seven::from_case_at_deal(villain, five).unwrap().bard();

        let hero_rank = pkcore::analysis::store::bcm::binary_card_map::BC_RANK_HASHMAP
            .get(&hero7)
            .unwrap();
        let villain_rank = pkcore::analysis::store::bcm::binary_card_map::BC_RANK_HASHMAP
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
