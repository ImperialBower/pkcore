use clap::{Parser, ValueEnum};
use clap_repl::ClapEditor;
use clap_repl::reedline::{DefaultPrompt, DefaultPromptSegment, FileBackedHistory, Reedline, Signal};
use pkcore::prelude::*;
use std::path::PathBuf;

/// A simple example of using `clap_repl` to create a REPL for a poker table.
///
/// -
/// - [Writing a CLI Tool in Rust with Clap](https://www.shuttle.dev/blog/2023/12/08/clap-rust)
/// `cargo run --example table0`
#[derive(Debug, Parser)]
#[command(name = "")] // This name will show up in clap's error messages, so it is important to set it to "".
enum SampleCommand {
    #[command(alias = "s")]
    Status,
    #[command(alias = "fb")]
    Blinds,
    #[command(alias = "dc")]
    Deal,
    Deck,
    Download {
        path: PathBuf,
        /// Check the integrity of the downloaded object
        ///
        /// Uses SHA256
        #[arg(long)]
        check_sha: bool,
    },
    /// A command to upload things.
    Upload,
    /// Login into the system.
    Login {
        /// Optional. You will be prompted if you don't provide it.
        #[arg(short, long)]
        username: Option<String>,
        #[arg(short, long, value_enum, default_value_t = Mode::Secure)]
        mode: Mode,
    },
    /// Exit the REPL
    #[command(alias = "q")]
    Exit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Encrypt the password
    Secure,
    /// Send the password plain
    ///
    /// This paragraph is ignored because there is no long help text for possible values in clap.
    Insecure,
}

fn main() {
    env_logger::init();

    // let seats = Table::generate_seats(6, 2);
    // let forced = ForcedBets::new(50, 100);
    let table = TestData::the_hand_table();

    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("table".to_owned()),
        ..DefaultPrompt::default()
    };
    let rl = ClapEditor::<SampleCommand>::builder()
        .with_prompt(Box::new(prompt))
        .with_editor_hook(|reed| {
            // Do custom things with `Reedline` instance here
            reed.with_history(Box::new(
                FileBackedHistory::with_file(10000, "./generated/clap-repl-simple-example-history".into()).unwrap(),
            ))
        })
        .build();
    rl.repl(|command| {
        match command {
            SampleCommand::Status => {
                println!("{}", table.get_game_state());
                println!("{table}");
            }
            SampleCommand::Deck => {
                println!("Deck: {}", table.deck);
            }
            SampleCommand::Blinds => {
                table.act_forced_bets().unwrap();
                println!("Forced bets: {}", table.forced);
            }
            SampleCommand::Deal => {
                if table.seats.are_dealt() {
                    println!("Cards have already been dealt to seats");
                } else {
                    table.deal_cards_to_seats().expect("Failed to deal cards to seats");
                    println!("Dealt cards to seats");
                }
            }
            SampleCommand::Download { path, check_sha } => {
                println!("Downloaded {path:?} with checking = {check_sha}");
            }
            SampleCommand::Upload => {
                println!("Uploaded");
            }
            SampleCommand::Login { username, mode } => {
                // You can use another `reedline::Reedline` inside the loop.
                let mut rl = Reedline::create();
                let username = username.unwrap_or_else(|| read_line_with_reedline(&mut rl, "What is your username? "));
                let password = read_line_with_reedline(&mut rl, "What is your password? ");
                println!("Logged in with {username} and {password} in mode {mode:?}");
            }
            SampleCommand::Exit => {
                println!("Goodbye! 👋");
                std::process::exit(0);
            }
        }
    });
}

fn read_line_with_reedline(rl: &mut Reedline, prompt: &str) -> String {
    let Signal::Success(x) = rl
        .read_line(&DefaultPrompt::new(
            DefaultPromptSegment::Basic(prompt.to_owned()),
            DefaultPromptSegment::Empty,
        ))
        .unwrap()
    else {
        panic!();
    };
    x
}
