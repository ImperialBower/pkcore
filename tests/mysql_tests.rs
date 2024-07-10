#[allow(non_snake_case)]
mod mysql_integration_tests {
    use mysql::prelude::Queryable;
    use pkcore::analysis::store::db::mysql::{HeadsUpQuery, HeadsUpRawResult, DB};
    use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
    use pkcore::arrays::two::Two;
    use pkcore::bard::Bard;
    use std::str::FromStr;

    const GOOD_HIGH_BARD: u64 = 70368748371968;
    const GOOD_LOW_BARD: u64 = 549890031616;

    fn good_sorted_heads_up() -> SortedHeadsUp {
        SortedHeadsUp::new(
            Two::try_from(Bard::from(GOOD_HIGH_BARD)).unwrap(),
            Two::try_from(Bard::from(GOOD_LOW_BARD)).unwrap(),
        )
    }

    #[test]
    fn test_existing_shu() {
        let mut conn = DB::get_connection().unwrap();
        let huq = HeadsUpQuery::from_str("J♦ 9♠ 3♥ 2♠").unwrap();

        // let query = conn.exec_map(
        //     "SELECT higher, lower, higher_wins, lower_wins, ties FROM nlh_headsup_result WHERE higher = ? AND lower = ?",
        //     (huq.higher, huq.lower),
        //     |(higher, lower, higher_wins, lower_wins, ties)| { HeadsUpRawResult {higher, lower, higher_wins, lower_wins, ties}},
        // ).unwrap();

        // let row = query.iter().next().unwrap();
        let result = huq.query(&mut conn).unwrap();

        // println!("{:?}", row);
        println!("{}", result);
    }
}
