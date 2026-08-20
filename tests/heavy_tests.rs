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
        let actual = HUPResult::from(&TestData::the_hand_sorted_headsup());

        assert_eq!(actual, TestData::the_hand_as_hup_result());
    }

    /// Trains CFR for 500k iterations and asserts exploitability converges below 0.001,
    /// confirming the average strategy reaches the analytical Nash equilibrium.
    #[test]
    #[ignore]
    fn kuhn_cfr__converges_to_nash_exploitability() {
        let mut cfr = KuhnCfr::new();
        cfr.train(500_000);
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
    /// Two limits are worth stating rather than hiding. Winners are not checked:
    /// a winner's payoff is the pot, not their own commitment. And hands whose
    /// last logged action ends the hand reach `end_hand`, which calls
    /// `Player::reset` and zeroes `chips_in_play` on every seat — those hands
    /// are skipped, because the evidence has already been cleared by the time
    /// the test can look. Roughly 18 500 losing seats across the corpus are
    /// still checked strictly.
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
                let nubi = match Nubificus::try_from(&plur) {
                    Ok(nubi) => nubi,
                    Err(e) => return Some(format!("Game #{idx}: {e}")),
                };
                if let Err(e) = nubi.play_hand() {
                    return Some(format!("Game #{idx}: {e}"));
                }

                // `end_hand` resets every seat, so a hand that finished has no
                // commitments left to compare against.
                let table_was_reset = (0..6).all(|n| {
                    nubi.table
                        .get_seat(n)
                        .map_or(true, |s| s.player.get_chips_in_play() == 0)
                });
                if table_was_reset {
                    return None;
                }

                for (seat_number, payoff) in plur.winnings.iter().enumerate() {
                    if *payoff >= 0 {
                        continue;
                    }
                    let seat_number = u8::try_from(seat_number).unwrap_or_default();
                    let committed = nubi
                        .table
                        .get_seat(seat_number)
                        .map_or(0, |seat| seat.player.get_chips_in_play());

                    if committed != payoff.unsigned_abs() {
                        return Some(format!(
                            "Game #{idx} seat {seat_number}: log says it lost {}, replay committed {committed}\n  {}",
                            payoff.unsigned_abs(),
                            plur.raw
                        ));
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
}
