//! The game-agnostic engine seam.
//!
//! [`GameRules`] is the trait the generic infrastructure (replay, bots,
//! networking, the mental-poker layer) targets — it knows nothing about tricks
//! or trump. [`TrickPlay`] lifts *any* [`TrickTaking`](crate::TrickTaking) game
//! into it, so bridge and spades become `GameRules` for free. `view_for` is the
//! hidden-information projection the crypto layer plugs into: a seat sees its
//! own cards and public table state, never an opponent's hand.

use crate::card::Card;
use crate::{legal_plays, resolve_trick, PlayState, PlayedCard, Seat, TrickTaking, Trick, Trump};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineError {
    NotYourTurn,
    IllegalPlay,
}

/// What the generic engine requires of any game. No card-game specifics here.
pub trait GameRules {
    type State;
    type Action: Clone;
    type View;
    type Outcome;
    type Error;

    /// Seat whose turn it is, or `None` if the game is over.
    fn to_act(&self, s: &Self::State) -> Option<Seat>;
    fn legal_actions(&self, s: &Self::State, seat: Seat) -> Vec<Self::Action>;
    fn apply(&self, s: Self::State, seat: Seat, a: Self::Action)
        -> Result<Self::State, Self::Error>;
    /// Per-seat projection — only what `seat` is entitled to see.
    fn view_for(&self, s: &Self::State, seat: Seat) -> Self::View;
    fn outcome(&self, s: &Self::State) -> Option<Self::Outcome>;
}

/// Drive any `GameRules` to completion with a trivial "first legal action"
/// strategy. The engine never mentions cards — it only sequences turns.
pub fn run<G: GameRules>(rules: &G, mut state: G::State) -> Result<G::State, G::Error> {
    while let Some(seat) = rules.to_act(&state) {
        let action = rules
            .legal_actions(&state, seat)
            .into_iter()
            .next()
            .expect("a legal action always exists while a hand is non-empty");
        state = rules.apply(state, seat, action)?;
    }
    Ok(state)
}

// ── Adapter: TrickTaking game -> GameRules ───────────────────────────────────

/// Play-phase state for a trick-taking game inside the generic engine.
pub struct TrickState {
    pub play: PlayState,
}

/// What a seat is allowed to see: its own hand + public table state. Opponents'
/// hands appear only as sizes. This is the seam the mental-poker layer realizes
/// cryptographically (a hidden hand = masked cards).
pub struct TrickView {
    pub seat: Seat,
    pub my_hand: Vec<Card>,
    pub trump: Trump,
    pub current: Trick,
    pub completed_count: usize,
    pub tricks_won: Vec<u8>,
    pub opponent_hand_sizes: Vec<usize>,
}

/// Adapter pairing a trick-taking ruleset with the contract its auction produced.
pub struct TrickPlay<G: TrickTaking> {
    pub game: G,
    pub contract: G::Contract,
}

fn actor(p: &PlayState) -> Option<Seat> {
    if p.hands.iter().all(|h| h.is_empty()) {
        return None;
    }
    Some(((p.current.leader as usize + p.current.plays.len()) % p.seats) as Seat)
}

impl<G: TrickTaking> GameRules for TrickPlay<G> {
    type State = TrickState;
    type Action = Card; // play this card to the current trick
    type View = TrickView;
    type Outcome = G::Score;
    type Error = EngineError;

    fn to_act(&self, s: &TrickState) -> Option<Seat> {
        actor(&s.play)
    }

    fn legal_actions(&self, s: &TrickState, seat: Seat) -> Vec<Card> {
        legal_plays(&self.game, &s.play, seat)
    }

    fn apply(&self, mut s: TrickState, seat: Seat, card: Card) -> Result<TrickState, EngineError> {
        if actor(&s.play) != Some(seat) {
            return Err(EngineError::NotYourTurn);
        }
        if !legal_plays(&self.game, &s.play, seat).contains(&card) {
            return Err(EngineError::IllegalPlay);
        }
        let hand = &mut s.play.hands[seat as usize];
        if let Some(i) = hand.iter().position(|c| *c == card) {
            hand.remove(i);
        }
        s.play.current.plays.push(PlayedCard { seat, card });
        if s.play.current.is_full(s.play.seats) {
            resolve_trick(&mut s.play); // credits winner, opens next trick led by them
        }
        Ok(s)
    }

    fn view_for(&self, s: &TrickState, seat: Seat) -> TrickView {
        TrickView {
            seat,
            my_hand: s.play.hands[seat as usize].clone(),
            trump: s.play.trump,
            current: s.play.current.clone(),
            completed_count: s.play.completed.len(),
            tricks_won: s.play.tricks_won.clone(),
            opponent_hand_sizes: s.play.hands.iter().map(|h| h.len()).collect(),
        }
    }

    fn outcome(&self, s: &TrickState) -> Option<G::Score> {
        if s.play.hands.iter().all(|h| h.is_empty()) {
            Some(self.game.score(&self.contract, &s.play.tricks_won))
        } else {
            None
        }
    }
}
