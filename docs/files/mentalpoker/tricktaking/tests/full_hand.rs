//! A full hand of spades driven entirely through the *generic* `GameRules`
//! engine: the driver (`run`) only sequences turns; all the trick-taking logic
//! lives behind the `TrickPlay` adapter. Demonstrates the `PlayState` loop in
//! motion and the engine layering at once.

use tricktaking::card::{Card, Rank, Suit};
use tricktaking::engine::{run, GameRules, TrickPlay, TrickState};
use tricktaking::spades::{Bid, Spades as SpadesGame};
use tricktaking::{PlayState, TrickTaking};

fn standard_deck() -> Vec<Card> {
    let mut d = Vec::with_capacity(52);
    for &s in &Suit::ALL {
        for &r in &Rank::ALL {
            d.push(Card::new(r, s));
        }
    }
    d
}

fn deal(deck: &[Card], seats: usize) -> Vec<Vec<Card>> {
    let per = deck.len() / seats;
    (0..seats).map(|i| deck[i * per..(i + 1) * per].to_vec()).collect()
}

#[test]
fn full_spades_hand_through_generic_engine() {
    let game = SpadesGame;
    let bids = [Bid { tricks: 3 }, Bid { tricks: 3 }, Bid { tricks: 3 }, Bid { tricks: 4 }];

    let hands = deal(&standard_deck(), 4);
    let trump = game.trump(&bids);
    let rules = TrickPlay { game, contract: bids };
    let state = TrickState { play: PlayState::new(trump, hands, 0) };

    // Hidden-information projection: seat 0 sees its own 13 cards, and only the
    // *sizes* of everyone's hands — never an opponent's actual cards.
    let v0 = rules.view_for(&state, 0);
    assert_eq!(v0.my_hand.len(), 13);
    assert_eq!(v0.opponent_hand_sizes, vec![13, 13, 13, 13]);

    // Play the whole hand. `run` is game-agnostic; it never mentions cards.
    let final_state = run(&rules, state).expect("legal hand");

    // 13 tricks played, all accounted for, hands empty.
    assert_eq!(final_state.play.completed.len(), 13);
    assert_eq!(final_state.play.tricks_won.iter().map(|&n| n as usize).sum::<usize>(), 13);
    assert!(final_state.play.hands.iter().all(|h| h.is_empty()));

    // Outcome only exists once the hand is complete.
    let score = rules.outcome(&final_state).expect("hand complete");

    println!("tricks won by seat: {:?}", final_state.play.tricks_won);
    println!("spades score [team {{0,2}}, team {{1,3}}]: {score:?}");
}

#[test]
fn outcome_is_none_until_complete() {
    let game = SpadesGame;
    let bids = [Bid { tricks: 3 }; 4];
    let hands = deal(&standard_deck(), 4);
    let trump = game.trump(&bids);
    let rules = TrickPlay { game, contract: bids };
    let state = TrickState { play: PlayState::new(trump, hands, 0) };

    assert!(rules.outcome(&state).is_none()); // nothing played yet
    assert_eq!(rules.to_act(&state), Some(0)); // seat 0 leads
}
