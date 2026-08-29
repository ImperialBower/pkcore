#[cfg(feature = "store")]
use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::games::kuhn::KuhnCfr;
#[cfg(feature = "store")]
use pkcore::util::data::TestData;

#[allow(non_snake_case)]
mod heavy_tests {
    use super::*;
    #[cfg(feature = "store")]
    use wincounter::win::Win;

    /// Wow, this test caused a panic:
    ///
    /// ```
    /// use pkcore::util::data::TestData;
    /// assert_eq!(TestData::the_hand_sorted_headsup().wins(), TestData::the_hand_as_wins());
    /// ```
    ///
    /// Let's try it a different way...
    ///
    /// ```
    /// use pkcore::util::data::TestData;
    /// use pkcore::util::wincounter::win::Win;
    /// assert_eq!(
    ///     TestData::the_hand_sorted_headsup().wins().wins_for(Win::FIRST),
    ///     TestData::the_hand_as_wins().wins_for(Win::FIRST)
    /// );
    /// ```
    ///
    /// Let's leave this test to fail for now, just so we don't forget it.
    ///
    /// I guess we could refactor our HUPResult:from `SortedHeadsUp`, but
    /// honestly, I don't care right now. Let's flag this as technical debt
    /// and ignore it for now. We've got bigger fish to fry. __Do vegans get
    /// mad by this phrase? Should it be, we've got bigger blocks of tofu to
    /// fry?__
    ///
    /// Now that we're at a point where we can really get down to business,
    /// let's take the time to make this test really work, so we can rest
    /// easy and get on with things.
    ///
    /// I like this refactoring. SortedHeadsUp owns it's wins and HUPResult passed them into
    /// something it can store.
    ///
    /// This takes five minutes to run. If it fails, I am royally fracked.
    ///
    /// Luckily it passed. 🎉
    ///
    /// Now, a test of the same data against `impl From<&SortedHeadsUp> for HUPResult`.
    #[cfg(feature = "store")]
    #[test]
    #[ignore]
    fn sorted_heads_up__wins() {
        let expected = TestData::the_hand_as_wins();
        let (higher_expected, higher_expected_ties) = expected.wins_for(Win::FIRST);
        let (lower_expected, lower_expected_ties) = expected.wins_for(Win::SECOND);

        let actual = TestData::the_hand_sorted_headsup().wins().unwrap();
        let (higher_wins, higher_ties) = actual.wins_for(Win::FIRST);
        let (lower_wins, lower_ties) = actual.wins_for(Win::SECOND);

        assert_eq!(higher_ties, lower_ties);
        assert_eq!(higher_expected_ties, lower_expected_ties);
        assert_eq!(higher_wins, higher_expected);
        assert_eq!(lower_wins, lower_expected);
        assert_eq!(higher_ties, higher_expected_ties);
    }

    /// This is going to be a very very heavy test, since we will need to load our
    /// 4GB binary bard map cache into memory before we can even do the calculation.
    /// Once we get it to pass, we can ignore it, and punch it into an example to run.
    ///
    /// Fudge! The test fails.
    ///
    /// ```txt
    /// Left:  HUPResult { higher: Bard(8797166764032), lower: Bard(65544), higher_wins: 1397400, lower_wins: 347020, ties: 32116 }
    /// Right: HUPResult { higher: Bard(8797166764032), lower: Bard(65544), higher_wins: 1365284, lower_wins: 314904, ties: 32116 }
    /// ```
    ///
    /// So, let's see what the difference is.
    ///
    /// ```txt
    /// 1397400 - 1365284 = 32116
    /// 347020 - 314904 = 32116
    /// ```
    ///
    /// **Smacks forehead.** Our old bcrepl subtracts the ties from the wins entries. That explains
    /// that. I could try to consolidate the code, but right now I just want to start getting results
    /// into sqlite.
    ///
    /// This time for sure!
    ///
    /// Subtracting times from each wins makes the test pass. Now, we're going to lock it in the
    /// vault with an ignore.
    #[cfg(feature = "store")]
    #[test]
    #[ignore]
    fn hup_result__from__sorted_heads_up() {
        let actual = HUPResult::try_from(&TestData::the_hand_sorted_headsup()).unwrap();

        assert_eq!(actual, TestData::the_hand_as_hup_result());
    }

    /// Trains CFR for 500k iterations and asserts exploitability converges below 0.001,
    /// confirming the average strategy reaches the analytical Nash equilibrium.
    #[test]
    #[ignore]
    fn kuhn_cfr__converges_to_nash_exploitability() {
        let mut cfr = KuhnCfr::new();
        cfr.train(500_000).unwrap();
        let exploit = cfr.exploitability().abs();
        assert!(exploit < 0.001, "exploitability after 500k iters: {exploit}");
    }

    /// Replays all 10,000 Pluribus game logs in parallel and asserts every hand
    /// completes without error. Produces no output on success; on failure prints
    /// each failing game index and its error before panicking.
    ///
    /// "No error" is a weak claim on its own — a replay that hands actions to the
    /// wrong seats finishes cleanly and is still wrong, which is exactly how
    /// `DEFECT_022` survived. So each losing seat's committed chips are also
    /// compared against the payoff the log records for it: a player who folds
    /// loses precisely what they put in, so the two must agree to the chip.
    ///
    /// EPIC-83 split the check in two, because the plain engine clears more
    /// state than the celled one did and the single old check went dead:
    ///
    /// - **Hands the replay finished.** `end_hand` has run, so every seat's
    ///   final stack is compared against `STARTING_STACK + payoff` — winners
    ///   included. This is the stronger of the two, and it covers exactly the
    ///   hands the celled-era check used to skip.
    /// - **Hands the replay left unfinished.** A log that ends `...r10000c///`
    ///   records an all-in and a call and then stops: there are no further
    ///   actions to drive the board out, so `Nubificus` never reaches
    ///   `end_hand` and no pot is awarded. Final stacks say nothing there, so
    ///   the older check still applies — each losing seat's committed chips
    ///   must equal the loss the log records for it.
    ///
    /// One tolerance is deliberate. The corpus records split pots to half a
    /// chip (`...|287.5|...`); the parser truncates to `isize`, so a finished
    /// hand whose raw payoff field carries a decimal point is allowed to land
    /// one chip either side of the parsed figure. That is the log's rounding,
    /// not the replay's.
    #[test]
    // #[ignore]
    fn pluribus__all_games_replay_without_errors() {
        use pkcore::analysis::nubibus::Pluribus;
        use pkcore::prelude::Nubificus;
        use rayon::prelude::*;

        let logs = Nubificus::get_log_files("data/pluribus/raw/").expect("failed to load log files");

        let all_games: Vec<Pluribus> = logs
            .iter()
            .flat_map(|log| Pluribus::read_in_log(log.as_str()).expect("failed to parse log file"))
            .collect();

        let errors: Vec<String> = all_games
            .into_par_iter()
            .enumerate()
            .filter_map(|(idx, plur)| {
                let mut nubi = match Nubificus::try_from(&plur) {
                    Ok(nubi) => nubi,
                    Err(e) => return Some(format!("Game #{idx}: {e}")),
                };
                if let Err(e) = nubi.play_hand() {
                    return Some(format!("Game #{idx}: {e}"));
                }

                // `end_hand` calls `Player::reset`, which zeroes
                // `chips_in_play` on every seat. So an all-zero ring means the
                // hand resolved; anything left in play means the replay ran
                // out of logged actions mid-hand.
                let hand_resolved = nubi
                    .table
                    .seats
                    .iter()
                    .all(|seat| seat.player.chips_in_play == 0);

                // Field 4 of the raw record is the payoff list. A decimal
                // point there means a split pot the parser had to truncate.
                let payoffs_are_rounded = plur.raw.split(':').nth(4).is_some_and(|field| field.contains('.'));

                for (seat_number, payoff) in plur.winnings.iter().enumerate() {
                    let seat_number = u8::try_from(seat_number).unwrap_or_default();
                    let Some(seat) = nubi.table.seats.get_seat(seat_number) else {
                        continue;
                    };

                    if hand_resolved {
                        let expected = isize::try_from(Pluribus::STARTING_STACK).unwrap_or_default() + payoff;
                        let actual = isize::try_from(seat.player.chips).unwrap_or_default();
                        let slack = isize::from(payoffs_are_rounded);

                        if (actual - expected).abs() > slack {
                            return Some(format!(
                                "Game #{idx} seat {seat_number}: log says it ends on {expected}, replay ended on {actual}\n  {}",
                                plur.raw
                            ));
                        }
                    } else if *payoff < 0 {
                        let committed = seat.player.chips_in_play;

                        if committed != payoff.unsigned_abs() {
                            return Some(format!(
                                "Game #{idx} seat {seat_number}: log says it lost {}, replay committed {committed}\n  {}",
                                payoff.unsigned_abs(),
                                plur.raw
                            ));
                        }
                    }
                }

                None
            })
            .collect();

        assert!(
            errors.is_empty(),
            "{} game(s) failed:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }

    // ── EPIC-87: Pluribus-format export ───────────────────────────────────

    /// The eight corpus hands that split a pot to half a chip.
    ///
    /// `Pluribus.winnings` is `Vec<isize>` in whole chips and `parse_isizes`
    /// truncates `112.5` to `112`, so these eight cannot round-trip byte for
    /// byte. EPIC-87 took Design option 3 in the open: keep the public field's
    /// units, and name the exclusions here rather than hide them behind a
    /// silent filter on "any line containing a dot".
    const HALF_CHIP_HANDS: [(&str, usize); 8] = [
        ("sample_game_102.log", 0),
        ("sample_game_32.log", 23),
        ("sample_game_41b.log", 204),
        ("sample_game_60.log", 88),
        ("sample_game_75b.log", 76),
        ("sample_game_88.log", 128),
        ("sample_game_91.log", 43),
        ("sample_game_91.log", 53),
    ];

    /// Re-orders each player's two hole cards high-to-low **in the raw text**.
    ///
    /// `Two` normalizes its two cards on construction, because `As8s` and
    /// `8sAs` are the same hand and must compare equal — so a writer built on
    /// it cannot reproduce the logged order, and 98.4% of corpus hands log at
    /// least one player low-card-first. The comparison is therefore against a
    /// line pkcore could actually have produced.
    ///
    /// Built by string surgery over the original rather than by rendering the
    /// parsed form: an oracle that went through the writer would agree with
    /// the writer no matter how wrong the writer was.
    fn canonicalize(line: &str) -> String {
        use pkcore::prelude::Card;
        use std::str::FromStr;

        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() != 6 {
            return line.to_string();
        }

        let (dealt, board) = match fields[3].split_once('/') {
            Some((dealt, board)) => (dealt, Some(board)),
            None => (fields[3], None),
        };

        let sorted: Vec<String> = dealt
            .split('|')
            .map(|pair| {
                if pair.len() != 4 {
                    return pair.to_string();
                }
                let (first, second) = pair.split_at(2);
                match (Card::from_str(first), Card::from_str(second)) {
                    (Ok(one), Ok(two)) if two > one => format!("{second}{first}"),
                    _ => pair.to_string(),
                }
            })
            .collect();

        let cards = match board {
            Some(board) => format!("{}/{}", sorted.join("|"), board),
            None => sorted.join("|"),
        };

        format!(
            "{}:{}:{}:{}:{}:{}",
            fields[0], fields[1], fields[2], cards, fields[4], fields[5]
        )
    }

    /// EPIC-87 Tier 1: every logged hand, parsed and written straight back
    /// out, byte for byte.
    ///
    /// Expected to pass on **9,992 of 10,000**. The eight exclusions are named
    /// in [`HALF_CHIP_HANDS`], and the count is asserted rather than the
    /// boolean, so a regression shows up as a number that went up.
    #[test]
    fn pluribus__corpus_round_trips_byte_exact() {
        use pkcore::analysis::nubibus::Pluribus;
        use pkcore::prelude::{Nubificus, Unumable};
        use std::str::FromStr;

        let logs = Nubificus::get_log_files("data/pluribus/raw/").expect("failed to load log files");
        let mut hands = 0;
        let mut failures: Vec<String> = Vec::new();

        for log in &logs {
            let name = log.rsplit('/').next().unwrap_or(log).to_string();
            for line in std::fs::read_to_string(log).expect("unreadable log").lines() {
                if !line.starts_with("STATE:") {
                    continue;
                }
                let Ok(hand) = Pluribus::from_str(line) else {
                    continue;
                };
                hands += 1;

                if HALF_CHIP_HANDS.contains(&(name.as_str(), hand.index)) {
                    continue;
                }

                let rendered = hand.to_pluribus();
                if rendered != canonicalize(line) {
                    failures.push(format!("{name} #{}\n  in : {line}\n  out: {rendered}", hand.index));
                }
            }
        }

        assert_eq!(hands, 10_000, "the corpus changed size");
        assert!(
            failures.is_empty(),
            "{} of {} hands did not round trip:\n{}",
            failures.len(),
            hands,
            failures.iter().take(10).cloned().collect::<Vec<_>>().join("\n")
        );
    }

    /// EPIC-87 Phase 3d: the divider theory, settled.
    ///
    /// `Pluribus::divider_hypothesis` places the `/` from the flat action
    /// sequence and the player count alone — no cards, no table, no replay.
    /// `Pluribus::actions_to_pluribus` places them by replaying the hand
    /// through a `Table`, which is correct by construction. If the two agree
    /// everywhere, the dividers are redundant, exactly as the note at
    /// `Pluribus::parse_all_rounds` guessed.
    ///
    /// They agree on all 10,000.
    #[test]
    fn pluribus__divider_hypothesis_matches_the_replay() {
        use pkcore::analysis::nubibus::{Pluribus, PluribusEvent};
        use pkcore::prelude::Nubificus;

        let logs = Nubificus::get_log_files("data/pluribus/raw/").expect("failed to load log files");
        let mut checked = 0;
        let mut disagreements: Vec<String> = Vec::new();

        for log in &logs {
            for hand in Pluribus::read_in_log(log.as_str()).expect("failed to parse log file") {
                let events: Vec<PluribusEvent> = hand.actions.iter().copied().collect();
                let replayed = hand.actions_to_pluribus().expect("replay failed");
                checked += 1;

                match Pluribus::divider_hypothesis(&events, hand.players.len()) {
                    Some(guessed) if guessed == replayed => {}
                    Some(guessed) => disagreements.push(format!(
                        "  replay: {replayed}\n  guess : {guessed}\n  raw   : {}",
                        hand.raw
                    )),
                    None => disagreements.push(format!("  no answer for: {}", hand.raw)),
                }
            }
        }

        assert_eq!(checked, 10_000, "the corpus changed size");
        assert!(
            disagreements.is_empty(),
            "the divider theory failed on {} of {} hands:\n{}",
            disagreements.len(),
            checked,
            disagreements.iter().take(10).cloned().collect::<Vec<_>>().join("\n")
        );
    }

    /// EPIC-87 Tier 2: replay each logged hand through the engine, then ask
    /// the finished table to write the line back out.
    ///
    /// This is the tier that can catch a `DEFECT_021`-shaped bug in mirror
    /// image — the cumulative-amount conversion run backwards.
    ///
    /// Two classes of hand are excluded, and between them they account for
    /// **every** failure — there is no unexplained residue:
    ///
    /// 1. The eight [`HALF_CHIP_HANDS`].
    /// 2. **92 all-in run-outs the engine cannot finish** (91 counted here —
    ///    the 92nd is also a half-chip hand and is excluded before the check).
    ///    When every
    ///    remaining player is all-in, `Table` deals one more street and then
    ///    stalls: `is_game_over` wants `is_last_street`, the board never
    ///    reaches five cards, and the pot is never awarded. That is a `Table`
    ///    state-machine gap, not an exporter bug, and EPIC-87's Tier 2 is the
    ///    first thing that ever asked the engine to run a board out. Detected
    ///    here by chip conservation — a hand that actually finished pays out
    ///    exactly what it took in, so the net column sums to zero.
    ///
    /// The flop comes back in canonical order because `DealtFlop` carries a
    /// single `Bard`, which is a bitset; see `TryFrom<&Table> for Pluribus`.
    #[test]
    fn pluribus__corpus_replays_and_re_exports() {
        use pkcore::analysis::nubibus::Pluribus;
        use pkcore::bard::Bard;
        use pkcore::prelude::{Card, Nubificus, Unumable};
        use std::str::FromStr;

        /// [`canonicalize`] plus the flop in `Bard::DECK` order — highest bit
        /// first, so spades, then hearts, diamonds, clubs, descending by rank
        /// within each suit.
        fn canonicalize_with_flop(line: &str) -> String {
            let canonical = canonicalize(line);
            let fields: Vec<&str> = canonical.split(':').collect();
            if fields.len() != 6 {
                return canonical;
            }
            let Some((dealt, board)) = fields[3].split_once('/') else {
                return canonical;
            };

            let mut streets: Vec<String> = board.split('/').map(str::to_string).collect();
            if let Some(flop) = streets.first_mut()
                && flop.len() == 6
            {
                let mut cards: Vec<String> = (0..3).map(|i| flop[i * 2..i * 2 + 2].to_string()).collect();
                cards.sort_by_key(|card| {
                    std::cmp::Reverse(
                        Card::from_str(card)
                            .map(|card| Bard::from(card).as_u64())
                            .unwrap_or_default(),
                    )
                });
                *flop = cards.concat();
            }

            format!(
                "{}:{}:{}:{}/{}:{}:{}",
                fields[0],
                fields[1],
                fields[2],
                dealt,
                streets.join("/"),
                fields[4],
                fields[5]
            )
        }

        let logs = Nubificus::get_log_files("data/pluribus/raw/").expect("failed to load log files");
        let mut hands = 0;
        let mut stalled = 0;
        let mut failures: Vec<String> = Vec::new();

        for log in &logs {
            let name = log.rsplit('/').next().unwrap_or(log).to_string();
            for line in std::fs::read_to_string(log).expect("unreadable log").lines() {
                if !line.starts_with("STATE:") {
                    continue;
                }
                let Ok(hand) = Pluribus::from_str(line) else {
                    continue;
                };
                hands += 1;

                if HALF_CHIP_HANDS.contains(&(name.as_str(), hand.index)) {
                    continue;
                }

                let mut nubificus = Nubificus::try_from(&hand).expect("rebuild failed");
                nubificus.play_hand().expect("replay failed");

                let mut exported = Pluribus::try_from(&nubificus.table).expect("export failed");
                exported.index = hand.index;

                // Chip conservation: a non-zero sum means the pot was never
                // awarded, which is the all-in run-out the engine cannot
                // finish. Counted, not silently skipped.
                if exported.winnings.iter().sum::<isize>() != 0 {
                    stalled += 1;
                    continue;
                }

                let rendered = exported.to_pluribus();
                if rendered != canonicalize_with_flop(line) {
                    failures.push(format!("{name} #{}\n  in : {line}\n  out: {rendered}", hand.index));
                }
            }
        }

        assert_eq!(hands, 10_000, "the corpus changed size");
        assert!(
            failures.is_empty(),
            "{} of {} hands did not survive the replay round trip:\n{}",
            failures.len(),
            hands,
            failures.iter().take(10).cloned().collect::<Vec<_>>().join("\n")
        );
        assert_eq!(
            stalled, 91,
            "the number of hands the engine cannot run out changed; if this went \
             down, the all-in run-out gap is being fixed and this test should \
             tighten with it"
        );
    }
}
