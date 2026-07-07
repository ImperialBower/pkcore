use crate::analysis::case_eval::CaseEval;
use crate::prelude::TableLog;

/// Represents the result from a hand played at the Table.
///
/// # DIARY
///
/// I wanted to see how good `CoPi` would be if I asked it to create the struct. Here's my prompt:
///
/// > I need a struct that returns the result from the hand played at the Table. It should include
/// > the `CaseEval` and `Results` table log.
///
/// Here's what it created:
///
/// ```txt
/// use crate::casino::case_eval::CaseEval;
/// use crate::casino::table::log::Results;
///
/// /// Represents the result from a hand played at the Table
/// #[derive(Debug, Clone, PartialEq)]
/// pub struct Result {
///     /// The case evaluation for the hand
///     pub case_eval: CaseEval,
///     /// The results log from the table
///     pub results: Results,
/// }
///
/// impl Result {
///     /// Creates a new Result with the given case evaluation and results log
///     pub fn new(case_eval: CaseEval, results: Results) -> Self {
///         Self {
///             case_eval,
///             results,
///         }
///     }
/// }
/// ```
///
/// OK, some hits and some misses. None of the structs it added to it exist. For `CaseEval` it got
/// the path wrong. And for `Results`, that struct doesn't even exist. Now, to be fair, I did say
/// `Results` instead of results, implying that there was a struct, instead of the `Vec<TableLog>`.
#[derive(Debug, Clone, PartialEq)]
pub struct HandResult {
    /// The case evaluation for the hand
    pub case_eval: CaseEval,
    /// The results log from the table
    pub log: TableLog,
}

impl HandResult {
    /// Creates a new Result with the given case evaluation and results log
    pub fn new(case_eval: CaseEval, log: TableLog) -> Self {
        Self { case_eval, log }
    }
}

impl std::fmt::Display for HandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Hand Result:")?;
        writeln!(f, "{}", self.case_eval)?;
        // writeln!(f, "\nTable Log:")?;
        // let log = self.log.clone();
        // for (i, entry) in log.iter().enumerate() {
        //     writeln!(f, "  {}: {}", i + 1, entry)?;
        // }
        Ok(())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table_celled__result_tests {
    // use super::*;
    // use crate::prelude::*;
}
