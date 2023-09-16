use fudd::analysis::chances::Chances;
use fudd::games::holdem::seat::Seat;
use fudd::games::holdem::seats::Seats;
use fudd::types::playing_card::PlayingCard;

fn main() {
    rayon();
}

fn rayon() {
    let now = std::time::Instant::now();
    // let table = get_table();
    // let mut evals = fudd::games::holdem::case_evals::CaseEvals::default();
    //
    // table.remaining_at_deal().combinations(5).into_par_iter().for_each(|v| {
    //     let cycle = fudd::types::playing_cards::PlayingCards::from(v);
    //     evals.push(table.players.case_eval(&cycle));
    // });
    //
    // let chances = evals.chances();
    // display(&table, chances);

    println!("Elapsed: {:.2?}", now.elapsed());
}

fn _fudd_bruteforce_no_bcm() {
    let now = std::time::Instant::now();

    let table = _get_table();

    let mut evals = fudd::games::holdem::case_evals::CaseEvals::default();
    for v in table.remaining_at_deal().combinations(5) {
        let cycle = fudd::types::playing_cards::PlayingCards::from(v);
        evals.push(table.players.case_eval(&cycle));
    }
    // Elapsed: 1076.15s
    // 18 minutes

    let chances = evals.chances();
    _display(&table, chances);

    println!("Elapsed: {:.2?}", now.elapsed());
}

fn _display(table: &fudd::games::holdem::table::Table, chances: Chances) {
    for k in chances.keys() {
        println!(
            "Seat #{k} {}: {}",
            table.players.get(*k).unwrap(),
            chances.get(*k)
        );
    }
}

fn _get_table() -> fudd::games::holdem::table::Table {
    let hero = Seat::new_with_hole_cards(
        0,
        fudd::types::slots::hole_cards::HoleCards::new(
            PlayingCard::ACE_SPADES,
            PlayingCard::FIVE_SPADES,
        ),
    );
    let villain = Seat::new_with_hole_cards(
        1,
        fudd::types::slots::hole_cards::HoleCards::new(
            PlayingCard::KING_SPADES,
            PlayingCard::KING_HEARTS,
        ),
    );
    let minion = Seat::new_with_hole_cards(
        2,
        fudd::types::slots::hole_cards::HoleCards::new(
            PlayingCard::EIGHT_CLUBS,
            PlayingCard::SEVEN_CLUBS,
        ),
    );

    fudd::games::holdem::table::Table {
        players: Seats::from(vec![hero, villain, minion]),
        board: Default::default(),
    }
}
