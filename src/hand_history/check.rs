//! `HandHistory::validate`: checks a hand history's claims against the rules
//! of the game (EPIC-93, Faithful Replay).
//!
//! A hand history is a claim. [`HandHistory::validate`] reads one and returns a
//! [`Finding`] for every place the claim breaks a rule, each one naming the
//! YAML path of the field at fault and the [`Rule`] it broke. It returns all of
//! them, not the first, so an author fixes a file in one pass instead of one
//! build per mistake.
//!
//! `validate` is a sequence of rule passes, one function per rule in the
//! private `passes` module. Each pass reads the hand and appends to the same
//! list of findings, so a pass added later (order, legality, amounts, pots,
//! results, money) joins the sequence in `validate` without touching the
//! others. A rule that finds nothing wrong adds nothing: absent optional fields
//! are not findings.
//!
//! # Seats
//!
//! Seats are **0-based**, the engine's seat index. A seat that acts but is not
//! in `players` is a [`Rule::Seats`] finding: a history must list everyone dealt
//! in. Two things are legal and are not findings: a dead button (the button on
//! an empty chair, TDA Rule 32) and a `post` of amount 0 or none by an unlisted
//! seat (the dead small blind that goes with it). The rule passes that walk the
//! hand (order, legality, amounts; EPIC-93 Task 5) are to seat an unlisted
//! actor with the largest listed stack so they can go on past the finding; no
//! pass does that yet.

use std::fmt;

/// One place where a hand history breaks a rule.
///
/// `path` is the YAML path of the offending field, with list indices:
/// `players[1].hole_cards`, `streets.turn.actions[1].amount`, `board`.
///
/// # Domain
///
/// - **Role:** value object
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::{Finding, Rule};
///
/// let finding = Finding {
///     path: "board".to_string(),
///     rule: Rule::Cards,
///     message: "A♠ appears twice".to_string(),
/// };
/// assert_eq!("board: A♠ appears twice (Cards)", finding.to_string());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// YAML path of the offending field, e.g. `streets.river.actions[1].amount`.
    pub path: String,
    /// The rule the field breaks.
    pub rule: Rule,
    /// One sentence, with the numbers: `calls 205000 with 201700 behind`.
    pub message: String,
}

impl fmt::Display for Finding {
    /// `"{path}: {message} ({rule:?})"`, e.g. `board: A♠ appears twice (Cards)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} ({:?})", self.path, self.message, self.rule)
    }
}

/// The rule a [`Finding`] says a hand history breaks.
///
/// # Domain
///
/// - **Role:** value object
///
/// # Examples
///
/// ```
/// use pkcore::hand_history::Rule;
///
/// assert_eq!("seats", Rule::Seats.to_string());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Rule {
    /// A card string does not parse, a card appears twice, or the board
    /// disagrees with the streets.
    Cards,
    /// Seats are not unique or not within the table, the button is outside the table,
    /// or an action is by a seat that is not listed or has already folded.
    Seats,
    /// Actions do not follow the engine's turn order.
    Order,
    /// An action is not legal at that point.
    Legality,
    /// A recorded amount is not the engine's amount.
    Amount,
    /// A recorded pot is not the engine's pot.
    Pot,
    /// A recorded result disagrees with the evaluation of the hand.
    Result,
    /// A recorded net is not the engine's net, or the nets do not sum to zero.
    Money,
    /// The game is one `validate` does not check (stud and razz).
    Unsupported,
}

impl fmt::Display for Rule {
    /// The lower-case name of the rule: `cards`, `seats`, `unsupported`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Rule::Cards => "cards",
            Rule::Seats => "seats",
            Rule::Order => "order",
            Rule::Legality => "legality",
            Rule::Amount => "amount",
            Rule::Pot => "pot",
            Rule::Result => "result",
            Rule::Money => "money",
            Rule::Unsupported => "unsupported",
        };
        f.write_str(name)
    }
}

impl crate::hand_history::HandHistory {
    /// Checks this hand history against the rules and returns every
    /// disagreement. Empty means no rule found a fault.
    ///
    /// Stud, stud hi-lo and razz are not checked: they return one
    /// [`Rule::Unsupported`] finding at `hand.game` and nothing else. For every
    /// other game the rules run in turn (cards, then seats) and each appends
    /// its findings, so one file's several faults come back together. Absent
    /// optional fields (`board`, `streets`, hole cards, `table.seats`,
    /// `table.button`) are not findings.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::hand_history::{HandHistory, Rule};
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { seats: 6, stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 6, name: B, stack: 1000.0 }
    /// ").unwrap();
    ///
    /// let findings = hand.validate();
    /// assert_eq!(1, findings.len());
    /// assert_eq!(Rule::Seats, findings[0].rule);
    /// assert_eq!("players[1].seat", findings[0].path);
    /// ```
    #[cfg(feature = "hand-histories")]
    #[must_use]
    pub fn validate(&self) -> Vec<Finding> {
        if let Some(finding) = passes::unsupported(self) {
            return vec![finding];
        }
        let mut findings = Vec::new();
        passes::cards(self, &mut findings);
        passes::seats(self, &mut findings);
        findings
    }
}

/// The rule passes, one function per rule. `validate` calls them in order.
#[cfg(feature = "hand-histories")]
mod passes {
    use super::{Finding, Rule};
    use crate::card::Card;
    use crate::hand_history::{Action, ActionType, HandHistory, HandVariant};
    use crate::util::terminal::Terminal;
    use std::str::FromStr;

    // ── Unsupported ──────────────────────────────────────────────────────────

    /// The text of a game the validator does not check, `None` for the rest.
    fn unsupported_game(game: &HandVariant) -> Option<&'static str> {
        match game {
            HandVariant::Stud => Some("stud"),
            HandVariant::StudHiLo => Some("stud hi-lo"),
            HandVariant::Razz => Some("razz"),
            _ => None,
        }
    }

    /// Stud and razz are outside what the engine plays, so they stop here.
    pub(super) fn unsupported(hand: &HandHistory) -> Option<Finding> {
        unsupported_game(&hand.hand.game).map(|name| Finding {
            path: "hand.game".to_string(),
            rule: Rule::Unsupported,
            message: format!("{name} hands are not validated"),
        })
    }

    // ── Cards ────────────────────────────────────────────────────────────────

    /// The cards the streets show, in order, each with the path it came from.
    #[derive(Default)]
    struct StreetCards {
        cards: Vec<(Card, &'static str)>,
        /// False when a street's card string is malformed or a street is
        /// missing before a later one, so the board cannot be compared to it.
        sound: bool,
    }

    /// Cards rule: every card string parses; no card appears twice across the
    /// hole cards and the board; the board is 3, 4 or 5 cards and agrees with
    /// the streets. The first holder of a card is not blamed: the second is.
    pub(super) fn cards(hand: &HandHistory, findings: &mut Vec<Finding>) {
        let mut seen: Vec<(Card, String)> = Vec::new();

        for (index, player) in hand.players.iter().enumerate() {
            let path = format!("players[{index}].hole_cards");
            if let Some(text) = &player.hole_cards
                && let Some(parsed) = parse_cards(text, &path, findings)
            {
                record(&parsed, &path, &mut seen, findings);
            }
        }

        let board = hand
            .board
            .as_deref()
            .and_then(|text| parse_cards(text, "board", findings));
        let streets = street_cards(hand, findings);

        match &board {
            Some(parsed) => {
                record(parsed, "board", &mut seen, findings);
                check_board_length(parsed, findings);
                check_board_against_streets(parsed, &streets, findings);
            }
            None => {
                for (card, path) in &streets.cards {
                    record(&[*card], path, &mut seen, findings);
                }
            }
        }
    }

    /// Splits `text` into cards the way `Cards::from_str` does (whitespace,
    /// commas and hyphens separate), reporting the first bad token at `path`.
    fn parse_cards(text: &str, path: &str, findings: &mut Vec<Finding>) -> Option<Vec<Card>> {
        let cleaned = Terminal::index_cleaner(text);
        let mut parsed = Vec::new();
        for token in cleaned.split_whitespace() {
            match Card::from_str(token) {
                Ok(card) => parsed.push(card),
                Err(error) => {
                    findings.push(Finding {
                        path: path.to_string(),
                        rule: Rule::Cards,
                        message: format!("{token:?} is not a card ({error})"),
                    });
                    return None;
                }
            }
        }
        if parsed.is_empty() {
            findings.push(Finding {
                path: path.to_string(),
                rule: Rule::Cards,
                message: "has no cards".to_string(),
            });
            return None;
        }
        Some(parsed)
    }

    /// Notes `cards` as seen at `path`, and reports any already seen elsewhere.
    fn record(cards: &[Card], path: &str, seen: &mut Vec<(Card, String)>, findings: &mut Vec<Finding>) {
        for card in cards {
            if let Some((_, first)) = seen.iter().find(|(earlier, _)| earlier == card) {
                findings.push(Finding {
                    path: path.to_string(),
                    rule: Rule::Cards,
                    message: format!("{card} appears twice; it is already at {first}"),
                });
            } else {
                seen.push((*card, path.to_string()));
            }
        }
    }

    /// Reads the flop, turn and river strings, reporting malformed ones.
    fn street_cards(hand: &HandHistory, findings: &mut Vec<Finding>) -> StreetCards {
        let mut found = StreetCards {
            cards: Vec::new(),
            sound: true,
        };
        let Some(streets) = &hand.streets else {
            return found;
        };

        if let Some(flop) = &streets.flop {
            take_street(&flop.cards, "streets.flop.cards", 3, findings, &mut found);
        }
        if let Some(turn) = &streets.turn {
            if streets.flop.is_none() {
                missing_street("streets.turn.card", "turn", "flop", findings, &mut found);
            }
            take_street(&turn.card, "streets.turn.card", 1, findings, &mut found);
        }
        if let Some(river) = &streets.river {
            if streets.turn.is_none() {
                missing_street("streets.river.card", "river", "turn", findings, &mut found);
            }
            take_street(&river.card, "streets.river.card", 1, findings, &mut found);
        }
        found
    }

    fn missing_street(path: &str, street: &str, before: &str, findings: &mut Vec<Finding>, found: &mut StreetCards) {
        found.sound = false;
        findings.push(Finding {
            path: path.to_string(),
            rule: Rule::Cards,
            message: format!("the {street} is recorded without the {before}"),
        });
    }

    fn take_street(
        text: &str,
        path: &'static str,
        expected: usize,
        findings: &mut Vec<Finding>,
        found: &mut StreetCards,
    ) {
        let Some(parsed) = parse_cards(text, path, findings) else {
            found.sound = false;
            return;
        };
        if parsed.len() != expected {
            found.sound = false;
            findings.push(Finding {
                path: path.to_string(),
                rule: Rule::Cards,
                message: format!("has {} cards; this street deals {expected}", parsed.len()),
            });
            return;
        }
        found.cards.extend(parsed.into_iter().map(|card| (card, path)));
    }

    fn check_board_length(board: &[Card], findings: &mut Vec<Finding>) {
        if !(3..=5).contains(&board.len()) {
            findings.push(Finding {
                path: "board".to_string(),
                rule: Rule::Cards,
                message: format!("has {} cards; a board holds 3, 4 or 5", board.len()),
            });
        }
    }

    /// The board must start with the cards the streets show. A longer board is
    /// not a fault: a hand that ran out after an all-in records only the
    /// streets that had betting.
    fn check_board_against_streets(board: &[Card], streets: &StreetCards, findings: &mut Vec<Finding>) {
        if !streets.sound {
            return;
        }
        for (index, (street_card, path)) in streets.cards.iter().enumerate() {
            match board.get(index) {
                None => {
                    findings.push(Finding {
                        path: "board".to_string(),
                        rule: Rule::Cards,
                        message: format!("has {} cards but {path} is card {}", board.len(), index + 1),
                    });
                    return;
                }
                Some(board_card) if board_card != street_card => {
                    findings.push(Finding {
                        path: "board".to_string(),
                        rule: Rule::Cards,
                        message: format!("card {} is {board_card} but {path} is {street_card}", index + 1),
                    });
                }
                Some(_) => {}
            }
        }
    }

    // ── Seats ────────────────────────────────────────────────────────────────

    /// The streets that were played, each with the path prefix its actions
    /// live under. Later rule passes walk the same list.
    pub(super) fn street_actions(hand: &HandHistory) -> Vec<(&'static str, &[Action])> {
        let mut played: Vec<(&'static str, &[Action])> = Vec::new();
        if let Some(streets) = &hand.streets {
            if let Some(preflop) = &streets.preflop {
                played.push(("streets.preflop", preflop.actions.as_slice()));
            }
            if let Some(flop) = &streets.flop {
                played.push(("streets.flop", flop.actions.as_slice()));
            }
            if let Some(turn) = &streets.turn {
                played.push(("streets.turn", turn.actions.as_slice()));
            }
            if let Some(river) = &streets.river {
                played.push(("streets.river", river.actions.as_slice()));
            }
        }
        played
    }

    /// Seats rule: seats unique and inside `table.seats`; the button inside the table (a dead button, on an empty chair, is legal);
    /// every action by a listed seat that has not folded. Gaps between seats
    /// are fine: a table is rarely full.
    pub(super) fn seats(hand: &HandHistory, findings: &mut Vec<Finding>) {
        let listed = |seat: u8| hand.players.iter().any(|player| player.seat == seat);

        for (index, player) in hand.players.iter().enumerate() {
            let path = format!("players[{index}].seat");
            if let Some(first) = hand.players[..index]
                .iter()
                .position(|earlier| earlier.seat == player.seat)
            {
                findings.push(Finding {
                    path: path.clone(),
                    rule: Rule::Seats,
                    message: format!("seat {} is already taken by players[{first}]", player.seat),
                });
            }
            if let Some(seats) = hand.table.seats
                && player.seat >= seats
            {
                findings.push(Finding {
                    path,
                    rule: Rule::Seats,
                    message: format!(
                        "seat {} is outside a {seats}-seat table; seats are 0-based, so they run 0 to {}",
                        player.seat,
                        seats.saturating_sub(1)
                    ),
                });
            }
        }

        // A button on an empty chair is a dead button (TDA Rule 32), which is
        // legal. Only a button outside the table is a fault.
        if let Some(button) = hand.table.button
            && let Some(seats) = hand.table.seats
            && button >= seats
        {
            findings.push(Finding {
                path: "table.button".to_string(),
                rule: Rule::Seats,
                message: format!(
                    "the button is on seat {button}, outside a {seats}-seat table; seats are 0-based, so they run 0 to {}",
                    seats.saturating_sub(1)
                ),
            });
        }

        let mut folded: Vec<u8> = Vec::new();
        for (prefix, actions) in street_actions(hand) {
            for (index, action) in actions.iter().enumerate() {
                let path = format!("{prefix}.actions[{index}].seat");
                if !listed(action.seat) {
                    // The dead small blind: a zero or amountless post by an empty chair.
                    if action.action == ActionType::Post && action.amount.is_none_or(|amount| amount == 0.0) {
                        continue;
                    }
                    findings.push(Finding {
                        path,
                        rule: Rule::Seats,
                        message: format!(
                            "seat {} acts but is not in players; list everyone dealt in (seats are 0-based)",
                            action.seat
                        ),
                    });
                } else if folded.contains(&action.seat) {
                    findings.push(Finding {
                        path,
                        rule: Rule::Seats,
                        message: format!("seat {} acts after it folded", action.seat),
                    });
                }
                if action.action == ActionType::Fold {
                    folded.push(action.seat);
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_history__check_tests {
    use super::*;
    use crate::hand_history::HandHistory;

    /// The two-player hand from the `replay_with` doc test.
    const CLEAN: &str = r"
hand: { id: ex, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: raise, amount: 300.0 }
      - { seat: 1, action: fold }
";

    fn hand(yaml: &str) -> HandHistory {
        HandHistory::from_yaml(yaml).unwrap()
    }

    #[test]
    fn validate__clean_heads_up_hand_has_no_findings() {
        assert_eq!(Vec::<Finding>::new(), hand(CLEAN).validate());
    }

    #[test]
    fn validate__duplicate_card_is_a_cards_finding() {
        let yaml = r"
hand: { id: dup, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0, hole_cards: 'A♠ K♠' }
  - { seat: 1, name: B, stack: 1000.0, hole_cards: 'Q♦ J♦' }
board: 'A♠ 7♣ 2♥'
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Cards, findings[0].rule);
        assert_eq!("board", findings[0].path);
        assert!(findings[0].message.contains("A♠"), "{}", findings[0].message);
    }

    #[test]
    fn validate__duplicate_between_two_hole_cards_names_the_later_player() {
        let yaml = r"
hand: { id: dup2, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0, hole_cards: 'A♠ K♠' }
  - { seat: 1, name: B, stack: 1000.0, hole_cards: 'K♠ J♦' }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!("players[1].hole_cards", findings[0].path);
    }

    #[test]
    fn validate__unparseable_card_is_a_cards_finding() {
        let yaml = r"
hand: { id: bad, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0, hole_cards: 'A♠ Zz' }
  - { seat: 1, name: B, stack: 1000.0 }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Cards, findings[0].rule);
        assert_eq!("players[0].hole_cards", findings[0].path);
    }

    #[test]
    fn validate__board_that_disagrees_with_the_streets_is_a_cards_finding() {
        let yaml = r"
hand: { id: board, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
board: '9♣ 6♦ 5♥ 5♠ 8♠'
streets:
  flop: { cards: '9♣ 6♦ 5♥', actions: [] }
  turn: { card: '4♠', actions: [] }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Cards, findings[0].rule);
        assert_eq!("board", findings[0].path);
        assert!(
            findings[0].message.contains("streets.turn.card"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn validate__board_that_matches_the_streets_is_clean() {
        let yaml = r"
hand: { id: board, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
board: '9♣ 6♦ 5♥ 5♠ 8♠'
streets:
  flop: { cards: '9♣ 6♦ 5♥', actions: [] }
  turn: { card: '5♠', actions: [] }
  river: { card: '8♠', actions: [] }
";
        assert_eq!(Vec::<Finding>::new(), hand(yaml).validate());
    }

    #[test]
    fn validate__board_of_two_cards_is_a_cards_finding() {
        let yaml = r"
hand: { id: short, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
board: '9♣ 6♦'
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Cards, findings[0].rule);
        assert_eq!("board", findings[0].path);
    }

    #[test]
    fn validate__acting_seat_not_in_players_is_a_seats_finding() {
        let yaml = r"
hand: { id: ghost, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: raise, amount: 300.0 }
      - { seat: 5, action: fold }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Seats, findings[0].rule);
        assert_eq!("streets.preflop.actions[1].seat", findings[0].path);
    }

    #[test]
    fn validate__acting_after_folding_is_a_seats_finding() {
        let yaml = r"
hand: { id: zombie, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
board: '9♣ 6♦ 5♥'
streets:
  preflop:
    actions:
      - { seat: 0, action: raise, amount: 300.0 }
      - { seat: 1, action: fold }
  flop:
    cards: '9♣ 6♦ 5♥'
    actions:
      - { seat: 1, action: check }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Seats, findings[0].rule);
        assert_eq!("streets.flop.actions[0].seat", findings[0].path);
        assert!(findings[0].message.contains("folded"), "{}", findings[0].message);
    }

    #[test]
    fn validate__seat_equal_to_table_seats_is_off_by_one() {
        let yaml = r"
hand: { id: off, game: holdem }
table: { seats: 6, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 6, name: B, stack: 1000.0 }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Seats, findings[0].rule);
        assert_eq!("players[1].seat", findings[0].path);
        assert!(findings[0].message.contains("0-based"), "{}", findings[0].message);
    }

    #[test]
    fn validate__duplicate_seat_is_a_seats_finding() {
        let yaml = r"
hand: { id: twice, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 1, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Seats, findings[0].rule);
        assert_eq!("players[1].seat", findings[0].path);
    }

    #[test]
    fn validate__button_outside_the_table_is_a_seats_finding() {
        let yaml = r"
hand: { id: button, game: holdem }
table: { seats: 6, button: 6, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Seats, findings[0].rule);
        assert_eq!("table.button", findings[0].path);
        assert!(findings[0].message.contains("0-based"), "{}", findings[0].message);
    }

    #[test]
    fn validate__dead_button_on_an_empty_chair_is_not_a_finding() {
        let yaml = r"
hand: { id: dead, game: holdem }
table: { seats: 6, button: 4, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
";
        let findings = hand(yaml).validate();
        assert!(
            findings.iter().all(|finding| finding.rule != Rule::Seats),
            "{findings:?}"
        );
    }

    #[test]
    fn validate__zero_or_amountless_post_by_an_unlisted_seat_is_not_a_finding() {
        let yaml = r"
hand: { id: deadblind, game: holdem }
table: { seats: 6, button: 4, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 5, action: post, amount: 0.0 }
      - { seat: 3, action: post }
      - { seat: 0, action: post, amount: 100.0 }
      - { seat: 1, action: fold }
";
        let findings = hand(yaml).validate();
        assert_eq!(Vec::<Finding>::new(), findings);
    }

    #[test]
    fn validate__positive_post_by_an_unlisted_seat_is_a_seats_finding() {
        let yaml = r"
hand: { id: ghostblind, game: holdem }
table: { seats: 6, button: 4, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 5, action: post, amount: 50.0 }
      - { seat: 0, action: fold }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Seats, findings[0].rule);
        assert_eq!("streets.preflop.actions[0].seat", findings[0].path);
    }

    #[test]
    fn validate__zero_post_by_an_unlisted_seat_is_not_a_free_pass_for_other_actions() {
        let yaml = r"
hand: { id: ghostfold, game: holdem }
table: { seats: 6, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 5, action: post, amount: 0.0 }
      - { seat: 5, action: fold }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!("streets.preflop.actions[1].seat", findings[0].path);
    }

    #[test]
    fn validate__seat_gap_is_not_a_finding() {
        let yaml = r"
hand: { id: gap, game: holdem }
table: { seats: 6, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 2, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: raise, amount: 300.0 }
      - { seat: 2, action: fold }
";
        let findings = hand(yaml).validate();
        assert!(
            findings.iter().all(|finding| finding.rule != Rule::Seats),
            "{findings:?}"
        );
    }

    #[test]
    fn validate__stud_is_unsupported() {
        let yaml = r"
hand: { id: stud, game: stud }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0, hole_cards: 'A♠ A♠' }
  - { seat: 9, name: B, stack: 1000.0 }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Unsupported, findings[0].rule);
        assert_eq!("hand.game", findings[0].path);
    }

    #[test]
    fn validate__razz_and_stud_hi_lo_are_unsupported() {
        for game in ["razz", "stud_hi_lo"] {
            let yaml = format!(
                "hand: {{ id: s, game: {game} }}\ntable: {{ stakes: {{ small_blind: 1.0, big_blind: 2.0 }} }}\nplayers: []\n"
            );
            let findings = hand(&yaml).validate();
            assert_eq!(1, findings.len(), "{game}: {findings:?}");
            assert_eq!(Rule::Unsupported, findings[0].rule, "{game}");
        }
    }

    #[test]
    fn finding__displays_path_message_and_rule() {
        let finding = Finding {
            path: "board".to_string(),
            rule: Rule::Cards,
            message: "A♠ appears twice".to_string(),
        };
        assert_eq!("board: A♠ appears twice (Cards)", finding.to_string());
    }

    #[test]
    fn rule__displays_lower_case_name() {
        let rules = [
            (Rule::Cards, "cards"),
            (Rule::Seats, "seats"),
            (Rule::Order, "order"),
            (Rule::Legality, "legality"),
            (Rule::Amount, "amount"),
            (Rule::Pot, "pot"),
            (Rule::Result, "result"),
            (Rule::Money, "money"),
            (Rule::Unsupported, "unsupported"),
        ];
        for (rule, name) in rules {
            assert_eq!(name, rule.to_string());
        }
    }
}
