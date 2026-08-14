//! # pkcore-mp — traits + mock impls for the mental-poker layer.
//! See README: `CardCrypto` (Barnett–Smart VTMF + shuffle) and `Coordinator`
//! (transport/ordering), with `PlaintextCrypto` / `InProcCoordinator` mocks.

pub mod card;

use crate::card::{Card, DECK_ARRAY};
use std::cell::Cell;
use std::collections::BTreeSet;
use std::rc::Rc;

pub type Hash = [u8; 32];
pub type Sig = [u8; 0];
pub type Seat = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MpError {
    StillMasked,
    UnknownCard,
    BadProof,
    OutOfOrder,
}

impl std::fmt::Display for MpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for MpError {}

pub trait CardCrypto {
    type SecretKey;
    type PublicKey: Clone;
    type AggregateKey: Clone;
    type MaskedCard: Clone + Eq;
    type RevealToken: Clone;
    type KeyProof: Clone;
    type MaskProof: Clone;
    type ShuffleProof: Clone;
    type Error: std::error::Error;

    fn keygen(&self) -> (Self::SecretKey, Self::PublicKey, Self::KeyProof);
    fn verify_key(&self, pk: &Self::PublicKey, proof: &Self::KeyProof) -> Result<(), Self::Error>;
    fn aggregate(&self, pks: &[Self::PublicKey]) -> Self::AggregateKey;

    fn encode(&self, card: Card) -> Result<Self::MaskedCard, Self::Error>;
    fn decode(&self, unmasked: &Self::MaskedCard) -> Result<Card, Self::Error>;

    fn mask(&self, agg: &Self::AggregateKey, m: &Self::MaskedCard)
        -> (Self::MaskedCard, Self::MaskProof);
    fn remask(&self, agg: &Self::AggregateKey, c: &Self::MaskedCard)
        -> (Self::MaskedCard, Self::MaskProof);
    fn verify_mask(
        &self,
        agg: &Self::AggregateKey,
        input: &Self::MaskedCard,
        output: &Self::MaskedCard,
        proof: &Self::MaskProof,
    ) -> Result<(), Self::Error>;

    fn shuffle(&self, agg: &Self::AggregateKey, deck: &[Self::MaskedCard])
        -> (Vec<Self::MaskedCard>, Self::ShuffleProof);
    fn verify_shuffle(
        &self,
        agg: &Self::AggregateKey,
        input: &[Self::MaskedCard],
        output: &[Self::MaskedCard],
        proof: &Self::ShuffleProof,
    ) -> Result<(), Self::Error>;

    fn reveal_token(
        &self,
        sk: &Self::SecretKey,
        pk: &Self::PublicKey,
        c: &Self::MaskedCard,
    ) -> Self::RevealToken;
    fn verify_reveal_token(
        &self,
        pk: &Self::PublicKey,
        c: &Self::MaskedCard,
        t: &Self::RevealToken,
    ) -> Result<(), Self::Error>;
    fn unmask(
        &self,
        c: &Self::MaskedCard,
        tokens: &[Self::RevealToken],
    ) -> Result<Self::MaskedCard, Self::Error>;
}

/// Mock masked card: deck index + set of seats whose "padlock" is attached.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlainMasked {
    pub card_ix: u8,
    pub padlocks: BTreeSet<Seat>,
}

#[derive(Clone, Debug)]
pub struct PlainToken {
    pub seat: Seat,
    pub card_ix: u8,
}

#[derive(Default)]
pub struct PlaintextCrypto {
    next_seat: Cell<Seat>,
}

impl PlaintextCrypto {
    pub fn new() -> Self {
        Self::default()
    }
}

fn card_index(card: Card) -> Option<u8> {
    DECK_ARRAY.iter().position(|c| *c == card).map(|i| i as u8)
}

impl CardCrypto for PlaintextCrypto {
    type SecretKey = Seat;
    type PublicKey = Seat;
    type AggregateKey = BTreeSet<Seat>;
    type MaskedCard = PlainMasked;
    type RevealToken = PlainToken;
    type KeyProof = ();
    type MaskProof = ();
    type ShuffleProof = ();
    type Error = MpError;

    fn keygen(&self) -> (Seat, Seat, ()) {
        let s = self.next_seat.get();
        self.next_seat.set(s + 1);
        (s, s, ())
    }

    fn verify_key(&self, _pk: &Seat, _proof: &()) -> Result<(), MpError> {
        Ok(())
    }

    fn aggregate(&self, pks: &[Seat]) -> BTreeSet<Seat> {
        pks.iter().copied().collect()
    }

    fn encode(&self, card: Card) -> Result<PlainMasked, MpError> {
        let card_ix = card_index(card).ok_or(MpError::UnknownCard)?;
        Ok(PlainMasked { card_ix, padlocks: BTreeSet::new() })
    }

    fn decode(&self, unmasked: &PlainMasked) -> Result<Card, MpError> {
        if unmasked.padlocks.is_empty() {
            Ok(DECK_ARRAY[unmasked.card_ix as usize])
        } else {
            Err(MpError::StillMasked)
        }
    }

    fn mask(&self, agg: &BTreeSet<Seat>, m: &PlainMasked) -> (PlainMasked, ()) {
        (PlainMasked { card_ix: m.card_ix, padlocks: agg.clone() }, ())
    }

    fn remask(&self, _agg: &BTreeSet<Seat>, c: &PlainMasked) -> (PlainMasked, ()) {
        (c.clone(), ())
    }

    fn verify_mask(
        &self,
        _agg: &BTreeSet<Seat>,
        _input: &PlainMasked,
        _output: &PlainMasked,
        _proof: &(),
    ) -> Result<(), MpError> {
        Ok(())
    }

    fn shuffle(&self, agg: &BTreeSet<Seat>, deck: &[PlainMasked]) -> (Vec<PlainMasked>, ()) {
        let n = deck.len();
        let mut out = deck.to_vec();
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15 ^ (n as u64);
        for i in (1..n).rev() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (state >> 33) as usize % (i + 1);
            out.swap(i, j);
        }
        for c in &mut out {
            *c = self.remask(agg, c).0;
        }
        (out, ())
    }

    fn verify_shuffle(
        &self,
        _agg: &BTreeSet<Seat>,
        input: &[PlainMasked],
        output: &[PlainMasked],
        _proof: &(),
    ) -> Result<(), MpError> {
        let mut a: Vec<u8> = input.iter().map(|c| c.card_ix).collect();
        let mut b: Vec<u8> = output.iter().map(|c| c.card_ix).collect();
        a.sort_unstable();
        b.sort_unstable();
        if a == b { Ok(()) } else { Err(MpError::BadProof) }
    }

    fn reveal_token(&self, sk: &Seat, _pk: &Seat, c: &PlainMasked) -> PlainToken {
        PlainToken { seat: *sk, card_ix: c.card_ix }
    }

    fn verify_reveal_token(
        &self,
        pk: &Seat,
        c: &PlainMasked,
        t: &PlainToken,
    ) -> Result<(), MpError> {
        if t.seat == *pk && t.card_ix == c.card_ix { Ok(()) } else { Err(MpError::BadProof) }
    }

    fn unmask(&self, c: &PlainMasked, tokens: &[PlainToken]) -> Result<PlainMasked, MpError> {
        let mut padlocks = c.padlocks.clone();
        for t in tokens {
            if t.card_ix != c.card_ix {
                return Err(MpError::BadProof);
            }
            padlocks.remove(&t.seat);
        }
        Ok(PlainMasked { card_ix: c.card_ix, padlocks })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Bet(u32),
    Call,
    Check,
    Raise(u32),
    AllIn,
    Fold,
    DealHand,
    DealFlop,
    DealTurn,
    DealRiver,
    EndHand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevealTarget {
    ToAll,
    ToSeat(Seat),
}

#[derive(Clone)]
pub enum EventPayload<C: CardCrypto> {
    KeyShare { seat: Seat, pk: C::PublicKey, proof: C::KeyProof },
    DeckInit { deck: Vec<C::MaskedCard> },
    Shuffle { seat: Seat, deck: Vec<C::MaskedCard>, proof: C::ShuffleProof },
    Reveal { seat: Seat, slot: u8, target: RevealTarget, token: C::RevealToken },
    Action(Action),
}

#[derive(Clone)]
pub struct SignedEvent<C: CardCrypto> {
    pub seq: u64,
    pub prev_hash: Hash,
    pub author: Seat,
    pub payload: EventPayload<C>,
    pub sig: Sig,
}

pub trait Coordinator {
    type Event: Clone;
    type Error: std::error::Error;

    fn publish(&mut self, event: Self::Event) -> Result<(), Self::Error>;
    fn next_event(&mut self) -> Result<Option<Self::Event>, Self::Error>;
    fn head(&self) -> Hash;
}

#[derive(Clone)]
pub struct InProcCoordinator<E> {
    log: Rc<std::cell::RefCell<Vec<E>>>,
    cursor: usize,
}

impl<E: Clone> InProcCoordinator<E> {
    pub fn new() -> Self {
        Self { log: Rc::new(std::cell::RefCell::new(Vec::new())), cursor: 0 }
    }
    pub fn subscribe(&self) -> Self {
        Self { log: Rc::clone(&self.log), cursor: 0 }
    }
    pub fn len(&self) -> usize {
        self.log.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<E: Clone> Default for InProcCoordinator<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Clone> Coordinator for InProcCoordinator<E> {
    type Event = E;
    type Error = MpError;

    fn publish(&mut self, event: E) -> Result<(), MpError> {
        self.log.borrow_mut().push(event);
        Ok(())
    }

    fn next_event(&mut self) -> Result<Option<E>, MpError> {
        let log = self.log.borrow();
        if self.cursor < log.len() {
            let e = log[self.cursor].clone();
            self.cursor += 1;
            Ok(Some(e))
        } else {
            Ok(None)
        }
    }

    fn head(&self) -> Hash {
        let mut h = [0u8; 32];
        let n = self.log.borrow().len() as u64;
        h[..8].copy_from_slice(&n.to_le_bytes());
        h
    }
}
