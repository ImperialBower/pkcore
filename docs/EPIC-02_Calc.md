# Epic 1: Calc

We are now reaching the next level of complexity with our project. Up till
now we've been focusing on the value of a poker hand in abstract, divorced from any
relationship to actual game play. Now we will need to deal with how the player's
hands relate to each other in an actual game of Hold'em. For this we will create
a new example file called calc.

When I was initially driving through this work, long before I thought it would
be interesting as a book, I created an initial version of calc. Here's how it 
turned out:

```
❯ cargo run --example calc -- -n -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/examples/calc -n -d '6♠ 6♥ 5♦ 5♣' -b '9♣ 6♦ 5♥ 5♠ 8♠'`
Cards Dealt: 6♠ 6♥ 5♦ 5♣ 9♣ 6♦ 5♥ 5♠ 8♠

[Seat 0: 6♠ 6♥, Seat 1: 5♦ 5♣]
[FLOP:  9♣ 6♦ 5♥, TURN:  5♠, RIVER: 8♠]

The Flop: 9♣ 6♦ 5♥
Chances of winning:
Seat #0 6♠ 6♥: 95.7% - CURRENT HAND: 6♠ 6♥ 6♦ 9♣ 5♥ HandRank { value: 2185, name: ThreeOfAKind, class: ThreeSixes }
Seat #1 5♦ 5♣: 6.0% - CURRENT HAND: 5♥ 5♦ 5♣ 9♣ 6♦ HandRank { value: 2251, name: ThreeOfAKind, class: ThreeFives }

The Nuts would be: 9♣ 8♠ 7♠ 6♦ 5♥ HandRank { value: 1605, name: Straight, class: NineHighStraight }

Possible hands at the flop, sorted by strength:
CKC #1605 9♣ 8♠ 7♠ 6♦ 5♥ HandRank { value: 1605, name: Straight, class: NineHighStraight }
CKC #1996 9♠ 9♥ 9♣ 6♦ 5♥ HandRank { value: 1996, name: ThreeOfAKind, class: ThreeNines }
CKC #2185 6♠ 6♥ 6♦ 9♣ 5♥ HandRank { value: 2185, name: ThreeOfAKind, class: ThreeSixes }
CKC #2251 5♠ 5♥ 5♦ 9♣ 6♦ HandRank { value: 2251, name: ThreeOfAKind, class: ThreeFives }
CKC #3047 9♠ 9♣ 6♠ 6♦ 5♥ HandRank { value: 3047, name: TwoPair, class: NinesAndSixes }
CKC #3058 9♠ 9♣ 5♠ 5♥ 6♦ HandRank { value: 3058, name: TwoPair, class: NinesAndFives }
CKC #3221 6♠ 6♦ 5♠ 5♥ 9♣ HandRank { value: 3221, name: TwoPair, class: SixesAndFives }
CKC #3501 A♠ A♥ 9♣ 6♦ 5♥ HandRank { value: 3501, name: Pair, class: PairOfAces }
CKC #3721 K♠ K♥ 9♣ 6♦ 5♥ HandRank { value: 3721, name: Pair, class: PairOfKings }
CKC #3941 Q♠ Q♥ 9♣ 6♦ 5♥ HandRank { value: 3941, name: Pair, class: PairOfQueens }
CKC #4161 J♠ J♥ 9♣ 6♦ 5♥ HandRank { value: 4161, name: Pair, class: PairOfJacks }
CKC #4381 T♠ T♥ 9♣ 6♦ 5♥ HandRank { value: 4381, name: Pair, class: PairOfTens }
CKC #4471 9♠ 9♣ A♠ 6♦ 5♥ HandRank { value: 4471, name: Pair, class: PairOfNines }
CKC #4836 8♠ 8♥ 9♣ 6♦ 5♥ HandRank { value: 4836, name: Pair, class: PairOfEights }
CKC #5056 7♠ 7♥ 9♣ 6♦ 5♥ HandRank { value: 5056, name: Pair, class: PairOfSevens }
CKC #5122 6♠ 6♦ A♠ 9♣ 5♥ HandRank { value: 5122, name: Pair, class: PairOfSixes }
CKC #5342 5♠ 5♥ A♠ 9♣ 6♦ HandRank { value: 5342, name: Pair, class: PairOfFives }
CKC #5720 4♠ 4♥ 9♣ 6♦ 5♥ HandRank { value: 5720, name: Pair, class: PairOfFours }
CKC #5940 3♠ 3♥ 9♣ 6♦ 5♥ HandRank { value: 5940, name: Pair, class: PairOfTreys }
CKC #6160 2♠ 2♥ 9♣ 6♦ 5♥ HandRank { value: 6160, name: Pair, class: PairOfDeuces }
CKC #6305 A♠ K♠ 9♣ 6♦ 5♥ HandRank { value: 6305, name: HighCard, class: AceHigh }
CKC #6753 K♠ Q♠ 9♣ 6♦ 5♥ HandRank { value: 6753, name: HighCard, class: KingHigh }
CKC #7046 Q♠ J♠ 9♣ 6♦ 5♥ HandRank { value: 7046, name: HighCard, class: QueenHigh }
CKC #7227 J♠ T♠ 9♣ 6♦ 5♥ HandRank { value: 7227, name: HighCard, class: JackHigh }
CKC #7346 T♠ 9♣ 8♠ 6♦ 5♥ HandRank { value: 7346, name: HighCard, class: TenHigh }
CKC #7420 9♣ 8♠ 6♦ 5♥ 4♠ HandRank { value: 7420, name: HighCard, class: NineHigh }
See https://suffe.cool/poker/7462.html for a listing of all CKC numbers.

The Turn: 5♠
Chances of winning:
Seat 0: 2.3% - Outs: 6♣
Seat 1: 97.7%

The River: 8♠
Seat 0: 0.0%
Seat 1: 100.0%

Winners:
   Seat 1: 5♠ 5♥ 5♦ 5♣ 9♣ HandRank { value: 124, name: FourOfAKind, class: FourFives }
Time taken performing calc: 1.400875004s

Command:
❯ cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"
```



## Calculating Odds at Flop

## PHASE 2.1/DEFECT DETOUR

We want to display the five card hand with the winning parts first.
For that we'll need to add frequency bits to the hand and then map them.

### Cards.map_by_rank()

```
fn map_by_rank(&self) -> HashMap<Rank, Cards> {
    let mut mappie: HashMap<Rank, Cards> = HashMap::new();
    for rank in Rank::iter() {
        let pile: Vec<Card> = self.iter().map(|c| *c).filter(|card| card.get_rank() == rank).collect();
        mappie.insert(rank, Cards::from(pile));
    }
    mappie
}
```

This generates the following clippy warning:

```
warning: you are using an explicit closure for copying elements
   --> src/cards.rs:120:35
    |
120 |             let pile: Vec<Card> = self.iter().map(|c| *c).filter(|card| card.get_rank() == rank).collect();
    |                                   ^^^^^^^^^^^^^^^^^^^^^^^ help: consider calling the dedicated `copied` method: `self.iter().copied()`
    |
    = note: `#[warn(clippy::map_clone)]` on by default
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#map_clone
```

Don't you love Rust? This fixes it, just like the compiler said:

```
fn map_by_rank(&self) -> HashMap<Rank, Cards> {
    let mut mappie: HashMap<Rank, Cards> = HashMap::new();
    for rank in Rank::iter() {
        let pile: Vec<Card> = self.iter().copied().filter(|card| card.get_rank() == rank).collect();
        mappie.insert(rank, Cards::from(pile));
    }
    mappie
}
```