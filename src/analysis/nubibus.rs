use crate::games::GamePhase;
use crate::play::board::Board;
use crate::play::hole_cards::HoleCards;
use crate::prelude::{BoxedCards, Card, Cards, ForcedBets, Seats, Table};
use crate::util::Util;
use crate::util::terminal::Terminal;
use crate::{PKError, Pile, Plurable};
use regex::Regex;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::fs;
use std::ops::Index;
use std::path::Path;
use std::str::FromStr;
#[cfg(all(unix, feature = "terminal"))]
use termion::color;
#[cfg(not(all(unix, feature = "terminal")))]
mod color {
    pub struct Fg<T>(pub T);
    impl<T> std::fmt::Display for Fg<T> {
        fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
            Ok(())
        }
    }
    pub struct Cyan;
    pub struct LightBlack;
    pub struct Yellow;
    pub struct LightBlue;
    pub struct LightRed;
    pub struct Green;
    pub struct LightGreen;
    pub struct Magenta;
    pub struct Reset;
}

/// `nūbĭfĭcus , a, um nubes-facio, - producing clouds`
///
/// The name "Nubificus" is derived from the Latin word "nubes," meaning "cloud," and the verb
/// "facio," meaning "to make" or "to produce."
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Nubificus {
    pub pluribus: Pluribus,
    pub table: Table,
    pub queue: VecDeque<PluribusEvent>,
}

impl Nubificus {
    /// Applies one logged Pluribus action to `table`.
    ///
    /// # Errors
    ///
    /// Returns whatever the underlying [`Table`] action returns —
    /// typically `PKError::TableActionOutOfOrder` when `seat_to_act` is not the
    /// seat the table expects, or a betting error when the logged amount is not
    /// legal in the current state.
    ///
    /// A rejected action is not recoverable during replay: the table has
    /// diverged from the log it is reproducing, so every action after this one
    /// is applied to a state the log never described. Discarding the error here
    /// — as this did before `DEFECT_020` — produced a silently wrong hand
    /// rather than a failed one.
    pub fn act(table: &mut Table, action: &PluribusEvent, seat_to_act: u8) -> Result<(), PKError> {
        log::trace!("...Nubificus.act({action}, {seat_to_act});)");

        let _chips_remaining = match action {
            PluribusEvent::Fold => table.act_fold(seat_to_act)?,
            PluribusEvent::Call => table.act_call(seat_to_act)?,
            PluribusEvent::Raise(amount) => {
                let target = Self::street_bet_target(table, seat_to_act, *amount)?;
                table.act_bet(seat_to_act, target)?
            }
        };

        Ok(())
    }

    /// Converts a logged Pluribus raise amount into the street bet target
    /// [`Table::act_bet`] expects.
    ///
    /// Pluribus log amounts are **cumulative per-player totals for the whole
    /// hand**, while `act_bet` takes the bet target for the *current street*.
    /// On the first street with action the two coincide, which is why this went
    /// unnoticed; from the flop on they differ by whatever the player already
    /// put in on earlier streets.
    ///
    /// `STATE:154:fr250ffr1150fc/r2050c/r3750c/r6250f` with payoffs
    /// `3850|-100|0|-3750|0|0` is the worked example: the losing player's
    /// payoff is exactly `-3750`, his last logged number, not the per-street sum
    /// `1150 + 2050 + 3750 = 6950`. Read per-street, the same hand asks a
    /// 10 000-chip stack for 6 950 and then 6 250 more, and the table rightly
    /// answers `PKError::InsufficientChips`. See `DEFECT_021`.
    ///
    /// # Errors
    ///
    /// `PKError::InvalidPluribusIndex` if `seat_number` is not an occupied seat.
    fn street_bet_target(table: &Table, seat_number: u8, logged_amount: usize) -> Result<usize, PKError> {
        let Some(seat) = table.seats.get_seat(seat_number) else {
            return Err(PKError::InvalidPluribusIndex);
        };

        // `chips_in_play` accumulates across the whole hand; `bet` is only the
        // current street, so their difference is what earlier streets took.
        let earlier_streets = seat.player.chips_in_play.saturating_sub(seat.player.bet);

        Ok(logged_amount.saturating_sub(earlier_streets))
    }

    /// # Errors
    ///
    /// Never returns an error today. Its one fallible call — [`Self::ff`] — has
    /// its result discarded, so a replay that diverges reads as success. The
    /// `Result` is kept because propagating is the intended fix, the same
    /// swallowed-error shape `DEFECT_020` closed on [`Nubificus::act`].
    pub fn boop(&mut self) -> Result<(), PKError> {
        let _ = self.ff(1, true);
        match self.queue.pop_front() {
            Some(_) | None => {}
        }
        Ok(())
    }

    /// # Errors
    ///
    /// - Whatever [`Table::act`] propagates while advancing the table —
    ///   `PKError::NotEnoughCards` when the deck cannot cover the street, and
    ///   any error from posting blinds, dealing, or ending the hand.
    /// - Whatever [`Self::do_action`] returns for the first replayed action the
    ///   table rejects.
    ///
    /// Replay stops at that action. The queue is not rewound, so the table and
    /// the log have diverged and the remaining actions are not applied.
    pub fn ff(&mut self, number_of_actions: usize, display: bool) -> Result<(), PKError> {
        self.table.act()?;

        // `PluribusEvent` is `Copy`, so take a snapshot of the actions to
        // replay before handing `&mut self` to `do_action`. The celled engine
        // could iterate and mutate at once; the plain one cannot, and that is
        // the borrow checker doing its job.
        let actions: Vec<PluribusEvent> = self.queue.iter().take(number_of_actions).copied().collect();
        for action in &actions {
            self.do_action(action, display)?;
        }
        Ok(())
    }

    /// # Errors
    ///
    /// - `PKError::NotEnoughCards` or `PKError::InvalidSeatNumber` from
    ///   [`Table::deal_cards_to_seats`] when the hand has not been dealt yet.
    /// - Whatever [`Table::act`] propagates while advancing the table.
    /// - Whatever [`Self::do_action`] returns for the first logged action the
    ///   table rejects; the actions after it are not applied.
    pub fn play_hand(&mut self) -> Result<(), PKError> {
        if !self.table.seats.are_dealt() {
            self.table.deal_cards_to_seats()?;
        }
        self.table.act()?;

        for action in self.pluribus.actions.clone() {
            self.do_action(&action, false)?;
        }
        Ok(())
    }

    /// # Errors
    ///
    /// The same set as [`Self::play_hand`]: dealing errors from
    /// [`Table::deal_cards_to_seats`], anything [`Table::act`] propagates, and
    /// the first rejected action from [`Self::do_action`].
    ///
    /// Output already written to stdout is not withdrawn, so a failed replay
    /// leaves a partial transcript on screen.
    pub fn play_hand_display(&mut self) -> Result<(), PKError> {
        log::trace!("Nubibus.play_hand_display()");
        if !self.table.seats.are_dealt() {
            self.table.deal_cards_to_seats()?;
        }

        // Display header with color
        println!("{}{}{}", color::Fg(color::Cyan), self.pluribus, color::Fg(color::Reset));
        println!(
            "{}{}{}",
            color::Fg(color::LightBlack),
            self.pluribus.raw,
            color::Fg(color::Reset)
        );

        self.table.act()?;

        println!(
            "{}--------------------------------{}",
            color::Fg(color::Yellow),
            color::Fg(color::Reset)
        );
        println!("{}", self.table);

        for (i, action) in self.pluribus.actions.clone().iter().enumerate() {
            println!("#{i} {action}");
            log::trace!("...PluribusEvent #{i}: {action}");
            self.do_action(action, true)?;
        }

        Ok(())
    }

    /// # Errors
    ///
    /// - `PKError::InvalidPluribusIndex` if the seat the table says is next to
    ///   act is not an occupied seat.
    /// - Whatever the matching [`Table`] action rejects the logged move with —
    ///   typically `PKError::TableActionOutOfOrder` when the log and the table
    ///   disagree about who acts, or `PKError::InsufficientChips` on a raise
    ///   the stack cannot cover.
    ///
    /// See [`Nubificus::act`] for why a rejected action ends the replay instead
    /// of being skipped.
    #[allow(clippy::too_many_lines)]
    pub fn do_action(&mut self, action: &PluribusEvent, display: bool) -> Result<(), PKError> {
        let seat_to_act = self.table.next_to_act();
        let handle_to_act = self.table.get_seat_handle(seat_to_act);
        log::debug!("...Nubificus.do_action() {handle_to_act} Seat {seat_to_act} is next to act: {action}");

        Nubificus::act(&mut self.table, action, seat_to_act)?;

        log::debug!(
            "......{}",
            self.table.commentary_last_player_action().unwrap_or_default()
        );
        if display {
            let commentary = self.table.commentary_last_player_action().unwrap_or_default();
            // Color player actions based on action type
            match action {
                PluribusEvent::Fold => {
                    println!(
                        "{}{}{}",
                        color::Fg(color::LightBlack),
                        commentary,
                        color::Fg(color::Reset)
                    );
                }
                PluribusEvent::Call => {
                    println!(
                        "{}{}{}",
                        color::Fg(color::LightBlue),
                        commentary,
                        color::Fg(color::Reset)
                    );
                }
                PluribusEvent::Raise(_) => {
                    println!(
                        "{}{}{}",
                        color::Fg(color::LightRed),
                        commentary,
                        color::Fg(color::Reset)
                    );
                }
            }
        }

        let betting_phase = self.table.determine_betting_phase();
        log::trace!("......Betting phase is {betting_phase}");

        if self.table.is_game_over() {
            log::trace!("......is_game_over() == true");
            let hand_result = self.table.end_hand()?;

            if display {
                println!();
                println!(
                    "{}================================{}",
                    color::Fg(color::Green),
                    color::Fg(color::Reset)
                );
                println!(
                    "{}================================{}",
                    color::Fg(color::Green),
                    color::Fg(color::Reset)
                );
                println!(
                    "{}{}{}",
                    color::Fg(color::LightGreen),
                    hand_result.first(),
                    color::Fg(color::Reset)
                );
                println!("{}", self.pluribus.display_results());
            }
        } else {
            log::trace!("......is_game_over() == false");
            match betting_phase {
                GamePhase::BettingPreFlop if self.table.seats.is_betting_complete() => {
                    log::trace!("......betting_phase == GamePhase::BettingPreFlop && betting is complete");
                    if display {
                        println!("{}", self.table);
                    }
                    self.table.act()?;
                    if display {
                        println!(
                            "\n{}Betting round ends. Dealing the flop...{}",
                            color::Fg(color::Magenta),
                            color::Fg(color::Reset)
                        );
                    }
                    log::debug!("Board: {}", self.table.board);
                    if display {
                        self.table.eval_flop_display();
                        println!(); // TODO: why the spacing issues?
                    }
                }
                GamePhase::BettingFlop if self.table.seats.is_betting_complete() => {
                    log::trace!("......betting_phase == GamePhase::BettingFlop && betting is complete");
                    if display {
                        println!("{}", self.table);
                    }
                    self.table.act()?;
                    if display {
                        println!(
                            "\n{}Betting round ends. Dealing the turn...{}",
                            color::Fg(color::Magenta),
                            color::Fg(color::Reset)
                        );
                    }
                    log::debug!("Board: {}", self.table.board);
                    if display {
                        self.table.eval_turn_display();
                    }
                }
                GamePhase::BettingTurn if self.table.seats.is_betting_complete() => {
                    log::trace!("......betting_phase == GamePhase::BettingTurn && betting is complete");
                    if display {
                        println!("{}", self.table);
                    }
                    self.table.act()?;
                    log::debug!("Board: {}", self.table.board);
                    if display {
                        self.table.eval_river_display();
                        println!(); // TODO: why the spacing issues?
                    }
                }
                _ => {
                    log::trace!("......is_game_over() == false");
                }
            }
        }
        Ok(())
    }

    /// # Errors
    ///
    /// Throws an error if path doesn't exist.
    pub fn get_log_files(path: &str) -> Result<Vec<String>, PKError> {
        let dir = Path::new(path);
        let mut log_files = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(extension) = path.extension()
                && extension == "log"
                && let Some(path_str) = path.to_str()
            {
                log_files.push(path_str.to_string());
            }
        }

        log_files.sort();
        Ok(log_files)
    }
}

impl FromStr for Nubificus {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Nubificus::try_from(Pluribus::from_str(s)?)
    }
}

/// Rebuilds the hand a Pluribus log describes as a playable [`Table`].
///
/// A log gives player names, hole cards, the board, and the actions — but not
/// a deck. So the deck is *stacked*: hole cards first, then each street's
/// cards, with a burn card slipped in before the flop, turn, and river. The
/// burns come from cards the log never mentions, so which ones they are cannot
/// affect any hand. Without them the deck runs dry, because `deal_flop`,
/// `deal_turn`, and `deal_river` each burn one.
///
/// Every seat is staked to 10,000 — the Pluribus experiment's fixed stack —
/// and the button sits on the last seat, which makes seat 0 the small blind.
///
/// Ported from `TryFrom<&Pluribus> for TableCelled` by EPIC-83. Lives here
/// rather than in `casino::table` to keep the log-replay concern next to
/// [`Pluribus`].
impl TryFrom<&Pluribus> for Table {
    type Error = PKError;

    fn try_from(pluribus: &Pluribus) -> Result<Self, Self::Error> {
        let mut seats = Seats::from(pluribus.players.clone());
        for seat in seats.iter_mut() {
            seat.player.chips = Pluribus::STARTING_STACK;
            seat.cards = BoxedCards::blanks(2);
        }

        let hole_cards = pluribus.hole_cards.cards();
        let board_cards = pluribus.board.cards();
        let unseen = Cards::deck_minus(&(hole_cards.clone() + board_cards.clone()));
        let mut burns = unseen.into_iter();

        let board: Vec<Card> = board_cards.into_iter().collect();
        let mut stacked: Vec<Card> = hole_cards.into_iter().collect();

        if board.len() >= 3 {
            stacked.push(burns.next().ok_or(PKError::NotEnoughCards)?);
            stacked.extend_from_slice(&board[0..3]);
        }
        if board.len() >= 4 {
            stacked.push(burns.next().ok_or(PKError::NotEnoughCards)?);
            stacked.push(board[3]);
        }
        if board.len() >= 5 {
            stacked.push(burns.next().ok_or(PKError::NotEnoughCards)?);
            stacked.push(board[4]);
        }

        let mut table = Table::nlh_primed(
            seats,
            &Cards::from(stacked),
            ForcedBets::new(Pluribus::SMALL_BLIND, Pluribus::BIG_BLIND),
        );

        table.button = table.seats.size().saturating_sub(1);
        for seat_number in 0..table.seats.size() {
            table.deal_card_to_seat(seat_number)?;
            table.deal_card_to_seat(seat_number)?;
        }

        Ok(table)
    }
}

impl TryFrom<Pluribus> for Nubificus {
    type Error = PKError;

    fn try_from(pluribus: Pluribus) -> Result<Self, Self::Error> {
        let queue = pluribus.actions.clone();
        let table = Table::try_from(&pluribus)?;
        Ok(Nubificus { pluribus, table, queue })
    }
}

impl TryFrom<&Pluribus> for Nubificus {
    type Error = PKError;

    fn try_from(pluribus: &Pluribus) -> Result<Self, Self::Error> {
        Nubificus::try_from(pluribus.clone())
    }
}

impl Display for Nubificus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Nubificus(index={})", self.pluribus.index)?;
        writeln!(f, "queue:")?;

        if self.queue.is_empty() {
            writeln!(f, "  (empty)")?;
        } else {
            for event in &self.queue {
                writeln!(f, "  - {event}")?;
            }
        }

        write!(f, "table:\n{}", self.table)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pluribus {
    pub index: usize,
    pub rounds: Vec<String>,
    pub actions: VecDeque<PluribusEvent>,
    pub hole_cards: HoleCards,
    pub board: Board,
    pub winnings: Vec<isize>,
    pub players: Vec<String>,
    pub raw: String,
}

impl Pluribus {
    pub const SMALL_BLIND: usize = 50;
    pub const BIG_BLIND: usize = 100;
    /// Every seat in the Pluribus experiment started each hand with a fixed
    /// 10,000-chip stack, so a rebuilt table stakes everyone to the same.
    pub const STARTING_STACK: usize = 10_000;

    fn parse_isizes(s: &str) -> Vec<isize> {
        s.split('|')
            .map(|raw| {
                // Split pots are logged to half a chip (`287.5`). A plain
                // `isize` parse rejects those and used to report the payoff as
                // `0`, hiding a real win. Fall back to the integer part, which
                // truncates toward zero exactly as the chip does.
                raw.parse::<isize>()
                    .or_else(|_| raw.split('.').next().unwrap_or(raw).parse::<isize>())
                    .unwrap_or(0)
            })
            .collect()
    }

    /// I have a theory that the divider between rounds isn't needed. That we can just take
    /// a vector of all the actions, and they pause when the round is over.
    #[must_use]
    pub fn parse_all_rounds(rounds: &Vec<String>) -> VecDeque<PluribusEvent> {
        let mut events = Vec::new();
        for round_str in rounds {
            events.extend(Pluribus::parse_round(round_str));
        }
        VecDeque::from(events)
    }

    #[must_use]
    pub fn parse_round_at(&self, i: usize) -> Vec<PluribusEvent> {
        if let Some(round_str) = self.rounds.get(i) {
            Pluribus::parse_round(round_str)
        } else {
            Vec::new()
        }
    }

    #[must_use]
    pub fn parse_round(rounds_str: &str) -> Vec<PluribusEvent> {
        let mut events = Vec::new();
        let chars: Vec<char> = rounds_str.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                'f' => {
                    events.push(PluribusEvent::Fold);
                    i += 1;
                }
                'c' => {
                    events.push(PluribusEvent::Call);
                    i += 1;
                }
                'r' => {
                    i += 1; // Skip 'r'
                    let mut amount_str = String::new();
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        amount_str.push(chars[i]);
                        i += 1;
                    }
                    if let Ok(amount) = amount_str.parse::<usize>() {
                        events.push(PluribusEvent::Raise(amount));
                    }
                }
                _ => i += 1,
            }
        }

        events
    }

    fn parse_usize(s: &str) -> Result<usize, PKError> {
        match s.to_string().parse() {
            Ok(i) => Ok(i),
            Err(_) => Err(PKError::InvalidPluribusIndex),
        }
    }

    fn parse_string(s: &str) -> Result<Vec<String>, PKError> {
        let v = Util::str_splitter(s, ":");
        if v.len() == 6 {
            Ok(v)
        } else {
            Err(PKError::InvalidPluribusIndex)
        }
    }

    // FirstPass { index: 27, bets: ["r200ffcfc", "cr850cf", "cr1825r3775c", "r10000c"], cards: ["Qc4h", "Tc9c", "8sAs", "Qh7c", "JcQd", "5h5d/3h7s5c/Qs/6c"], winnings: [], players: [] }
    #[allow(clippy::unwrap_used)]
    fn parse_cards(s: &str) -> (HoleCards, Board) {
        if s.contains('/') {
            let re = Regex::new(r"^(?<dealt>[0-9a-zA-Z|]+)/(?<board>.+)$").unwrap();
            let mut res = re.captures_iter(s);

            let Some(caps) = res.next() else {
                return (HoleCards::default(), Board::default());
            };
            (
                HoleCards::from_pluribus(&caps["dealt"]).unwrap_or_default(),
                Board::from_pluribus(&caps["board"]).unwrap_or_default(),
            )
        } else {
            (HoleCards::from_pluribus(s).unwrap_or_default(), Board::default())
        }
    }

    /// I love how this code evolved from a double flipmode clippy lint:
    ///
    /// First:
    ///
    /// ```txt
    /// /Users/gaoler/.cargo/bin/cargo clippy --color=always --message-format=json-diagnostic-rendered-ansi
    ///     Checking pkcore v0.0.15 (/Users/gaoler/src/github.com/ImperialBower/pkcore)
    /// warning: unnecessary `if let` since only the `Ok` variant of the iterator element is used
    ///   --> src/analysis/store/nubibus/pluribus.rs:71:13
    ///    |
    /// 71 | /             for line in lines {
    /// 72 | |                 if let Ok(ip) = line {
    /// 73 | |                     match Pluribus::from_str(ip.as_str()) {
    /// 74 | |                         Ok(pl) => games.push(pl),
    /// ...  |
    /// 78 | |             }
    ///    | |_____________^
    ///    |
    /// help: try `.flatten()` and remove the `if let` statement in the for loop
    ///   --> src/analysis/store/nubibus/pluribus.rs:72:17
    ///    |
    /// 72 | /                 if let Ok(ip) = line {
    /// 73 | |                     match Pluribus::from_str(ip.as_str()) {
    /// 74 | |                         Ok(pl) => games.push(pl),
    /// 75 | |                         Err(_) => {}
    /// 76 | |                     }
    /// 77 | |                 }
    ///    | |_________________^
    ///    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#manual_flatten
    ///    = note: `#[warn(clippy::manual_flatten)]` on by default
    /// help: try
    ///    |
    /// 71 ~             for ip in lines.flatten() {
    /// 72 +                 match Pluribus::from_str(ip.as_str()) {
    /// 73 +                     Ok(pl) => games.push(pl),
    /// 74 +                     Err(_) => {}
    /// 75 +                 }
    /// 76 +             }
    ///    |
    ///
    /// warning: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`
    ///   --> src/analysis/store/nubibus/pluribus.rs:73:21
    ///    |
    /// 73 | /                     match Pluribus::from_str(ip.as_str()) {
    /// 74 | |                         Ok(pl) => games.push(pl),
    /// 75 | |                         Err(_) => {}
    /// 76 | |                     }
    ///    | |_____________________^ help: try: `if let Ok(pl) = Pluribus::from_str(ip.as_str()) { games.push(pl) }`
    ///    |
    ///    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#single_match
    ///    = note: `#[warn(clippy::single_match)]` on by default
    ///
    ///     Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.44s
    /// Process finished with exit code 0
    /// ```
    ///
    /// ```txt
    /// /Users/gaoler/.cargo/bin/cargo clippy --color=always --message-format=json-diagnostic-rendered-ansi
    ///     Checking pkcore v0.0.15 (/Users/gaoler/src/github.com/ImperialBower/pkcore)
    /// warning: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`
    ///   --> src/analysis/store/nubibus/pluribus.rs:72:17
    ///    |
    /// 72 | /                 match Pluribus::from_str(ip.as_str()) {
    /// 73 | |                     Ok(pl) => games.push(pl),
    /// 74 | |                     Err(_) => {} // Invalid lines get eaten :-P
    /// 75 | |                 }
    ///    | |_________________^ help: try: `if let Ok(pl) = Pluribus::from_str(ip.as_str()) { games.push(pl) }`
    ///    |
    ///    = note: you might want to preserve the comments from inside the `match`
    ///    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#single_match
    ///    = note: `#[warn(clippy::single_match)]` on by default
    ///
    ///     Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.42s
    /// Process finished with exit code 0.
    /// ```
    ///
    /// Returns a formatted string showing the winnings/losses for each player
    /// with their seat number and name.
    ///
    /// # Returns
    ///
    /// A string with each player's results on a separate line in the format:
    /// "Seat {n}: {name} {+/-amount}"
    #[must_use]
    pub fn display_results(&self) -> String {
        use std::fmt::Write;
        let mut result = String::new();

        for (seat, (player_name, winnings)) in self.players.iter().zip(self.winnings.iter()).enumerate() {
            match (*winnings).cmp(&0) {
                std::cmp::Ordering::Greater => {
                    let _ = writeln!(
                        result,
                        "{}Seat {} {} wins {}!",
                        Terminal::random_happy(),
                        seat,
                        player_name,
                        winnings
                    );
                }
                std::cmp::Ordering::Equal => {
                    let _ = writeln!(result, "  Seat {seat} {player_name} wins {winnings}");
                }
                std::cmp::Ordering::Less => {
                    let _ = writeln!(
                        result,
                        "{}Seat {} {} {}",
                        Terminal::random_sad(),
                        seat,
                        player_name,
                        winnings
                    );
                }
            }
        }

        result
    }

    /// # Errors
    ///
    /// `PKError::InvalidPluribusIndex`
    pub fn read_in_log(filename: &str) -> Result<Vec<Pluribus>, PKError> {
        let mut games = Vec::new();

        if let Ok(lines) = Util::read_lines(filename) {
            for ip in lines.map_while(Result::ok) {
                if let Ok(pl) = Pluribus::from_str(ip.as_str()) {
                    games.push(pl);
                }
            }
        }

        Ok(games)
    }
}

impl Display for Pluribus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{} rounds: {:?} HANDS: {} BOARD: {} WINNINGS: {:?} PLAYERS: {:?}",
            self.index, self.rounds, self.hole_cards, self.board, self.winnings, self.players,
        )
    }
}

impl FromStr for Pluribus {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Pluribus::parse_string(s) {
            Ok(v) => {
                let (hole_cards, board) = Pluribus::parse_cards(v.index(3));

                let rounds = Util::str_splitter(v.index(2), "/");
                let actions = Pluribus::parse_all_rounds(&rounds);

                Ok(Pluribus {
                    index: Pluribus::parse_usize(v.index(1))?,
                    rounds,
                    actions,
                    hole_cards,
                    board,
                    winnings: Pluribus::parse_isizes(v.index(4)),
                    players: Util::str_splitter(v.index(5), "|"),
                    raw: s.to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub enum PluribusEvent {
    #[default]
    Fold,
    Call,
    Raise(usize),
}

impl PluribusEvent {
    #[must_use]
    pub fn is_fold(&self) -> bool {
        matches!(self, PluribusEvent::Fold)
    }

    #[must_use]
    pub fn is_call(&self) -> bool {
        matches!(self, PluribusEvent::Call)
    }

    #[must_use]
    pub fn is_raise(&self) -> bool {
        matches!(self, PluribusEvent::Raise(_))
    }

    #[must_use]
    pub fn raise_amount(&self) -> Option<usize> {
        if let PluribusEvent::Raise(amount) = self {
            Some(*amount)
        } else {
            None
        }
    }
}

impl Display for PluribusEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluribusEvent::Fold => write!(f, "Fold"),
            PluribusEvent::Call => write!(f, "Call"),
            PluribusEvent::Raise(amount) => write!(f, "Raise({amount})"),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod store_pluribus_tests {
    use super::*;
    use rstest::rstest;

    const LOG: &str = "STATE:27:r200ffcfc/cr850cf/cr1825r3775c/r10000c:Qc4h|Tc9c|8sAs|Qh7c|JcQd|5h5d/3h7s5c/Qs/6c:-50|-200|-10000|0|0|10250:Eddie|Bill|Pluribus|MrWhite|Gogo|Budd";

    /// Asserts each seat's final stack is its starting stack plus the payoff
    /// the log records for it.
    ///
    /// Reads the outcome from the stacks rather than from `chips_in_play`,
    /// which `Table::end_hand` clears when it resets for the next hand.
    fn assert_payoffs(nubi: &Nubificus, payoffs: &[isize]) {
        for (seat_number, payoff) in payoffs.iter().enumerate() {
            let seat = nubi.table.seats.get_seat(u8::try_from(seat_number).unwrap()).unwrap();
            let expected = usize::try_from(isize::try_from(Pluribus::STARTING_STACK).unwrap() + payoff).unwrap();
            assert_eq!(
                expected, seat.player.chips,
                "seat {seat_number} ({}) should end on {expected}",
                seat.player.handle
            );
        }
    }

    // ── EPIC-83: rebuilding the hand on the plain Table ──────────────────────

    #[test]
    fn table_from_pluribus_seats_every_named_player() {
        let pluribus = Pluribus::from_str(LOG).unwrap();

        let table = Table::try_from(&pluribus).unwrap();

        assert_eq!(6, table.seats.size());
        let handles: Vec<&str> = table.seats.iter().map(|seat| seat.player.handle.as_str()).collect();
        assert_eq!(vec!["Eddie", "Bill", "Pluribus", "MrWhite", "Gogo", "Budd"], handles);
    }

    #[test]
    fn table_from_pluribus_stakes_every_seat_to_ten_thousand() {
        let pluribus = Pluribus::from_str(LOG).unwrap();

        let table = Table::try_from(&pluribus).unwrap();

        for seat in table.seats.iter() {
            assert_eq!(10_000, seat.player.chips, "{}", seat.player.handle);
        }
    }

    #[test]
    fn table_from_pluribus_deals_the_logged_hole_cards() {
        // The log reads `Qc4h|Tc9c|8sAs|...`, seat by seat.
        let pluribus = Pluribus::from_str(LOG).unwrap();

        let table = Table::try_from(&pluribus).unwrap();

        assert_eq!(
            "Q♣ 4♥",
            table.seats.get_seat(0).unwrap().cards.to_string(),
            "seat 0 holds what the log says"
        );
        assert_eq!("T♣ 9♣", table.seats.get_seat(1).unwrap().cards.to_string());
    }

    #[test]
    fn table_from_pluribus_puts_the_button_on_the_last_seat() {
        // Pluribus logs are six-handed with the button fixed at seat 5, which
        // makes seat 0 the small blind.
        let pluribus = Pluribus::from_str(LOG).unwrap();

        let table = Table::try_from(&pluribus).unwrap();

        assert_eq!(5, table.button);
    }

    #[test]
    fn table_from_pluribus_leaves_the_logged_board_on_top_of_the_deck() {
        // Board `3h7s5c/Qs/6c` must still be dealable, with burn cards
        // interleaved, or the streets run out of cards.
        let pluribus = Pluribus::from_str(LOG).unwrap();
        let mut table = Table::try_from(&pluribus).unwrap();

        table.deal_flop().unwrap();
        table.deal_turn().unwrap();
        table.deal_river().unwrap();

        assert_eq!("3♥ 7♠ 5♣ Q♠ 6♣", table.board.to_string());
    }

    /// `DEFECT_020`: `Nubificus::act` discarded every action `Result`, so a
    /// rejected action during log replay vanished and the table drifted out of
    /// sync with the log it is supposed to reproduce.
    #[test]
    fn act_propagates_a_rejected_action() {
        let mut nubi = Nubificus::from_str(LOG).unwrap();
        let out_of_turn = nubi.table.next_to_act() + 1;

        let result = Nubificus::act(&mut nubi.table, &PluribusEvent::Fold, out_of_turn);

        assert!(result.is_err(), "an out-of-turn fold must not report success");
    }

    #[test]
    fn act_propagates_a_rejected_raise() {
        let mut nubi = Nubificus::from_str(LOG).unwrap();
        let out_of_turn = nubi.table.next_to_act() + 1;

        let result = Nubificus::act(&mut nubi.table, &PluribusEvent::Raise(200), out_of_turn);

        assert!(result.is_err(), "an out-of-turn raise must not report success");
    }

    #[test]
    fn act_propagates_a_rejected_call() {
        let mut nubi = Nubificus::from_str(LOG).unwrap();
        let out_of_turn = nubi.table.next_to_act() + 1;

        let result = Nubificus::act(&mut nubi.table, &PluribusEvent::Call, out_of_turn);

        assert!(result.is_err(), "an out-of-turn call must not report success");
    }

    /// `DEFECT_021`: Pluribus log amounts are cumulative per-hand totals, while
    /// [`Table::act_bet`] takes a per-street target.
    ///
    /// The losing player's payoff is exactly `-3750`, his last logged number.
    /// Read per-street the same hand asks a 10 000-chip stack for
    /// `1150 + 2050 + 3750 = 6950` and then 6 250 more, which the table rightly
    /// refuses.
    #[test]
    fn replay_reads_logged_amounts_as_cumulative_totals() {
        const CUMULATIVE: &str = "STATE:154:fr250ffr1150fc/r2050c/r3750c/r6250f:Qc4h|Tc9c|8sAs|Qh7c|JcQd|5h5d/3h7s5c/Qs/6c:3850|-100|0|-3750|0|0:Eddie|Bill|Pluribus|MrWhite|Gogo|Budd";

        let mut nubi = Nubificus::from_str(CUMULATIVE).unwrap();

        assert!(nubi.play_hand().is_ok(), "the hand must replay without a betting error");

        // EPIC-83: the celled engine left `chips_in_play` standing after the
        // hand, so this used to read commitments straight off the seats. The
        // plain engine clears it in `reset()` — correctly, since the next hand
        // must start from zero. Assert the log's own payoffs instead, which is
        // a stronger check: it pins what each seat committed *and* what it won.
        assert_payoffs(&nubi, &[3850, -100, 0, -3750, 0, 0]);
    }

    /// `DEFECT_022`: a re-raise used to put the action on the wrong seat, so the
    /// replay applied logged actions to players who were not next in turn.
    ///
    /// On the flop of this hand seat 5 raises to 475, seat 1 calls, and seat 3
    /// re-raises to 1200. Seat 5 is next in turn and folds. The old order gave
    /// the fold to seat 1 instead, rotating every later action by one live seat
    /// and landing seat 3's turn raise of 5200 on seat 5.
    #[test]
    fn replay_gives_a_re_raise_the_correct_seat() {
        const RE_RAISE: &str = "STATE:153:fr200fcfc/ccr475cr1200fc/cr5200f:9dAh|6sJs|Jh3c|7h8h|Td2s|JcQc/5h8sQs/Kc:-50|-1200|0|1725|0|-475:MrBrown|MrBlue|Pluribus|Eddie|MrPink|MrOrange";

        let mut nubi = Nubificus::from_str(RE_RAISE).unwrap();

        assert!(nubi.play_hand().is_ok(), "the hand must replay without a betting error");

        // Every seat ends exactly where the log says it should. If a re-raise
        // landed on the wrong seat, these stacks would not line up.
        assert_payoffs(&nubi, &[-50, -1200, 0, 1725, 0, -475]);
    }

    #[test]
    fn log_to_string_vec() {
        assert!(Pluribus::parse_string(LOG).is_ok())
    }

    #[rstest]
    #[case("3c9s|6d5s|9dTs|2sQs|AdKd|7cTc", "3c9s|6d5s|9dTs|2sQs|AdKd|7cTc", "")]
    #[case("8sQc|2s8d|7dTs|5d8h|2h9s|6cQd", "8sQc|2s8d|7dTs|5d8h|2h9s|6cQd", "")]
    #[case("JhJs|7d7c|7sKc|4d6s|8hAs|8s4c", "JhJs|7d7c|7sKc|4d6s|8hAs|8s4c", "")]
    #[case("Qd4c|7h9d|6s3h|7s9c|JcKc|Ks7c", "Qd4c|7h9d|6s3h|7s9c|JcKc|Ks7c", "")]
    #[case("9cAd|4h7c|Ts2s|6s8c|6c8s|QhAh", "9cAd|4h7c|Ts2s|6s8c|6c8s|QhAh", "")]
    fn parse_cards(#[case] raw: &str, #[case] expected_hands: &str, #[case] expected_board: &str) {
        let (hands, board) = Pluribus::parse_cards(raw);

        assert_eq!(hands, HoleCards::from_pluribus(expected_hands).unwrap());
        assert_eq!(board, Board::from_pluribus(expected_board).unwrap());
    }

    #[test]
    fn parse_isizes() {
        let expected = vec![-50, -200, -10000, 0, 0, 10250];

        let actual = Pluribus::parse_isizes(Pluribus::parse_string(LOG).unwrap().index(4));

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_isizes_keeps_a_split_pots_half_chip_payoff() {
        // A chopped pot is logged to half a chip. Reading `112.5` as `0` would
        // report a winner as having broken even.
        let actual = Pluribus::parse_isizes("112.5|-225|0|112.5|0|0");

        assert_eq!(vec![112, -225, 0, 112, 0, 0], actual);
    }

    #[test]
    fn parse_isizes_truncates_a_half_chip_loss_toward_zero() {
        let actual = Pluribus::parse_isizes("-287.5|287.5");

        assert_eq!(vec![-287, 287], actual);
    }

    #[test]
    fn parse_isizes_still_reads_nonsense_as_zero() {
        let actual = Pluribus::parse_isizes("|abc|1x2");

        assert_eq!(vec![0, 0, 0], actual);
    }

    #[test]
    fn parse_rounds() {
        // Test basic fold and call
        let events = Pluribus::parse_round("ffc");
        assert_eq!(events.len(), 3);
        matches!(events[0], PluribusEvent::Fold);
        matches!(events[1], PluribusEvent::Fold);
        matches!(events[2], PluribusEvent::Call);

        // Test raise with amount
        let events = Pluribus::parse_round("r200ffcfc");
        assert_eq!(events.len(), 6);
        matches!(events[0], PluribusEvent::Raise(200));
        matches!(events[1], PluribusEvent::Fold);
        matches!(events[2], PluribusEvent::Fold);
        matches!(events[3], PluribusEvent::Call);
        matches!(events[4], PluribusEvent::Fold);
        matches!(events[5], PluribusEvent::Call);

        // Test multiple raises
        let events = Pluribus::parse_round("cr850cf");
        assert_eq!(events.len(), 4);
        matches!(events[0], PluribusEvent::Call);
        matches!(events[1], PluribusEvent::Raise(850));
        matches!(events[2], PluribusEvent::Call);
        matches!(events[3], PluribusEvent::Fold);

        // Test complex round with multiple raises
        let events = Pluribus::parse_round("cr1825r3775c");
        assert_eq!(events.len(), 4);
        matches!(events[0], PluribusEvent::Call);
        matches!(events[1], PluribusEvent::Raise(1825));
        matches!(events[2], PluribusEvent::Raise(3775));
        matches!(events[3], PluribusEvent::Call);
    }

    #[test]
    fn parse_usize() {
        assert_eq!(
            27usize,
            Pluribus::parse_usize(Pluribus::parse_string(LOG).unwrap().index(1)).unwrap()
        );
        assert_eq!(
            PKError::InvalidPluribusIndex,
            Pluribus::parse_usize("23skidoo").unwrap_err()
        );
    }

    #[test]
    fn parse_string() {
        let expected = vec!["r200ffcfc", "cr850cf", "cr1825r3775c", "r10000c"];

        let actual = Util::str_splitter(Pluribus::parse_string(LOG).unwrap().index(2), "/");

        assert_eq!(expected, actual);
    }

    #[test]
    fn from_str() {
        let actual = Pluribus::from_str(LOG).unwrap();

        assert_eq!(27, actual.index);
        assert_eq!(vec!["r200ffcfc", "cr850cf", "cr1825r3775c", "r10000c"], actual.rounds);
        assert_eq!(
            HoleCards::from_str("Qc 4h Tc 9c 8s As Qh 7c Jc Qd 5h 5d").unwrap(),
            actual.hole_cards
        );
        assert_eq!(Board::from_str("3h 7s 5c Qs 6c").unwrap(), actual.board);
        assert_eq!(
            vec!["Eddie", "Bill", "Pluribus", "MrWhite", "Gogo", "Budd"],
            actual.players
        );
    }

    #[rstest]
    #[case("STATE:0:ffr225fff:3c9s|6d5s|9dTs|2sQs|AdKd|7cTc:-50|-100|0|0|150|0:MrWhite|Gogo|Budd|Eddie|Bill|Pluribus")]
    #[case("STATE:1:ffffr300f:8sQc|2s8d|7dTs|5d8h|2h9s|6cQd:100|-100|0|0|0|0:Gogo|Budd|Eddie|Bill|Pluribus|MrWhite")]
    #[case(
        "STATE:5:ffr200fr950ff:JhJs|7d7c|7sKc|4d6s|8hAs|8s4c:300|-100|0|0|-200|0:Pluribus|MrWhite|Gogo|Budd|Eddie|Bill"
    )]
    #[case("STATE:6:ffr225fff:Qd4c|7h9d|6s3h|7s9c|JcKc|Ks7c:-50|-100|0|0|150|0:MrWhite|Gogo|Budd|Eddie|Bill|Pluribus")]
    #[case("STATE:11:fffr250ff:9cAd|4h7c|Ts2s|6s8c|6c8s|QhAh:-50|-100|0|0|0|150:Pluribus|MrWhite|Gogo|Budd|Eddie|Bill")]
    fn from_str__errors(#[case] row: &str) {
        let _nl = Pluribus {
            index: 0,
            rounds: Vec::new(),
            actions: Default::default(),
            hole_cards: HoleCards::default(),
            board: Board::default(),
            winnings: Vec::new(),
            players: Vec::new(),
            raw: String::new(),
        };
        let _result = match Pluribus::parse_string(row) {
            Ok(v) => {
                let (hole_cards, board) = Pluribus::parse_cards(v.index(3));
                let rounds = Util::str_splitter(v.index(2), "/");
                let actions = Pluribus::parse_all_rounds(&rounds);
                Ok(Pluribus {
                    index: Pluribus::parse_usize(v.index(1)).unwrap(),
                    rounds,
                    actions,
                    hole_cards,
                    board,
                    winnings: Pluribus::parse_isizes(v.index(4)),
                    players: Util::str_splitter(v.index(5), "|"),
                    raw: row.to_string(),
                })
            }
            Err(e) => Err(e),
        };
    }

    #[test]
    fn do_test() {
        let row =
            "STATE:0:ffr225fff:3c9s|6d5s|9dTs|2sQs|AdKd|7cTc:-50|-100|0|0|150|0:MrWhite|Gogo|Budd|Eddie|Bill|Pluribus";
        let v = Pluribus::parse_string(row).unwrap();
        let (player_cards, board) = Pluribus::parse_cards(v.index(3));

        let _index = Pluribus::parse_usize(v.index(1)).unwrap();
        let _rounds = Util::str_splitter(v.index(2), "/");
        let _hole_cards = player_cards;
        let _board = board;
        let _winnings = Pluribus::parse_isizes(v.index(4));
        let _players = Util::str_splitter(v.index(5), "|");
    }

    #[test]
    fn nubificus_display_contains_key_fields() {
        let pluribus = Pluribus::from_str(LOG).unwrap();
        let nubificus = Nubificus::try_from(pluribus).unwrap();

        let rendered = nubificus.to_string();

        assert!(rendered.contains("Nubificus(index=27)"));
        assert!(rendered.contains("queue:\n"));
        assert!(rendered.contains("  - Raise(200)"));
        assert!(rendered.contains("table:\n"));
    }

    #[test]
    fn nubificus_display_shows_empty_queue() {
        let mut nubificus = Nubificus::try_from(Pluribus::from_str(LOG).unwrap()).unwrap();
        nubificus.queue.clear();

        let rendered = nubificus.to_string();

        assert!(rendered.contains("queue:\n"));
        assert!(rendered.contains("  (empty)"));
    }

    #[test]
    fn isolate_15() {
        let s = "STATE:14:fr200cfff/cc/cc/cc:4cJs|5s9h|Kh7h|9sQs|2d2h|5dTh/3s3dAd/Qc/Td:-50|-100|0|350|-200|0:MrWhite|MrPink|MrBrown|Pluribus|MrBlue|MrBlonde";
        let pl = Pluribus::from_str(s).unwrap();
        let nub = Nubificus::try_from(pl).unwrap().play_hand_display().unwrap();
        println!("{:?}", nub);
    }
}
