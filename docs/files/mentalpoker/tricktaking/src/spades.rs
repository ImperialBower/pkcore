//! Spades as a thin [`TrickTaking`](crate::TrickTaking) impl.
//!
//! Differences from the shared core: trump is always Spades; you may not *lead*
//! spades until they're "broken" (a spade has been played) unless that's all
//! you hold; scoring is per partnership on bid accuracy, with bags and nil.
//! (Bag-penalty accumulation across hands and blind-nil are omitted.)

use crate::card::{Card, Suit};
use crate::{PlayState, Seat, TrickTaking, Trump};

#[derive(Clone, Copy, Debug)]
pub struct Bid {
    pub tricks: u8, // 0 = nil
}

pub struct Spades;

impl TrickTaking for Spades {
    type Contract = [Bid; 4]; // each seat's bid
    /// Per-partnership score: index 0 = seats {0,2}, index 1 = seats {1,3}.
    type Score = [i32; 2];

    fn trump(&self, _c: &Self::Contract) -> Trump {
        Trump::Suit(Suit::Spades)
    }

    fn can_play(&self, st: &PlayState, seat: Seat, card: Card, is_lead: bool) -> bool {
        if is_lead && card.suit == Suit::Spades {
            let broken = st
                .completed
                .iter()
                .any(|t| t.plays.iter().any(|p| p.card.suit == Suit::Spades));
            let only_spades = st.hands[seat as usize]
                .iter()
                .all(|c| c.suit == Suit::Spades);
            broken || only_spades
        } else {
            true
        }
    }

    fn score(&self, bids: &Self::Contract, tricks_won: &[u8]) -> [i32; 2] {
        let mut out = [0i32; 2];
        for team in 0..2 {
            let seats = [team, team + 2];
            let mut team_bid = 0u8;
            let mut team_tricks = 0u8;
            let mut score = 0i32;
            for &s in &seats {
                team_tricks += tricks_won[s];
                if bids[s].tricks == 0 {
                    // nil: +100 if the player took no tricks, else -100
                    score += if tricks_won[s] == 0 { 100 } else { -100 };
                } else {
                    team_bid += bids[s].tricks;
                }
            }
            if team_bid > 0 {
                if team_tricks >= team_bid {
                    let bags = (team_tricks - team_bid) as i32;
                    score += 10 * team_bid as i32 + bags; // 1 point per bag
                } else {
                    score -= 10 * team_bid as i32; // set
                }
            }
            out[team] = score;
        }
        out
    }
}
