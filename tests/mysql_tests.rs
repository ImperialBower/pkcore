#[allow(non_snake_case)]
mod mysql_integration_tests {
    use mysql::PooledConn;
    use pkcore::analysis::store::db::mysql::{DbConnectOps, HeadsUpQuery, HeadsUpRawResult, MySqlDB};
    use pkcore::PKError;
    use std::str::FromStr;

    #[test]
    #[ignore]
    fn test_all() {
        match MySqlDB::get_connection() {
            Ok(pooled_connection) => {
                let mut conn: PooledConn = pooled_connection;
                let hurrs = HeadsUpRawResult::all_as_hup_results(&mut conn).unwrap();

                for hurr in hurrs {
                    println!("{hurr}");
                }
            }
            Err(e) => {
                println!("Unable to run integration test: {:?}", e);
            }
        }
    }

    #[test]
    fn test_query_existing_shu() {
        match MySqlDB::get_connection() {
            Ok(pooled_connection) => {
                let mut conn: PooledConn = pooled_connection;
                let huq = HeadsUpQuery::from_str("J♦ 9♠ 3♥ 2♠").unwrap();

                let result = huq.query(&mut conn);

                assert!(result.is_ok());
                println!("{}", result.unwrap());
            }
            Err(e) => {
                println!("Unable to run integration test: {:?}", e);
            }
        }
    }

    /// How it starts:
    /// ```rust
    /// let HeadsUpRawResult = HeadsUpRawResult { // Press ENTER
    /// };
    /// ```
    ///
    /// Copilot generates:
    ///
    /// ```rust
    /// let HeadsUpRawResult = HeadsUpRawResult {
    ///     higher: 70368748371968,
    ///     lower: 549890031616,
    ///     wins: 523851,
    ///     ties: 0,
    ///     losses: 1118235,
    /// };
    /// ```
    ///
    /// Notice the names for the fields don't always match.
    #[test]
    fn test_query_nonexisting_shu() {
        match MySqlDB::get_connection() {
            Ok(pooled_connection) => {
                let mut conn: PooledConn = pooled_connection;
                let huq = HeadsUpQuery::from_str("7♠ 2♦ 2♥ 2♠").unwrap();
                // let huq = HeadsUpQuery::from_str("T♠ 7♥ 7♠ 4♠").unwrap();

                let result = huq.query(&mut conn);

                assert!(!result.is_ok());
                assert_eq!(result.unwrap_err(), PKError::SqlEmptyResult);

                let _HeadsUpRawResult = HeadsUpRawResult {
                    higher: 17592186052608,
                    lower: 549822922752,
                    higher_wins: 523851,
                    lower_wins: 1118235,
                    ties: 70218,
                };

                // let huq = HeadsUpQuery::from_str("J♦ 9♠ 3♥ 2♠").unwrap();
                //
                // assert!(huq.exists(&mut conn));
            }
            Err(e) => {
                println!("Unable to run integration test: {:?}", e);
            }
        }
    }

    #[test]
    fn test_no_shu() {
        match MySqlDB::get_connection() {
            Ok(pooled_connection) => {
                let mut conn: PooledConn = pooled_connection;

                let huq = HeadsUpQuery::from_str("2♦ 3♠ 3♥ 2♠").unwrap();

                let result = huq.query(&mut conn);

                assert_eq!(result.unwrap_err(), PKError::SqlEmptyResult);
            }
            Err(e) => {
                println!("Unable to run integration test: {:?}", e);
            }
        }
    }

    #[test]
    fn test_exists() {
        match MySqlDB::get_connection() {
            Ok(pooled_connection) => {
                let mut conn: PooledConn = pooled_connection;

                let huq = HeadsUpQuery::from_str("J♦ 9♠ 3♥ 2♠").unwrap();

                assert!(huq.exists(&mut conn));
            }
            Err(e) => {
                println!("Unable to run integration test: {:?}", e);
            }
        }
    }
}
