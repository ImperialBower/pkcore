//! Interactive gRPC client for the Dealer service.
//!
//! Connects to a remote Dealer gRPC server and provides a command-line
//! interface similar to `dealer_repl.rs`, but communicating over the network.
//!
//! # Usage
//!
//! First, start the server:
//! ```bash
//! cargo run --example dealer_grpc_server --features grpc
//! ```
//!
//! Then, in another terminal, run the client:
//! ```bash
//! cargo run --example dealer_grpc_client --features grpc
//! ```
//!
//! # Quick-start session
//!
//! ```text
//! dealer❯ seat Alice 10000
//! dealer❯ seat Bob 10000
//! dealer❯ seat Carol 10000
//! dealer❯ start
//! dealer❯ status
//! dealer❯ bet 2 300
//! dealer❯ call 3
//! dealer❯ fold 0
//! dealer❯ street
//! dealer❯ end
//! dealer❯ quit
//! ```

#[cfg(not(feature = "grpc"))]
fn main() {
    eprintln!("This example requires the `grpc` feature. Run:");
    eprintln!("  cargo run --example dealer_grpc_client --features grpc");
}

#[cfg(feature = "grpc")]
use clap::Parser;
#[cfg(feature = "grpc")]
use clap_repl::ClapEditor;
#[cfg(feature = "grpc")]
use reedline::{DefaultPrompt, DefaultPromptSegment, FileBackedHistory};

// Include the generated proto code
#[cfg(feature = "grpc")]
pub mod dealer_proto {
    tonic::include_proto!("pkcore.dealer");
}

#[cfg(feature = "grpc")]
use dealer_proto::{dealer_service_client::DealerServiceClient, *};

// ── Commands ─────────────────────────────────────────────────────────────────

#[cfg(feature = "grpc")]
#[derive(Debug, Parser)]
#[command(name = "", about = "pkcore Dealer gRPC Client - drive a poker hand remotely")]
enum Command {
    /// Seat a new player. Chips default to 10 000 if omitted.
    #[command(alias = "s")]
    Seat {
        name: String,
        #[arg(default_value_t = 10_000)]
        chips: u32,
    },

    /// Seat a player at a specific seat number.
    #[command(alias = "sa")]
    SeatAt {
        seat: u32,
        name: String,
        #[arg(default_value_t = 10_000)]
        chips: u32,
    },

    /// Remove a player from their seat.
    #[command(alias = "rm")]
    Remove { seat: u32 },

    /// Start a new hand.
    #[command(alias = "st")]
    Start,

    /// Advance to the next street.
    #[command(alias = "sv")]
    Street,

    /// End the current hand.
    #[command(alias = "e")]
    End,

    /// Bet a specific amount.
    #[command(alias = "b")]
    Bet { seat: u32, amount: u32 },

    /// Call the current bet.
    #[command(alias = "c")]
    Call { seat: u32 },

    /// Check.
    #[command(alias = "ck")]
    Check { seat: u32 },

    /// Raise to a total amount.
    #[command(alias = "r")]
    Raise { seat: u32, amount: u32 },

    /// Go all-in.
    #[command(alias = "ai")]
    Allin { seat: u32 },

    /// Fold.
    #[command(alias = "f")]
    Fold { seat: u32 },

    /// Show the full table state.
    #[command(alias = "sh")]
    Status,

    /// Show who is next to act.
    #[command(alias = "n")]
    Next,

    /// Show the community cards.
    #[command(alias = "bo")]
    Board,

    /// Show chip counts.
    #[command(alias = "ch")]
    Chips,

    /// Show the pot.
    #[command(alias = "p")]
    Pot,

    /// Show the event log.
    #[command(alias = "l")]
    Log,

    /// Exit the client.
    #[command(alias = "q")]
    Quit,
}

// ── Main ─────────────────────────────────────────────────────────────────────

#[cfg(feature = "grpc")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;

    println!("╔══════════════════════════════════════════════════╗");
    println!("║       pkcore  Dealer  gRPC  Client  v0.1         ║");
    println!("║  Tab-complete commands · Ctrl-D or quit to exit  ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    let server_addr = std::env::var("DEALER_SERVER").unwrap_or_else(|_| "http://localhost:50051".to_string());
    println!("  Connecting to: {}", server_addr);

    let mut client = rt.block_on(DealerServiceClient::connect(server_addr.clone()))?;
    println!("  ✓ Connected!");
    println!();

    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("dealer".to_owned()),
        ..DefaultPrompt::default()
    };

    let rl = ClapEditor::<Command>::builder()
        .with_prompt(Box::new(prompt))
        .with_editor_hook(|reed| {
            reed.with_history(Box::new(
                FileBackedHistory::with_file(1_000, "./generated/dealer-grpc-client-history".into())
                    .unwrap_or_default(),
            ))
        })
        .build();

    rl.repl(|command| {
        if let Err(e) = rt.block_on(handle(&mut client, command)) {
            println!("✗ Error: {}", e);
        }
    });

    Ok(())
}

// ── Command dispatch ──────────────────────────────────────────────────────────

#[cfg(feature = "grpc")]
type CliResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "grpc")]
async fn handle(client: &mut DealerServiceClient<tonic::transport::Channel>, command: Command) -> CliResult {
    match command {
        Command::Seat { name, chips } => handle_seat(client, name, chips).await?,
        Command::SeatAt { seat, name, chips } => handle_seat_at(client, seat, name, chips).await?,
        Command::Remove { seat } => handle_remove(client, seat).await?,
        Command::Start => handle_start(client).await?,
        Command::Street => handle_street(client).await?,
        Command::End => handle_end(client).await?,
        Command::Bet { seat, amount } => {
            perform_player_action(
                client,
                seat,
                ActionType::Bet,
                amount,
                format!("✓ Seat {seat} bets {amount}"),
            )
            .await?;
        }
        Command::Call { seat } => {
            perform_player_action(client, seat, ActionType::Call, 0, format!("✓ Seat {seat} calls")).await?;
        }
        Command::Check { seat } => {
            perform_player_action(client, seat, ActionType::Check, 0, format!("✓ Seat {seat} checks")).await?;
        }
        Command::Raise { seat, amount } => {
            perform_player_action(
                client,
                seat,
                ActionType::Raise,
                amount,
                format!("✓ Seat {seat} raises to {amount}"),
            )
            .await?;
        }
        Command::Allin { seat } => {
            perform_player_action(client, seat, ActionType::AllIn, 0, format!("✓ Seat {seat} is all-in")).await?;
        }
        Command::Fold { seat } => {
            perform_player_action(client, seat, ActionType::Fold, 0, format!("✓ Seat {seat} folds")).await?;
        }
        Command::Status => handle_status(client).await?,
        Command::Next => handle_next(client).await?,
        Command::Board => handle_board(client).await?,
        Command::Chips => handle_chips(client).await?,
        Command::Pot => handle_pot(client).await?,
        Command::Log => handle_log(client).await?,
        Command::Quit => {
            println!("Goodbye! 👋");
            std::process::exit(0);
        }
    }

    Ok(())
}

#[cfg(feature = "grpc")]
async fn handle_seat(
    client: &mut DealerServiceClient<tonic::transport::Channel>,
    name: String,
    chips: u32,
) -> CliResult {
    let response = client
        .seat_player(SeatPlayerRequest {
            name: name.clone(),
            chips,
        })
        .await?
        .into_inner();

    match response.result {
        Some(seat_player_response::Result::SeatNumber(seat)) => {
            println!("✓ {name} seated at seat {seat} with {chips} chips");
        }
        Some(seat_player_response::Result::Error(e)) => println!("✗ {e}"),
        None => println!("✗ Invalid response from server"),
    }

    Ok(())
}

#[cfg(feature = "grpc")]
async fn handle_seat_at(
    client: &mut DealerServiceClient<tonic::transport::Channel>,
    seat: u32,
    name: String,
    chips: u32,
) -> CliResult {
    let response = client
        .seat_player_at(SeatPlayerAtRequest {
            seat,
            name: name.clone(),
            chips,
        })
        .await?
        .into_inner();

    match response.result {
        Some(seat_player_at_response::Result::Success(_)) => {
            println!("✓ {name} seated at seat {seat} with {chips} chips");
        }
        Some(seat_player_at_response::Result::Error(e)) => println!("✗ {e}"),
        None => println!("✗ Invalid response from server"),
    }

    Ok(())
}

#[cfg(feature = "grpc")]
async fn handle_remove(client: &mut DealerServiceClient<tonic::transport::Channel>, seat: u32) -> CliResult {
    let response = client.remove_player(RemovePlayerRequest { seat }).await?.into_inner();

    match response.result {
        Some(remove_player_response::Result::PlayerName(name)) => {
            println!("✓ {name} removed from seat {seat}");
        }
        Some(remove_player_response::Result::Error(e)) => println!("✗ {e}"),
        None => println!("✗ Invalid response from server"),
    }

    Ok(())
}

#[cfg(feature = "grpc")]
async fn handle_start(client: &mut DealerServiceClient<tonic::transport::Channel>) -> CliResult {
    let response = client.start_hand(StartHandRequest {}).await?.into_inner();

    match response.result {
        Some(start_hand_response::Result::Status(status)) => {
            println!("✓ Hand started — blinds posted and hole cards dealt");
            print_status(&status);
        }
        Some(start_hand_response::Result::Error(e)) => println!("✗ {e}"),
        None => println!("✗ Invalid response from server"),
    }

    Ok(())
}

#[cfg(feature = "grpc")]
async fn handle_street(client: &mut DealerServiceClient<tonic::transport::Channel>) -> CliResult {
    let response = client.advance_street(AdvanceStreetRequest {}).await?.into_inner();

    match response.result {
        Some(advance_street_response::Result::StreetResult(result)) => {
            if result.board.trim().is_empty() {
                println!("✓ Bets consolidated");
            } else {
                println!("✓ Board: {}", result.board);
            }
            println!("  Action to seat {}  pot: {}", result.next_to_act, result.pot);
        }
        Some(advance_street_response::Result::Error(e)) => println!("✗ {e}"),
        None => println!("✗ Invalid response from server"),
    }

    Ok(())
}

#[cfg(feature = "grpc")]
async fn handle_end(client: &mut DealerServiceClient<tonic::transport::Channel>) -> CliResult {
    let response = client.end_hand(EndHandRequest {}).await?.into_inner();

    handle_result_or_error(response.result, |r| match r {
        end_hand_response::Result::HandResult(result) => {
            println!("✓ Hand complete");
            println!("{}", result.result_text);
            println!();
            print_chips(&result.final_chips);
            Some(String::new())
        }
        end_hand_response::Result::Error(_) => None,
    })
}

#[cfg(feature = "grpc")]
async fn handle_status(client: &mut DealerServiceClient<tonic::transport::Channel>) -> CliResult {
    let response = client.get_status(GetStatusRequest {}).await?.into_inner();
    if let Some(status) = response.status {
        print_status(&status);
    }
    Ok(())
}

#[cfg(feature = "grpc")]
async fn handle_next(client: &mut DealerServiceClient<tonic::transport::Channel>) -> CliResult {
    let response = client.get_next_to_act(GetNextToActRequest {}).await?.into_inner();

    handle_result_or_error(response.result, |r| match r {
        get_next_to_act_response::Result::Info(info) => Some(format!(
            "  Action to seat {} ({})  chips: {}  pot: {}",
            info.seat, info.player_name, info.chips, info.pot
        )),
        get_next_to_act_response::Result::Message(msg) => Some(format!("  {msg}")),
    })
}

#[cfg(feature = "grpc")]
async fn handle_board(client: &mut DealerServiceClient<tonic::transport::Channel>) -> CliResult {
    let response = client.get_board(GetBoardRequest {}).await?.into_inner();
    if response.board.trim().is_empty() {
        println!("Board: (no community cards yet)");
    } else {
        println!("Board: {}", response.board);
    }
    Ok(())
}

#[cfg(feature = "grpc")]
async fn handle_chips(client: &mut DealerServiceClient<tonic::transport::Channel>) -> CliResult {
    let response = client.get_chips(GetChipsRequest {}).await?.into_inner();
    print_chips(&response.chips);
    Ok(())
}

#[cfg(feature = "grpc")]
async fn handle_pot(client: &mut DealerServiceClient<tonic::transport::Channel>) -> CliResult {
    let response = client.get_pot(GetPotRequest {}).await?.into_inner();
    println!("Pot: {}", response.pot);
    Ok(())
}

#[cfg(feature = "grpc")]
async fn handle_log(client: &mut DealerServiceClient<tonic::transport::Channel>) -> CliResult {
    let response = client.get_event_log(GetEventLogRequest {}).await?.into_inner();
    println!("{}", "─".repeat(60));
    println!("{}", response.log);
    println!("{}", "─".repeat(60));
    Ok(())
}

#[cfg(feature = "grpc")]
async fn perform_player_action(
    client: &mut DealerServiceClient<tonic::transport::Channel>,
    seat: u32,
    action_type: ActionType,
    amount: u32,
    success_message: String,
) -> CliResult {
    let response = client
        .act(ActRequest {
            action: Some(PlayerAction {
                seat,
                action_type: action_type as i32,
                amount,
            }),
        })
        .await?
        .into_inner();

    match response.result {
        Some(act_response::Result::ActionResult(result)) => {
            println!("{success_message}");
            if !result.hand_complete {
                println!("  Action to seat {}  pot: {}", result.next_to_act, result.pot);
            }
        }
        Some(act_response::Result::Error(e)) => {
            println!("✗ {e}");
        }
        None => println!("✗ Invalid response from server"),
    }

    Ok(())
}

// ── Display helpers ───────────────────────────────────────────────────────────

#[cfg(feature = "grpc")]
fn print_status(status: &TableStatus) {
    println!("{}", "═".repeat(60));
    println!("Table Status:");
    println!();
    for seat in &status.seats {
        println!(
            "  Seat {}  {}  →  {} chips  [{}]",
            seat.seat_number, seat.player_name, seat.chips, seat.state
        );
    }
    println!();
    if !status.board.trim().is_empty() {
        println!("  Board: {}", status.board);
    }
    println!("  Pot: {}", status.pot);
    if status.hand_in_progress && !status.game_over {
        println!("  Next to act: seat {}", status.next_to_act);
    }
    println!("{}", "═".repeat(60));
}

#[cfg(feature = "grpc")]
fn print_chips(chips: &[PlayerChips]) {
    println!("{}", "─".repeat(40));
    for pc in chips {
        println!("  Seat {}  {}  →  {} chips", pc.seat, pc.player_name, pc.chips);
    }
    println!("{}", "─".repeat(40));
}

/// Generic helper for handling gRPC responses with a success/error/none pattern.
/// The closure should return Ok(message) for success cases, Err(error_string) for error cases.
#[cfg(feature = "grpc")]
fn handle_result_or_error<T, F>(result: Option<T>, success_handler: F) -> CliResult
where
    F: FnOnce(T) -> Result<String, String>,
{
    match result {
        Some(r) => match success_handler(r) {
            Ok(msg) => {
                if !msg.is_empty() {
                    println!("{msg}");
                }
            }
            Err(e) => println!("✗ {e}"),
        },
        None => println!("✗ Invalid response from server"),
    }
    Ok(())
}
