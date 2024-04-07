# Omaha

This library is infested with a plague, and that plague's name is No Limit Hold'Em. With this sprint, I try to 
escape from its grasp by coding in ways to evaluate hands other than No Limit. Omaha is a good candidate, because
it kinda-sorta uses the same rules, with one notable exception: you must use two of your hole cards. 

## Strangeness

Why is this code creating a floppy test?

```rust
pub struct OmahaHigh {
    pub hand: Four,
}

impl OmahaHigh {
    #[must_use]
    pub fn permutations(&self, board: &Five) -> HashSet<Eval> {
        let mut permutations = HashSet::new();
        for hand_perm in &OMAHA_HAND_PERMUTATIONS {
            for board_perm in &OMAHA_BOARD_PERMUTATIONS {
                let card1 = self.hand.0[hand_perm[0]];
                let card2 = self.hand.0[hand_perm[1]];
                let card3 = board.0[board_perm[0]];
                let card4 = board.0[board_perm[1]];
                let card5 = board.0[board_perm[2]];
                let five = Five::from([card1, card2, card3, card4, card5]).sort();
                permutations.insert(five.eval());
            }
        }
        permutations
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod games__omaha_high_tests {
    use crate::Pile;
    use super::*;

    /// The hand:
    /// Robl: AS QS QD JC
    /// Antonius: 9H 8D 6D 5D
    /// board: 4D AD 7S JD AC
    /// https://www.youtube.com/watch?v=iXmrtiqoUKM
    const ROBL_HAND: [Card; 4] = [
        Card::ACE_SPADES,
        Card::QUEEN_SPADES,
        Card::QUEEN_DIAMONDS,
        Card::JACK_CLUBS,
    ];

    const BOARD: [Card; 5] = [
        Card::FOUR_DIAMONDS,
        Card::ACE_DIAMONDS,
        Card::SEVEN_SPADES,
        Card::JACK_DIAMONDS,
        Card::ACE_CLUBS,
    ];

    #[test]
    fn permutations() {
        let hand = Four::from(ROBL_HAND);
        let board = Five::from(BOARD);

        let actual = OmahaHigh::from(hand).permutations(&board);

        // >> Line that fails   
        assert_eq!(60, actual.len());
        let mut v = actual.iter().collect::<Vec<&Eval>>();
        v.sort();
        v.reverse();

        for eval in v {
            assert_eq!(2, eval.hand.how_many(hand.cards()));
            assert_eq!(3, eval.hand.how_many(board.cards()));
            println!("{eval}");
        }
        
    }
}
```