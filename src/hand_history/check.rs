//! `HandHistory::validate`: checks a hand history's claims against the rules
//! of the game (EPIC-93, Faithful Replay).
//!
//! A hand history is a claim. [`HandHistory::validate`](crate::hand_history::HandHistory::validate) reads one and returns a
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
//! are not findings, except that a record must run to the end of the hand
//! (the Incomplete rule), so `validate` coming back empty means `replay` and
//! `cursor` succeed.
//!
//! # Seats
//!
//! Seats are **0-based**, the engine's seat index. A seat that acts but is not
//! in `players` is a [`Rule::Seats`] finding: a history must list everyone dealt
//! in. Two things are legal and are not findings: a dead button (the button on
//! an empty chair, TDA Rule 32) and a `post` of amount 0 or none by an unlisted
//! seat (the dead small blind that goes with it). The walk (below) seats an
//! unlisted actor with the largest listed stack so it can go on past the
//! finding; a seat whose only action is such a `post` is not seated.
//!
//! # The walk
//!
//! The Order, Legality and Amount rules are one pass that plays the hand
//! through the engine, street by street, as `replay_with` does, checking each
//! recorded action before applying it. An action out of turn cannot be
//! applied, so the walk stops there. An illegal bet, raise or check is
//! reported and repaired (to the minimum, a call or a check) so the walk goes
//! on; the first repair adds the finding "pots, nets and amounts after this
//! point are not checked", because from there the engine's chips are not the
//! record's. After it only record-only checks (whole chips) still run.
//!
//! # What a recorded amount means
//!
//! A `call` records the chips it adds or the caller's street total after it;
//! both readings are accepted, and a call that matches neither is a finding. A
//! `bet`, `raise` or `all_in` records the total the seat has bet on the street
//! after the action. The comparison is one
//! function that `validate` and `replay_with` share, so a history `validate`
//! passes on amounts is one `replay` accepts.
//!
//! # Pots, results and money
//!
//! A street's `pot` is checked by the walk, at the end of the street: it is
//! the engine's pot with the street's bets in it, antes and blinds included.
//! The Result rule and most of the Money rule need no walk, so they run on the
//! record alone and still report on a file whose walk stopped at its first
//! action: `best_hand` and `hand_rank` against pkcore's evaluation, `outcome`
//! against the ranking at showdown, no loss larger than the stack, and nets
//! that sum to zero. Each `net` is compared with the engine's only when the
//! walk played every action without a repair.
//!
//! One fault, one finding: when the walk reports an Amount finding, the
//! engine has moved chips the record did not, so the walked Pot and Money
//! rules are skipped for that hand rather than repeat the same fault in every
//! pot and net. The record-only Money checks still run.

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
    /// an action is by a seat that is not listed or has already folded, a
    /// starting stack is not a whole number of chips, or the engine cannot
    /// seat the players and post their forced bets.
    Seats,
    /// Actions do not follow the engine's turn order.
    Order,
    /// An action is not legal at that point.
    Legality,
    /// A recorded amount is not the engine's amount, or an amount or stake is
    /// not a whole number of chips, or the engine cannot post the stakes.
    Amount,
    /// A recorded pot is not the engine's pot.
    Pot,
    /// A recorded result disagrees with the evaluation of the hand.
    Result,
    /// A recorded net is not the engine's net, or the nets do not sum to zero.
    Money,
    /// The game is one `validate` does not check (stud and razz).
    Unsupported,
    /// The record stops before the hand is over: two or more seats are still
    /// in, and an action or a street is missing. A hand history runs to the
    /// end of the hand, so it can be replayed.
    Incomplete,
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
            Rule::Incomplete => "incomplete",
        };
        f.write_str(name)
    }
}

impl crate::hand_history::HandHistory {
    /// Checks this hand history against the rules and returns every
    /// disagreement.
    ///
    /// Empty means the hand is a complete, legal record that replays: no rule
    /// found a fault, the record runs to the end of the hand, and
    /// [`Self::replay`] and [`Self::cursor`] both succeed on it. A record
    /// that stops before the hand is over (two or more seats still in, with
    /// an action or a street missing, or no `streets` at all) is a
    /// [`Rule::Incomplete`] finding; so is a board given only in `board` for a
    /// hand that went all in, since the streets the engine plays are not
    /// recorded. Partial records are not supported.
    ///
    /// Stud, stud hi-lo and razz are not checked: they return one
    /// [`Rule::Unsupported`] finding at `hand.game` and nothing else. For every
    /// other game the rules run in turn (cards, seats, stakes and stacks, then
    /// a walk through the engine for order, legality, amounts, pots and
    /// completeness, then results and money) and each appends its findings,
    /// so one file's several faults come back together. Other absent optional
    /// fields (`board`, hole cards, `table.seats`, `table.button`, `pot`,
    /// `results`, `net`) are not findings.
    ///
    /// Every stake is a whole, non-negative number of chips (a
    /// [`Rule::Amount`] finding at `table.stakes.*` otherwise) and so is every
    /// starting stack (a [`Rule::Seats`] finding at `players[i].stack`): NaN,
    /// negative and infinite numbers are findings. A hand the engine cannot
    /// set up at all, such as a listed player with no chips in a blind, is
    /// one finding at `players` ([`Rule::Seats`]), or at `table.stakes`
    /// ([`Rule::Amount`]) when the stakes are at fault, naming the engine's
    /// error.
    ///
    /// A street's `pot` is the engine's pot after the street's last action,
    /// antes, blinds and the street's bets included. Results are checked on
    /// the record alone, for Hold'em: `best_hand` is a best five of the seat's
    /// hole cards and the board, `hand_rank` is pkcore's rank of it, and at
    /// showdown the best hand is marked `win` (or `tie`, when it is split).
    /// Money is checked on the record alone (no loss larger than the starting
    /// stack; nets sum to zero when every seat that put chips in has one) and,
    /// when the walk played every action without a repair, against the
    /// engine: each `net` is the seat's chips after the engine's showdown less
    /// its starting stack.
    ///
    /// The walk stops at an action out of turn, which the engine cannot apply.
    /// An illegal bet, raise or check is repaired so the walk goes on, and the
    /// first repair adds a [`Rule::Legality`] finding that says pots, nets and
    /// amounts after it are not checked. A recorded amount is checked the way
    /// [`Self::replay`] checks it: a `call` records the chips it adds or the
    /// caller's street total after it (either is accepted); a `bet`, `raise` or
    /// `all_in` the seat's street total after it.
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
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: fold }
    /// ").unwrap();
    ///
    /// let findings = hand.validate();
    /// assert_eq!(1, findings.len());
    /// assert_eq!(Rule::Seats, findings[0].rule);
    /// assert_eq!("players[1].seat", findings[0].path);
    /// ```
    ///
    /// A call records the chips it adds or the caller's street total after it.
    /// The big blind calling a raise to 300 adds 200 and has 300 in, so a call
    /// of 250 is neither. The hand plays on to its end, so that is the only
    /// finding:
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::{HandHistory, Rule};
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 1, name: B, stack: 1000.0 }
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: raise, amount: 300.0 }
    ///       - { seat: 1, action: call, amount: 250.0 }
    ///   flop:
    ///     cards: 'K♠ K♣ 9♦'
    ///     actions:
    ///       - { seat: 1, action: bet, amount: 100.0 }
    ///       - { seat: 0, action: fold }
    /// ").unwrap();
    ///
    /// let findings = hand.validate();
    /// assert_eq!(1, findings.len(), "{findings:?}");
    /// assert_eq!(Rule::Amount, findings[0].rule);
    /// assert_eq!("streets.preflop.actions[1].amount", findings[0].path);
    /// # }
    /// ```
    ///
    /// A winner whose hand is weaker is a [`Rule::Result`] finding, found
    /// without playing the hand. Seat 1's quad fives (124) beat seat 0's sixes
    /// full (271); lower is stronger:
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::{HandHistory, Rule};
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0, hole_cards: '6♠ 6♥' }
    ///   - { seat: 1, name: B, stack: 1000.0, hole_cards: '5♦ 5♣' }
    /// board: '9♣ 6♦ 5♥ 5♠ 8♠'
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: all_in, amount: 1000.0, all_in: true }
    ///       - { seat: 1, action: call, amount: 900.0 }
    ///   flop: { cards: '9♣ 6♦ 5♥', actions: [] }
    ///   turn: { card: '5♠', actions: [] }
    ///   river: { card: '8♠', actions: [] }
    /// results:
    ///   - { seat: 0, outcome: win }
    ///   - { seat: 1, outcome: lose }
    /// ").unwrap();
    ///
    /// let findings = hand.validate();
    /// assert!(findings.iter().all(|finding| finding.rule == Rule::Result));
    /// assert!(findings.iter().any(|finding| finding.path == "results[1].outcome"));
    /// # }
    /// ```
    ///
    /// A record that stops before the hand is over is a [`Rule::Incomplete`]
    /// finding: both players are all in, but the streets the engine deals are
    /// not recorded, only the `board`:
    ///
    /// ```
    /// # #[cfg(feature = "hand-histories")]
    /// # {
    /// use pkcore::hand_history::{HandHistory, Rule};
    ///
    /// let hand = HandHistory::from_yaml(r"
    /// hand: { id: ex, game: holdem }
    /// table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
    /// players:
    ///   - { seat: 0, name: A, stack: 1000.0 }
    ///   - { seat: 1, name: B, stack: 1000.0 }
    /// board: '9♣ 6♦ 5♥ 5♠ 8♠'
    /// streets:
    ///   preflop:
    ///     actions:
    ///       - { seat: 0, action: all_in, amount: 1000.0, all_in: true }
    ///       - { seat: 1, action: call, amount: 900.0 }
    /// ").unwrap();
    ///
    /// let findings = hand.validate();
    /// assert_eq!(1, findings.len());
    /// assert_eq!(Rule::Incomplete, findings[0].rule);
    /// assert_eq!("streets.preflop", findings[0].path);
    /// assert!(hand.cursor().is_err());
    /// # }
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
        passes::chips(self, &mut findings);
        let walk = passes::walk(self, &mut findings);
        passes::results(self, &mut findings);
        passes::money(self, &mut findings);
        passes::walked_money(self, &walk, &mut findings);
        findings
    }
}

/// A recorded amount the engine disagrees with, in whole chips.
#[cfg(feature = "hand-histories")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AmountMismatch {
    /// The recorded amount, rounded down to whole chips.
    pub(super) recorded: usize,
    /// The amount the engine moves.
    pub(super) engine: usize,
}

/// Compares `action`'s recorded amount with the engine's, from where `table`
/// stands with `action.seat` to act. `None` when they agree, or when there is
/// nothing to compare (no amount recorded, or an action that moves no chips).
///
/// A `call` may record either the chips it adds (the amount owed, or the whole
/// stack when the stack cannot cover it), which is what the engine logs, or the
/// caller's street total after it, which is how hands are often typed. Either
/// reading is accepted; a mismatch is a call that matches neither, and its
/// `engine` is the chips the call adds. A `bet`, `raise` or `all_in` records
/// the seat's total on the street after the action, which is never more than
/// the seat's chips plus its bet. An amount that is not a whole, non-negative
/// number of chips is always a mismatch, with `recorded` rounded down.
/// [`HandHistory::validate`] and
/// [`HandHistory::replay_with`] both call this, so they cannot disagree on what
/// an amount means.
///
/// [`HandHistory::validate`]: crate::hand_history::HandHistory::validate
/// [`HandHistory::replay_with`]: crate::hand_history::HandHistory::replay_with
#[cfg(feature = "hand-histories")]
pub(super) fn amount_mismatch(
    table: &crate::casino::table::Table,
    action: &crate::hand_history::Action,
) -> Option<AmountMismatch> {
    let amount = action.amount?;
    let recorded = whole_chips(amount);
    let engine = engine_amount(table, action)?;
    if !is_whole_chips(amount) {
        // Chips are whole: a fractional or negative amount is never the engine's,
        // even when its whole part agrees.
        return Some(AmountMismatch { recorded, engine });
    }
    if recorded == engine {
        return None;
    }
    if action.action == crate::hand_history::ActionType::Call
        && let Some(seat) = table.seats.get_seat(action.seat)
        && recorded == seat.player.bet + engine
    {
        // The caller's street total after the call: the other accepted reading.
        return None;
    }
    Some(AmountMismatch { recorded, engine })
}

/// The amount the engine moves for `action`, in the units the record uses.
#[cfg(feature = "hand-histories")]
fn engine_amount(table: &crate::casino::table::Table, action: &crate::hand_history::Action) -> Option<usize> {
    use crate::hand_history::ActionType;

    let player = &table.seats.get_seat(action.seat)?.player;
    match action.action {
        ActionType::Call => Some(table.to_call(action.seat).min(player.chips)),
        ActionType::Bet | ActionType::Raise => action
            .amount
            .map(|amount| whole_chips(amount).min(player.total_chip_count())),
        ActionType::AllIn => {
            // A shove can become a call (TDA 47-A) or a capped raise, so the
            // engine is asked rather than second-guessed.
            let mut probe = table.clone();
            probe.act_all_in(action.seat).ok()?;
            probe.seats.get_seat(action.seat).map(|seat| seat.player.bet)
        }
        ActionType::Fold | ActionType::Check | ActionType::Post => None,
    }
}

/// True when `amount` is a whole, non-negative number of chips.
#[cfg(feature = "hand-histories")]
fn is_whole_chips(amount: f64) -> bool {
    amount.fract() == 0.0 && amount >= 0.0
}

/// `amount` rounded down to whole chips; a negative amount is 0.
#[cfg(feature = "hand-histories")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn whole_chips(amount: f64) -> usize {
    amount.floor().max(0.0) as usize
}

/// The rule passes, one function per rule. `validate` calls them in order.
#[cfg(feature = "hand-histories")]
mod passes {
    use super::{Finding, Rule, amount_mismatch, is_whole_chips, whole_chips};
    use crate::analysis::eval::Eval;
    use crate::arrays::HandRanker;
    use crate::arrays::five::Five;
    use crate::card::Card;
    use crate::cards::Cards;
    use crate::casino::action::PlayerAction;
    use crate::casino::table::Table;
    use crate::hand_history::{
        Action, ActionType, HandHistory, HandVariant, Outcome, PlayerEntry, ReplayStreet, ResultEntry,
    };
    use crate::util::terminal::Terminal;
    use std::borrow::Cow;
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

    // ── Chips: stakes and stacks ─────────────────────────────────────────────

    /// The stakes, each with the path it lives at.
    fn stakes(hand: &HandHistory) -> Vec<(&'static str, f64)> {
        let stakes = &hand.table.stakes;
        let mut named = vec![
            ("table.stakes.small_blind", stakes.small_blind),
            ("table.stakes.big_blind", stakes.big_blind),
        ];
        for (path, value) in [
            ("table.stakes.ante", stakes.ante),
            ("table.stakes.straddle", stakes.straddle),
            ("table.stakes.bring_in", stakes.bring_in),
        ] {
            if let Some(value) = value {
                named.push((path, value));
            }
        }
        named
    }

    /// Amount and Seats rules, on the record alone: every stake (an
    /// [`Rule::Amount`] finding) and every starting stack (a [`Rule::Seats`]
    /// finding) is a whole, non-negative number of chips. NaN, a negative
    /// number and infinity are not, and the engine would read each of them
    /// as some other number.
    pub(super) fn chips(hand: &HandHistory, findings: &mut Vec<Finding>) {
        for (path, value) in stakes(hand) {
            if !is_whole_chips(value) {
                findings.push(Finding {
                    path: path.to_string(),
                    rule: Rule::Amount,
                    message: format!("records {value}, which is not a whole number of chips"),
                });
            }
        }
        for (index, player) in hand.players.iter().enumerate() {
            if !is_whole_chips(player.stack) {
                findings.push(Finding {
                    path: format!("players[{index}].stack"),
                    rule: Rule::Seats,
                    message: format!(
                        "seat {} starts with {}, which is not a whole number of chips",
                        player.seat, player.stack
                    ),
                });
            }
        }
    }

    /// The finding for a hand the engine cannot set up: its stakes when the
    /// Amount rule found fault with them, otherwise its players. `None` when
    /// a hole-card string does not parse: the Cards rule has said so.
    fn unseated(error: crate::PKError, findings: &[Finding]) -> Option<Finding> {
        let at_fault = |prefix: &str, rule: Rule| {
            findings
                .iter()
                .any(|finding| finding.rule == rule && finding.path.starts_with(prefix))
        };
        if at_fault("players[", Rule::Cards) {
            return None;
        }
        Some(if at_fault("table.stakes", Rule::Amount) {
            Finding {
                path: "table.stakes".to_string(),
                rule: Rule::Amount,
                message: format!("the engine cannot post these stakes ({error:?}: {error})"),
            }
        } else {
            Finding {
                path: "players".to_string(),
                rule: Rule::Seats,
                message: format!(
                    "the engine cannot seat these players and post the forced bets ({error:?}: {error}); every stack must cover its blind and ante"
                ),
            }
        })
    }

    // ── The walk: Order, Legality, Amount ────────────────────────────────────

    /// The message of the one finding that marks the first repair.
    pub(super) const REPAIRED: &str = "pots, nets and amounts after this point are not checked";

    /// What the walk leaves for the rules that read the engine's table.
    pub(super) struct Walk {
        /// The table after the last action the walk applied; `None` when the
        /// starting table could not be built (a Seats or Amount finding at
        /// `players` or `table.stakes` says why, or the Cards rule does).
        pub(super) table: Option<Table>,
        /// True once an action was repaired: from there the engine's chips are
        /// not the record's, so pots, nets and amounts compared with the engine
        /// say nothing.
        pub(super) repaired: bool,
        /// True when the walk applied every recorded action: it did not stop
        /// at an action out of turn, one the engine refused, or a street
        /// whose cards do not parse.
        pub(super) complete: bool,
        /// True when the walk reported an amount: the engine then moved
        /// chips the record did not, so its pots and nets would only repeat
        /// that one fault.
        pub(super) amount_found: bool,
    }

    /// The walk's running state.
    struct Walker<'a> {
        findings: &'a mut Vec<Finding>,
        /// `.seat` paths the Seats rule already blamed.
        blamed: Vec<String>,
        repaired: bool,
        stopped: bool,
        amount_found: bool,
        /// The largest street bet a seat held when it folded, this street.
        folded_bet: usize,
    }

    /// Order, Legality and Amount rules: plays the hand through the engine,
    /// street by street, the way [`HandHistory::replay_with`] does, and checks
    /// each recorded action before applying it.
    ///
    /// - **Order**: the seat is the engine's next to act. An action out of
    ///   turn cannot be applied, so the walk stops there. A path the Seats rule
    ///   already blamed is not reported again.
    /// - **Legality**: a bet or raise below the minimum and not all in, a bet
    ///   or raise larger than the stack, a bet or raise when no raise is open,
    ///   and a check facing a bet. Each is repaired (to the minimum, an
    ///   all-in, a call, or a check) and the walk goes on; the first repair
    ///   adds one more finding, [`REPAIRED`].
    /// - **Amount**: the recorded amount is a whole number of chips and is the
    ///   engine's ([`amount_mismatch`]). After a repair the engine's numbers
    ///   are no longer the record's, so only the whole-chips check runs.
    ///
    /// - **Pot**: at the end of each street the walk played through, the
    ///   street's recorded `pot` is the engine's pot with the street's bets
    ///   in it, antes and blinds included ([`check_pot`]). A street whose
    ///   walk stopped, or that ends after a repair, is not checked, and no pot
    ///   is reported for a hand with an Amount finding.
    ///
    /// - **Incomplete**: a walk that played every recorded action, or ran out
    ///   of record mid-street, and left two or more seats in a hand that is
    ///   not over, reports it once ([`incomplete`]).
    ///
    /// A hand the engine cannot set up is one finding ([`unseated`]) and no
    /// walk. A seat that acts but is not listed is seated with the largest
    /// listed stack, so the walk can go on past the Seats finding.
    pub(super) fn walk(hand: &HandHistory, findings: &mut Vec<Finding>) -> Walk {
        let recorded = hand;
        let hand = seat_unlisted_actors(hand);
        let mut table = match hand.replay_table() {
            Ok(table) => table,
            Err(error) => {
                findings.extend(unseated(error, findings));
                return Walk {
                    table: None,
                    repaired: false,
                    complete: false,
                    amount_found: false,
                };
            }
        };
        let blamed = findings
            .iter()
            .filter(|finding| finding.rule == Rule::Seats)
            .map(|finding| finding.path.clone())
            .collect();
        let mut walker = Walker {
            findings,
            blamed,
            repaired: false,
            stopped: false,
            amount_found: false,
            folded_bet: 0,
        };
        let mut pots: Vec<Finding> = Vec::new();
        let mut last: Option<ReplayStreet> = None;
        // A street whose cards do not parse ends the walk; the Cards rule has
        // already reported it.
        let played = hand.play_streets(&mut table, |table, street, actions| {
            last = Some(street);
            let prefix = street_prefix(street);
            walker.folded_bet = 0;
            for (index, action) in actions.iter().enumerate() {
                if !walker.step(table, &format!("{prefix}.actions[{index}]"), action) {
                    walker.stopped = true;
                    return Ok(false);
                }
            }
            if !walker.repaired
                && let Some(pot) = street_pot(recorded, street)
                && let Some(finding) = check_pot(table, prefix, street, pot, walker.folded_bet)
            {
                pots.push(finding);
            }
            Ok(true)
        });
        // One fault, one finding: a wrong amount makes every later pot wrong.
        if !walker.amount_found {
            walker.findings.append(&mut pots);
        }
        // The walk ran out of record rather than stopping at a fault: every
        // street played, or the next street could not be dealt because the
        // last one's betting is not done.
        let ran_out = !walker.stopped && matches!(played, Ok(()) | Err(crate::PKError::ActionIsntFinished));
        if ran_out && let Some(finding) = incomplete(&table, last) {
            walker.findings.push(finding);
        }
        Walk {
            table: Some(table),
            repaired: walker.repaired,
            complete: played.is_ok() && !walker.stopped,
            amount_found: walker.amount_found,
        }
    }

    /// Incomplete rule: after a walk that played every recorded action, the
    /// hand is over: one seat is left, or the river's betting is done. `last`
    /// is the last street the walk played; the finding is at its path, or at
    /// `streets` when no action was recorded.
    fn incomplete(table: &Table, last: Option<ReplayStreet>) -> Option<Finding> {
        if table.is_game_over() {
            return None;
        }
        let left = table.seats.count_active_in_hand();
        let Some(street) = last else {
            return Some(Finding {
                path: "streets".to_string(),
                rule: Rule::Incomplete,
                message: format!(
                    "no action is recorded, with seat {} first to act and {left} seats in; record every action to the end of the hand",
                    table.next_to_act()
                ),
            });
        };
        let path = street_prefix(street).to_string();
        let message = if table.seats.is_betting_complete() {
            format!(
                "the record stops after the {street} with {left} seats still in; record every street to the end of the hand"
            )
        } else {
            format!(
                "the record stops on the {street} with seat {} to act and {left} seats still in; record every action to the end of the hand",
                table.next_to_act()
            )
        };
        Some(Finding {
            path,
            rule: Rule::Incomplete,
            message,
        })
    }

    /// The `pot` recorded at the end of `street`, if any.
    fn street_pot(hand: &HandHistory, street: ReplayStreet) -> Option<f64> {
        let streets = hand.streets.as_ref()?;
        match street {
            ReplayStreet::Preflop => streets.preflop.as_ref()?.pot,
            ReplayStreet::Flop => streets.flop.as_ref()?.pot,
            ReplayStreet::Turn => streets.turn.as_ref()?.pot,
            ReplayStreet::River => streets.river.as_ref()?.pot,
        }
    }

    /// Pot rule: `recorded`, the pot at the end of `street`, is the engine's
    /// pot with the street's bets in it ([`Table::effective_pot`]): antes,
    /// blinds and every bet, called or not.
    ///
    /// A street that ends with everyone else folding has three accepted
    /// readings: that full pot; the pot less the uncalled bet (the winner's
    /// street bet above `folded_bet`, the largest street bet a seat folded
    /// to), which is returned; and `Table::pot`, the pot without the winner's
    /// street bet, which is what the engine's own log writes, so a history
    /// the engine wrote passes.
    fn check_pot(
        table: &Table,
        prefix: &str,
        street: ReplayStreet,
        recorded: f64,
        folded_bet: usize,
    ) -> Option<Finding> {
        let engine = table.effective_pot();
        let chips = whole_chips(recorded);
        let accepted = if let [winner] = table.seats.active_in_hand().as_slice() {
            let bet = table.seats.get_seat(*winner).map_or(0, |seat| seat.player.bet);
            let uncalled = bet.saturating_sub(folded_bet);
            vec![engine, engine.saturating_sub(uncalled), table.pot]
        } else {
            vec![engine]
        };
        if is_whole_chips(recorded) && accepted.contains(&chips) {
            return None;
        }
        Some(Finding {
            path: format!("{prefix}.pot"),
            rule: Rule::Pot,
            message: format!(
                "records a pot of {recorded}; the engine's pot after the {street} is {engine}, antes and blinds included"
            ),
        })
    }

    /// The path prefix of a street's actions.
    fn street_prefix(street: ReplayStreet) -> &'static str {
        match street {
            ReplayStreet::Preflop => "streets.preflop",
            ReplayStreet::Flop => "streets.flop",
            ReplayStreet::Turn => "streets.turn",
            ReplayStreet::River => "streets.river",
        }
    }

    /// `hand`, with every seat that takes a non-`post` action but is not in
    /// `players` seated with the largest listed stack. A seat whose only
    /// action is a `post` (the dead small blind) is not seated.
    fn seat_unlisted_actors(hand: &HandHistory) -> Cow<'_, HandHistory> {
        let mut unlisted: Vec<u8> = Vec::new();
        for (_, actions) in street_actions(hand) {
            for action in actions {
                if action.action != ActionType::Post
                    && !hand.players.iter().any(|player| player.seat == action.seat)
                    && !unlisted.contains(&action.seat)
                {
                    unlisted.push(action.seat);
                }
            }
        }
        if unlisted.is_empty() {
            return Cow::Borrowed(hand);
        }
        let stack = hand.players.iter().map(|player| player.stack).fold(0.0, f64::max);
        let mut patched = hand.clone();
        patched.players.extend(unlisted.into_iter().map(|seat| PlayerEntry {
            seat,
            name: format!("seat {seat}"),
            stack,
            player_id: None,
            hole_cards: None,
            posted: None,
            hole_cards_visibility: None,
            withdrawn: None,
        }));
        Cow::Owned(patched)
    }

    impl Walker<'_> {
        fn push(&mut self, path: String, rule: Rule, message: String) {
            if rule == Rule::Amount {
                self.amount_found = true;
            }
            self.findings.push(Finding { path, rule, message });
        }

        /// Checks and applies one recorded action at path `at`. Returns
        /// `false` when the walk must stop.
        fn step(&mut self, table: &mut Table, at: &str, action: &Action) -> bool {
            if action.action == ActionType::Post {
                return true;
            }
            let seat = action.seat;
            let next = table.next_to_act();
            if seat != next {
                let path = format!("{at}.seat");
                if !self.blamed.contains(&path) {
                    self.push(
                        path,
                        Rule::Order,
                        format!("seat {seat} acts out of turn; seat {next} is next to act"),
                    );
                }
                return false;
            }

            self.check_amount(table, at, action);
            let (play, repaired) = self.choose(table, at, action);
            if repaired && !self.repaired {
                self.repaired = true;
                self.push(at.to_string(), Rule::Legality, REPAIRED.to_string());
            }
            if play == PlayerAction::Fold
                && let Some(folder) = table.seats.get_seat(seat)
            {
                self.folded_bet = self.folded_bet.max(folder.player.bet);
            }
            if let Err(error) = table.apply_action(seat, play) {
                self.push(
                    at.to_string(),
                    Rule::Legality,
                    format!("the engine cannot apply {play} for seat {seat}: {error}"),
                );
                return false;
            }
            true
        }

        /// Amount rule: a whole number of chips, and, until the first repair,
        /// the engine's amount.
        fn check_amount(&mut self, table: &Table, at: &str, action: &Action) {
            let Some(amount) = action.amount else {
                return;
            };
            let path = format!("{at}.amount");
            if !is_whole_chips(amount) {
                self.push(
                    path,
                    Rule::Amount,
                    format!("records {amount}, which is not a whole number of chips"),
                );
                return;
            }
            if self.repaired {
                return;
            }
            let Some(mismatch) = amount_mismatch(table, action) else {
                return;
            };
            let (recorded, engine) = (mismatch.recorded, mismatch.engine);
            let Some(player) = table.seats.get_seat(action.seat).map(|seat| &seat.player) else {
                return;
            };
            let behind = player.chips;
            let total = player.bet + engine;
            let message = match action.action {
                // More than the stack can put in, under either reading.
                ActionType::Call if recorded > player.total_chip_count() && action.all_in != Some(true) => {
                    if table.to_call(action.seat) > behind {
                        format!(
                            "calls {recorded} with {behind} behind; a call the stack cannot cover is all in for {engine}"
                        )
                    } else {
                        format!("calls {recorded} with {behind} behind; the call adds {engine}")
                    }
                }
                ActionType::Call => format!(
                    "calls {recorded}, which is neither the {engine} the call adds nor the street total of {total} after it"
                ),
                ActionType::AllIn => format!(
                    "goes all in for {recorded}, but the engine's all-in is to {engine}; an all-in records the street total"
                ),
                _ => format!(
                    "{} {recorded} with {} in all; a bet the stack cannot cover is all in for {engine}",
                    if action.action == ActionType::Bet {
                        "bets"
                    } else {
                        "raises to"
                    },
                    player.total_chip_count()
                ),
            };
            self.push(path, Rule::Amount, message);
        }

        /// Legality rule: the action to apply for `action`, and whether it had
        /// to be repaired to be legal.
        fn choose(&mut self, table: &Table, at: &str, action: &Action) -> (PlayerAction, bool) {
            let seat = action.seat;
            let owed = table.to_call(seat);
            match action.action {
                ActionType::Fold => (PlayerAction::Fold, false),
                ActionType::Call => (PlayerAction::Call, false),
                ActionType::AllIn => (PlayerAction::AllIn, false),
                ActionType::Check if owed > 0 => {
                    self.push(
                        at.to_string(),
                        Rule::Legality,
                        format!("checks facing a bet; seat {seat} owes {owed}"),
                    );
                    (PlayerAction::Call, true)
                }
                ActionType::Check | ActionType::Post => (PlayerAction::Check, false),
                ActionType::Bet | ActionType::Raise => self.choose_raise(table, at, action, owed),
            }
        }

        fn choose_raise(&mut self, table: &Table, at: &str, action: &Action, owed: usize) -> (PlayerAction, bool) {
            let seat = action.seat;
            let call_or_check = if owed > 0 {
                PlayerAction::Call
            } else {
                PlayerAction::Check
            };
            let minimum = table
                .legal_actions(seat)
                .into_iter()
                .find(|legal| matches!(legal, PlayerAction::Bet(_) | PlayerAction::Raise(_)));
            let Some(recorded) = action.amount.map(whole_chips) else {
                self.push(
                    format!("{at}.amount"),
                    Rule::Amount,
                    "a bet or raise records no amount".to_string(),
                );
                return (minimum.unwrap_or(call_or_check), true);
            };
            let stack = table
                .seats
                .get_seat(seat)
                .map_or(0, |seat| seat.player.total_chip_count());
            let (verb, as_recorded) = if action.action == ActionType::Bet {
                ("bets", PlayerAction::Bet(recorded))
            } else {
                ("raises to", PlayerAction::Raise(recorded))
            };
            if recorded > stack {
                // The Amount rule has reported the amount; the stack goes in.
                self.push(
                    at.to_string(),
                    Rule::Legality,
                    format!("{verb} {recorded} with {stack} in all; it is applied as all in for {stack}"),
                );
                return (PlayerAction::AllIn, true);
            }
            if recorded == stack {
                return (as_recorded, false);
            }
            match minimum {
                Some(PlayerAction::Bet(least) | PlayerAction::Raise(least)) if recorded < least => {
                    self.push(
                        at.to_string(),
                        Rule::Legality,
                        format!("{verb} {recorded}; the minimum is {least}, and {recorded} is not all in"),
                    );
                    (minimum.unwrap_or(call_or_check), true)
                }
                Some(_) => (as_recorded, false),
                None => {
                    self.push(
                        at.to_string(),
                        Rule::Legality,
                        format!("{verb} {recorded}, but no raise is open to seat {seat} here"),
                    );
                    (call_or_check, true)
                }
            }
        }
    }

    // ── Result ───────────────────────────────────────────────────────────────

    /// A `results` entry that pkcore can rank: its seat's hole cards on a
    /// five-card board.
    struct Ranked<'a> {
        index: usize,
        entry: &'a ResultEntry,
        eval: Eval,
        /// The seat's starting stack, when the seat is listed.
        stack: Option<f64>,
    }

    /// The seats that fold, by the record's actions.
    fn folded_seats(hand: &HandHistory) -> Vec<u8> {
        street_actions(hand)
            .into_iter()
            .flat_map(|(_, actions)| actions.iter())
            .filter(|action| action.action == ActionType::Fold)
            .map(|action| action.seat)
            .collect()
    }

    /// Result rule, on the record alone (no walk): for every `results` entry
    /// whose seat has hole cards on a five-card board, `best_hand` is a best
    /// five of those seven cards and `hand_rank` is pkcore's rank of it, field
    /// by field. Among the seats that reach showdown (not folded by the
    /// record's actions or `outcome: fold`, not mucked), the best rank is
    /// marked `win` (or `tie` when several share it), and a weaker hand is not
    /// marked `win` or `tie` when a best hand covers its stack, so no side pot
    /// can be its. The best hand must win only when every seat still in is
    /// ranked: a seat without hole cards or result, or a `mucked: true` entry,
    /// turns that check off for the hand.
    ///
    /// Hold'em only: `HandHistory::rank_seven` is a seven-card evaluation. A
    /// hand with a Cards finding is not ranked; its cards are not to be
    /// trusted.
    pub(super) fn results(hand: &HandHistory, findings: &mut Vec<Finding>) {
        if hand.hand.game != HandVariant::Holdem || findings.iter().any(|finding| finding.rule == Rule::Cards) {
            return;
        }
        let (Some(results), Some(board)) = (&hand.results, &hand.board) else {
            return;
        };
        // Folded by the record's actions, or marked `fold` in the results (a
        // history may record the outcome without the action).
        let mut folded = folded_seats(hand);
        folded.extend(
            results
                .iter()
                .filter(|entry| entry.outcome == Outcome::Fold)
                .map(|entry| entry.seat),
        );
        let mut showdown: Vec<Ranked> = Vec::new();
        for (index, entry) in results.iter().enumerate() {
            let player = hand.players.iter().find(|player| player.seat == entry.seat);
            let Some(hole) = player.and_then(|player| player.hole_cards.as_deref()) else {
                continue;
            };
            let Some(eval) = HandHistory::rank_seven(hole, board) else {
                continue;
            };
            check_best_hand(index, entry, hole, board, &eval, findings);
            check_hand_rank(index, entry, &eval, findings);
            if !folded.contains(&entry.seat) && entry.mucked != Some(true) {
                showdown.push(Ranked {
                    index,
                    entry,
                    eval,
                    stack: player.map(|player| player.stack),
                });
            }
        }
        // The best hand is known only when every seat still in has been ranked.
        // A mucked seat is still in but unranked, so a `mucked: true` entry
        // turns the best-hand-must-win check off for the whole hand.
        let everyone_ranked = hand
            .players
            .iter()
            .filter(|player| !folded.contains(&player.seat))
            .all(|player| showdown.iter().any(|ranked| ranked.entry.seat == player.seat));
        check_outcomes(&showdown, everyone_ranked, findings);
    }

    fn check_best_hand(
        index: usize,
        entry: &ResultEntry,
        hole: &str,
        board: &str,
        eval: &Eval,
        findings: &mut Vec<Finding>,
    ) {
        let Some(text) = &entry.best_hand else {
            return;
        };
        let path = format!("results[{index}].best_hand");
        let Ok(five) = Five::from_str(text) else {
            findings.push(Finding {
                path,
                rule: Rule::Result,
                message: format!("{text:?} is not five cards"),
            });
            return;
        };
        // More than one five can be best (a pair from three on the board), so
        // the record's five is checked for where its cards come from and for
        // its rank, not compared card for card with pkcore's.
        let seven = Cards::from_str(&format!("{hole} {board}")).unwrap_or_default();
        let drawn = five.to_arr().iter().all(|card| seven.0.contains(card));
        if drawn && five.hand_rank().value == eval.hand_rank.value {
            return;
        }
        let reason = if drawn {
            format!("ranks {}", five.hand_rank().value)
        } else {
            format!("is not drawn from {hole} and {board}")
        };
        findings.push(Finding {
            path,
            rule: Rule::Result,
            message: format!(
                "records {text}, which {reason}; the best five is {} at {}",
                eval.hand, eval.hand_rank.value
            ),
        });
    }

    fn check_hand_rank(index: usize, entry: &ResultEntry, eval: &Eval, findings: &mut Vec<Finding>) {
        let Some(recorded) = entry.hand_rank else {
            return;
        };
        let engine = eval.hand_rank;
        let five = &eval.hand;
        let mut push = |field: &str, message: String| {
            findings.push(Finding {
                path: format!("results[{index}].hand_rank.{field}"),
                rule: Rule::Result,
                message,
            });
        };
        if recorded.value != engine.value {
            push(
                "value",
                format!(
                    "records {}; pkcore ranks {five} ({:?}) at {}",
                    recorded.value, engine.class, engine.value
                ),
            );
        }
        if recorded.name != engine.name {
            push(
                "name",
                format!("records {:?}; pkcore names {five} {:?}", recorded.name, engine.name),
            );
        }
        if recorded.class != engine.class {
            push(
                "class",
                format!("records {:?}; pkcore classes {five} {:?}", recorded.class, engine.class),
            );
        }
    }

    /// The outcomes of the seats that reach showdown. Lower values are
    /// stronger.
    fn check_outcomes(showdown: &[Ranked], everyone_ranked: bool, findings: &mut Vec<Finding>) {
        let Some(best) = showdown.iter().map(|ranked| ranked.eval.hand_rank.value).min() else {
            return;
        };
        let bests: Vec<&Ranked> = showdown
            .iter()
            .filter(|ranked| ranked.eval.hand_rank.value == best)
            .collect();
        let split = bests.len() > 1;
        for ranked in showdown {
            let path = format!("results[{}].outcome", ranked.index);
            let outcome = &ranked.entry.outcome;
            let (five, value) = (&ranked.eval.hand, ranked.eval.hand_rank.value);
            let takes = matches!(outcome, Outcome::Win | Outcome::Tie);
            if value == best {
                let right = *outcome == Outcome::Win || (split && *outcome == Outcome::Tie);
                if everyone_ranked && !right {
                    findings.push(Finding {
                        path,
                        rule: Rule::Result,
                        message: format!(
                            "is {}, but {five} ({value}) is the best hand at showdown{}",
                            outcome_name(outcome),
                            if split { ", split" } else { "" }
                        ),
                    });
                }
            } else if takes
                && let Some(cover) = bests.iter().find(|winner| {
                    winner
                        .stack
                        .is_some_and(|stack| ranked.stack.is_some_and(|own| stack >= own))
                })
            {
                findings.push(Finding {
                    path,
                    rule: Rule::Result,
                    message: format!(
                        "is {}, but {five} ({value}) loses to seat {}'s {} ({best}), which covers it; lower is stronger",
                        outcome_name(outcome),
                        cover.entry.seat,
                        cover.eval.hand
                    ),
                });
            }
        }
    }

    fn outcome_name(outcome: &Outcome) -> &'static str {
        match outcome {
            Outcome::Win => "win",
            Outcome::Lose => "lose",
            Outcome::Tie => "tie",
            Outcome::Fold => "fold",
        }
    }

    // ── Money ────────────────────────────────────────────────────────────────

    /// Money rule, on the record alone (no walk): a seat loses no more than
    /// its starting stack, and when `results` gives a net for every seat that
    /// put chips in, the nets sum to zero.
    ///
    /// The seats that put chips in are every listed seat when there is an
    /// ante, the blinds the engine posts, and every seat that calls, bets,
    /// raises, goes all in or posts chips.
    pub(super) fn money(hand: &HandHistory, findings: &mut Vec<Finding>) {
        let Some(results) = &hand.results else {
            return;
        };
        for (index, entry) in results.iter().enumerate() {
            if let Some(net) = entry.net
                && let Some(player) = hand.players.iter().find(|player| player.seat == entry.seat)
                && -net > player.stack
            {
                findings.push(Finding {
                    path: format!("results[{index}].net"),
                    rule: Rule::Money,
                    message: format!(
                        "records a loss of {}, more than the {} seat {} started with",
                        -net, player.stack, entry.seat
                    ),
                });
            }
        }
        let net_of = |seat: u8| {
            results
                .iter()
                .find(|entry| entry.seat == seat)
                .and_then(|entry| entry.net)
        };
        let covered = contributors(hand).into_iter().all(|seat| net_of(seat).is_some());
        let sum: f64 = results.iter().filter_map(|entry| entry.net).sum();
        if covered && sum.abs() >= 0.005 {
            findings.push(Finding {
                path: "results".to_string(),
                rule: Rule::Money,
                message: format!("the nets sum to {sum}, not 0; chips are won only from the other seats"),
            });
        }
    }

    /// The seats that put chips in: see [`money`]. Every listed seat when the
    /// blinds cannot be placed.
    fn contributors(hand: &HandHistory) -> Vec<u8> {
        let listed = hand.players.iter().map(|player| player.seat);
        let ante = hand.table.stakes.ante.is_some_and(|ante| ante > 0.0);
        let mut seats: Vec<u8> = match hand.replay_table() {
            Ok(table) if !ante => listed
                .filter(|&seat| table.seats.get_seat(seat).is_some_and(|seat| seat.player.bet > 0))
                .collect(),
            _ => listed.collect(),
        };
        for (_, actions) in street_actions(hand) {
            for action in actions {
                let moves_chips = match action.action {
                    ActionType::Call | ActionType::Bet | ActionType::Raise | ActionType::AllIn => true,
                    ActionType::Post => action.amount.is_some_and(|amount| amount > 0.0),
                    ActionType::Fold | ActionType::Check => false,
                };
                if moves_chips && !seats.contains(&action.seat) {
                    seats.push(action.seat);
                }
            }
        }
        seats
    }

    /// Money rule, against the engine: when the walk applied every recorded
    /// action without a repair or an Amount finding, each recorded `net` is
    /// the seat's chips after
    /// the engine's showdown, on a copy of the walked table, less its starting
    /// stack. Not checked when the record stops before the hand is over, or
    /// when a seat still in at showdown has no hole cards or the board is not
    /// complete, since then the engine cannot know who won.
    #[allow(clippy::cast_precision_loss)]
    pub(super) fn walked_money(hand: &HandHistory, walk: &Walk, findings: &mut Vec<Finding>) {
        let (Some(results), Some(walked)) = (&hand.results, &walk.table) else {
            return;
        };
        if walk.repaired || !walk.complete || walk.amount_found {
            return;
        }
        let mut table = walked.clone();
        let still_in = table.seats.active_in_hand();
        let has_cards = |seat: u8| {
            hand.players
                .iter()
                .any(|player| player.seat == seat && player.hole_cards.is_some())
        };
        if still_in.len() > 1 && (table.board.len() < 5 || !still_in.iter().all(|&seat| has_cards(seat))) {
            return;
        }
        if table.showdown().is_err() {
            return;
        }
        for (index, entry) in results.iter().enumerate() {
            let Some(net) = entry.net else {
                continue;
            };
            let Some(player) = hand.players.iter().find(|player| player.seat == entry.seat) else {
                continue;
            };
            let Some(seat) = table.seats.get_seat(entry.seat) else {
                continue;
            };
            let engine = seat.player.chips as f64 - player.stack;
            if (net - engine).abs() >= 0.005 {
                findings.push(Finding {
                    path: format!("results[{index}].net"),
                    rule: Rule::Money,
                    message: format!(
                        "records a net of {net}; the engine's is {engine}, from {} to {} chips",
                        player.stack, seat.player.chips
                    ),
                });
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_history__check_tests {
    use super::*;
    #[cfg(feature = "hand-histories")]
    use crate::hand_history::HandHistory;

    /// The two-player hand from the `replay_with` doc test.
    #[cfg(feature = "hand-histories")]
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

    #[cfg(feature = "hand-histories")]
    fn hand(yaml: &str) -> HandHistory {
        HandHistory::from_yaml(yaml).unwrap()
    }

    /// `hand`'s findings less the one [`Rule::Incomplete`] finding. Many
    /// fixtures here are partial records on purpose, stopping once their rule
    /// has something to say; this keeps each test on its own rule. The
    /// Incomplete rule has tests of its own.
    #[cfg(feature = "hand-histories")]
    fn partial(hand: &HandHistory) -> Vec<Finding> {
        let findings = hand.validate();
        let incomplete = findings
            .iter()
            .filter(|finding| finding.rule == Rule::Incomplete)
            .count();
        assert!(incomplete <= 1, "{findings:?}");
        findings
            .into_iter()
            .filter(|finding| finding.rule != Rule::Incomplete)
            .collect()
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__clean_heads_up_hand_has_no_findings() {
        assert_eq!(Vec::<Finding>::new(), hand(CLEAN).validate());
    }

    #[cfg(feature = "hand-histories")]
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
        let findings = partial(&hand(yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Cards, findings[0].rule);
        assert_eq!("board", findings[0].path);
        assert!(findings[0].message.contains("A♠"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__duplicate_between_two_hole_cards_names_the_later_player() {
        let yaml = r"
hand: { id: dup2, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0, hole_cards: 'A♠ K♠' }
  - { seat: 1, name: B, stack: 1000.0, hole_cards: 'K♠ J♦' }
";
        let findings = partial(&hand(yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!("players[1].hole_cards", findings[0].path);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__unparseable_card_is_a_cards_finding() {
        let yaml = r"
hand: { id: bad, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0, hole_cards: 'A♠ Zz' }
  - { seat: 1, name: B, stack: 1000.0 }
";
        let findings = partial(&hand(yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Cards, findings[0].rule);
        assert_eq!("players[0].hole_cards", findings[0].path);
    }

    #[cfg(feature = "hand-histories")]
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
        let findings = partial(&hand(yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Cards, findings[0].rule);
        assert_eq!("board", findings[0].path);
        assert!(
            findings[0].message.contains("streets.turn.card"),
            "{}",
            findings[0].message
        );
    }

    #[cfg(feature = "hand-histories")]
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
        assert_eq!(Vec::<Finding>::new(), partial(&hand(yaml)));
    }

    #[cfg(feature = "hand-histories")]
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
        let findings = partial(&hand(yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Cards, findings[0].rule);
        assert_eq!("board", findings[0].path);
    }

    #[cfg(feature = "hand-histories")]
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

    #[cfg(feature = "hand-histories")]
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

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__seat_equal_to_table_seats_is_off_by_one() {
        let yaml = r"
hand: { id: off, game: holdem }
table: { seats: 6, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 6, name: B, stack: 1000.0 }
";
        let findings = partial(&hand(yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Seats, findings[0].rule);
        assert_eq!("players[1].seat", findings[0].path);
        assert!(findings[0].message.contains("0-based"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
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

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__button_outside_the_table_is_a_seats_finding() {
        let yaml = r"
hand: { id: button, game: holdem }
table: { seats: 6, button: 6, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
";
        let findings = partial(&hand(yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Seats, findings[0].rule);
        assert_eq!("table.button", findings[0].path);
        assert!(findings[0].message.contains("0-based"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
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

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__zero_or_amountless_post_by_an_unlisted_seat_is_not_a_finding() {
        // The engine places the small blind on seat 0 when a dead SB wraps
        // (suspected engine defect, docs/TECHNICAL_DEBT.md), so the
        // walk reports seat 1's fold as out of turn. This test is about the
        // Seats rule only.
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
        assert!(with_rule(&findings, Rule::Seats).is_empty(), "{findings:?}");
    }

    #[cfg(feature = "hand-histories")]
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

    #[cfg(feature = "hand-histories")]
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

    #[cfg(feature = "hand-histories")]
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

    #[cfg(feature = "hand-histories")]
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

    #[cfg(feature = "hand-histories")]
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

    /// The findings with `rule`, as `path: message` lines for a failure message.
    #[cfg(feature = "hand-histories")]
    fn with_rule(findings: &[Finding], rule: Rule) -> Vec<&Finding> {
        findings.iter().filter(|finding| finding.rule == rule).collect()
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__out_of_turn_is_an_order_finding() {
        // Heads-up the button posts the small blind and acts first pre-flop.
        let yaml = r"
hand: { id: order, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 1, action: raise, amount: 300.0 }
      - { seat: 0, action: fold }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Order, findings[0].rule);
        assert_eq!("streets.preflop.actions[0].seat", findings[0].path);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__raise_below_minimum_is_a_legality_finding() {
        // The turn of The Hand: a bet of 52,000 makes the minimum raise 104,000.
        // The raise to 100,000 is repaired to 104,000. From there the engine's
        // numbers are not the record's, so the call is not compared with them,
        // but its non-integral amount is wrong by the record alone.
        let findings = hand(&min_raise_then_call("100000.5")).validate();
        let legality = with_rule(&findings, Rule::Legality);
        assert!(
            legality
                .iter()
                .any(|finding| finding.path == "streets.flop.actions[1]" && finding.message.contains("104000")),
            "{findings:?}"
        );
        assert!(
            legality.iter().any(|finding| finding.path == "streets.flop.actions[1]"
                && finding.message == "pots, nets and amounts after this point are not checked"),
            "{findings:?}"
        );
        let amount = with_rule(&findings, Rule::Amount);
        assert_eq!(1, amount.len(), "{findings:?}");
        assert_eq!("streets.flop.actions[2].amount", amount[0].path);
        assert!(
            amount[0].message.contains("not a whole number"),
            "{}",
            amount[0].message
        );
        assert!(
            amount
                .iter()
                .all(|finding| !finding.message.contains("104000") && !finding.message.contains("52000")),
            "{findings:?}"
        );
    }

    /// Heads-up at 400/800: on the flop seat 1 bets 52,000 and seat 0 raises to
    /// 100,000, below the 104,000 minimum; seat 1 then calls `call`.
    #[cfg(feature = "hand-histories")]
    fn min_raise_then_call(call: &str) -> String {
        format!(
            "
hand: {{ id: minraise, game: holdem }}
table: {{ stakes: {{ small_blind: 400.0, big_blind: 800.0 }} }}
players:
  - {{ seat: 0, name: A, stack: 1000000.0 }}
  - {{ seat: 1, name: B, stack: 1000000.0 }}
streets:
  preflop:
    actions:
      - {{ seat: 0, action: call, amount: 400.0 }}
      - {{ seat: 1, action: check }}
  flop:
    cards: '9♣ 6♦ 5♥'
    actions:
      - {{ seat: 1, action: bet, amount: 52000.0 }}
      - {{ seat: 0, action: raise, amount: 100000.0 }}
      - {{ seat: 1, action: call, amount: {call} }}
"
        )
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__repair_sets_repaired_and_suppresses_later_amounts() {
        // The call of 100,000 matches neither 52,000 added nor 104,000 in all,
        // but after the repair the engine's numbers are not the record's.
        let history = hand(&min_raise_then_call("100000.0"));
        let mut findings = Vec::new();
        let walk = super::passes::walk(&history, &mut findings);
        assert!(walk.repaired);
        assert!(with_rule(&findings, Rule::Amount).is_empty(), "{findings:?}");
        assert!(with_rule(&history.validate(), Rule::Amount).is_empty());
    }

    /// Heads-up at 50/100 with 1,000 each: seat 0 acts first with `first`, and
    /// seat 1 answers with `second`.
    #[cfg(feature = "hand-histories")]
    fn heads_up(first: &str, second: &str) -> String {
        format!(
            "
hand: {{ id: headsup, game: holdem }}
table: {{ stakes: {{ small_blind: 50.0, big_blind: 100.0 }} }}
players:
  - {{ seat: 0, name: A, stack: 1000.0 }}
  - {{ seat: 1, name: B, stack: 1000.0 }}
streets:
  preflop:
    actions:
      - {{ seat: 0, {first} }}
      - {{ seat: 1, {second} }}
"
        )
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__bet_larger_than_the_stack_is_applied_all_in_and_repaired() {
        // Seat 0 has 1,000 in all and raises to 5,000: applied as all in for
        // 1,000. Seat 1's call of 777 is then not compared with the engine.
        let history = hand(&heads_up(
            "action: raise, amount: 5000.0",
            "action: call, amount: 777.0",
        ));
        let mut findings = Vec::new();
        let walk = super::passes::walk(&history, &mut findings);
        assert!(walk.repaired);
        let legality = with_rule(&findings, Rule::Legality);
        assert!(
            legality
                .iter()
                .any(|finding| finding.path == "streets.preflop.actions[0]" && finding.message.contains("all in")),
            "{findings:?}"
        );
        assert!(
            legality
                .iter()
                .any(|finding| finding.message == "pots, nets and amounts after this point are not checked"),
            "{findings:?}"
        );
        assert!(
            findings
                .iter()
                .all(|finding| !finding.path.starts_with("streets.preflop.actions[1]")),
            "{findings:?}"
        );
        let table = walk.table.expect("the walk built a table");
        assert!(table.seats.get_seat(0).is_some_and(|seat| seat.player.is_all_in()));
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__all_in_amount_mismatch_is_an_amount_finding() {
        // Seat 0 has 1,000 in all; an all-in records the street total.
        let findings = hand(&heads_up("action: all_in, amount: 900.0", "action: fold")).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Amount, findings[0].rule);
        assert_eq!("streets.preflop.actions[0].amount", findings[0].path);
        assert!(findings[0].message.contains("900"), "{}", findings[0].message);
        assert!(findings[0].message.contains("1000"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn replay__rejects_an_all_in_amount_mismatch() {
        let history = hand(&heads_up("action: all_in, amount: 900.0", "action: fold"));
        assert_eq!(
            Err(crate::PKError::ReplayAmountMismatch {
                seat: 0,
                recorded: 900,
                engine: 1000,
            }),
            history.replay()
        );
    }

    /// Heads-up at 400/800: the flop goes bet 8,000, raise to 26,000, call. The
    /// call adds 18,000 and brings the caller's street total to 26,000.
    #[cfg(feature = "hand-histories")]
    fn flop_call(amount: &str) -> String {
        format!(
            "
hand: {{ id: flopcall, game: holdem }}
table: {{ stakes: {{ small_blind: 400.0, big_blind: 800.0 }} }}
players:
  - {{ seat: 0, name: A, stack: 1000000.0 }}
  - {{ seat: 1, name: B, stack: 1000000.0 }}
streets:
  preflop:
    actions:
      - {{ seat: 0, action: call, amount: 400.0 }}
      - {{ seat: 1, action: check }}
  flop:
    cards: '9♣ 6♦ 5♥'
    actions:
      - {{ seat: 1, action: bet, amount: 8000.0 }}
      - {{ seat: 0, action: raise, amount: 26000.0 }}
      - {{ seat: 1, action: call, amount: {amount} }}
"
        )
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__call_recorded_as_added_or_total_is_accepted() {
        for amount in ["18000.0", "26000.0"] {
            let findings = partial(&hand(&flop_call(amount)));
            assert_eq!(Vec::<Finding>::new(), findings, "call {amount}");
        }
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__call_matching_neither_reading_is_an_amount_finding() {
        let findings = partial(&hand(&flop_call("20000.0")));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Amount, findings[0].rule);
        assert_eq!("streets.flop.actions[2].amount", findings[0].path);
        assert!(findings[0].message.contains("18000"), "{}", findings[0].message);
        assert!(findings[0].message.contains("26000"), "{}", findings[0].message);
    }

    /// Seat 1 has 49,200 behind once the big blind is posted, and records a
    /// call of 100,000 without marking it all in.
    #[cfg(feature = "hand-histories")]
    const UNAFFORDABLE_CALL: &str = r"
hand: { id: short, game: holdem }
table: { stakes: { small_blind: 400.0, big_blind: 800.0 } }
players:
  - { seat: 0, name: A, stack: 1000000.0 }
  - { seat: 1, name: B, stack: 50000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: raise, amount: 100000.0 }
      - { seat: 1, action: call, amount: 100000.0 }
";

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__unaffordable_call_is_an_amount_finding() {
        let findings = partial(&hand(UNAFFORDABLE_CALL));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Amount, findings[0].rule);
        assert_eq!("streets.preflop.actions[1].amount", findings[0].path);
        assert!(findings[0].message.contains("100000"), "{}", findings[0].message);
        assert!(findings[0].message.contains("49200"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__fractional_amount_is_an_amount_finding() {
        let yaml = r"
hand: { id: half, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: call, amount: 0.5 }
      - { seat: 1, action: check }
";
        let findings = partial(&hand(yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Amount, findings[0].rule);
        assert_eq!("streets.preflop.actions[0].amount", findings[0].path);
        assert!(findings[0].message.contains("0.5"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__unlisted_actor_is_seated_so_the_walk_goes_on() {
        // Seat 2 is not listed. It is seated with the largest listed stack, so
        // its call, which matches neither reading (200 added, 300 in all), is
        // still reported.
        let yaml = r"
hand: { id: seated, game: holdem }
table: { button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 5000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: raise, amount: 300.0 }
      - { seat: 1, action: fold }
      - { seat: 2, action: call, amount: 250.0 }
";
        let findings = partial(&hand(yaml));
        assert_eq!(2, findings.len(), "{findings:?}");
        assert_eq!(Rule::Seats, findings[0].rule);
        assert_eq!("streets.preflop.actions[2].seat", findings[0].path);
        assert_eq!(Rule::Amount, findings[1].rule);
        assert_eq!("streets.preflop.actions[2].amount", findings[1].path);
        assert!(findings[1].message.contains("200"), "{}", findings[1].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__check_facing_a_bet_is_a_legality_finding() {
        let yaml = r"
hand: { id: check, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: raise, amount: 300.0 }
      - { seat: 1, action: check }
";
        let findings = hand(yaml).validate();
        let legality = with_rule(&findings, Rule::Legality);
        assert!(
            legality
                .iter()
                .any(|finding| finding.path == "streets.preflop.actions[1]"),
            "{findings:?}"
        );
    }

    // ── Pot ──────────────────────────────────────────────────────────────────

    /// Six seats at 400/800 with a 200 ante, button on seat 0: seats 1 and 2
    /// post the blinds and fold, seat 4 raises to 3,500 and the button calls.
    /// The pot is 2 × 3,500 + 400 + 800 + 6 × 200 = 9,400. `pot` is the
    /// recorded preflop pot.
    #[cfg(feature = "hand-histories")]
    fn blinds_fold(pot: &str) -> String {
        format!(
            "
hand: {{ id: blinds, game: holdem }}
table: {{ seats: 6, button: 0, stakes: {{ small_blind: 400.0, big_blind: 800.0, ante: 200.0 }} }}
players:
  - {{ seat: 0, name: A, stack: 100000.0 }}
  - {{ seat: 1, name: B, stack: 100000.0 }}
  - {{ seat: 2, name: C, stack: 100000.0 }}
  - {{ seat: 3, name: D, stack: 100000.0 }}
  - {{ seat: 4, name: E, stack: 100000.0 }}
  - {{ seat: 5, name: F, stack: 100000.0 }}
streets:
  preflop:
    actions:
      - {{ seat: 3, action: fold }}
      - {{ seat: 4, action: raise, amount: 3500.0 }}
      - {{ seat: 5, action: fold }}
      - {{ seat: 0, action: call, amount: 3500.0 }}
      - {{ seat: 1, action: fold }}
      - {{ seat: 2, action: fold }}
    pot: {pot}
"
        )
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__street_pot_including_blinds_and_antes_is_clean() {
        assert_eq!(Vec::<Finding>::new(), partial(&hand(&blinds_fold("9400.0"))));
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__street_pot_excluding_blinds_is_a_pot_finding() {
        // DEFECT_040 item 6: 8,200 is the raise, the call and the antes, and
        // leaves out the blinds of the two seats that folded.
        let findings = partial(&hand(&blinds_fold("8200.0")));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Pot, findings[0].rule);
        assert_eq!("streets.preflop.pot", findings[0].path);
        assert!(findings[0].message.contains("9400"), "{}", findings[0].message);
        assert!(findings[0].message.contains("8200"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__pots_after_a_repair_are_not_checked() {
        // The preflop pot (1,600) is wrong and is checked. The flop's raise
        // is repaired, so the flop's pot is not compared with the engine's.
        let yaml = min_raise_then_call("104000.0")
            .replace(
                "      - { seat: 1, action: check }\n",
                "      - { seat: 1, action: check }\n    pot: 1500.0\n",
            )
            .replace(
                "      - { seat: 1, action: call, amount: 104000.0 }\n",
                "      - { seat: 1, action: call, amount: 104000.0 }\n    pot: 1.0\n",
            );
        let findings = hand(&yaml).validate();
        let pot = with_rule(&findings, Rule::Pot);
        assert_eq!(1, pot.len(), "{findings:?}");
        assert_eq!("streets.preflop.pot", pot[0].path);
        assert!(pot[0].message.contains("1600"), "{}", pot[0].message);
    }

    /// The clean heads-up hand with a preflop `pot` and results that give
    /// seat 0 `net0` and seat 1 `net1`.
    #[cfg(feature = "hand-histories")]
    fn folded_with_results(pot: &str, net0: &str, net1: &str) -> String {
        format!(
            "{CLEAN}    pot: {pot}
results:
  - {{ seat: 0, outcome: win, net: {net0} }}
  - {{ seat: 1, outcome: fold, net: {net1} }}
"
        )
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__true_record_of_a_folded_hand_is_clean() {
        // The pot with the uncalled raise (400), or the pot as the engine's
        // log writes it when a fold ends the hand (100): both are accepted.
        for pot in ["400.0", "100.0"] {
            let findings = hand(&folded_with_results(pot, "100.0", "-100.0")).validate();
            assert_eq!(Vec::<Finding>::new(), findings, "pot {pot}");
        }
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__pot_of_a_folded_hand_matching_neither_reading_is_a_pot_finding() {
        let findings = hand(&folded_with_results("300.0", "100.0", "-100.0")).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Pot, findings[0].rule);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__fold_out_pot_after_uncalled_bet_returned_passes() {
        // Seat 0 raises to 300 and the big blind folds: the full pot is 400,
        // 200 once the uncalled 200 is returned, 100 as the engine logs it.
        for pot in ["400.0", "200.0", "100.0"] {
            let findings = hand(&folded_with_results(pot, "100.0", "-100.0")).validate();
            assert_eq!(Vec::<Finding>::new(), findings, "pot {pot}");
        }
        let findings = hand(&folded_with_results("250.0", "100.0", "-100.0")).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Pot, findings[0].rule);
        assert!(findings[0].message.contains("250"), "{}", findings[0].message);
        assert!(findings[0].message.contains("400"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__amount_finding_does_not_cascade_into_money() {
        // The small blind's call of 75 is neither the 50 it adds nor the 100
        // it brings its total to. The pot and nets follow the record's 75, so
        // the engine's 400 and +/-100 would only repeat that one fault.
        let yaml = r"
hand: { id: cascade, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: call, amount: 75.0 }
      - { seat: 1, action: raise, amount: 300.0 }
      - { seat: 0, action: fold }
    pot: 375.0
results:
  - { seat: 0, outcome: fold, net: -75.0 }
  - { seat: 1, outcome: win, net: 75.0 }
";
        let findings = hand(yaml).validate();
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Amount, findings[0].rule);
        assert_eq!("streets.preflop.actions[0].amount", findings[0].path);
    }

    // ── Result ───────────────────────────────────────────────────────────────

    /// The Hand's cards heads-up, no action: seat 0 holds sixes full of fives
    /// (271), seat 1 quad fives with a nine kicker (124). `results` is
    /// appended as is.
    #[cfg(feature = "hand-histories")]
    fn quads_and_boat(results: &str) -> String {
        format!(
            "
hand: {{ id: quads, game: holdem }}
table: {{ stakes: {{ small_blind: 50.0, big_blind: 100.0 }} }}
players:
  - {{ seat: 0, name: Boat, stack: 1000.0, hole_cards: '6♠ 6♥' }}
  - {{ seat: 1, name: Quads, stack: 1000.0, hole_cards: '5♦ 5♣' }}
board: '9♣ 6♦ 5♥ 5♠ 8♠'
results:
{results}"
        )
    }

    /// The `results` entry for seat 0 (the boat) with `outcome`.
    #[cfg(feature = "hand-histories")]
    fn boat(outcome: &str) -> String {
        format!(
            "  - {{ seat: 0, best_hand: '6♠ 6♥ 6♦ 5♥ 5♠', outcome: {outcome}, hand_rank: {{ value: 271, name: FullHouse, class: SixesOverFives }} }}\n"
        )
    }

    /// The `results` entry for seat 1 (the quads) with `value` and `outcome`.
    #[cfg(feature = "hand-histories")]
    fn quads(value: &str, outcome: &str) -> String {
        format!(
            "  - {{ seat: 1, best_hand: '5♥ 5♠ 5♦ 5♣ 9♣', outcome: {outcome}, hand_rank: {{ value: {value}, name: FourOfAKind, class: FourFives }} }}\n"
        )
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__true_results_are_clean() {
        let yaml = quads_and_boat(&format!("{}{}", boat("lose"), quads("124", "win")));
        assert_eq!(Vec::<Finding>::new(), partial(&hand(&yaml)));
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__winner_with_weaker_rank_is_a_result_finding() {
        // DEFECT_040 item 2: the full house (271) is marked the winner over
        // the quads (124). Lower is stronger.
        let yaml = quads_and_boat(&format!("{}{}", boat("win"), quads("124", "lose")));
        let findings = partial(&hand(&yaml));
        assert!(!findings.is_empty(), "{findings:?}");
        assert!(
            findings.iter().all(|finding| finding.rule == Rule::Result),
            "{findings:?}"
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.path == "results[1].outcome" && finding.message.contains("124")),
            "{findings:?}"
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.path == "results[0].outcome" && finding.message.contains("271")),
            "{findings:?}"
        );
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__rank_value_not_matching_class_is_a_result_finding() {
        // DEFECT_040 item 3: 117 is in the FourSixes range; quad fives with a
        // nine kicker is 124.
        let yaml = quads_and_boat(&format!("{}{}", boat("lose"), quads("117", "win")));
        let findings = partial(&hand(&yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Result, findings[0].rule);
        assert_eq!("results[1].hand_rank.value", findings[0].path);
        assert!(findings[0].message.contains("124"), "{}", findings[0].message);
        assert!(findings[0].message.contains("117"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__best_hand_not_the_best_five_is_a_result_finding() {
        let yaml = quads_and_boat(&format!(
            "{}{}",
            boat("lose").replace("6♠ 6♥ 6♦ 5♥ 5♠", "6♠ 6♥ 6♦ 9♣ 8♠"),
            quads("124", "win")
        ));
        let findings = hand(&yaml).validate();
        assert!(
            findings
                .iter()
                .any(|finding| finding.rule == Rule::Result && finding.path == "results[0].best_hand"),
            "{findings:?}"
        );
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__split_pot_marks_both_best_hands_win() {
        // The board plays: a royal flush for both.
        let yaml = r"
hand: { id: split, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0, hole_cards: '2♣ 3♦' }
  - { seat: 1, name: B, stack: 1000.0, hole_cards: '4♣ 5♦' }
board: 'A♠ K♠ Q♠ J♠ T♠'
results:
  - { seat: 0, outcome: win }
  - { seat: 1, outcome: tie }
";
        assert_eq!(Vec::<Finding>::new(), partial(&hand(yaml)));
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__fold_outcome_in_results_is_not_ranked_at_showdown() {
        // No streets: seat 1's fold is in the results only. Its quads are
        // stronger, but it is not at showdown.
        let yaml = quads_and_boat("  - { seat: 0, outcome: win }\n  - { seat: 1, outcome: fold }\n");
        let findings = hand(&yaml).validate();
        assert!(with_rule(&findings, Rule::Result).is_empty(), "{findings:?}");
    }

    // ── Money ────────────────────────────────────────────────────────────────

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__net_larger_than_stack_is_a_money_finding() {
        // DEFECT_040 item 5: a seat cannot lose more than it sat down with.
        let yaml = r"
hand: { id: net, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
results:
  - { seat: 0, outcome: lose, net: -1500.0 }
  - { seat: 1, outcome: win, net: 1500.0 }
";
        let findings = partial(&hand(yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Money, findings[0].rule);
        assert_eq!("results[0].net", findings[0].path);
        assert!(findings[0].message.contains("1500"), "{}", findings[0].message);
        assert!(findings[0].message.contains("1000"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__nets_that_do_not_sum_to_zero_are_a_money_finding() {
        let yaml = r"
hand: { id: sum, game: holdem }
table: { stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
results:
  - { seat: 0, outcome: lose, net: -500.0 }
  - { seat: 1, outcome: win, net: 400.0 }
";
        let findings = partial(&hand(yaml));
        assert_eq!(1, findings.len(), "{findings:?}");
        assert_eq!(Rule::Money, findings[0].rule);
        assert_eq!("results", findings[0].path);
        assert!(findings[0].message.contains("-100"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__net_not_the_engines_is_a_money_finding() {
        // Seat 0 wins seat 1's big blind: +100 and -100, not +200 and -200.
        let findings = hand(&folded_with_results("400.0", "200.0", "-200.0")).validate();
        let money = with_rule(&findings, Rule::Money);
        assert_eq!(2, money.len(), "{findings:?}");
        assert_eq!("results[0].net", money[0].path);
        assert!(money[0].message.contains("the engine's is 100"), "{}", money[0].message);
        assert_eq!("results[1].net", money[1].path);
        assert_eq!(2, findings.len(), "{findings:?}");
    }

    // ── validate ⇒ cursor (EPIC-93 final review I3) ─────────────────────────

    /// Asserts `findings` hold a `rule` finding at `path`, and that the hand
    /// it came from has no cursor: a clean `validate` would have lied.
    #[cfg(feature = "hand-histories")]
    fn assert_refused(yaml: &str, rule: Rule, path: &str) -> Vec<Finding> {
        let hand = hand(yaml);
        let findings = hand.validate();
        assert!(
            findings
                .iter()
                .any(|finding| finding.rule == rule && finding.path == path),
            "no {rule} finding at {path}: {findings:?}"
        );
        assert!(hand.cursor().is_err(), "the cursor builds, so the finding is wrong");
        findings
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__board_only_all_in_is_incomplete() {
        let findings = assert_refused(
            r"
hand: { id: board-only, game: holdem }
table: { seats: 2, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0, hole_cards: '7♦ 2♣' }
  - { seat: 1, name: B, stack: 1000.0, hole_cards: 'A♠ A♥' }
board: 'K♠ K♣ 9♦ 8♠ 4♥'
streets:
  preflop:
    actions:
      - { seat: 0, action: all_in, amount: 1000.0, all_in: true }
      - { seat: 1, action: call, amount: 900.0 }
",
            Rule::Incomplete,
            "streets.preflop",
        );
        assert_eq!(1, findings.len(), "{findings:?}");
        assert!(findings[0].message.contains("2 seats"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__record_that_stops_mid_flop_is_incomplete() {
        let findings = assert_refused(
            r"
hand: { id: mid-flop, game: holdem }
table: { seats: 2, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: call, amount: 50.0 }
      - { seat: 1, action: check }
  flop:
    cards: 'K♠ K♣ 9♦'
    actions:
      - { seat: 1, action: bet, amount: 100.0 }
",
            Rule::Incomplete,
            "streets.flop",
        );
        assert_eq!(1, findings.len(), "{findings:?}");
        assert!(findings[0].message.contains("seat 0 to act"), "{}", findings[0].message);
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__zero_stack_in_the_blind_is_a_seats_finding() {
        let findings = assert_refused(
            r"
hand: { id: zero-stack, game: holdem }
table: { seats: 3, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
  - { seat: 2, name: C, stack: 0.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: fold }
      - { seat: 1, action: fold }
",
            Rule::Seats,
            "players",
        );
        assert_eq!(1, findings.len(), "{findings:?}");
        assert!(
            findings[0].message.contains("InsufficientChips"),
            "{}",
            findings[0].message
        );
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__nan_negative_and_infinite_stakes_and_stacks_are_findings() {
        let findings = assert_refused(
            r"
hand: { id: nan, game: holdem }
table: { seats: 2, button: 0, stakes: { small_blind: .nan, big_blind: -100.0, ante: .inf } }
players:
  - { seat: 0, name: A, stack: .nan }
  - { seat: 1, name: B, stack: -5.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: raise, amount: .inf }
      - { seat: 1, action: fold }
",
            Rule::Amount,
            "table.stakes.small_blind",
        );
        for (rule, path) in [
            (Rule::Amount, "table.stakes.big_blind"),
            (Rule::Amount, "table.stakes.ante"),
            (Rule::Seats, "players[0].stack"),
            (Rule::Seats, "players[1].stack"),
        ] {
            assert!(
                findings
                    .iter()
                    .any(|finding| finding.rule == rule && finding.path == path),
                "no {rule} finding at {path}: {findings:?}"
            );
        }
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__stakes_the_engine_cannot_post_are_an_amount_finding() {
        // A negative big blind with whole, positive stacks: the stakes are at
        // fault, so the finding is at `table.stakes`.
        let findings = hand(
            r"
hand: { id: negative-blind, game: holdem }
table: { seats: 2, button: 0, stakes: { small_blind: 50.0, big_blind: -100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: fold }
",
        )
        .validate();
        assert!(
            findings
                .iter()
                .any(|finding| finding.rule == Rule::Amount && finding.path == "table.stakes.big_blind"),
            "{findings:?}"
        );
    }

    #[cfg(feature = "hand-histories")]
    #[test]
    fn validate__finished_hands_are_not_incomplete() {
        // A fold-out ends when one seat is left; a showdown after the river.
        let checked_down = r"
hand: { id: checked-down, game: holdem }
table: { seats: 2, button: 0, stakes: { small_blind: 50.0, big_blind: 100.0 } }
players:
  - { seat: 0, name: A, stack: 1000.0 }
  - { seat: 1, name: B, stack: 1000.0 }
streets:
  preflop:
    actions:
      - { seat: 0, action: call, amount: 50.0 }
      - { seat: 1, action: check }
  flop:
    cards: 'K♠ K♣ 9♦'
    actions:
      - { seat: 1, action: check }
      - { seat: 0, action: check }
  turn:
    card: '8♠'
    actions:
      - { seat: 1, action: check }
      - { seat: 0, action: check }
  river:
    card: '4♥'
    actions:
      - { seat: 1, action: check }
      - { seat: 0, action: check }
";
        for yaml in [CLEAN, checked_down] {
            let findings = hand(yaml).validate();
            assert!(
                findings.iter().all(|finding| finding.rule != Rule::Incomplete),
                "{findings:?}"
            );
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
            (Rule::Incomplete, "incomplete"),
        ];
        for (rule, name) in rules {
            assert_eq!(name, rule.to_string());
        }
    }
}
