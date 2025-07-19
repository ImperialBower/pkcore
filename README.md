# pkcore AKA Spawn of [Fudd](https://github.com/ImperialBower/fudd)

🚧 **Work In Progress** 🚧

[Rust](https://www.rust-lang.org/) poker library. Code inspired by [Cactus Kev's](https://suffe.cool)
[work in C](https://suffe.cool/poker/code/). An isolated version of the core hand evaluation library is available at [ckc-rs](https://github.com/ContractBridge/ckc-rs).

Currently only supports [hold'em](https://en.wikipedia.org/wiki/Texas_hold_%27em), but working on [Omaha](https://en.wikipedia.org/wiki/Omaha_hold_%27em) and want to add more types of games. Supporting
things like [Razz](https://en.wikipedia.org/wiki/Razz_(poker)) would be a total kick.

This code is a complete rewrite from scratch of my [Fudd](https://github.com/ImperialBower/fudd) crate. Changes:

* Folded [ckc-rs](https://github.com/ContractBridge/ckc-rs) crate into the repo.
* Folded [wincounter](https://github.com/ImperialBower/wincounter) crate into the repo.
* Removed [cardpack.rs](https://github.com/ImperialBower/cardpack.rs) dependency

## TODO:

* Roadmap
* Clear release breakdowns.

## Examples

Check out the examples directory for various ways to use the library. 

The best example is [calc](examples/calc.rs), which allows you to do a breakdown of the odds of a specific hand
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

## Value Stories

* I want a tool that will help me get better at [GTO](https://www.888poker.com/magazine/strategy/beginners-guide-gto-poker) style poker playing.
* I want a library that can be reused for poker applications.

## Resources

* Poker
  * [Mike Caro's Website](https://www.poker1.com/)
  * GTO
    * [Minimum Defense Frequency vs Pot Odds](https://upswingpoker.com/minimum-defense-frequency-vs-pot-odds/)
    * [A Beginner’s Guide to Poker Combinatorics](https://blog.gtowizard.com/a-beginners-guide-to-poker-combinatorics/)
  * Terms
    * [The Nuts](https://en.wikipedia.org/wiki/Nut_hand)
      * 888poker > [What is Nuts in Poker?](https://www.888poker.com/magazine/poker-terms/nuts)
      * GetMega > [Nuts in Poker](https://www.getmega.com/cards/poker/terms/nuts-in-poker/)
    * 888poker > [What is Texture in Poker?](https://www.888poker.com/magazine/poker-terms/texture)
  * Pluribus
    * [Superhuman AI for multiplayer poker](https://www.science.org/doi/10.1126/science.aay2400)
    * [pluribus-hand-parser](https://github.com/VitamintK/pluribus-hand-parser)
    * [Let's analyze Pluribus's Hands!](http://kevinwang.us/lets-analyze-pluribuss-hands/)
      * [reddit](https://www.reddit.com/r/poker/comments/cdhasb/download_all_10000_hands_that_pluribus_poker_ai/)
    * [fedden / poker_ai](https://github.com/fedden/poker_ai) - An Open Source Texas Hold'em AI
    * [Remembering Pluribus: The Techniques that Facebook Used to Master World’s Most Difficult Poker Game](https://www.kdnuggets.com/2020/12/remembering-pluribus-facebook-master-difficult-poker-game.html)
    * [PokerHandEvaluator](https://github.com/HenryRLee/PokerHandEvaluator)
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
  
