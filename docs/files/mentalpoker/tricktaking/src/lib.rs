//! # tricktaking
//!
//! The shared core for trick-taking card games (bridge, spades, hearts, euchre).
//! It owns the logic that is identical across all of them — following suit,
//! trump-aware trick resolution, and lead rotation — and exposes two hooks for
//! the parts that differ per game: **bidding** (via the `Contract` associated
//! type a game produces) and **scoring**.
//!
//! A game becomes a thin [`TrickTaking`] impl: say what `trump` a contract
//! implies, add any `can_play` constraints beyond follow-suit, and `score` a
//! finished hand. See `bridge` and `spades`.

pub mod bridge;
pub mod card;
pub mod engine;
pub mod spades;

use card::{Card, Suit};

pub type Seat = u8;

/// The trump for the play phase. `NoTrump` = highest card of the led suit wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trump {
    Suit(Suit),
    NoTrump,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayedCard {
    pub seat: Seat,
    pub card: Card,
}

/// Cards played to the current trick, in play order starting from `leader`.
#[derive(Clone, Debug, Default)]
pub struct Trick {
    pub leader: Seat,
    pub plays: Vec<PlayedCard>,
}

impl Trick {
    pub fn new(leader: Seat) -> Self {
        Self { leader, plays: Vec::new() }
    }
    /// The suit that was led, if any card has been played.
    pub fn led_suit(&self) -> Option<Suit> {
        self.plays.first().map(|p| p.card.suit)
    }
    pub fn is_full(&self, seats: usize) -> bool {
        self.plays.len() == seats
    }
}

/// State the engine maintains through the play phase. Game-agnostic: games
/// derive their own flags (e.g. "spades broken") from `completed`.
#[derive(Clone, Debug)]
pub struct PlayState {
    pub seats: usize,
    pub trump: Trump,
    pub hands: Vec<Vec<Card>>, // per seat
    pub current: Trick,
    pub completed: Vec<Trick>,
    pub tricks_won: Vec<u8>, // per seat
}

impl PlayState {
    pub fn new(trump: Trump, hands: Vec<Vec<Card>>, leader: Seat) -> Self {
        let seats = hands.len();
        Self {
            seats,
            trump,
            hands,
            current: Trick::new(leader),
            completed: Vec::new(),
            tricks_won: vec![0; seats],
        }
    }
}

/// Comparable strength of a card *in the context of a trick*. Ordered tuple:
/// (category, rank) where category is 2 = trump, 1 = led suit, 0 = off-suit.
/// An off-suit, non-trump card can never win.
fn trick_value(card: Card, led: Suit, trump: Trump) -> (u8, u8) {
    let category = match trump {
        Trump::Suit(t) if card.suit == t => 2,
        _ if card.suit == led => 1,
        _ => 0,
    };
    (category, card.rank as u8)
}

/// The seat that wins a completed trick. The led suit is the first card's suit;
/// the highest trump wins, else the highest card of the led suit.
pub fn trick_winner(trick: &Trick, trump: Trump) -> Seat {
    let led = trick.plays[0].card.suit;
    trick
        .plays
        .iter()
        .max_by(|a, b| trick_value(a.card, led, trump).cmp(&trick_value(b.card, led, trump)))
        .expect("trick has at least one play")
        .seat
}

/// The base follow-suit rule: if the player holds the led suit they must play
/// it; otherwise any card is allowed. (When leading, `led` is `None`.)
pub fn must_follow(hand: &[Card], led: Option<Suit>) -> Vec<Card> {
    match led {
        None => hand.to_vec(),
        Some(s) => {
            let of_suit: Vec<Card> = hand.iter().copied().filter(|c| c.suit == s).collect();
            if of_suit.is_empty() {
                hand.to_vec()
            } else {
                of_suit
            }
        }
    }
}

/// The per-game ruleset. Bidding/auction is represented by whatever `Contract`
/// the game's auction produces; this crate focuses on play + scoring, which is
/// where trick-taking games overlap most.
pub trait TrickTaking {
    /// Output of the auction: enough to determine trump (and, for bridge,
    /// declarer/level/vulnerability).
    type Contract;
    /// Per-hand score. Deliberately game-specific — do not try to unify it.
    type Score;

    fn seats(&self) -> usize {
        4
    }
    fn hand_size(&self) -> usize {
        13
    }

    /// Trump for the play phase, derived from the contract.
    fn trump(&self, contract: &Self::Contract) -> Trump;

    /// Extra legality beyond follow-suit, e.g. "can't lead spades until broken"
    /// or hearts' first-trick restrictions. Default: no extra constraint.
    fn can_play(&self, _st: &PlayState, _seat: Seat, _card: Card, _is_lead: bool) -> bool {
        true
    }

    /// Score a completed hand. `tricks_won` is indexed by seat.
    fn score(&self, contract: &Self::Contract, tricks_won: &[u8]) -> Self::Score;
}

/// Generic helper every game shares: legal plays = follow-suit ∩ game constraints.
pub fn legal_plays<G: TrickTaking>(game: &G, st: &PlayState, seat: Seat) -> Vec<Card> {
    let is_lead = st.current.plays.is_empty();
    let led = st.current.led_suit();
    must_follow(&st.hands[seat as usize], led)
        .into_iter()
        .filter(|c| game.can_play(st, seat, *c, is_lead))
        .collect()
}

/// Resolve the current (full) trick: record the winner, move it to `completed`,
/// credit the winner, and open the next trick led by them. Shared by all games.
pub fn resolve_trick(st: &mut PlayState) -> Seat {
    let winner = trick_winner(&st.current, st.trump);
    st.tricks_won[winner as usize] += 1;
    let finished = std::mem::replace(&mut st.current, Trick::new(winner));
    st.completed.push(finished);
    winner
}
