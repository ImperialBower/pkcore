use pkcore::cards::Cards;

fn main() {
    env_logger::init();

    let deck = Cards::deck();
    for v in deck.combinations(7) {
        println!("{}", Cards::from(v));
    }
    // for v in hands.combinations_after(2, &board.cards()) {
    //     let tx = tx.clone();
    //     let my_hands = hands.clone();
    //
    //     thread::spawn(move || {
    //         let case = Two::from(v);
    //         if let Ok(ce) = CaseEval::from_holdem_at_flop(board, case, &my_hands) {
    //             tx.send(ce).unwrap();
    //         }
    //     });
    // }
}