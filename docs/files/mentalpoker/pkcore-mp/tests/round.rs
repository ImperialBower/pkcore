//! End-to-end two-seat round using the mock backends: set up keys, mask and
//! shuffle the deck, deal hole cards via staged unmask, reveal the board, and
//! confirm the l-out-of-l threshold (one seat alone cannot read another's card).

use pkcore_mp::card::Card;
use pkcore_mp::*;
use std::rc::Rc;

const N: usize = 2; // heads-up
const SEAT0: Seat = 0;
const SEAT1: Seat = 1;

/// Build a fully masked, shuffled deck for the given aggregate key.
fn masked_shuffled_deck(
    crypto: &PlaintextCrypto,
    agg: &std::collections::BTreeSet<Seat>,
) -> Vec<PlainMasked> {
    // Step 1: encode + mask all 52 cards.
    let mut deck: Vec<PlainMasked> = Vec::with_capacity(52);
    for c in pkcore_mp::card::DECK_ARRAY {
        let plain = crypto.encode(c).expect("encode");
        deck.push(crypto.mask(agg, &plain).0);
    }
    // Step 2: each seat shuffles in turn; every peer would verify each proof.
    for _seat in 0..N {
        let (next, proof) = crypto.shuffle(agg, &deck);
        crypto.verify_shuffle(agg, &deck, &next, &proof).expect("shuffle verifies");
        deck = next;
    }
    deck
}

/// Reveal the card at `slot` to `target`, returning the recovered plaintext.
/// Every seat *except* the recipient publishes a token; the recipient finishes.
fn reveal(
    crypto: &PlaintextCrypto,
    deck: &[PlainMasked],
    sks: &[Seat],
    pks: &[Seat],
    slot: usize,
    target: RevealTarget,
) -> Card {
    let card = &deck[slot];

    let contributors: Vec<Seat> = match target {
        RevealTarget::ToAll => (0..N as Seat).collect(),
        RevealTarget::ToSeat(j) => (0..N as Seat).filter(|s| *s != j).collect(),
    };

    let mut tokens: Vec<PlainToken> = Vec::new();
    for s in &contributors {
        let t = crypto.reveal_token(&sks[*s as usize], &pks[*s as usize], card);
        crypto.verify_reveal_token(&pks[*s as usize], card, &t).expect("token verifies");
        tokens.push(t);
    }

    if let RevealTarget::ToSeat(j) = target {
        // Threshold check: with only the other seats' tokens the card is still
        // locked — nobody but the recipient can read it yet.
        let partial = crypto.unmask(card, &tokens).expect("partial unmask");
        assert_eq!(crypto.decode(&partial), Err(MpError::StillMasked));
        // Recipient adds their own token and finishes privately.
        let own = crypto.reveal_token(&sks[j as usize], &pks[j as usize], card);
        tokens.push(own);
    }

    let full = crypto.unmask(card, &tokens).expect("full unmask");
    crypto.decode(&full).expect("decode")
}

#[test]
fn two_seat_deal_and_reveal() {
    let crypto = PlaintextCrypto::new();

    // Step 0: keys + aggregate.
    let mut sks = Vec::new();
    let mut pks = Vec::new();
    for _ in 0..N {
        let (sk, pk, proof) = crypto.keygen();
        crypto.verify_key(&pk, &proof).expect("key verifies");
        sks.push(sk);
        pks.push(pk);
    }
    let agg = crypto.aggregate(&pks);
    assert_eq!(sks, vec![SEAT0, SEAT1]);

    // Steps 1–2: masked, shuffled deck.
    let deck = masked_shuffled_deck(&crypto, &agg);
    assert_eq!(deck.len(), 52);
    // Every card starts behind both seats' padlocks.
    assert!(deck.iter().all(|c| c.padlocks.len() == N));

    // Slot layout (heads-up): alternate hole cards, then burn+board.
    let seat0_hole = [0usize, 2];
    let seat1_hole = [1usize, 3];
    let board = [5usize, 6, 7, 8, 9]; // flop(5,6,7) turn(8) river(9); slot 4 burned

    // Step 4: deal hole cards (reveal-to-one).
    let h0: Vec<Card> = seat0_hole
        .iter()
        .map(|&s| reveal(&crypto, &deck, &sks, &pks, s, RevealTarget::ToSeat(SEAT0)))
        .collect();
    let h1: Vec<Card> = seat1_hole
        .iter()
        .map(|&s| reveal(&crypto, &deck, &sks, &pks, s, RevealTarget::ToSeat(SEAT1)))
        .collect();

    // Steps 5–6: reveal the board (reveal-to-all).
    let community: Vec<Card> = board
        .iter()
        .map(|&s| reveal(&crypto, &deck, &sks, &pks, s, RevealTarget::ToAll))
        .collect();

    // Each player learned exactly two distinct hole cards.
    assert_eq!(h0.len(), 2);
    assert_ne!(h0[0], h0[1]);
    assert_eq!(h1.len(), 2);
    assert_ne!(h1[0], h1[1]);
    assert_eq!(community.len(), 5);

    // Deck integrity: all dealt + board cards are distinct.
    let mut all: Vec<Card> = Vec::new();
    all.extend(&h0);
    all.extend(&h1);
    all.extend(&community);
    let mut sorted = all.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), all.len(), "no card revealed twice");

    println!("seat0 hole: {} {}", h0[0], h0[1]);
    println!("seat1 hole: {} {}", h1[0], h1[1]);
    print!("board:");
    for c in &community {
        print!(" {c}");
    }
    println!();
}

#[test]
fn coordinator_orders_and_chains_events() {
    // Two peers share one in-process log (the broadcast channel).
    let mut writer: InProcCoordinator<Rc<SignedEvent<PlaintextCrypto>>> = InProcCoordinator::new();
    let mut reader = writer.subscribe();

    let crypto = PlaintextCrypto::new();
    let agg = crypto.aggregate(&[SEAT0, SEAT1]);
    let masked =
        crypto.mask(&agg, &crypto.encode(pkcore_mp::card::DECK_ARRAY[0]).unwrap()).0;
    let token = crypto.reveal_token(&SEAT1, &SEAT1, &masked);

    let e0 = Rc::new(SignedEvent {
        seq: 0,
        prev_hash: writer.head(),
        author: SEAT0,
        payload: EventPayload::Shuffle { seat: SEAT0, deck: vec![masked.clone()], proof: () },
        sig: [],
    });
    writer.publish(e0).unwrap();

    let e1 = Rc::new(SignedEvent {
        seq: 1,
        prev_hash: writer.head(), // chains onto the now-longer log
        author: SEAT1,
        payload: EventPayload::Reveal {
            seat: SEAT1,
            slot: 0,
            target: RevealTarget::ToSeat(SEAT0),
            token,
        },
        sig: [],
    });
    writer.publish(e1).unwrap();

    // The reader sees both events in order; the second chains onto the first.
    let got0 = reader.next_event().unwrap().expect("event 0");
    let got1 = reader.next_event().unwrap().expect("event 1");
    assert_eq!(got0.seq, 0);
    assert_eq!(got1.seq, 1);
    assert_ne!(got0.prev_hash, got1.prev_hash, "head advanced between publishes");
    assert!(reader.next_event().unwrap().is_none(), "caught up");
    assert_eq!(writer.len(), 2);
}
