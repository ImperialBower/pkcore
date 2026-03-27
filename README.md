# pkcore AKA Spawn of [Fudd](https://github.com/ImperialBower/fudd)

[![Crates.io](https://img.shields.io/crates/v/pkcore.svg)](https://crates.io/crates/pkcore)

🚧 **Work In Progress** 🚧

[Rust](https://www.rust-lang.org/) poker library. Code inspired by [Cactus Kev's](https://suffe.cool)
[work in C](https://suffe.cool/poker/code/). An isolated version of the core hand evaluation library is available at [ckc-rs](https://github.com/ContractBridge/ckc-rs).

Currently only supports [hold'em](https://en.wikipedia.org/wiki/Texas_hold_%27em), but working on [Omaha](https://en.wikipedia.org/wiki/Omaha_hold_%27em) and want to add more types of games. Supporting
things like [Razz](https://en.wikipedia.org/wiki/Razz_(poker)) would be a total kick.

This code is a complete rewrite from scratch of my [Fudd](https://github.com/ImperialBower/fudd) crate. Changes:

* Folded [ckc-rs](https://github.com/ContractBridge/ckc-rs) crate into the repo.
* ~~Folded [wincounter](https://github.com/ImperialBower/wincounter) crate into the repo.~~ 
* Removed [cardpack.rs](https://github.com/ImperialBower/cardpack.rs) dependency

## Setup

This program uses [cargo make](https://github.com/sagiegurari/cargo-make) to manage tasks. Install it with:

```shell
cargo install cargo-make
```

The default `cargo make` runs the following tasks:

* `cargo fmt`
* `cargo clean`
* `cargo build`
* `carg test`
* `cargo clippy` with `clippy::pedantic` lint settings
* `cargo doc --no-deps`

```shell
❯ cargo make
````

To open the generated docs in your browser:

```shell
❯ cargo make docs
```

### .env

Some of the library and examples will be looking for a `.env` file in the root of the project. Simply copy
`.env.example` to `.env` and modify as needed.

## Documentation

For comprehensive API documentation, run:

```shell
cargo doc --open
````

## Examples

### The Hand

The best way to see the library in action is to run the example file that does a step by step of my favorite hand
of broadcast television poker: "The Hand" between Daniel Negreanu and Gus Hansen on High Stakes Poker. You can
watch it here on [YouTube](https://www.youtube.com/watch?v=vjM60lqRhPg).

`cargo run --example the_hand`

### [cck](examples/cck.rs)

[cck](examples/cck.rs) is a simple command line program that prints out a poker hand's
[Cactus Kev](https://suffe.cool/poker/evaluator.html) value.

```shell
❯ cargo run --example cck -- -c "AS 9C KS QS 2D JS TS"                                               ✔ ▓▒░
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running `target/debug/examples/cck -c 'AS 9C KS QS 2D JS TS'`
CARDS: A♠ 9♣ K♠ Q♠ 2♦ J♠ T♠ - BEST HAND: A♠ K♠ Q♠ J♠ T♠ - 1: RoyalFlush
{TEN: 1, KING: 1, JACK: 1, QUEEN: 1, ACE: 1}
Elapsed: 2.45ms

❯ cargo run --example cck -- -c "7S 9C JS 3S 2D 7S 4C"                                               ✔ ▓▒░
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.44s
     Running `target/debug/examples/cck -c '7S 9C JS 3S 2D 7S 4C'`
CARDS: 7♠ 9♣ J♠ 3♠ 2♦ 4♣ - BEST HAND: J♠ 9♣ 7♠ 4♣ 3♠ - 7294: JackHigh
{NINE: 1, SEVEN: 1, JACK: 1, TREY: 1, FOUR: 1}
Elapsed: 2.16ms

```


### [calc](examples/calc.rs)

[calc](examples/calc.rs) allows you to do a breakdown of the odds of a specific hand
of poker. Here it is running [the famous hand](https://www.youtube.com/watch?v=vjM60lqRhPg) quads vs full
house between Gus Hansen and Daniel Negreanu on High Stakes Poker:

```shell
❯ cargo run --example calc -- -d "6s 6h 5d 5c" -b "9c 6d 5h 5d 8d"
    Finished dev [unoptimized + debuginfo] target(s) in 0.10s
     Running `target/debug/examples/calc -d '6s 6h 5d 5c' -b '9c 6d 5h 5d 8d'`
DEALT: [6♠ 6♥, 5♦ 5♣] FLOP: 9♣ 6♦ 5♥, TURN: 5♦, RIVER: 8♦

The Flop: 9♣ 6♦ 5♥
  Player #1 [6♠ 6♥] 95.7% (94.04%/1.62%) [931/16]
     6♠ 6♥ 6♦ 9♣ 5♥ (2185-ThreeSixes)
  Player #2 [5♦ 5♣] 6.0% (4.34%/1.62%) [43/16]
     5♥ 5♦ 5♣ 9♣ 6♦ (2251-ThreeFives)

The Turn: 5♦
  Player #1 [6♠ 6♥] 97.8% (97.78%/0.00%) [44/0]
    HAND: 6♠ 6♥ 6♦ 5♥ 5♦ (271-SixesOverFives)
  Player #2 [5♦ 5♣] 2.2% (2.22%/0.00%) [1/0]
    HAND: 5♥ 5♦ 5♣ 9♣ 6♦ (2251-ThreeFives)
    OUTS: 5♠

The River: 8♦
 Winning Hand: 271-SixesOverFives
   Player #1: 6♠ 6♥ 6♦ 5♥ 5♦ - 271-SixesOverFives WINS!
   Player #2: 5♥ 5♦ 5♣ 9♣ 8♦ - 2249-ThreeFives

cargo run --example calc -- -d  "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♦ 8♦"
Elapsed: 467.50ms
```

Add the -n flag and it will add all possible hands at the flop, sorted by strength:

```shell
❯ cargo run --example calc -- -d "6s 6h 5d 5c" -b "9c 6d 5h 5d 8d" -n
    Finished dev [unoptimized + debuginfo] target(s) in 0.09s
     Running `target/debug/examples/calc -d '6s 6h 5d 5c' -b '9c 6d 5h 5d 8d' -n`
DEALT: [6♠ 6♥, 5♦ 5♣] FLOP: 9♣ 6♦ 5♥, TURN: 5♦, RIVER: 8♦

The Flop: 9♣ 6♦ 5♥
  Player #1 [6♠ 6♥] 95.7% (94.04%/1.62%) [931/16]
     6♠ 6♥ 6♦ 9♣ 5♥ (2185-ThreeSixes)
  Player #2 [5♦ 5♣] 6.0% (4.34%/1.62%) [43/16]
     5♥ 5♦ 5♣ 9♣ 6♦ (2251-ThreeFives)

The Nuts @ Flop:
  #1: 9♣ 8♠ 7♠ 6♦ 5♥ - 1605-NineHighStraight
  #2: 9♠ 9♥ 9♣ 6♦ 5♥ - 1996-ThreeNines
  #3: 6♠ 6♥ 6♦ 9♣ 5♥ - 2185-ThreeSixes
  #4: 5♠ 5♥ 5♦ 9♣ 6♦ - 2251-ThreeFives
  #5: 9♠ 9♣ 6♠ 6♦ 5♥ - 3047-NinesAndSixes
  #6: 9♠ 9♣ 5♠ 5♥ 6♦ - 3058-NinesAndFives
  #7: 6♠ 6♦ 5♠ 5♥ 9♣ - 3221-SixesAndFives
  #8: A♠ A♥ 9♣ 6♦ 5♥ - 3501-PairOfAces
  #9: K♠ K♥ 9♣ 6♦ 5♥ - 3721-PairOfKings
  #10: Q♠ Q♥ 9♣ 6♦ 5♥ - 3941-PairOfQueens
  #11: J♠ J♥ 9♣ 6♦ 5♥ - 4161-PairOfJacks
  #12: T♠ T♥ 9♣ 6♦ 5♥ - 4381-PairOfTens
  #13: 9♠ 9♣ A♠ 6♦ 5♥ - 4471-PairOfNines
  #14: 8♠ 8♥ 9♣ 6♦ 5♥ - 4836-PairOfEights
  #15: 7♠ 7♥ 9♣ 6♦ 5♥ - 5056-PairOfSevens
  #16: 6♠ 6♦ A♠ 9♣ 5♥ - 5122-PairOfSixes
  #17: 5♠ 5♥ A♠ 9♣ 6♦ - 5342-PairOfFives
  #18: 4♠ 4♣ 9♣ 6♦ 5♥ - 5720-PairOfFours
  #19: 3♠ 3♥ 9♣ 6♦ 5♥ - 5940-PairOfTreys
  #20: 2♠ 2♥ 9♣ 6♦ 5♥ - 6160-PairOfDeuces
  #21: A♠ K♠ 9♣ 6♦ 5♥ - 6305-AceHigh
  #22: K♠ Q♠ 9♣ 6♦ 5♥ - 6753-KingHigh
  #23: Q♠ J♠ 9♣ 6♦ 5♥ - 7046-QueenHigh
  #24: J♠ T♠ 9♣ 6♦ 5♥ - 7227-JackHigh
  #25: T♠ 9♣ 8♠ 6♦ 5♥ - 7346-TenHigh
  #26: 9♣ 8♠ 6♦ 5♥ 4♠ - 7420-NineHigh

The Turn: 5♦
  Player #1 [6♠ 6♥] 97.8% (97.78%/0.00%) [44/0]
    HAND: 6♠ 6♥ 6♦ 5♥ 5♦ (271-SixesOverFives)
  Player #2 [5♦ 5♣] 2.2% (2.22%/0.00%) [1/0]
    HAND: 5♥ 5♦ 5♣ 9♣ 6♦ (2251-ThreeFives)
    OUTS: 5♠

The River: 8♦
 Winning Hand: 271-SixesOverFives
   Player #1: 6♠ 6♥ 6♦ 5♥ 5♦ - 271-SixesOverFives WINS!
   Player #2: 5♥ 5♦ 5♣ 9♣ 8♦ - 2249-ThreeFives

cargo run --example calc -- -d  "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♦ 8♦"
Elapsed: 484.90ms
```

### GTO

Here's an example of a preliminary GTO combo string calculator
that shows how a player would fare against an opponent with a common opening range.

```rust
use clap::Parser;
use pkcore::prelude::*;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  #[clap(short = 'p', long)]
  player: String,

  #[clap(short = 'v', long)]
  villain: String,

  #[clap(short = 'b', long, required = false)]
  board: Option<Board>,

  #[clap(short = 'n', long)]
  nuts: bool,
}

/// `cargo run -- -p "K♠ K♥" -v "66+,AJs+,KQs,AJo+,KQo"`
fn main() -> Result<(), PKError> {
  let now = std::time::Instant::now();
  env_logger::init();

  let args = Args::parse();

  let solver: Versus;

  if let Some(board) = args.board {
    solver = Versus::new_with_board(Two::from_str(&args.player)?, Combos::from_str(&args.villain)?, board);
  } else {
    solver = Versus::new(Two::from_str(&args.player)?, Combos::from_str(&args.villain)?);
  }

  println!("{solver}");
  println!();
  println!("Villain combos before your blockers:");
  println!("{}", solver.villain.combo_pairs());

  println!();

  println!("Villain combos after your blockers:");
  println!("{}", solver.combo_pairs());
  println!();

  let hups = solver.hups_at_deal();

  println!("Odds per hand matchup:");
  for key in hups.keys() {
    println!("{}", hups.get(key).unwrap());
  }

  let results = Versus::combined_odds_at_deal(&hups.values().collect::<Vec<&HUPResult>>());
  println!();
  println!("Consolidated odds:");
  println!("{results}");

  if solver.has_board() {
    let games = solver.games_at_flop();
    for game in &games {
      let fe = FlopEval::try_from(game.clone()).unwrap();
      println!("{fe}");
      println!("{}", WinLoseDraw::from(fe));
    }
    println!("FLOP: {}", solver.combined_odds_at_flop());
    println!("TURN: {}", solver.combined_odds_at_turn());
  }

  println!();
  println!("Elapsed: {:.2?}", now.elapsed());
  Ok(())
}
```

produces:

```shell
❯ cargo run -- -p "K♠ K♥" -v "66+,AJs+,KQs,AJo+,KQo"                                                                                                      ─╯
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.40s
     Running `target/debug/gto -p 'K♠ K♥' -v 66+,AJs+,KQs,AJo+,KQo`
Solver { hero: K♠ K♥, villain: 66+, KQs, AJs+, AJo+, KQo }

Villain combos before your blockers:
 AA  6 of  6: A♠ A♥, A♠ A♦, A♠ A♣, A♥ A♦, A♥ A♣, A♦ A♣
AKs  4 of  4: A♠ K♠, A♥ K♥, A♦ K♦, A♣ K♣
AKo 12 of 12: A♠ K♥, A♠ K♦, A♠ K♣, A♥ K♠, A♥ K♦, A♥ K♣, A♦ K♠, A♦ K♥, A♦ K♣, A♣ K♠, A♣ K♥, A♣ K♦
AQs  4 of  4: A♠ Q♠, A♥ Q♥, A♦ Q♦, A♣ Q♣
AQo 12 of 12: A♠ Q♥, A♠ Q♦, A♠ Q♣, A♥ Q♠, A♥ Q♦, A♥ Q♣, A♦ Q♠, A♦ Q♥, A♦ Q♣, A♣ Q♠, A♣ Q♥, A♣ Q♦
AJs  4 of  4: A♠ J♠, A♥ J♥, A♦ J♦, A♣ J♣
AJo 12 of 12: A♠ J♥, A♠ J♦, A♠ J♣, A♥ J♠, A♥ J♦, A♥ J♣, A♦ J♠, A♦ J♥, A♦ J♣, A♣ J♠, A♣ J♥, A♣ J♦
 KK  6 of  6: K♠ K♥, K♠ K♦, K♠ K♣, K♥ K♦, K♥ K♣, K♦ K♣
KQs  4 of  4: K♠ Q♠, K♥ Q♥, K♦ Q♦, K♣ Q♣
KQo 12 of 12: K♠ Q♥, K♠ Q♦, K♠ Q♣, K♥ Q♠, K♥ Q♦, K♥ Q♣, K♦ Q♠, K♦ Q♥, K♦ Q♣, K♣ Q♠, K♣ Q♥, K♣ Q♦
 QQ  6 of  6: Q♠ Q♥, Q♠ Q♦, Q♠ Q♣, Q♥ Q♦, Q♥ Q♣, Q♦ Q♣
 JJ  6 of  6: J♠ J♥, J♠ J♦, J♠ J♣, J♥ J♦, J♥ J♣, J♦ J♣
 TT  6 of  6: T♠ T♥, T♠ T♦, T♠ T♣, T♥ T♦, T♥ T♣, T♦ T♣
 99  6 of  6: 9♠ 9♥, 9♠ 9♦, 9♠ 9♣, 9♥ 9♦, 9♥ 9♣, 9♦ 9♣
 88  6 of  6: 8♠ 8♥, 8♠ 8♦, 8♠ 8♣, 8♥ 8♦, 8♥ 8♣, 8♦ 8♣
 77  6 of  6: 7♠ 7♥, 7♠ 7♦, 7♠ 7♣, 7♥ 7♦, 7♥ 7♣, 7♦ 7♣
 66  6 of  6: 6♠ 6♥, 6♠ 6♦, 6♠ 6♣, 6♥ 6♦, 6♥ 6♣, 6♦ 6♣


Villain combos after your blockers:
 AA  6 of  6: A♠ A♥, A♠ A♦, A♠ A♣, A♥ A♦, A♥ A♣, A♦ A♣
AKs  2 of  4: A♦ K♦, A♣ K♣
AKo  6 of 12: A♠ K♦, A♠ K♣, A♥ K♦, A♥ K♣, A♦ K♣, A♣ K♦
AQs  4 of  4: A♠ Q♠, A♥ Q♥, A♦ Q♦, A♣ Q♣
AQo 12 of 12: A♠ Q♥, A♠ Q♦, A♠ Q♣, A♥ Q♠, A♥ Q♦, A♥ Q♣, A♦ Q♠, A♦ Q♥, A♦ Q♣, A♣ Q♠, A♣ Q♥, A♣ Q♦
AJs  4 of  4: A♠ J♠, A♥ J♥, A♦ J♦, A♣ J♣
AJo 12 of 12: A♠ J♥, A♠ J♦, A♠ J♣, A♥ J♠, A♥ J♦, A♥ J♣, A♦ J♠, A♦ J♥, A♦ J♣, A♣ J♠, A♣ J♥, A♣ J♦
 KK  1 of  6: K♦ K♣
KQs  2 of  4: K♦ Q♦, K♣ Q♣
KQo  6 of 12: K♦ Q♠, K♦ Q♥, K♦ Q♣, K♣ Q♠, K♣ Q♥, K♣ Q♦
 QQ  6 of  6: Q♠ Q♥, Q♠ Q♦, Q♠ Q♣, Q♥ Q♦, Q♥ Q♣, Q♦ Q♣
 JJ  6 of  6: J♠ J♥, J♠ J♦, J♠ J♣, J♥ J♦, J♥ J♣, J♦ J♣
 TT  6 of  6: T♠ T♥, T♠ T♦, T♠ T♣, T♥ T♦, T♥ T♣, T♦ T♣
 99  6 of  6: 9♠ 9♥, 9♠ 9♦, 9♠ 9♣, 9♥ 9♦, 9♥ 9♣, 9♦ 9♣
 88  6 of  6: 8♠ 8♥, 8♠ 8♦, 8♠ 8♣, 8♥ 8♦, 8♥ 8♣, 8♦ 8♣
 77  6 of  6: 7♠ 7♥, 7♠ 7♦, 7♠ 7♣, 7♥ 7♦, 7♥ 7♣, 7♦ 7♣
 66  6 of  6: 6♠ 6♥, 6♠ 6♦, 6♠ 6♣, 6♥ 6♦, 6♥ 6♣, 6♦ 6♣


Odds per hand matchup:
K♠ K♥ 71.32% (1221273) A♠ J♥ 28.19% (482674) ties: 0.49% (8357)
K♠ K♥ 90.96% (1557553) K♣ Q♥ 7.78% (133242) ties: 1.26% (21509)
K♠ K♥ 71.21% (1219342) A♥ Q♣ 28.37% (485735) ties: 0.42% (7227)
K♠ K♥ 71.21% (1219342) A♠ Q♦ 28.37% (485735) ties: 0.42% (7227)
K♠ K♥ 90.14% (1543487) K♣ Q♦ 8.70% (148916) ties: 1.16% (19901)
K♠ K♥ 90.14% (1543487) K♦ Q♣ 8.70% (148916) ties: 1.16% (19901)
K♠ K♥ 81.70% (1398993) Q♥ Q♦ 17.83% (305351) ties: 0.46% (7960)
K♠ K♥ 80.93% (1385813) T♥ T♦ 18.66% (319499) ties: 0.41% (6992)
K♠ K♥ 69.38% (1187967) A♥ K♣ 29.74% (509318) ties: 0.88% (15019)
K♠ K♥ 80.62% (1380450) 9♠ 9♣ 18.99% (325103) ties: 0.39% (6751)
K♠ K♥ 71.95% (1231930) A♦ Q♠ 27.63% (473146) ties: 0.42% (7228)
K♠ K♥ 71.32% (1221273) A♥ J♠ 28.19% (482674) ties: 0.49% (8357)
K♠ K♥ 71.72% (1228029) A♥ Q♠ 27.78% (475675) ties: 0.50% (8600)
K♠ K♥ 79.68% (1364328) 7♦ 7♣ 20.05% (343300) ties: 0.27% (4676)
K♠ K♥ 80.31% (1375234) 6♠ 6♦ 19.33% (330994) ties: 0.35% (6076)
K♠ K♥ 80.31% (1375234) 6♠ 6♣ 19.33% (330994) ties: 0.35% (6076)
K♠ K♥ 80.31% (1375234) 6♥ 6♦ 19.33% (330994) ties: 0.35% (6076)
K♠ K♥ 90.96% (1557553) K♦ Q♠ 7.78% (133242) ties: 1.26% (21509)
K♠ K♥ 81.32% (1392403) J♠ J♦ 18.25% (312425) ties: 0.44% (7476)
K♠ K♥ 79.70% (1364648) 6♦ 6♣ 20.03% (342914) ties: 0.28% (4742)
K♠ K♥ 71.95% (1231930) A♦ Q♥ 27.63% (473146) ties: 0.42% (7228)
K♠ K♥ 68.19% (1167667) A♥ Q♥ 31.33% (536525) ties: 0.47% (8112)
K♠ K♥ 85.54% (1464659) K♦ Q♦ 13.27% (227282) ties: 1.19% (20363)
K♠ K♥ 82.35% (1410122) Q♠ Q♥ 17.10% (292846) ties: 0.55% (9336)
K♠ K♥ 18.55% (317694) A♦ A♣ 81.06% (1388072) ties: 0.38% (6538)
K♠ K♥ 80.62% (1380450) 9♠ 9♦ 18.99% (325103) ties: 0.39% (6751)
K♠ K♥ 80.68% (1381504) 8♠ 8♦ 18.94% (324287) ties: 0.38% (6513)
K♠ K♥ 71.44% (1223243) A♣ Q♦ 28.22% (483206) ties: 0.34% (5855)
K♠ K♥ 67.68% (1158934) A♦ Q♦ 31.92% (546637) ties: 0.39% (6733)
K♠ K♥ 81.32% (1392403) J♥ J♣ 18.25% (312425) ties: 0.44% (7476)
K♠ K♥ 81.57% (1396658) T♠ T♥ 17.95% (307282) ties: 0.49% (8364)
K♠ K♥ 80.05% (1370776) 8♦ 8♣ 19.65% (336398) ties: 0.30% (5130)
K♠ K♥ 81.70% (1398993) Q♥ Q♣ 17.83% (305351) ties: 0.46% (7960)
K♠ K♥ 70.82% (1212727) A♠ J♣ 28.77% (492592) ties: 0.41% (6985)
K♠ K♥ 71.56% (1225273) A♣ J♥ 28.04% (480045) ties: 0.41% (6986)
K♠ K♥ 80.30% (1374914) 7♠ 7♣ 19.35% (331347) ties: 0.35% (6043)
K♠ K♥ 80.93% (1385813) T♠ T♣ 18.66% (319499) ties: 0.41% (6992)
K♠ K♥ 2.17% (37210) K♦ K♣ 2.17% (37210) ties: 95.65% (1637884)
K♠ K♥ 80.62% (1380450) 9♥ 9♦ 18.99% (325103) ties: 0.39% (6751)
K♠ K♥ 80.93% (1385820) 6♠ 6♥ 18.63% (319074) ties: 0.43% (7410)
K♠ K♥ 71.06% (1216727) A♦ J♣ 28.61% (489963) ties: 0.33% (5614)
K♠ K♥ 69.61% (1191993) A♣ K♦ 29.60% (506801) ties: 0.79% (13510)
K♠ K♥ 80.68% (1381504) 8♥ 8♦ 18.94% (324287) ties: 0.38% (6513)
K♠ K♥ 67.33% (1152901) A♣ J♣ 32.29% (552858) ties: 0.38% (6545)
K♠ K♥ 81.32% (1392403) J♠ J♣ 18.25% (312425) ties: 0.44% (7476)
K♠ K♥ 67.83% (1161493) A♥ J♥ 31.71% (542888) ties: 0.46% (7923)
K♠ K♥ 71.95% (1231930) A♣ Q♠ 27.63% (473146) ties: 0.42% (7228)
K♠ K♥ 80.62% (1380450) 9♥ 9♣ 18.99% (325103) ties: 0.39% (6751)
K♠ K♥ 65.48% (1121238) A♦ K♦ 33.69% (576944) ties: 0.82% (14122)
K♠ K♥ 90.96% (1557553) K♦ Q♥ 7.78% (133242) ties: 1.26% (21509)
K♠ K♥ 80.93% (1385813) T♥ T♣ 18.66% (319499) ties: 0.41% (6992)
K♠ K♥ 80.30% (1374968) T♦ T♣ 19.37% (331716) ties: 0.33% (5620)
K♠ K♥ 80.93% (1385813) T♠ T♦ 18.66% (319499) ties: 0.41% (6992)
K♠ K♥ 80.68% (1381504) 8♥ 8♣ 18.94% (324287) ties: 0.38% (6513)
K♠ K♥ 80.30% (1374914) 7♠ 7♦ 19.35% (331347) ties: 0.35% (6043)
K♠ K♥ 81.25% (1391172) 9♠ 9♥ 18.28% (313002) ties: 0.47% (8130)
K♠ K♥ 69.38% (1187967) A♠ K♣ 29.74% (509318) ties: 0.88% (15019)
K♠ K♥ 79.99% (1369728) 9♦ 9♣ 19.69% (337204) ties: 0.31% (5372)
K♠ K♥ 80.68% (1381504) 8♠ 8♣ 18.94% (324287) ties: 0.38% (6513)
K♠ K♥ 80.30% (1374914) 7♥ 7♦ 19.35% (331347) ties: 0.35% (6043)
K♠ K♥ 71.21% (1219342) A♥ Q♦ 28.37% (485735) ties: 0.42% (7227)
K♠ K♥ 17.82% (305177) A♥ A♦ 81.71% (1399204) ties: 0.46% (7923)
K♠ K♥ 81.70% (1398993) Q♠ Q♦ 17.83% (305351) ties: 0.46% (7960)
K♠ K♥ 81.70% (1398993) Q♠ Q♣ 17.83% (305351) ties: 0.46% (7960)
K♠ K♥ 67.33% (1152901) A♦ J♦ 32.29% (552858) ties: 0.38% (6545)
K♠ K♥ 81.32% (1392403) J♥ J♦ 18.25% (312425) ties: 0.44% (7476)
K♠ K♥ 71.06% (1216727) A♣ J♦ 28.61% (489963) ties: 0.33% (5614)
K♠ K♥ 69.38% (1187967) A♠ K♦ 29.74% (509318) ties: 0.88% (15019)
K♠ K♥ 65.48% (1121238) A♣ K♣ 33.69% (576944) ties: 0.82% (14122)
K♠ K♥ 80.31% (1375234) 6♥ 6♣ 19.33% (330994) ties: 0.35% (6076)
K♠ K♥ 17.82% (305177) A♠ A♣ 81.71% (1399204) ties: 0.46% (7923)
K♠ K♥ 71.56% (1225273) A♦ J♥ 28.04% (480045) ties: 0.41% (6986)
K♠ K♥ 80.30% (1374914) 7♥ 7♣ 19.35% (331347) ties: 0.35% (6043)
K♠ K♥ 80.68% (1381416) J♦ J♣ 18.97% (324786) ties: 0.36% (6102)
K♠ K♥ 81.96% (1403390) J♠ J♥ 17.52% (300064) ties: 0.52% (8850)
K♠ K♥ 70.82% (1212727) A♥ J♦ 28.77% (492592) ties: 0.41% (6985)
K♠ K♥ 80.91% (1385500) 7♠ 7♥ 18.65% (319394) ties: 0.43% (7410)
K♠ K♥ 90.96% (1557553) K♣ Q♠ 7.78% (133242) ties: 1.26% (21509)
K♠ K♥ 70.82% (1212727) A♥ J♣ 28.77% (492592) ties: 0.41% (6985)
K♠ K♥ 17.09% (292660) A♠ A♥ 82.36% (1410336) ties: 0.54% (9308)
K♠ K♥ 17.82% (305177) A♠ A♦ 81.71% (1399204) ties: 0.46% (7923)
K♠ K♥ 71.21% (1219342) A♠ Q♣ 28.37% (485735) ties: 0.42% (7227)
K♠ K♥ 70.82% (1212727) A♠ J♦ 28.77% (492592) ties: 0.41% (6985)
K♠ K♥ 17.82% (305177) A♥ A♣ 81.71% (1399204) ties: 0.46% (7923)
K♠ K♥ 71.56% (1225273) A♦ J♠ 28.04% (480045) ties: 0.41% (6986)
K♠ K♥ 67.83% (1161493) A♠ J♠ 31.71% (542888) ties: 0.46% (7923)
K♠ K♥ 71.95% (1231930) A♣ Q♥ 27.63% (473146) ties: 0.42% (7228)
K♠ K♥ 71.44% (1223243) A♦ Q♣ 28.22% (483206) ties: 0.34% (5855)
K♠ K♥ 81.05% (1387864) Q♦ Q♣ 18.56% (317856) ties: 0.38% (6584)
K♠ K♥ 81.31% (1392232) 8♠ 8♥ 18.23% (312176) ties: 0.46% (7896)
K♠ K♥ 85.54% (1464659) K♣ Q♣ 13.27% (227282) ties: 1.19% (20363)
K♠ K♥ 69.61% (1191993) A♦ K♣ 29.60% (506801) ties: 0.79% (13510)
K♠ K♥ 71.72% (1228029) A♠ Q♥ 27.78% (475675) ties: 0.50% (8600)
K♠ K♥ 68.19% (1167667) A♠ Q♠ 31.33% (536525) ties: 0.47% (8112)
K♠ K♥ 67.68% (1158934) A♣ Q♣ 31.92% (546637) ties: 0.39% (6733)
K♠ K♥ 71.56% (1225273) A♣ J♠ 28.04% (480045) ties: 0.41% (6986)
K♠ K♥ 69.38% (1187967) A♥ K♦ 29.74% (509318) ties: 0.88% (15019)

Consolidated odds:
72.40% (120246594), 26.11% (43363520), 1.50% (2483374)

Elapsed: 2.72s
```

## TODO:

* Roadmap
* Clear release breakdowns.
* Consolidate card collection structs. We currently have:
  * [Bard](src/bard.rs) - binary representation of one or more [Cards](src/cards.rs)
  * [BoxedCards](src/arrays/sliced.rs) - Boxed slice of [Card](src/cards.rs) values.
  * [Cards](src/cards.rs) - IndexSet collection of unique [Card](src/cards.rs) values.
  * [Board](src/play/board.rs) - The community cards in a game of Hold'em and Omaha.
  * [HoleCards](src/play/hole_cards.rs) - The cards dealt to each player in Hold'em and Omaha.
  * Fixed array [Card](src/cards.rs) structs. Can these all be replaced by [BoxedCards](src/arrays/sliced.rs)?
    * [Deck](src/cards.rs)
    * [Two](src/arrays/two.rs)
    * [Three](src/arrays/three.rs)
    * [Four](src/arrays/four.rs)
    * [Five](src/arrays/five.rs)
    * [Six](src/arrays/six.rs)
    * [Seven](src/arrays/seven.rs)

## Value Stories

* I want a tool that will help me get better at [GTO](https://www.888poker.com/magazine/strategy/beginners-guide-gto-poker) style poker playing.
* I want a library that can be reused for poker applications.

## Resources

* Poker
  * [Mike Caro's Website](https://www.poker1.com/)
  * Solvers
    * [Pro Poker Tools Odds Oracle](http://www.propokertools.com/)
  * GTO
    * [Minimum Defense Frequency vs Pot Odds](https://upswingpoker.com/minimum-defense-frequency-vs-pot-odds/)
    * [A Beginner’s Guide to Poker Combinatorics](https://blog.gtowizard.com/a-beginners-guide-to-poker-combinatorics/)
  * Terms
    * [The Nuts](https://en.wikipedia.org/wiki/Nut_hand)
      * 888poker > [What is Nuts in Poker?](https://www.888poker.com/magazine/poker-terms/nuts)
      * GetMega > [Nuts in Poker](https://www.getmega.com/cards/poker/terms/nuts-in-poker/)
    * 888poker > [What is Texture in Poker?](https://www.888poker.com/magazine/poker-terms/texture)
  * AI
    * [Pluribus](https://en.wikipedia.org/wiki/Pluribus_(poker_bot))
      * [Superhuman AI for multiplayer poker](https://www.science.org/doi/10.1126/science.aay2400)
      * [pluribus-hand-parser](https://github.com/VitamintK/pluribus-hand-parser)
      * [Let's analyze Pluribus's Hands!](http://kevinwang.us/lets-analyze-pluribuss-hands/)
        * [reddit](https://www.reddit.com/r/poker/comments/cdhasb/download_all_10000_hands_that_pluribus_poker_ai/)
      * [fedden / poker_ai](https://github.com/fedden/poker_ai) - An Open Source Texas Hold'em AI
        * FORK: [pluribus-poker-AI(https://github.com/apcode/pluribus-poker-AI)
      * [Remembering Pluribus: The Techniques that Facebook Used to Master World’s Most Difficult Poker Game](https://www.kdnuggets.com/2020/12/remembering-pluribus-facebook-master-difficult-poker-game.html)
      * [PokerHandEvaluator](https://github.com/HenryRLee/PokerHandEvaluator)
      * Naom Brown: [5 years ago we revealed Pluribus](https://x.com/polynoamial/status/1816347598623834365?lang=en)
    * [Neuron Poker: OpenAi gym environment for texas holdem poker](https://github.com/dickreuter/neuron_poker)]
    * [clubs_gym](https://github.com/fschlatt/clubs_gym)
    * [PokerRL](https://github.com/EricSteinberger/PokerRL) Framework for Multi-Agent Deep Reinforcement Learning in Poker games.
    * [Best AI Poker Training Tools](https://thepokeragent.com/best-ai-poker-training-tools/)
  * Probability
    * Wikipedia > [Poker probability](https://en.wikipedia.org/wiki/Poker_probability)
    * [Distinct head-to-head match ups in holdem](https://poker.stackexchange.com/questions/5682/distinct-head-to-head-match-ups-in-holdem)
    * [Texas Hold’em Poker Odds (over 100 Poker Probabilities)](https://www.primedope.com/texas-holdem-poker-probabilities-odds/)
    * Heads up
      * [Mathmatrucker > Preflop High Hand Equity and Tie Percentages](https://www.mathematrucker.com/poker/matchups.php)
  * Cheating
    * [FTX’s ‘chief regulatory officer’ Dan Friedberg tied to online poker scandal](https://nypost.com/2022/11/20/ftxs-ex-chief-regulatory-officer-tied-to-online-poker-scandal/)
  * Cool Resources
    * YouTube > [I created an AI to Play Poker](https://www.youtube.com/watch?v=MWRXx2saLw4)
      * The code: [Gongsta / Poker-AI](https://github.com/Gongsta/Poker-AI/)
      * Related Stuff
        * [Counterfactual regret minimization in Rust](https://github.com/erikbrinkman/cfr)
* Rust
  * [The Rust Programming Language](https://doc.rust-lang.org/stable/book/)
  * [Rust Design Patterns](https://rust-unofficial.github.io/patterns/intro.html)
  * [Are we game yet?](https://arewegameyet.rs/)
  * [Are we GUI Yet?](https://www.areweguiyet.com/)
  * [rustlings](https://github.com/rust-lang/rustlings)
  * frameworks
    * [Yew](https://yew.rs/)
    * [Flowistry: Information Flow for Rust](https://github.com/willcrichton/flowistry)
    * Graphic Libraries
      * [tui-rs](https://github.com/fdehau/tui-rs)
        * [Rust and TUI: Building a command-line interface in Rust](https://blog.logrocket.com/rust-and-tui-building-a-command-line-interface-in-rust/)
      * [Crossterm](https://github.com/crossterm-rs/crossterm)
    * [Creating successful open source projects - with @orhunp - RustShip 1](https://www.youtube.com/watch?v=_xABF_H8b3g)
  * OTel
    * [tracing.rs](https://tracing.rs/tracing/) [GitHub](https://github.com/tokio-rs/tracing)
      * [tracing-test](https://docs.rs/tracing-test/latest/tracing_test/)
  * articles
    * [Rust Is Hard, Or: The Misery of Mainstream Programming](https://hirrolot.github.io/posts/rust-is-hard-or-the-misery-of-mainstream-programming.html)
    * [Rust: Your code can be perfect](https://www.youtube.com/watch?v=IA4q0lzmyfM)
      * Probability
        * [How To Work Out Flop Probability In Texas Holdem](https://www.thepokerbank.com/tools/odds-charts/work-out-flop-probability/)
  * videos
    * [Poker Out Loud](https://solveforwhy.io/categories/poker-out-loud)
      * [Poker Our Loud Academy demo](https://www.youtube.com/watch?v=NpSDXJsej-o&t=634s)
      * [Great rant on stack sizes in 2022 WSOP](https://www.youtube.com/watch?v=a8801jTxt4Y&t=820s)
  * mobile
    * android
      * [Building pure Rust apps for Android](https://blog.traverseresearch.nl/building-pure-rust-apps-for-android-d1e388b431b8)
      * [Building and Deploying a Rust library on iOS](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-06-rust-on-ios.html)
      * [Building and Deploying a Rust library on Android](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html)
      * [Running Rust on Android](https://blog.svgames.pl/article/running-rust-on-android)
  * concurrency
    * [Rayon](https://github.com/rayon-rs/rayon)
      * [How Rust makes Rayon's data parallelism magical](https://developers.redhat.com/blog/2021/04/30/how-rust-makes-rayons-data-parallelism-magical#generic_constraints_in_rayon)
      * [Implementing Rayon’s Parallel Iterators - A Tutorial](https://geo-ant.github.io/blog/2022/implementing-parallel-iterators-rayon/)
  * Macros
    * [The Little Book of Rust Macros](https://github.com/Veykril/tlborm.git)
    * [What Every Rust Developer Should Know About Macro Support in IDEs](https://blog.jetbrains.com/rust/2022/12/05/what-every-rust-developer-should-know-about-macro-support-in-ides/)
  * DBs
    * [SurrealDB](https://surrealdb.com/)
  * Code Coverage
    * [How to do code coverage in Rust](https://blog.rng0.io/how-to-do-code-coverage-in-rust)
* Video
  * [Cloudinary's image overlay feature](https://cloudinary.com/documentation/video_manipulation_and_delivery#adding_image_overlays)
  * [Programmatically add 100s of image overlays on video clip](https://stackoverflow.com/questions/18750525/programatically-add-100s-of-image-overlays-on-video-clip)
  * open source
    * [How to Add Graphics and Overlays to Live Video With Open Broadcaster (OBS)](https://photography.tutsplus.com/tutorials/how-to-add-custom-graphics-obs-studio--cms-35066)
* Programming
  * [The Grug Brained Developer](https://grugbrain.dev/)
  * GUI
    * [Ratatui](https://github.com/tui-rs-revival/ratatui)

## Dependencies

* [assert-struct](https://github.com/carllerche/assert-struct)
* [bitvec](https://github.com/ferrilab/bitvec)
* [Burnt Sushi CSV](https://github.com/BurntSushi/rust-csv) with Serde support
* [itertools](https://github.com/rust-itertools/itertools)
* [Serde](https://serde.rs/)
  * [Serde JSON](https://github.com/serde-rs/json)
  * [Serde YAML](https://github.com/dtolnay/serde-yaml)
* [Termion](https://github.com/redox-os/termion)

## Potential Libraries

* [derive_more](https://github.com/JelteF/derive_more) (Recommended by Rust Power Tools)
* [mycelium-bitfield](https://crates.io/crates/mycelium-bitfield)
* [modular-bitfield](https://crates.io/crates/modular-bitfield)
* [RustyLine](https://github.com/kkawakam/rustyline)
* [sled](https://github.com/spacejam/sled)
* SQLite
  * [rusqlite](https://github.com/rusqlite/rusqlite)
    * [Rust Cookbook](https://rust-lang-nursery.github.io/rust-cookbook/database/sqlite.html)
    * [In-Memory Databases](https://www.sqlite.org/inmemorydb.html)
* UI
  * [Ratatui](https://github.com/ratatui-org/ratatui)
  * [shadcn/ui](https://ui.shadcn.com/)
    * [This UI Library is NEXT LEVEL](https://www.youtube.com/watch?v=dD1fpoGHuC8&t=29s)

## TMI

This library was intended to be the example code for a book tentatively titled _Rust for Failures_. Here's the intro:

Motto

Failure is a gift.

I am a failure. I've been a failure all my life. I’ve dropped out of college three times. I graduated from high school with a D average. I’ve lost every professional French Horn audition I’ve ever taken.
(I did win first horn at the California State Honor Band, but all the other horn players were so mad that I'd won that didn't talk to me, so I was miserable the whole time.) For a period of time I was homeless. (Ask me about sleeping in the San Francisco Symphony's Davies Symphony Hall musician's lounge.)

In the fifth grade, I got the silver medal for having the second-worst car in the Cub Scouts Pinewood Derby. I would have been last, but the Scout Master from a rival troupe took
the wheel that had fallen off from one of his kids' cars, hammered it back onto the car, and stole first from me by deliberately creating a car that wouldn’t roll. I even failed at failure. I was the worst. I should have gotten the gold, but no, I was the second worst. If you’re not last, you’re…?

There’s only one thing that I’ve ever really felt like a master of. Nobody ran the early morning solo coffee rush at a 7-11 better than me. At the Colma 7-11 I kept all five
pots of coffee humming, making sure to keep them in a steady rotation, so that those smart-asses who tried to get the fresher pot of coffee by grabbing the one in the
back were actually getting the oldest one. Not that it mattered. They were always fresh as fuck. A regular walked in, and I had their three packs of Benson & Hedges Ultra
Lights waiting for them at the register before they even asked. I was the shit, and all of my customers knew it, and were gracious to let me know, if in more family-friendly terms.

Then, one simple event solidified it all for me. A beautiful motor home drove into the parking lot, and out stepped a gentleman to buy a cup of coffee and a pack of gum.
We talked. He wanted to thank me. He was doing really well now. He was working in a restaurant in downtown San Francisco and had really turned his life around, all thanks to me! Great.

A year before, he was pointing a revolver at my face. He ran out with 204 dollars, only to be caught by the K-9 unit a few blocks away
shortly thereafter. This nimrod had fucking robbed me, and he was already doing better than me.
His life was on beautiful, and I was still that dumb schmuck making $9 an hour under the table, farting his life away at the Colma 7-11, where San Francisco buries its dead.

It wasn’t until I became a professional programmer that I realized failure is a gift IF you learn from it. By testing the shit out of the systems I build and practicing techniques
like test-driven development, I turned my ability to fail into a superpower. The faster you fail, the more you profit.

You see… it turns out that we’re all in the feedback business. You will get that feedback. It just depends on if you want to get it when you’re testing your tiles out in a controlled environment,
or if you want to find out when every kid in school is watching your [space shuttle explode with the first teacher in space](https://en.wikipedia.org/wiki/Space_Shuttle_Challenger_disaster). (I was one of those kids.)
