# pkcore AKA Rust for Imposters AKA Rust for Failures

## Value Stories

* I want a tool that will help me get better at GTO style poker playing.
* I want a library that can be reused for poker applications.

## Outline

* Got rust?
  * Cargo, your new best friend
  * Cargo clippy BEAST MODE
  * Cargo fmt
    * STORY TIME: Why I love clean code. (Migraines)
* [Setup wasm](https://rustwasm.github.io/docs/book/game-of-life/setup.html).
* Why Rust?
  * Inverting the curve
  * THE BIG IDEA: Better to eliminate a problem than to solve it.
  * Rust TDD loop
    * define
    * create fn sig returning default value
    * create failing test valid on expected value
    * Make test green
    * any more boundary conditions?
    * refactor
    * draw negative boundary refactor to Result for overdraw
* Letting the IDE do a lot of the work (Mad Dog Murdock)
  * Compare CLion to VSCode
* Create pkcore lib
  * Set #![warn(clippy::pedantic)]
* EPIC: Display HandRank
  * Rank
    * lib:PKError
    * create enum
    * ::from(char)
      * Testing: TELL THE HEROES STORY
        * Tests using [rstest](https://crates.io/crates/rstest) BRUTE FORCE
        * Readability of tests names (short, matching, clear)
        * Hub and spoke... take your time on the core.
        * Negative boundary conditions
    * ::from_str()
      * test neg scenarios #[allow(non_snake_case)]
  * Suit
    * create enum
    * ::from(char)
      * Tests using [rstest](https://crates.io/crates/rstest)
    * ::from_str()
      * test neg scenarios #[allow(non_snake_case)]
    * Card
      * CKC Card Number consts
        * Intro to CKC Numbers
      * ::as_u32()
      * ::from<u32> (filter)
        * #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        * Talk about brute force testing philosophy.
        * Add Rank.number(), .bits(), .prime(), and .shift8().
          * strum::EnumIter && tests
      * card_consts
      * ::new
      * .is_blank()
      * .from_str()
        * Boundary conditions tests.
        * THE WONDERFUL ? operator
      * Detour on Testing as the Hero's Journey
        * tell the story
        * scannable
      * Move card numbers and make private
      * .to_string() which means doing it for Rank and Suit.
    * Rank
      * .to_string() thus .to_char()
    * Suit
      * .to_string() thus .to_char()
    * Card
      * .to_string()
    * Cards
      * [indexmap::IndexSet](https://github.com/bluss/indexmap)
      * indexmap::set::Iter
      * .len() .is_empty()
      * .insert and .iter()
      * .to_string()
      * .sort() (adding Ord)
    * Card
      * .bit_string() -- Talk about expressive logging.
    * REFACTORING I don't expect you to copy out every part of the test code, although you are more than happy to. Think of this as a cooking show where I, your humble host, does most of the boring parts off camera so that you don't get bored. Voila! Losts of tests refactored!
      * CardNumber enum
      * into()
      * rstests: Consolidating  tests
        * The problem is that they aren't really testing for value Just number not number to actual card
    * Five INTRODUCING CACTUS KEV
    * Cards
      * implement FromStr so can be used by arrays like Five.
      * need get_index() to do Five.from_str()
    * Five
      * impl TryFrom<Cards>
      * now implement FromStr leveraging Cards.from_str()
      * .to_arr()
      * Get Five eval to return HandRank Number
        * .rank()
          * .or_bits()
          * .or_rank_bits()
          * .and_bits()
          * .is_flush()
          * .rank()
            * flushes
            * unique
            * the rest
              * .multiply_primes()
                * Card.get_prime() (Add to get_rank tests)
              * REFACTORING: Section for pub fn then private fn
              * not_unique()
                * find_in_products()
                  * refactoring from ckc (compare)
      * POSSIBLE DEFECT: Cards in different orders aren't equal
    * Cards
      * REFACTORING: Clippy found call to `str::trim` before `str::split_whitespace` cards.rs:72:20
    * Five
      * rank() basic tests
    * INTRODUCING HandRank struct
      * type aliases (HandRankValue, CactusKev)
        * CLIPPY: HandRankValue clippy::module_name_repetitions
      * hand_rank module (talk about separating files as old habit)
      * HandRankName
        * about strum::EnumIter;
        * refactoring HandRank.determine_name() to HandRankName::from (small test range... ranges of testing)
      * HandRankClass
        * refactoring HandRank.determine_class() to HandRankClass::from
          * CLIPPY #[warn(clippy::too_many_lines)]"
        * AUDIBLE (waiting for HandRank MEGA TEST to capture all) (AUDIBLE = change from standard flow)
      * REFACTORING: impl Default to derive
      * HandRank::from(HandRankValue)
        * Favor traits over functions
        * PROCESS NOTE::: Step by step
          * why I do small commits
          * git muscle memory get into the habit of knowing the commands
        * REFACTORING name and class no longer public (remove need for ckc.is_a_valid_hand_rank())
          * HARDENING (over thinking? Maybe)
        * TRAIT: SOK - I find boring names hard to remember
        * arrays::HandRanker
          * remove Five::rank() replace with HandRanker
          * REFACTOR: remove validated methods from Trait
      * Hand HandRanker BIG KAHUNA TEST
        * using //region
        * REFACTORING HandRankName  HandRankClass to Name and Class
    * DETOUR: Generating lookup tables
      * Is Straight Flush?
        * Is Straight?
          * Let's add tests for all types of straights
            * WHEEL???!!!!
        * Already have .is_flush()
        * .is_straight_flush()
        * INTRODUCING: IterTools
          * Cards.combinations()
          * Cards::from<Vec<Card>>
          * Five::try_from<Vec<Card>>
          * TEACHABLE: let rust show you what to return
    * Six
      * Before we get started lets update the repl so that we get more feedback
      * impl TryFrom<Cards>
      * Five.display
        * Cards::from(Card)
        * Cards::from(Five)
      * examples/repl.rs Update to use Cards
      * Cards.draw() reqs Cards.deck()
      * .hand_rank()
        * Let IDE drive with the TODO
          * yay todo!()
          * Understand the dynamics of teams, bend to the norms. Different people react differently to code styles. It's OK Be cool
        * adding .five_from_permutation to HandRanker
          * Adding to trait because I know I will need it for Seven as well. NOTE: This is not strict TDD.
          * REFACTORING: Originally a separate trait Permutator in ckc-rs. Decided to consolidate make cleaner.
          * REFACTORING: Move to Trait
        * Five.sort() no method named `sort` found for struct `Five` in the current scope
          * Adding to HandRanker trait
          * Later refactoring to handle less then five cards (Another trait?)
        * And we're green!!!
        * More tests
          * REFACTORING: Adding HandRanker.hand_rank_and_hand() to trait
          * Fleshing out
          * Deliberate using different forms of index strings
            * Be careful with your patterns and assumptions. TDD can be a self-fulfilling prophecy divorced from reality
      * .sort()
    * TODO: Note on Five.sort() for wheels Ace on top
      * ASIDE: I am fiercely pro TODOs. Many people hate them. That's OK. However, this is my kingdom and I rule it with an iron fist! When you write your book, you too can be Absolute Ruler of your one person empire (Unless you have an editor 😉).
    * Seven
      * Seven holdem boards etc
      * BONUS CREDIT:
        * OK, if you are seriously hard core and want to go for bonus points (awarded via my GFY Cryptocurrency guaranteed to be worthless!!!) Try coding Seven yourself.
          * What are its reqs? What's different about Six and Seven?
      * .from(array)
      * impl TryFrom<Cards>
      * Drive from implementing HandRanker
        * .five_from_permutation()
          * Get not yet implemented error
          * Fix error
          * make green
        * .hand_rank_value_and_hand()
          * Why AD 6S 4S AS 5D 3C 2S?
          * paste over test from Six (which will fail for Seven)
          * get rid of not yet implemented error
          * make green
          * Clippy
      * TODO: No assessors besides .to_arr() do I really need the others?
    * UPDATE repl to handle multiple card lengths
      * Print out HandRank info for best hand if five or more cards.
        * Update to look at Cards length and usher to the right struct
        * NOTE on driving through repl instead of tests
          * move index out to variable. (Why is it called index?)
            * check in repl again
          * let cards = Cards::from_str(index).unwrap();
            * check in repl again
          * Add match for default _
          * Add match for five
            * impl TryFrom<Cards> for Five
              * REFACTOR: move match from from_str to TryFrom
      * NEEDED: to_string() for Five, Six and Seven
        * impl Display for Five
          * 5 => println!("{}", Five::try_from(cards)?),
            * update main to return error
              * update sig to fn main() -> Result<(), PKError> {
                * must return Ok(())
                * Verify `❯ cargo run --example repl -- -c "AS KS QS JS TS"`
          * 6 => println!("Six: {}", Six::try_from(cards)?),
            * impl Display for Six
          * 7
            * .to_arr()
            * .display()
    * UPDATE REPL: Display HandRank
      * show() - Handle generic input for function
      * DEFECT: Make sure it sorts the best hand
        * Five.is_wheel()
        * Make sure it sorts wheels correctly
      * UPDATE: Make it display the original cards
        * Add Display to function's trait requirements
    * FEATURE COMPLETE!!!!
* ASIDE: Code Coverage
  * Through Clion
  * gcov etc...
* EPIC: GAME PLAY - Hold'Em play out hand
  * Two
    * Assessors
    * ::from_str()
      * TryFrom<Cards>
        * From<[Card; 2]>
  * Introducing example/calc
    * Clap Args: dealt, board
    * Display dealt and board
      * Have main return Result<(), PKError>
    * Introducing play::hands::Hands
      * Hands::try_from(Cards)
        * Two::new(card, card)
          * HP
          * Must be unique
            * RF: Returns result
            * Implements SOK to test for uniqueness
              * Drive through SOK
          * First card needs to be higher than second.
            * Add to happy path test
        * Hands::from_str()
        * Hands::to_string()
          * Two
            * .to_string()
              * .to_vec()
              * REFACTOR: use Cards to display instead of code duplication
                * Also refactor for other array types
                  * .to_vec()
                  * I LOVE DELETING CODE
      * THE FLOP
        * Show HandRank for each player.
        * INTRODUCING: Game
          * Flop: arrays::three::Three
            * DO IT!
            * NOTE ON READABILITY: I deal with a lot of code. I want to scan it
              * CODE SPEED READING
            * `From<[Card; 3]>`
            * `impl SOK for Three`
            * `impl TryFrom<Cards> for Three`
            * `impl Display for Three`
          * INTRODUCING: Board
            * The power of `pub` on struct fields.
            * TryFrom<Cards> for Board
              * too few
              * too many
              * three cards
                * display
                  * Default: "FLOP: ____ __, TURN:__, RIVER: __"
                  * THE BOARD from THE HAND
              * four cards
                * getting clunky
                * REFACTOR: Cards.draw_one from Option to Result
              * five cards
            * FromStr for Board
          * Update calc to display Hands and Board
            * REFACTOR: Game{Hands, Board}
            * Game.to_string()
              * TEST REFACTORING: `fn state() -> (Hands, Board, Game) {`
            * `DEALT: [6♠ 6♥, 5♦ 5♣] FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠`
    * PHASE TWO: Calc the Flop
      * THE PLAN:
        * Display HandRank for each hand
        * Display winning percentages
        * Display outs
      * PHASE 2.1: Display HandRank for each hand
        * Five::from_2and3()
        * calc: iterate through each player's hand and show it's `HandRank`.
          * REFACTOR: Make `Game` struct fields public.
          * Hands.iter()
          * Game.five_at_flop()
            * Hands.get()
            * DEFECT: HandRank for Five::default()
              * impl SOK for Five
        * DEFECT DETOUR: Sort Five by card frequency
          * .map_by_rank()
            * Card frequency flags
              * Card.frequency_paired()
              * Card.frequency_tripped()
            * Cards.map_by_rank()
              * impl indexmap::set::IntoIter for Cards (needs to be indexmap::set::IntoIter not std::vec::IntoIter)
                * impl From<Vec<&Card>> for Cards
              * Card.is_flagged_ ... (Used to filter cards)
                * REFACTOR: Card.is_flagged()
                  * REFACTOR: Delete is_flagged_ methods and just use .is_flagged()
          * Cards.flag_
          * Cards.frequency_weighted()
            * Cards.add()
              * Five test effect of adding weighted values on CKC eval
                * Cards.dump()  debugging helper
                  * DEFECT FOUND: Issue with Card.bit_string()
          * CLOSE IT OUT
            * Five #[test] sort__pair()
      * PHASE 2.2: Display winning percentages
        * THE PLAN:
          * Determine all cards in hands and on board at flop.
          * Get every remaining combination of turn and river cards.
          * Get HandRank for each hand against possible board.
          * Add up winning hand
          * Determine the percentages
        * Introducing [Win Counter](https://github.com/ContractBridge/wincounter)
          * AUDIBLE: moved crate code to util::wincounter for now for easy updating.
        * Three all possible at flop
          * Cards.deck_minus()
            * Cards.get()
              * Cards.shuffle() - Want this to harden the unit tests for deck_minus()
                * util::RandomOrdering
                  * Add [rand crate](https://crates.io/crates/rand)
          * REFACTOR: Hands.get() top return Option
            * REFACTOR: Game.five_at_flop() to return Result<Five, PKError>
        * Calc trait - Don't think that I like that name, but it will do for now.
          * .cards()
            * impl Calc for Card, Two, Three, Five, Six, Seven, Hands
          * REFACTOR: rename to Pile
          * REFACTOR: add .to_vec to trait
            * Delete Cards::from<Five> - No longer needed
            * I really love this refactor. It simplifies all the communication between card collections.
        * Game.remaining_cards_at_flop()
          * Game.play_out_flop() INTRODUCING PLAY OUT!!! WOOOOO!!!!! TPOS 1.1
            * Game.case_seven()
              * HandRank Eval
                * impl Ord for HandRank
                * STEP 1: Eval
                  * ::new(hand_rank: HandRank, hand: Five)
                  * impl From<Five> for Eval
                  * impl PartialEq for Eval
                    * impl Hash for Eval
                  * impl From<Seven> for Eval - Why did I do Five first??!!
                  * impl Display for Eval
                  * introducing Logging!
                    * [log crate](https://crates.io/crates/log)
                    * [env_logger](https://crates.io/crates/env_logger)
                  * .sort()
                    * verify that Vec<Card> sorts as desired
                    * verify that Vec<Five> sorts as desired
                      * sort__vector_of_fives()
                  * Now let's crunch some Wins
                  * TPOS 1.2: PlayOut Trait (_wild hare_)
                    * REFACTORING: Game.remaining_cards_at_flop() to Hands.remaining_after()
                      * REFACTORING: Sick question? Can I add remaining() and remaining_after to the Pile struct?
                        * The answer is yes :-)
                    * WE ARE STILL RED??!! Let's implement PlayerWins
                      * OK, we got it to build.
                      * VERY IMPORTANT DANGER DANGER WILL ROBINSON
                        * Commit before you do a major, structural refactoring!!!
                          * For now, we will have a private pof generic injection for game. Later on we may want to get fancier.
                          * NOTE: for now, I am playing without a unit test net. I am using the flow of the repl guide me. Later on, when I am done I will harden the code with tests, but for now we freefall. Weeeee!!!!
                    * OK, now let's clippy this MFer.
                      * There are few greater highs for me then deleting code after a refactoring. Deleting code is the highest calling of a programmer. Learn to treasure these moments. This is the way.
                        * Deleting Game.case_seven()
                    * REFACTORING: Introducing the analysis module
                      * REFACTORING: Adding combinations_after() and  enumerate_after() to Pile.
                      * REFACTORING: PlayOut.play_out_flop() Hands just needs a reference.
                      * RF: Hands.realize_case_at_flop()
                      * Three::from_str()
                      * DEFECT ALERT: The Five stored in case is frequency rated for sorting so doesn't match a raw Five, even though they are the same hand. We need to strip those flags before storing them in case.
                        * Adding .clean() to Pile trait.
                          * The beautiful todo!() macro
                            * Implement for Card
                              * Card::FREQUENCY_MASK_FILTER
                            * Five.clean()
                              * Add `assert_eq!(hand.sort().clean(), five);` to Five hand_rank mega test
                              * Added .clean() to Seven.hand_rank_value_and_hand() hand return
                              * Removed unneeded .strip() and .clean() from hand_rank_case_tests::from__seven() test
                * STEP 2: CaseEval
                  * .push()
                  * INTRODUCING: pub(crate) TestData mod
                    * Hands::from(Vec<Two>)
                    * Refactoring Hands test data to use TestData.
                    * Adding constants to Two
                  * Added util::wincounter from crate for easier updating
                  * CaseEval.win_count()
                    * Win::from(index)
                    * TODO RF: Refactor `Count` as a `struct Count(u16)`.
                    * DEFECT ALERT: Copied the wrong function in the test data.
                    * Test #3: A TIE
                      * Win::or()
                * STEP 3: CaseEvals
                  * examples/calc command() function to display command
                  * impl Pile for Board
                  * deprecate Game .play_out_flop() .pof() & .remaining_cards_at_flop()
                * Step 4: Results
                  * REFACTORING: util::Util::calculate_percentage()
                  * TestData::wins_the_hand()
                  * REFACTORING: Results struct to include case and player count.
                  * .wins_total()
                  * .wins_and_ties()
                  * REFACTORING: .wins_total() to use .wins_and_ties()
                  * .wins_and_ties_percentages()
                    * removed Wins::percent_calculator() since dupe of Util::calculate_percentage()
                    * Fixed Results::from_wins() to include case_count and player_count.
                  * .wins_total_percentage()
                  * .player_to_string()
                  * impl Display
            * Three: The nuts in order
              * TheNuts(Vec<Eval>)
                * impl From<Vec<Eval>> for TheNuts
                * .to_vec()
                * .sort()
                  * Two KT constants (for The Fold)
                  * REFACTOR: renamed to Hands to HoleCards
                  * HoleCards.three_into_fives(&self, three: Three) -> Vec<Five>
                  * HoleCards.three_into_evals(&self, three: Three) -> Vec<Eval>
                    * impl From<&Five> for Eval
                  * TestData::evals_the_fold()
              * Pile.the_nuts()
                * impl From<Vec<Card>> for Two
                * REFACTORING to HashMap
                  * SIDETRACK: impl Pile for Cards
                    * Pile.contains_blank() as replacement for SOK trait
                    * REFACTORING: Pile.is_dealt() to replace SOK.salright()
                  * Five overriding `Eq`
                    * CANCELLED: This experiment failed.
                  * SIDETRACK: Bard
                    * impl BitOr for Bard
                    * impl BitAnd for Bard
                    * BitXor, BitAndAssign, BitOrAssign, BitXorAssign
                    * DOUBLE SIDETRACK: Added bitwise todo!s to Cards.
                * Nutty.get()
                  * Evals
                  * examples/evals_per_class.rs
                    * HandRanker.eval()
                    * EvalsPerClass in TheNuts' rustdoc
                * REFACTOR: Pile.the_nuts() to .possible_evals()
              * Done!
                * REFACTOR: Moved possible_evals() test from TheNuts to Three
            * Two: .possible_evals()
              * impl From<Vec<Card>> for Three
    * PHASE THREE: Calc the Turn
      * examples.calc add display of the nuts at flop.
        * REFACTOR: move the_hand() from game tests to `TestData`. (Note the clippy warning from the refactoring. Nice catch 📎!)
      * Game.possible_evals_at_flop()
      * Game.possible_evals_at_turn()
        * !.is_dealt()
        * Four
          * impl From<Vec<Card>> for Four
          * impl Pile for Four
      * Game.flop_and_turn()
      * Four::from_turn()
      * REFACTORING: Move PlayerWins::seven_at_flop() to Seven::from_case_at_flop()
      * Seven::from_case_at_turn()
      * PlayerWins.case_evals_turn()
        * Calc.display_odds()
        * Game.five_at_turn()
      * PHASE 3.1: Outs
        * REFACTOR: possible_evals to evals
        * REFACTOR: Game.evals_at_turn() to .the_nuts_at_turn()
        * REFACTOR: HandRank struct to own mod
        * REFACTOR: All hand_rank mods to analysis
        * SPIKE: examples/outs.rs
          * REFACTOR: Move display odds methods to Game
          * REFACTOR: Move Game.player_wins_at_flop() to PlayerWins::at_flop()
          * REFACTOR: Move Game.player_wins_at_turn() to PlayerWins::at_turn()
        * Outs.touch()
        * Outs.add()
          * REFACTORING
        * Outs.len_for_player()
        * Outs.len_longest()
        * Outs.longest_player()
        * Outs.add_from_player_flag()
        * CaseEval adding case Cards field
          * .cards()
          * .card()
          * .cards_is_empty() & .cards_len()
        * Outs.add_from_case_eval()
        * Game.case_eval_at_turn()
          * Cards::from<&Card>
          * REFACTORING: Game.case_evals_at_turn() moved from PlayerWins.case_evals_flop()
            * REFACTORING: Game.turn_case_evals() & Game.turn_case_eval()
          * REFACTORING: All Game methods to phase (flop, turn, river) first naming
          * REFACTORING: Game turn and flop wins
            * CaseEvals.wins()
          * Game.turn_calculations()
          * Game.flop_calculations()
        * Game.turn_display_odds() check that turn is dealt
          * DEFECT: discrepancy with fudd
            * REFACTORING: Updated Results.player_to_string()
            * RESOLUTION: Game.turn_remaining()
            * REFACTORING: Game.turn_cards()
        * REFACTORING: Change to take in reference: Outs::from(&case_evals)
        * FINISHED Outs!
    * PHASE FOUR: Game.river_display_results()
      * TAKE 1: impl From<Board> for Five NOT NEEDED
      * TAKE 2: Seven::from_case_at_river() ABORT!
      * TAKE 3: HoleCards.river_case_eval()
        * Seven.from_case_and_board()
          * Meditation: Pass primitives by reference or by value?
            * TODO: Align around passing by reference or value for primitives.
        * Game.river_display_results()
          * CaseEval.winner()
            * REFACTORING to return PlayerFlag instead of a single winning index.
            * ERROR: Inverting winner call to use existing code
          * Game.river_case_eval()
    * DEFECT: Outs displayed wrong: `cargo run --example calc -- -d "A♠ K♥ 8♦ 6♣" -b "A♣ 8♥ 7♥ 9♠ 5♠" -n`
      * FIXED
    * TODO TD: Write real performance tests.
    * PHASE FIVE: Concurrency
      * REFACTOR: Turn
        * REFACTOR: Add .the_nuts() to Pile trait and have .evals() use it to return evals.
        * HARDENING: Pile trait .the_nuts() tests for arrays
          * HARDENING: Added getters to TheNuts against underlying evals array.
            * TheNuts.get()
      * REFACTOR: play/stages
        * Flop (renamed FlopEval)
          * .gen_case_evals()
          * CaseEval
            * .from_holdem_at_flop()
              * impl TryFrom<&[Card]> for Two
                * REFACTORING: impl TryFrom<Cards> for Two
                * Tests - Negative Boundary Conditions:
                  * .try_from__card_slice__empty_slice()
                  * .try_from__card_slice__one_card()
                  * .try_from__card_slice__three_cards()
                  * .try_from__card_slice__first_card_blank()
                    * BAD PASS!!! 😱
                    * impl TryFrom<Card> for Card
                      * FAIL
                    * TAKE TWO: Card::filter()
                    * PASS!
                  * .try_from__card_slice__second_card_blank()
                  * .try_from__card_slice__both_cards_blank()
                  * REFACTORING: rstest for blank tests
                  * REFACTORING PART DEUX: Mega Consolidation
                    * first add error param to existing three cases
                    * add remaining as failures (Don't change error case)
                    * Make em pass
                    * the cleanup
              * REFACTOR: sig to return Result
              * First test: happy path using The Hand
              * BUGFIX: Two::display() for blank cards
              * Finished tests with negative boundary conditions
          * CaseEvals
            * Two::from_from<Vec<Card>>
            * REFACTORING: Deleted Game.flop_case_eval() and flop_case_evals() moved to own structs
          * REFACTOR: Renamed FlopEval
          * REFACTOR: Eliminate Game.flop_display_the_nuts() move to Evals.display()
          * REFACTOR: Eliminate Game.display_evals()
          * Added test for CaseEvals::new()
          * impl TryFrom<Game> for FlopEval
            * Added PKError::NotDealt
          * Removed dupe of eval_for_player: eval_for_hand
          * Finished Display
        * TurnEval
      * Deconstructing Calc: DEALING WITH calc PERFORMANCE
        * Created examples/fixed and examples/long
          * log is going to work out in minute detail the flop calculations
          * Added Deck from fudd::types::poker_deck
        * VICTORY!!! CaseEvals::from_holdem_at_flop_mpsc()
          * finally figured out how to use concurrency to make this faster.
          * We got it down to under 1 second.
    * PHASE SIX: Pre flop DUN DUN DUNNNNNNN
      * 6.1 - Construct Bards from Card
      * 6.2 - Cards from Bard
        * 6.2.1 - Card try_from Bard
      * TODO: Store analysis Bard results in memory
  * Created [pkterm](https://github.com/ImperialBower/pkterm)
  * EPIC SEVEN: Transposition
    * Shift Suit trait
      * `impl SuitShift for Suit`
      * `impl SuitShift for Card`
      * `impl SuitShift for Two`
      * HeadsUp
        * Spike: Storage using [Sled](https://github.com/spacejam/sled/tree/main)
          * First relive BCM CSV fun
            * CSV without Bard. Pure Cards.
              * `SevenEval`: Need to store Evals with the original `Seven`.
                * Cards.remaining() (Trying to get only bcm repl example to work here)
                  * HA!!! Already exists in Pile trait. Wooo!!!!
                * Cards.into_twos() - Copying over bcrepl functionality
                  * Cards.divisible_by
              * examples/csv_card.rs
              * Serialize `Card` using symbols
              * analysis/store/heads_up/Row
  * EPIC _____: Bets
    * Added in sample data from pluribus, etc.
    *

## LATER

* improved hand hash <https://github.com/HenryRLee/PokerHandEvaluator/blob/master/Documentation/Algorithm.md>
  * split flushes out to only focus on rank brilliant

## Resources

* Poker
  * [Mike Caro's Website](https://www.poker1.com/)
  * GTO
    * [Minimum Defense Frequency vs Pot Odds](https://upswingpoker.com/minimum-defense-frequency-vs-pot-odds/)
  * Terms
    * [The Nuts](https://en.wikipedia.org/wiki/Nut_hand)
      * 888poker > [What is Nuts in Poker?](https://www.888poker.com/magazine/poker-terms/nuts)
      * GetMega > [Nuts in Poker](https://www.getmega.com/cards/poker/terms/nuts-in-poker/)
    * 888poker > [What is Texture in Poker?](https://www.888poker.com/magazine/poker-terms/texture)
  * Pluribus
    * [Superhuman AI for multiplayer poker](https://www.science.org/doi/10.1126/science.aay2400)
    * [pluribus-hand-parser](https://github.com/VitamintK/pluribus-hand-parser)
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

* [Serde](https://serde.rs/)
  * [Serde JSON](https://github.com/serde-rs/json)
  * [Serde YAML](https://github.com/dtolnay/serde-yaml)
* [Burnt Sushi CSV](https://github.com/BurntSushi/rust-csv) with Serde support

## Potential Libraries

* [mycelium-bitfield](https://crates.io/crates/mycelium-bitfield)
* [modular-bitfield](https://crates.io/crates/modular-bitfield)
