#[allow(non_snake_case)]
mod mysql_integration_tests {
    use pkcore::analysis::store::db::mysql::{HeadsUpQuery, DB};
    use pkcore::PKError;
    use std::str::FromStr;

    #[test]
    fn test_existing_shu() {
        let mut conn = DB::get_connection().unwrap();
        let huq = HeadsUpQuery::from_str("J♦ 9♠ 3♥ 2♠").unwrap();

        let result = huq.query(&mut conn);

        assert!(result.is_ok());

        println!("{}", result.unwrap());
    }

    #[test]
    fn test_no_shu() {
        let mut conn = DB::get_connection().unwrap();
        let huq = HeadsUpQuery::from_str("2♦ 3♠ 3♥ 2♠").unwrap();

        let result = huq.query(&mut conn);

        assert_eq!(result.unwrap_err(), PKError::SqlEmptyResult);
    }
}
