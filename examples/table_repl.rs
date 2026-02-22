use clap::{Parser, Subcommand};
use clap_repl::ClapRepl;
use pkcore::casino::cashier::chips::Stack;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::seat::Seat;
use pkcore::casino::table::seats::Seats;
use pkcore::casino::table::Table;
use pkcore::prelude::Player;
use pkcore::{BoxedCards, PKError};
use std::str::FromStr;

/// Interactive REPL for managing poker table actions
///
/// # Usage
///
/// ```bash
/// cargo run --example table_repl
/// ```
///
/// # Commands
///
/// - `setup <players> <sb> <bb>` - Create a new table with players
/// - `deal` - Deal cards to all players
/// - `flop` - Deal the flop
/// - `turn` - Deal the turn
/// - `river` - Deal the river
/// - `bet <seat> <amount>` - Player at seat bets amount
/// - `call <seat>` - Player at seat calls
/// - `raise <seat> <amount>` - Player at seat raises to amount
/// - `check <seat>` - Player at seat checks
/// - `fold <seat>` - Player at seat folds
/// - `allin <seat>` - Player at seat goes all-in
/// - `show` - Show current table state
/// - `log` - Show event log
/// - `reset` - Reset betting round
/// - `help` - Show available commands
/// - `quit` - Exit the REPL
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("╔═══════════════════════════════════════════════╗");
    println!("║      Poker Table Management REPL v0.1         ║");
    println!("║   Type 'help' for commands, 'quit' to exit    ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let mut state = TableState::default();
    let mut repl = ClapRepl::new(TableCommands::parse)
        .with_name("table")
        .with_prompt("table> ");

    loop {
        match repl.read_command() {
            Ok(cmd) => match cmd.command {
                Commands::Setup { players, sb, bb } => {
                    match state.setup(players, sb, bb) {
                        Ok(_) => println!("✓ Table created with {} players (SB: {}, BB: {})", players, sb, bb),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::Deal => {
                    match state.deal() {
                        Ok(_) => println!("✓ Cards dealt to all players"),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::Flop => {
                    match state.flop() {
                        Ok(cards) => println!("✓ Flop: {}", cards),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::Turn => {
                    match state.turn() {
                        Ok(cards) => println!("✓ Turn: {}", cards),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::River => {
                    match state.river() {
                        Ok(cards) => println!("✓ River: {}", cards),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::Bet { seat, amount } => {
                    match state.bet(seat, amount) {
                        Ok(_) => println!("✓ Seat {} bets {}", seat, amount),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::Call { seat } => {
                    match state.call(seat) {
                        Ok(amount) => println!("✓ Seat {} calls {}", seat, amount),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::Raise { seat, amount } => {
                    match state.raise(seat, amount) {
                        Ok(_) => println!("✓ Seat {} raises to {}", seat, amount),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::Check { seat } => {
                    match state.check(seat) {
                        Ok(_) => println!("✓ Seat {} checks", seat),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::Fold { seat } => {
                    match state.fold(seat) {
                        Ok(_) => println!("✓ Seat {} folds", seat),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::AllIn { seat } => {
                    match state.allin(seat) {
                        Ok(amount) => println!("✓ Seat {} goes all-in with {}", seat, amount),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::Show => {
                    state.show();
                }
                Commands::Board => {
                    state.show_board();
                }
                Commands::Log => {
                    state.show_log();
                }
                Commands::Reset => {
                    match state.reset() {
                        Ok(_) => println!("✓ Betting round reset"),
                        Err(e) => eprintln!("✗ Error: {:?}", e),
                    }
                }
                Commands::Next => {
                    state.show_next_to_act();
                }
                Commands::Pot => {
                    state.show_pot();
                }
                Commands::Quit => {
                    println!("Goodbye! 👋");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}

#[derive(Parser)]
#[clap(name = "", no_binary_name = true)]
struct TableCommands {
    #[clap(subcommand)]
    command: Commands,
}

impl std::ops::Deref for TableCommands {
    type Target = Commands;

    fn deref(&self) -> &Self::Target {
        &self.command
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new table with specified number of players and blinds
    #[clap(alias = "s")]
    Setup {
        /// Number of players (2-9)
        #[clap(value_parser = clap::value_parser!(u8).range(2..=9))]
        players: u8,
        /// Small blind amount
        #[clap(default_value = "50")]
        sb: usize,
        /// Big blind amount
        #[clap(default_value = "100")]
        bb: usize,
    },

    /// Deal hole cards to all players
    #[clap(alias = "d")]
    Deal,

    /// Deal the flop (3 community cards)
    #[clap(alias = "f")]
    Flop,

    /// Deal the turn (4th community card)
    #[clap(alias = "t")]
    Turn,

    /// Deal the river (5th community card)
    #[clap(alias = "r")]
    River,

    /// Player at seat bets amount
    #[clap(alias = "b")]
    Bet {
        /// Seat number
        seat: u8,
        /// Bet amount
        amount: usize,
    },

    /// Player at seat calls current bet
    #[clap(alias = "c")]
    Call {
        /// Seat number
        seat: u8,
    },

    /// Player at seat raises to amount
    #[clap(alias = "ra")]
    Raise {
        /// Seat number
        seat: u8,
        /// Raise amount
        amount: usize,
    },

    /// Player at seat checks
    #[clap(alias = "ch")]
    Check {
        /// Seat number
        seat: u8,
    },

    /// Player at seat folds
    #[clap(alias = "fo")]
    Fold {
        /// Seat number
        seat: u8,
    },

    /// Player at seat goes all-in
    #[clap(alias = "a")]
    AllIn {
        /// Seat number
        seat: u8,
    },

    /// Show current table state
    #[clap(alias = "sh")]
    Show,

    /// Show the community cards (board)
    #[clap(alias = "bo")]
    Board,

    /// Show event log
    #[clap(alias = "l")]
    Log,

    /// Reset betting round
    Reset,

    /// Show next player to act
    #[clap(alias = "n")]
    Next,

    /// Show pot size
    #[clap(alias = "p")]
    Pot,

    /// Exit the REPL
    #[clap(alias = "q")]
    Quit,
}

#[derive(Default)]
struct TableState {
    table: Option<Table>,
}

impl TableState {
    fn setup(&mut self, players: u8, sb: usize, bb: usize) -> Result<(), PKError> {
        let mut seats = Vec::new();
        for i in 0..players {
            let mut player = Player::default();
            player.name = format!("Player {}", i);
            player.chips.add_to(Stack::new(100_000)); // Start with 100k chips
            seats.push(Seat {
                player,
                cards: BoxedCards::blanks(2),
            });
        }

        let seats = Seats::new(seats);
        let forced_bets = ForcedBets::new(sb, bb);
        let table = Table::nlh_from_seats(seats, forced_bets);

        self.table = Some(table);
        Ok(())
    }

    fn get_table(&self) -> Result<&Table, PKError> {
        self.table.as_ref().ok_or(PKError::TableNotFound)
    }

    fn deal(&self) -> Result<(), PKError> {
        let table = self.get_table()?;
        table.deal_cards_to_seats()?;
        table.act_forced_bets()?;
        Ok(())
    }

    fn flop(&self) -> Result<String, PKError> {
        let table = self.get_table()?;
        table.deal_flop()?;
        Ok(table.board.to_string())
    }

    fn turn(&self) -> Result<String, PKError> {
        let table = self.get_table()?;
        table.deal_turn()?;
        Ok(table.board.to_string())
    }

    fn river(&self) -> Result<String, PKError> {
        let table = self.get_table()?;
        table.deal_river()?;
        Ok(table.board.to_string())
    }

    fn bet(&self, seat: u8, amount: usize) -> Result<(), PKError> {
        let table = self.get_table()?;
        table.act_bet(seat, amount)?;
        Ok(())
    }

    fn call(&self, seat: u8) -> Result<usize, PKError> {
        let table = self.get_table()?;
        table.act_call(seat)
    }

    fn raise(&self, seat: u8, amount: usize) -> Result<(), PKError> {
        let table = self.get_table()?;
        table.act_raise(seat, amount)?;
        Ok(())
    }

    fn check(&self, seat: u8) -> Result<(), PKError> {
        let table = self.get_table()?;
        table.act_check(seat)?;
        Ok(())
    }

    fn fold(&self, seat: u8) -> Result<usize, PKError> {
        let table = self.get_table()?;
        table.act_fold(seat)
    }

    fn allin(&self, seat: u8) -> Result<usize, PKError> {
        let table = self.get_table()?;
        table.act_all_in(seat)
    }

    fn reset(&self) -> Result<(), PKError> {
        let table = self.get_table()?;
        table.reset_betting();
        Ok(())
    }

    fn show(&self) {
        match self.get_table() {
            Ok(table) => {
                println!("\n{}", "═".repeat(60));
                println!("{}", table);
                println!("{}", "═".repeat(60));
            }
            Err(_) => println!("No table created. Use 'setup <players> <sb> <bb>' first."),
        }
    }

    fn show_board(&self) {
        match self.get_table() {
            Ok(table) => {
                let board = table.board.to_string();
                if board.is_empty() {
                    println!("Board: (no cards yet)");
                } else {
                    println!("Board: {}", board);
                }
            }
            Err(_) => println!("No table created."),
        }
    }

    fn show_log(&self) {
        match self.get_table() {
            Ok(table) => {
                println!("\n{}", "─".repeat(60));
                println!("Event Log:");
                println!("{}", "─".repeat(60));
                println!("{}", table.event_log);
                println!("{}", "─".repeat(60));
            }
            Err(_) => println!("No table created."),
        }
    }

    fn show_next_to_act(&self) {
        match self.get_table() {
            Ok(table) => {
                let next = table.next_to_act();
                println!("Next to act: Seat {}", next);
                if let Some(seat) = table.seats.get_seat(next) {
                    println!("  Player: {}", seat.player.name);
                    println!("  Chips: {}", seat.player.chips.count());
                    println!("  Current bet: {}", seat.player.bet.count());
                }
            }
            Err(_) => println!("No table created."),
        }
    }

    fn show_pot(&self) {
        match self.get_table() {
            Ok(table) => {
                println!("Pot: {}", table.pot.count());
            }
            Err(_) => println!("No table created."),
        }
    }
}

