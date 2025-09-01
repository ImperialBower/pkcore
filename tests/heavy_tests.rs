use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::util::data::TestData;
use pkcore::util::wincounter::win::Win;

#[allow(non_snake_case)]
mod heavy_tests {
    use super::*;

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
    #[test]
    #[ignore]
    fn hup_result__from__sorted_heads_up() {
        let actual = HUPResult::from(&TestData::the_hand_sorted_headsup());

        assert_eq!(actual, TestData::the_hand_as_hup_result());
    }
}
