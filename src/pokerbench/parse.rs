//! Shared parsing helpers for the PokerBench loaders.
//!
//! These turn the dataset's two surface forms — structured CSV columns and the
//! natural-language JSON `instruction` — into the same [`PokerBenchScenario`]
//! fields. The CSV helpers parse compact tokens (`"AhKs"`, `"UTG"`, `"raise 6"`);
//! the prose helpers (`extract_*`) pull the same facts out of the long
//! instruction text using its fixed phrasing.

use crate::card::Card;
use crate::casino::table::position::Position;
use crate::pokerbench::action::{PokerBenchAction, parse_chips};
use crate::pokerbench::error::PokerBenchError;
use std::str::FromStr;

/// Parses a PokerBench position code (`UTG`/`HJ`/`CO`/`BTN`/`SB`/`BB`,
/// case-insensitive) into a pkcore [`Position`].
///
/// # Errors
/// Returns [`PokerBenchError::Position`] for an unrecognized code.
pub(crate) fn parse_position(token: &str) -> Result<Position, PokerBenchError> {
    match token.trim().to_uppercase().as_str() {
        "UTG" => Ok(Position::UTG),
        "HJ" => Ok(Position::HJ),
        "CO" => Ok(Position::CO),
        "BTN" | "BU" | "BTN/D" => Ok(Position::BTN),
        "SB" => Ok(Position::SB),
        "BB" => Ok(Position::BB),
        other => Err(PokerBenchError::Position(other.to_string())),
    }
}

/// Parses a compact card list — concatenated (`"AhKs"`) or whitespace/comma
/// separated (`"Ks 7h 2d"`) two-character cards. An empty/whitespace input
/// yields an empty vec.
///
/// # Errors
/// Returns [`PokerBenchError::Card`] if the cleaned input has an odd character
/// count or a two-character chunk is not a valid card.
pub(crate) fn parse_cards_concat(input: &str) -> Result<Vec<Card>, PokerBenchError> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace() && *c != ',').collect();
    if cleaned.is_empty() {
        return Ok(Vec::new());
    }
    if !cleaned.len().is_multiple_of(2) {
        return Err(PokerBenchError::Card(input.to_string()));
    }
    let bytes = cleaned.as_bytes();
    let mut cards = Vec::with_capacity(cleaned.len() / 2);
    let mut idx = 0;
    while idx < bytes.len() {
        let token = &cleaned[idx..idx + 2];
        let card = Card::from_str(token).map_err(|_| PokerBenchError::Card(token.to_string()))?;
        cards.push(card);
        idx += 2;
    }
    Ok(cards)
}

/// Parses a list of legal moves, leniently. Tokens are split on `;` or `,`;
/// each is parsed as a [`PokerBenchAction`], with bare `bet`/`raise` (size open)
/// Unparsable tokens are skipped — `legal` is informational, never a hard
/// failure.
pub(crate) fn parse_legal(input: &str) -> Vec<PokerBenchAction> {
    input
        .trim()
        .trim_matches(['[', ']'])
        .split(',')
        .filter_map(|raw| {
            let token = raw.trim().trim_matches(['\'', '"', ' ']);
            if token.is_empty() {
                return None;
            }
            PokerBenchAction::from_str(token).ok()
        })
        .collect()
}

/// Parses one token of an action line into an action, across PokerBench's three
/// surface forms: the `/`-separated CSV pre-flop line (`"BTN"`, `"2.0bb"`,
/// `"call"`), the post-flop line (`"OOP_BET_5"`, `"IP_CALL"`, `"dealcards"`),
/// and the JSON instruction prose (`"BB bet 4 chips"`). Position names, dealt
/// cards, and board prose yield `None`.
fn parse_action_token(raw: &str) -> Option<PokerBenchAction> {
    // Normalize: drop list punctuation, lowercase, fold OOP_/IP_ prefixes and
    // `_` separators to spaces, and strip the "chips" noise word.
    let cleaned = raw
        .trim()
        .trim_matches(['[', ']', '\'', '"', '.'])
        .to_lowercase()
        .replace("oop_", "")
        .replace("ip_", "")
        .replace('_', " ")
        .replace("chips", "");
    let segment = cleaned.trim();
    if segment.is_empty() || segment == "dealcards" {
        return None;
    }
    // A bare position code is not an action.
    if !segment.chars().any(|c| c.is_ascii_digit()) && parse_position(segment).is_ok() {
        return None;
    }
    // Try progressively shorter trailing token-spans so a leading position/name
    // (`"bb bet 4"`) is tolerated while card phrases fall through to `None`.
    let words: Vec<&str> = segment.split_whitespace().collect();
    (0..words.len()).find_map(|start| PokerBenchAction::from_str(&words[start..].join(" ")).ok())
}

/// Extracts an ordered action history from any of the three action-line forms,
/// splitting on token/clause boundaries and keeping the segments that parse as
/// actions.
pub(crate) fn parse_history(text: &str) -> Vec<PokerBenchAction> {
    // Note: split on ". " (sentence boundary), never a bare '.', so decimal
    // amounts like "2.0bb" survive intact.
    text.split(['/', ','])
        .flat_map(|clause| clause.split(" and "))
        .flat_map(|clause| clause.split(" then "))
        .flat_map(|clause| clause.split(". "))
        .filter_map(parse_action_token)
        .collect()
}

/// Derives the chips the hero must call from the parsed action history.
///
/// Approximation (a Phase-1 covariate, not a scored quantity): the size of the
/// bet/raise the hero is currently facing — `0` if the last action is a check
/// or the history is empty, the last bet/raise size if that is the last action,
/// otherwise the most recent bet/raise level still standing behind a call/fold.
/// Ignores the hero's already-posted chips (e.g. a blind), so it can overstate
/// by up to one bet.
pub(crate) fn derive_to_call(history: &[PokerBenchAction]) -> u32 {
    match history.last() {
        None | Some(PokerBenchAction::Check) => 0,
        Some(PokerBenchAction::Bet(n) | PokerBenchAction::Raise(n)) => *n,
        Some(PokerBenchAction::Call | PokerBenchAction::Fold | PokerBenchAction::AllIn) => {
            history.iter().rev().find_map(|action| action.size()).unwrap_or(0)
        }
    }
}

/// Post-flop action order rank: lower acts earlier (more out of position).
fn postflop_order_rank(position: Position) -> u8 {
    match position {
        Position::SB => 0,
        Position::BB => 1,
        Position::UTG => 2,
        Position::UTGP1 => 3,
        Position::UTGP2 => 4,
        Position::EP => 5,
        Position::MP => 6,
        Position::LJ => 7,
        Position::HJ => 8,
        Position::CO => 9,
        Position::BTN => 10,
    }
}

/// Resolves the hero's absolute table position for a (heads-up) post-flop item.
///
/// The post-flop CSV labels the hero relatively (`IP`/`OOP`); the two table
/// positions live in `preflop_action`. The earlier post-flop actor is `OOP`,
/// the later is `IP`. A `hero_rel` that is already an absolute code is accepted
/// as a fallback.
///
/// # Errors
/// Returns [`PokerBenchError::Position`] if `preflop_action` does not yield
/// exactly two positions, or `hero_rel` is neither `IP`/`OOP` nor a known code.
pub(crate) fn resolve_postflop_hero(preflop_action: &str, hero_rel: &str) -> Result<Position, PokerBenchError> {
    let mut positions: Vec<Position> = Vec::new();
    for token in preflop_action.split('/') {
        if let Ok(position) = parse_position(token)
            && !positions.contains(&position)
        {
            positions.push(position);
        }
    }
    if positions.len() != 2 {
        return Err(PokerBenchError::Position(format!(
            "expected 2 post-flop positions in {preflop_action:?}, found {positions:?}"
        )));
    }
    positions.sort_by_key(|p| postflop_order_rank(*p));
    match hero_rel.trim().to_uppercase().as_str() {
        "OOP" => Ok(positions[0]),
        "IP" => Ok(positions[1]),
        other => parse_position(other),
    }
}

/// Seeds per-position stacks for a full 6-handed table, each at
/// [`PB_EFFECTIVE_STACK`](crate::pokerbench::PB_EFFECTIVE_STACK). PokerBench
/// carries no stacks; this is the documented 100 bb 6-max convention.
pub(crate) fn seed_stacks() -> Vec<(Position, u32)> {
    use crate::pokerbench::scenario::PB_EFFECTIVE_STACK;
    [
        Position::UTG,
        Position::HJ,
        Position::CO,
        Position::BTN,
        Position::SB,
        Position::BB,
    ]
    .into_iter()
    .map(|p| (p, PB_EFFECTIVE_STACK))
    .collect()
}

// --- Instruction-prose helpers (JSON form) ---------------------------------

/// Returns the substring immediately following `marker`, up to (excluding) the
/// first character not allowed by `keep`.
fn slice_after<'a>(text: &'a str, marker: &str, keep: impl Fn(char) -> bool) -> Option<&'a str> {
    let start = text.find(marker)? + marker.len();
    let rest = &text[start..];
    let end = rest.find(|c: char| !keep(c)).unwrap_or(rest.len());
    Some(rest[..end].trim_end())
}

/// Maps a full rank word (`"king"`) to its card-rank letter.
fn rank_letter(word: &str) -> Option<char> {
    Some(match word.trim().to_lowercase().as_str() {
        "ace" => 'A',
        "king" => 'K',
        "queen" => 'Q',
        "jack" => 'J',
        "ten" => 'T',
        "nine" => '9',
        "eight" => '8',
        "seven" => '7',
        "six" => '6',
        "five" => '5',
        "four" => '4',
        "three" => '3',
        "two" => '2',
        _ => return None,
    })
}

/// Maps a full suit word (`"spade"`, `"diamonds"`) to its card-suit letter.
fn suit_letter(word: &str) -> Option<char> {
    let normalized = word.trim().to_lowercase();
    let singular = normalized.strip_suffix('s').unwrap_or(&normalized);
    Some(match singular {
        "spade" => 's',
        "heart" => 'h',
        "diamond" => 'd',
        "club" => 'c',
        _ => return None,
    })
}

/// Parses a spelled-out card phrase (`"King of Diamond"`) into a [`Card`].
/// Returns `None` if the rank or suit word is unrecognized.
fn card_from_phrase(phrase: &str) -> Option<Card> {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    if words.len() < 2 {
        return None;
    }
    let rank = rank_letter(words[0])?;
    let suit = suit_letter(words[words.len() - 1])?;
    Card::from_str(&format!("{rank}{suit}")).ok()
}

/// Parses a comma/`and`-separated list of spelled-out card phrases, stopping at
/// the first phrase that is not a card (so trailing action prose like
/// `", then BB check"` is dropped).
fn cards_from_phrase_list(list: &str) -> Vec<Card> {
    let mut cards = Vec::new();
    for raw in list.split(',') {
        let phrase = raw.trim().trim_start_matches("and ").trim();
        match card_from_phrase(phrase) {
            Some(card) => cards.push(card),
            None => break,
        }
    }
    cards
}

/// Extracts the hero position from an instruction (`"your position is HJ,"`).
pub(crate) fn extract_position(instruction: &str) -> Result<Position, PokerBenchError> {
    let token = slice_after(instruction, "position is ", |c| c.is_ascii_alphanumeric())
        .ok_or_else(|| PokerBenchError::MissingField("position".to_string()))?;
    parse_position(token)
}

/// Extracts the hero hole cards from an instruction
/// (`"holding is [Ace of Heart and King of Spade]"`).
pub(crate) fn extract_holding(instruction: &str) -> Result<Vec<Card>, PokerBenchError> {
    let start = instruction
        .find("holding is [")
        .map(|i| i + "holding is [".len())
        .ok_or_else(|| PokerBenchError::MissingField("holding".to_string()))?;
    let rest = &instruction[start..];
    let end = rest
        .find(']')
        .ok_or_else(|| PokerBenchError::MissingField("holding".to_string()))?;
    let inner = &rest[..end];
    let cards: Vec<Card> = inner
        .split(" and ")
        .filter_map(|phrase| card_from_phrase(phrase.trim()))
        .collect();
    if cards.is_empty() {
        return Err(PokerBenchError::Card(inner.to_string()));
    }
    Ok(cards)
}

/// Extracts the full board (flop + turn + river, in order) from an instruction's
/// `"The flop comes ..."` / `"turn comes ..."` / `"river comes ..."` clauses.
pub(crate) fn extract_board(instruction: &str) -> Vec<Card> {
    let mut board = Vec::new();
    for marker in ["flop comes ", "turn comes ", "river comes "] {
        if let Some(segment) = slice_after(instruction, marker, |c| c != '.') {
            board.extend(cards_from_phrase_list(segment));
        }
    }
    board
}

/// Extracts the pot from an instruction (`"pot size is 24.0 chips"`).
///
/// # Errors
/// Returns [`PokerBenchError::MissingField`] if no pot phrase is present.
pub(crate) fn extract_pot(instruction: &str) -> Result<u32, PokerBenchError> {
    let token = slice_after(instruction, "pot size is ", |c| c.is_ascii_digit() || c == '.')
        .ok_or_else(|| PokerBenchError::MissingField("pot".to_string()))?;
    parse_chips(token)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod pokerbench__tests {
    use super::*;

    #[test]
    fn parse_position_known_codes() {
        assert_eq!(parse_position("utg").unwrap(), Position::UTG);
        assert_eq!(parse_position(" BTN ").unwrap(), Position::BTN);
        assert_eq!(parse_position("BB").unwrap(), Position::BB);
    }

    #[test]
    fn parse_position_unknown_is_err() {
        assert!(parse_position("XX").is_err());
    }

    #[test]
    fn parse_cards_concat_handles_concatenated() {
        let cards = parse_cards_concat("AhKs").unwrap();
        assert_eq!(cards.len(), 2);
    }

    #[test]
    fn parse_cards_concat_handles_spaced() {
        let cards = parse_cards_concat("Ks 7h 2d").unwrap();
        assert_eq!(cards.len(), 3);
    }

    #[test]
    fn parse_cards_concat_empty_is_empty_vec() {
        assert!(parse_cards_concat("   ").unwrap().is_empty());
    }

    #[test]
    fn parse_cards_concat_odd_length_is_err() {
        assert!(parse_cards_concat("AhK").is_err());
    }

    #[test]
    fn parse_legal_python_list_preflop() {
        let legal = parse_legal("['3.0bb', 'call', 'fold']");
        assert_eq!(
            legal,
            vec![
                PokerBenchAction::Raise(3),
                PokerBenchAction::Call,
                PokerBenchAction::Fold
            ]
        );
    }

    #[test]
    fn parse_legal_python_list_postflop() {
        let legal = parse_legal("['Check', 'Bet 24']");
        assert_eq!(legal, vec![PokerBenchAction::Check, PokerBenchAction::Bet(24)]);
    }

    #[test]
    fn parse_history_preflop_slash_line() {
        let history = parse_history("UTG/2.0bb/BTN/call/SB/allin/BB/fold");
        assert_eq!(
            history,
            vec![
                PokerBenchAction::Raise(2),
                PokerBenchAction::Call,
                PokerBenchAction::AllIn,
                PokerBenchAction::Fold
            ]
        );
    }

    #[test]
    fn parse_history_postflop_slash_line() {
        let history = parse_history("OOP_CHECK/IP_BET_5/OOP_RAISE_14/IP_CALL/dealcards/Jc/OOP_CHECK");
        assert_eq!(
            history,
            vec![
                PokerBenchAction::Check,
                PokerBenchAction::Bet(5),
                PokerBenchAction::Raise(14),
                PokerBenchAction::Call,
                PokerBenchAction::Check
            ]
        );
    }

    #[test]
    fn parse_history_prose_with_chips_and_cards() {
        let history = parse_history(
            "The flop comes King Of Spade, Seven Of Heart, and Two Of Diamond, \
             then BB bet 4 chips, and BTN call.",
        );
        assert_eq!(history, vec![PokerBenchAction::Bet(4), PokerBenchAction::Call]);
    }

    #[test]
    fn derive_to_call_faces_last_bet() {
        assert_eq!(derive_to_call(&[PokerBenchAction::Bet(4)]), 4);
        assert_eq!(derive_to_call(&[PokerBenchAction::Raise(3)]), 3);
    }

    #[test]
    fn derive_to_call_zero_when_checked_to() {
        assert_eq!(derive_to_call(&[PokerBenchAction::Bet(4), PokerBenchAction::Check]), 0);
        assert_eq!(derive_to_call(&[]), 0);
    }

    #[test]
    fn derive_to_call_uses_standing_level_behind_a_call() {
        // UTG raise 2, BTN call -> the hero (BB) still faces the 2bb level.
        assert_eq!(derive_to_call(&[PokerBenchAction::Raise(2), PokerBenchAction::Call]), 2);
    }

    #[test]
    fn resolve_postflop_hero_maps_ip_oop() {
        // HJ vs BB heads-up: BB acts first post-flop (OOP), HJ is IP.
        assert_eq!(resolve_postflop_hero("HJ/2.0bb/BB/call", "IP").unwrap(), Position::HJ);
        assert_eq!(resolve_postflop_hero("HJ/2.0bb/BB/call", "OOP").unwrap(), Position::BB);
    }

    #[test]
    fn resolve_postflop_hero_three_bet_pot_same_two_positions() {
        let hero = resolve_postflop_hero("SB/3.0bb/BB/10.0bb/SB/call", "OOP").unwrap();
        assert_eq!(hero, Position::SB); // SB acts before BB post-flop
    }

    #[test]
    fn resolve_postflop_hero_rejects_non_heads_up() {
        assert!(resolve_postflop_hero("UTG/2bb/CO/call/BTN/call", "IP").is_err());
    }

    #[test]
    fn seed_stacks_is_six_handed() {
        let stacks = seed_stacks();
        assert_eq!(stacks.len(), 6);
        assert!(stacks.iter().all(|(_, chips)| *chips == 100));
    }

    #[test]
    fn extract_position_from_prose() {
        let text = "... your position is HJ, and your holding is [Ace of Heart and King of Spade].";
        assert_eq!(extract_position(text).unwrap(), Position::HJ);
    }

    #[test]
    fn extract_holding_from_prose() {
        let text = "your holding is [Ace of Heart and King of Spade].";
        let hole = extract_holding(text).unwrap();
        assert_eq!(hole, parse_cards_concat("AhKs").unwrap());
    }

    #[test]
    fn extract_board_multistreet() {
        let text = "The flop comes King Of Spade, Seven Of Heart, and Two Of Diamond. \
                    The turn comes Jack Of Club. The river comes Seven Of Club, then BB check.";
        let board = extract_board(text);
        assert_eq!(board, parse_cards_concat("Ks7h2dJc7c").unwrap());
    }

    #[test]
    fn extract_board_empty_when_preflop() {
        let text = "your position is BTN, and your holding is [Ace of Heart and King of Spade].";
        assert!(extract_board(text).is_empty());
    }

    #[test]
    fn extract_pot_from_prose() {
        let text = "Now it is your turn. The current pot size is 24.0 chips.";
        assert_eq!(extract_pot(text).unwrap(), 24);
    }
}
