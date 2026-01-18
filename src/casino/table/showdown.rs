use crate::analysis::case_eval::CaseEval;
use crate::PKError;
use crate::play::game::Game;
use crate::prelude::Table;

#[derive(Clone, Debug, Default)]
pub struct Showdown {
    pub case_eval: CaseEval,
}

impl CaseEval {

}

impl TryFrom<&Table> for Showdown {
    type Error = PKError;

    fn try_from(table: &Table) -> Result<Self, Self::Error> {
        if !table.is_game_over() {
            return Err(PKError::ActionIsntFinished);
        }

        let mut case_eval = CaseEval::default();

        if table.is_river() {
            case_eval = Game::try_from(table)?.river_case_eval()?;
        }

        Ok(Showdown { case_eval })
    }
}