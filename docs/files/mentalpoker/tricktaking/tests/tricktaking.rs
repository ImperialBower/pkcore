use tricktaking::bridge::{Bridge, Contract, Strain};
use tricktaking::card::{Card, Rank::*, Suit::*};
use tricktaking::spades::{Bid, Spades as SpadesGame};
use tricktaking::*;

fn c(r: tricktaking::card::Rank, s: tricktaking::card::Suit) -> Card {
    Card::new(r, s)
}

fn trick(plays: &[(Seat, Card)]) -> Trick {
    let mut t = Trick::new(plays[0].0);
    for &(seat, card) in plays {
        t.plays.push(PlayedCard { seat, card });
    }
    t
}

#[test]
fn highest_of_led_suit_wins_no_trump() {
    // Led hearts; off-suit spade can't win under no-trump.
    let t = trick(&[(0, c(King, Hearts)), (1, c(Ace, Hearts)), (2, c(Two, Spades)), (3, c(Five, Hearts))]);
    assert_eq!(trick_winner(&t, Trump::NoTrump), 1); // A♥
}

#[test]
fn trump_beats_higher_led_card() {
    // Led hearts with the Ace, but seat 2 ruffs with the lowly 2♠ (trump).
    let t = trick(&[(0, c(King, Hearts)), (1, c(Ace, Hearts)), (2, c(Two, Spades)), (3, c(Queen, Hearts))]);
    assert_eq!(trick_winner(&t, Trump::Suit(Spades)), 2); // 2♠ ruffs
}

#[test]
fn highest_trump_wins_when_several_ruff() {
    let t = trick(&[(0, c(Ace, Hearts)), (1, c(Three, Spades)), (2, c(Ten, Spades)), (3, c(Four, Spades))]);
    assert_eq!(trick_winner(&t, Trump::Suit(Spades)), 2); // T♠ is the top trump
}

#[test]
fn must_follow_the_led_suit() {
    let hand = vec![c(Two, Hearts), c(King, Hearts), c(Ace, Spades)];
    let legal = must_follow(&hand, Some(Hearts));
    assert_eq!(legal, vec![c(Two, Hearts), c(King, Hearts)]); // the spade is illegal
}

#[test]
fn cannot_lead_spades_until_broken() {
    let game = SpadesGame;
    let hands = vec![
        vec![c(Two, Spades), c(King, Hearts)], // seat 0 has a non-spade
        vec![], vec![], vec![],
    ];
    let st = PlayState::new(Trump::Suit(Spades), hands, 0); // no completed tricks => unbroken
    assert!(!game.can_play(&st, 0, c(Two, Spades), true));  // leading a spade: illegal
    assert!(game.can_play(&st, 0, c(King, Hearts), true));  // leading the heart: fine
}

#[test]
fn bridge_scores_match_known_contracts() {
    let game = Bridge;
    // 4♠ by seat 0 (partner = seat 2), non-vul, making exactly 10 tricks => 420.
    let four_spades = Contract { level: 4, strain: Strain::Spades, declarer: 0, vulnerable: false };
    assert_eq!(game.score(&four_spades, &[6, 2, 4, 1]), 420); // 6+4 = 10 = needed

    // 2♣ part-score making exactly 8 => 40 + 50 = 90.
    let two_clubs = Contract { level: 2, strain: Strain::Clubs, declarer: 0, vulnerable: false };
    assert_eq!(game.score(&two_clubs, &[5, 0, 3, 5]), 90); // 5+3 = 8 = needed

    // 3NT non-vul making exactly 9 => 100 + 300 = 400.
    let three_nt = Contract { level: 3, strain: Strain::NoTrump, declarer: 0, vulnerable: false };
    assert_eq!(game.score(&three_nt, &[5, 2, 4, 2]), 400); // 5+4 = 9 = needed

    // 4♥ non-vul down one (9 tricks) => -50.
    let four_hearts = Contract { level: 4, strain: Strain::Hearts, declarer: 0, vulnerable: false };
    assert_eq!(game.score(&four_hearts, &[5, 2, 4, 2]), -50); // 5+4 = 9, needed 10
}

#[test]
fn spades_scores_bids_bags_and_nil() {
    let game = SpadesGame;

    // Team {0,2} bid 2+2=4, take 3+2=5 => 40 + 1 bag = 41.
    let bids = [Bid { tricks: 2 }, Bid { tricks: 2 }, Bid { tricks: 2 }, Bid { tricks: 2 }];
    let score = game.score(&bids, &[3, 2, 2, 6]); // team0 = 5 tricks, team1 = 8
    assert_eq!(score[0], 41);

    // Team {0,2} bid 4, take only 3 => set, -40.
    let score = game.score(&bids, &[1, 5, 2, 5]); // team0 = 3, team1 = 10
    assert_eq!(score[0], -40);

    // Seat 0 bids nil and makes it (0 tricks); partner bids 3 and the pair takes 3.
    let nil = [Bid { tricks: 0 }, Bid { tricks: 4 }, Bid { tricks: 3 }, Bid { tricks: 4 }];
    let score = game.score(&nil, &[0, 4, 3, 6]); // team0: seat0=0 (nil ok +100), seat2=3 = bid 3 met
    assert_eq!(score[0], 100 + 30); // nil bonus + made 3-bid

    // Nil broken: seat 0 bid nil but took a trick => -100 (plus partner's result).
    let score = game.score(&nil, &[1, 4, 2, 6]); // seat0 took 1 => nil fails; team tricks 3 vs bid 3
    assert_eq!(score[0], -100 + 30);
}
