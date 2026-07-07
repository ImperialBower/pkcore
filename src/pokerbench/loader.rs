//! CSV and JSON loaders for PokerBench scenarios.
//!
//! Both forms are reduced to the same [`PokerBenchScenario`]: the CSV path reads
//! the dataset's structured columns; the JSON path parses the natural-language
//! `instruction`. Fields PokerBench does not carry (per-position stacks, big
//! blind, explicit chips-to-call) are filled from documented conventions — see
//! [`seed_stacks`](crate::pokerbench::parse) usage and
//! [`PB_BIG_BLIND`](crate::pokerbench::PB_BIG_BLIND).

use crate::pokerbench::action::{PokerBenchAction, parse_chips};
use crate::pokerbench::error::PokerBenchError;
use crate::pokerbench::parse::{
    derive_to_call, extract_board, extract_holding, extract_position, extract_pot, parse_cards_concat, parse_history,
    parse_legal, parse_position, resolve_postflop_hero, seed_stacks,
};
use crate::pokerbench::scenario::{PB_BIG_BLIND, PokerBenchScenario, PokerBenchSplit};
use serde::Deserialize;
use std::path::Path;
use std::str::FromStr;

/// One row of the pre-flop CSV (only the columns we consume; extra columns such
/// as `num_players`/`num_bets` are ignored by the header-name mapping).
#[derive(Debug, Deserialize)]
struct PreflopRow {
    prev_line: String,
    hero_pos: String,
    hero_holding: String,
    correct_decision: String,
    available_moves: String,
    pot_size: String,
}

/// One row of the post-flop CSV (consumed columns only).
#[derive(Debug, Deserialize)]
struct PostflopRow {
    preflop_action: String,
    board_flop: String,
    board_turn: String,
    board_river: String,
    postflop_action: String,
    available_moves: String,
    pot_size: String,
    hero_position: String,
    holding: String,
    correct_decision: String,
}

/// One JSON item: the prompt prose plus the solver-optimal label.
#[derive(Debug, Deserialize)]
struct JsonItem {
    instruction: String,
    output: String,
}

impl PreflopRow {
    fn into_scenario(self) -> Result<PokerBenchScenario, PokerBenchError> {
        let history = parse_history(&self.prev_line);
        Ok(PokerBenchScenario {
            instruction: String::new(),
            hero: parse_position(&self.hero_pos)?,
            board: Vec::new(),
            hole: parse_cards_concat(&self.hero_holding)?,
            pot: parse_chips(&self.pot_size)?,
            to_call: derive_to_call(&history),
            big_blind: PB_BIG_BLIND,
            stacks: seed_stacks(),
            history,
            legal: parse_legal(&self.available_moves),
            optimal: PokerBenchAction::from_str(self.correct_decision.trim())?,
            split: PokerBenchSplit::Preflop,
        })
    }
}

impl PostflopRow {
    fn into_scenario(self) -> Result<PokerBenchScenario, PokerBenchError> {
        let mut board = parse_cards_concat(&self.board_flop)?;
        board.extend(parse_cards_concat(&self.board_turn)?);
        board.extend(parse_cards_concat(&self.board_river)?);
        let history = parse_history(&self.postflop_action);
        Ok(PokerBenchScenario {
            instruction: String::new(),
            hero: resolve_postflop_hero(&self.preflop_action, &self.hero_position)?,
            board,
            hole: parse_cards_concat(&self.holding)?,
            pot: parse_chips(&self.pot_size)?,
            to_call: derive_to_call(&history),
            big_blind: PB_BIG_BLIND,
            stacks: seed_stacks(),
            history,
            legal: parse_legal(&self.available_moves),
            optimal: PokerBenchAction::from_str(self.correct_decision.trim())?,
            split: PokerBenchSplit::Postflop,
        })
    }
}

impl JsonItem {
    fn into_scenario(self, split: PokerBenchSplit) -> Result<PokerBenchScenario, PokerBenchError> {
        let history = parse_history(&self.instruction);
        Ok(PokerBenchScenario {
            hero: extract_position(&self.instruction)?,
            board: extract_board(&self.instruction),
            hole: extract_holding(&self.instruction)?,
            pot: extract_pot(&self.instruction)?,
            to_call: derive_to_call(&history),
            big_blind: PB_BIG_BLIND,
            stacks: seed_stacks(),
            history,
            legal: Vec::new(),
            optimal: PokerBenchAction::from_str(self.output.trim())?,
            split,
            instruction: self.instruction,
        })
    }
}

impl PokerBenchScenario {
    /// Parses the structured CSV form of a split into scenarios.
    ///
    /// `split` selects the column schema (pre-flop vs post-flop). Stacks, big
    /// blind, and chips-to-call are filled from the documented conventions
    /// described on this module.
    ///
    /// # Errors
    /// Returns [`PokerBenchError`] if the file cannot be read, a row is
    /// malformed, or a card/position/label fails to parse.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use pkcore::pokerbench::{PokerBenchScenario, PokerBenchSplit};
    /// use std::path::Path;
    ///
    /// let path = format!("{}/src/pokerbench/fixtures/preflop.csv", env!("CARGO_MANIFEST_DIR"));
    /// let scenarios = PokerBenchScenario::load_csv(Path::new(&path), PokerBenchSplit::Preflop)?;
    /// assert!(!scenarios.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_csv(path: &Path, split: PokerBenchSplit) -> Result<Vec<Self>, PokerBenchError> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut scenarios = Vec::new();
        match split {
            PokerBenchSplit::Preflop => {
                for result in reader.deserialize::<PreflopRow>() {
                    scenarios.push(result?.into_scenario()?);
                }
            }
            PokerBenchSplit::Postflop => {
                for result in reader.deserialize::<PostflopRow>() {
                    scenarios.push(result?.into_scenario()?);
                }
            }
        }
        Ok(scenarios)
    }

    /// Parses the JSON (`instruction` + `output`) form of a split into scenarios
    /// by parsing the instruction prose for hero/hole/board/pot and the `output`
    /// label for the optimal action.
    ///
    /// # Errors
    /// Returns [`PokerBenchError`] if the file cannot be read, the JSON is
    /// malformed, or an instruction/label fails to parse.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use pkcore::pokerbench::{PokerBenchScenario, PokerBenchSplit};
    /// use std::path::Path;
    ///
    /// let path = format!("{}/src/pokerbench/fixtures/preflop.json", env!("CARGO_MANIFEST_DIR"));
    /// let scenarios = PokerBenchScenario::load_json(Path::new(&path), PokerBenchSplit::Preflop)?;
    /// assert!(!scenarios.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_json(path: &Path, split: PokerBenchSplit) -> Result<Vec<Self>, PokerBenchError> {
        let text = std::fs::read_to_string(path)?;
        let items: Vec<JsonItem> = serde_json::from_str(&text)?;
        items.into_iter().map(|item| item.into_scenario(split)).collect()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod pokerbench__tests {
    use super::*;
    use crate::casino::table_celled::position::Position;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/pokerbench/fixtures")
            .join(name)
    }

    #[test]
    fn load_csv_preflop_counts_and_parses() {
        let scenarios = PokerBenchScenario::load_csv(&fixture("preflop.csv"), PokerBenchSplit::Preflop).unwrap();
        assert_eq!(scenarios.len(), 2);
        assert_eq!(scenarios[0].hero, Position::SB);
        assert!(scenarios[0].board.is_empty());
        // Bare "3.0bb" label -> a raise to 3.
        assert_eq!(scenarios[0].optimal, PokerBenchAction::Raise(3));
        assert_eq!(scenarios[0].legal[0], PokerBenchAction::Raise(3));
    }

    #[test]
    fn load_csv_postflop_parses_board_and_resolves_hero() {
        let scenarios = PokerBenchScenario::load_csv(&fixture("postflop.csv"), PokerBenchSplit::Postflop).unwrap();
        assert_eq!(scenarios.len(), 2);
        // hero_position "IP" in a BTN-vs-BB pot resolves to the in-position BTN.
        assert_eq!(scenarios[0].hero, Position::BTN);
        assert_eq!(scenarios[0].board, parse_cards_concat("Th3s2d5d").unwrap());
        assert_eq!(scenarios[0].optimal, PokerBenchAction::Check);
        // Second hand: HJ vs BB, hero IP -> HJ; capitalized "Bet 18" label.
        assert_eq!(scenarios[1].hero, Position::HJ);
        assert_eq!(scenarios[1].optimal, PokerBenchAction::Bet(18));
    }

    #[test]
    fn load_csv_postflop_derives_to_call() {
        let scenarios = PokerBenchScenario::load_csv(&fixture("postflop.csv"), PokerBenchSplit::Postflop).unwrap();
        // Both hands are checked to the hero on the final street -> to_call 0.
        assert_eq!(scenarios[0].to_call, 0);
        assert_eq!(scenarios[1].to_call, 0);
    }

    #[test]
    fn load_json_preflop_parses_prose() {
        let scenarios = PokerBenchScenario::load_json(&fixture("preflop.json"), PokerBenchSplit::Preflop).unwrap();
        assert_eq!(scenarios.len(), 2);
        assert_eq!(scenarios[0].hero, Position::SB);
        assert!(!scenarios[0].instruction.is_empty());
    }

    #[test]
    fn csv_and_json_agree_on_structural_overlap() {
        // The same items in both forms must agree on hero/hole/board/pot/optimal.
        for (csv_name, json_name, split) in [
            ("preflop.csv", "preflop.json", PokerBenchSplit::Preflop),
            ("postflop.csv", "postflop.json", PokerBenchSplit::Postflop),
        ] {
            let csv = PokerBenchScenario::load_csv(&fixture(csv_name), split).unwrap();
            let json = PokerBenchScenario::load_json(&fixture(json_name), split).unwrap();
            assert_eq!(csv.len(), json.len());
            for (c, j) in csv.iter().zip(json.iter()) {
                assert_eq!(c.hero, j.hero, "hero mismatch");
                assert_eq!(c.hole, j.hole, "hole mismatch");
                assert_eq!(c.board, j.board, "board mismatch");
                assert_eq!(c.pot, j.pot, "pot mismatch");
                assert_eq!(c.optimal, j.optimal, "optimal mismatch");
            }
        }
    }

    #[test]
    fn load_csv_malformed_row_is_err() {
        let scenarios = PokerBenchScenario::load_csv(&fixture("malformed.csv"), PokerBenchSplit::Preflop);
        assert!(scenarios.is_err());
    }

    #[test]
    fn load_json_malformed_is_err() {
        let scenarios = PokerBenchScenario::load_json(&fixture("malformed.json"), PokerBenchSplit::Preflop);
        assert!(scenarios.is_err());
    }

    #[test]
    fn load_csv_missing_file_is_err() {
        let scenarios = PokerBenchScenario::load_csv(&fixture("does_not_exist.csv"), PokerBenchSplit::Preflop);
        assert!(scenarios.is_err());
    }
}
