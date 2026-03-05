//! gRPC server for the [`Dealer`] struct.
//!
//! This example demonstrates how to expose the Dealer functionality as a
//! remote service using gRPC/Protobuf, allowing clients in any language
//! to manage poker tables over the network.
//!
//! # Usage
//!
//! Start the server:
//! ```bash
//! cargo run --example dealer_grpc_server --features grpc
//! ```
//!
//! The server will listen on `0.0.0.0:50051` by default.
//!
//! # Features
//!
//! - All dealer operations (seating, starting hands, player actions)
//! - Thread-safe concurrent access to the dealer
//! - Proper error handling and conversion to gRPC status codes
//! - Optional event streaming for real-time updates
//!
//! # Client Examples
//!
//! See `dealer_grpc_client.rs` for a Rust client example.

#[cfg(not(feature = "grpc"))]
fn main() {
    eprintln!("This example requires the `grpc` feature. Run:");
    eprintln!("  cargo run --example dealer_grpc_server --features grpc");
}

#[cfg(feature = "grpc")]
use pkcore::casino::dealer::{Dealer, DealerAction};
#[cfg(feature = "grpc")]
use pkcore::casino::game::ForcedBets;
#[cfg(feature = "grpc")]
use pkcore::casino::player::Player;
#[cfg(feature = "grpc")]
use std::sync::Arc;
#[cfg(feature = "grpc")]
use tokio::sync::Mutex;
#[cfg(feature = "grpc")]
use tonic::{Request, Response, Status, transport::Server};

// Include the generated proto code
#[cfg(feature = "grpc")]
pub mod dealer_proto {
    tonic::include_proto!("pkcore.dealer");
}

#[cfg(feature = "grpc")]
use dealer_proto::{
    dealer_service_server::{DealerService, DealerServiceServer},
    *,
};

// ── Service Implementation ────────────────────────────────────────────────────

/// Implementation of the DealerService gRPC service.
///
/// Each instance manages a single Dealer/Table. For multi-table support,
/// you would maintain a HashMap<SessionId, Arc<Mutex<Dealer>>>.
#[cfg(feature = "grpc")]
pub struct DealerServiceImpl {
    dealer: Arc<Mutex<Dealer>>,
}

#[cfg(feature = "grpc")]
impl DealerServiceImpl {
    pub fn new(forced_bets: ForcedBets, max_seats: u8) -> Self {
        Self {
            dealer: Arc::new(Mutex::new(Dealer::new(forced_bets, max_seats))),
        }
    }
}

#[cfg(feature = "grpc")]
#[tonic::async_trait]
impl DealerService for DealerServiceImpl {
    type StreamEventsStream = std::pin::Pin<
        Box<dyn tonic::codegen::tokio_stream::Stream<Item = Result<TableEvent, Status>> + Send + 'static>,
    >;

    async fn seat_player(&self, request: Request<SeatPlayerRequest>) -> Result<Response<SeatPlayerResponse>, Status> {
        let req = request.into_inner();
        let chips = if req.chips == 0 { 10_000 } else { req.chips as usize };
        let player = Player::new_with_chips(req.name, chips);

        let dealer = self.dealer.lock().await;
        match dealer.seat_player(player) {
            Ok(seat) => Ok(Response::new(SeatPlayerResponse {
                result: Some(seat_player_response::Result::SeatNumber(seat as u32)),
            })),
            Err(e) => Ok(Response::new(SeatPlayerResponse {
                result: Some(seat_player_response::Result::Error(e.to_string())),
            })),
        }
    }

    async fn seat_player_at(
        &self,
        request: Request<SeatPlayerAtRequest>,
    ) -> Result<Response<SeatPlayerAtResponse>, Status> {
        let req = request.into_inner();
        let chips = if req.chips == 0 { 10_000 } else { req.chips as usize };
        let player = Player::new_with_chips(req.name, chips);

        let dealer = self.dealer.lock().await;
        match dealer.seat_player_at(player, req.seat as u8) {
            Ok(()) => Ok(Response::new(SeatPlayerAtResponse {
                result: Some(seat_player_at_response::Result::Success(true)),
            })),
            Err(e) => Ok(Response::new(SeatPlayerAtResponse {
                result: Some(seat_player_at_response::Result::Error(e.to_string())),
            })),
        }
    }

    async fn remove_player(
        &self,
        request: Request<RemovePlayerRequest>,
    ) -> Result<Response<RemovePlayerResponse>, Status> {
        let req = request.into_inner();
        let mut dealer = self.dealer.lock().await;

        match dealer.remove_player(req.seat as u8) {
            Ok(player) => Ok(Response::new(RemovePlayerResponse {
                result: Some(remove_player_response::Result::PlayerName(player.handle)),
            })),
            Err(e) => Ok(Response::new(RemovePlayerResponse {
                result: Some(remove_player_response::Result::Error(e.to_string())),
            })),
        }
    }

    async fn start_hand(&self, _request: Request<StartHandRequest>) -> Result<Response<StartHandResponse>, Status> {
        let mut dealer = self.dealer.lock().await;

        match dealer.start_hand() {
            Ok(()) => {
                let status = build_table_status(&dealer);
                Ok(Response::new(StartHandResponse {
                    result: Some(start_hand_response::Result::Status(status)),
                }))
            }
            Err(e) => Ok(Response::new(StartHandResponse {
                result: Some(start_hand_response::Result::Error(e.to_string())),
            })),
        }
    }

    async fn advance_street(
        &self,
        _request: Request<AdvanceStreetRequest>,
    ) -> Result<Response<AdvanceStreetResponse>, Status> {
        let mut dealer = self.dealer.lock().await;

        match dealer.advance_street() {
            Ok(()) => {
                let board = dealer.table.board.to_string();
                let next_to_act = dealer.next_to_act() as u32;
                let pot = dealer.pot() as u32;

                Ok(Response::new(AdvanceStreetResponse {
                    result: Some(advance_street_response::Result::StreetResult(StreetResult {
                        board,
                        next_to_act,
                        pot,
                    })),
                }))
            }
            Err(e) => Ok(Response::new(AdvanceStreetResponse {
                result: Some(advance_street_response::Result::Error(e.to_string())),
            })),
        }
    }

    async fn end_hand(&self, _request: Request<EndHandRequest>) -> Result<Response<EndHandResponse>, Status> {
        let mut dealer = self.dealer.lock().await;

        match dealer.end_hand() {
            Ok(result) => {
                let result_text = result.to_string();
                let final_chips = (0..dealer.table.seats.size())
                    .filter_map(|i| {
                        dealer.table.get_seat(i).and_then(|s| {
                            if !s.is_empty() {
                                Some(PlayerChips {
                                    seat: i as u32,
                                    player_name: s.player.handle.clone(),
                                    chips: s.player.chips.count() as u32,
                                })
                            } else {
                                None
                            }
                        })
                    })
                    .collect();

                Ok(Response::new(EndHandResponse {
                    result: Some(end_hand_response::Result::HandResult(HandResult {
                        result_text,
                        final_chips,
                    })),
                }))
            }
            Err(e) => Ok(Response::new(EndHandResponse {
                result: Some(end_hand_response::Result::Error(e.to_string())),
            })),
        }
    }

    async fn act(&self, request: Request<ActRequest>) -> Result<Response<ActResponse>, Status> {
        let req = request.into_inner();
        let action = req
            .action
            .ok_or_else(|| Status::invalid_argument("action is required"))?;

        let dealer_action = match ActionType::try_from(action.action_type) {
            Ok(ActionType::Bet) => DealerAction::Bet {
                seat: action.seat as u8,
                amount: action.amount as usize,
            },
            Ok(ActionType::Call) => DealerAction::Call {
                seat: action.seat as u8,
            },
            Ok(ActionType::Check) => DealerAction::Check {
                seat: action.seat as u8,
            },
            Ok(ActionType::Raise) => DealerAction::Raise {
                seat: action.seat as u8,
                amount: action.amount as usize,
            },
            Ok(ActionType::AllIn) => DealerAction::AllIn {
                seat: action.seat as u8,
            },
            Ok(ActionType::Fold) => DealerAction::Fold {
                seat: action.seat as u8,
            },
            Err(_) => return Err(Status::invalid_argument("invalid action type")),
        };

        let dealer = self.dealer.lock().await;
        match dealer.act(dealer_action) {
            Ok(()) => {
                let next_to_act = dealer.next_to_act() as u32;
                let pot = dealer.pot() as u32;
                let hand_complete = dealer.table.is_game_over();

                Ok(Response::new(ActResponse {
                    result: Some(act_response::Result::ActionResult(ActionResult {
                        next_to_act,
                        pot,
                        hand_complete,
                    })),
                }))
            }
            Err(e) => Ok(Response::new(ActResponse {
                result: Some(act_response::Result::Error(e.to_string())),
            })),
        }
    }

    async fn get_status(&self, _request: Request<GetStatusRequest>) -> Result<Response<GetStatusResponse>, Status> {
        let dealer = self.dealer.lock().await;
        let status = build_table_status(&dealer);
        Ok(Response::new(GetStatusResponse { status: Some(status) }))
    }

    async fn get_next_to_act(
        &self,
        _request: Request<GetNextToActRequest>,
    ) -> Result<Response<GetNextToActResponse>, Status> {
        let dealer = self.dealer.lock().await;

        if !dealer.is_hand_in_progress() {
            return Ok(Response::new(GetNextToActResponse {
                result: Some(get_next_to_act_response::Result::Message(
                    "No hand in progress".to_string(),
                )),
            }));
        }

        if dealer.table.is_game_over() {
            return Ok(Response::new(GetNextToActResponse {
                result: Some(get_next_to_act_response::Result::Message("Hand is over".to_string())),
            }));
        }

        let seat = dealer.next_to_act();
        if let Some(s) = dealer.table.get_seat(seat) {
            Ok(Response::new(GetNextToActResponse {
                result: Some(get_next_to_act_response::Result::Info(NextToActInfo {
                    seat: seat as u32,
                    player_name: s.player.handle.clone(),
                    chips: s.player.chips.count() as u32,
                    pot: dealer.pot() as u32,
                })),
            }))
        } else {
            Ok(Response::new(GetNextToActResponse {
                result: Some(get_next_to_act_response::Result::Message("Invalid seat".to_string())),
            }))
        }
    }

    async fn get_board(&self, _request: Request<GetBoardRequest>) -> Result<Response<GetBoardResponse>, Status> {
        let dealer = self.dealer.lock().await;
        let board = dealer.table.board.to_string();
        Ok(Response::new(GetBoardResponse { board }))
    }

    async fn get_chips(&self, _request: Request<GetChipsRequest>) -> Result<Response<GetChipsResponse>, Status> {
        let dealer = self.dealer.lock().await;
        let chips = (0..dealer.table.seats.size())
            .filter_map(|i| {
                dealer.table.get_seat(i).and_then(|s| {
                    if !s.is_empty() {
                        Some(PlayerChips {
                            seat: i as u32,
                            player_name: s.player.handle.clone(),
                            chips: s.player.chips.count() as u32,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(Response::new(GetChipsResponse { chips }))
    }

    async fn get_pot(&self, _request: Request<GetPotRequest>) -> Result<Response<GetPotResponse>, Status> {
        let dealer = self.dealer.lock().await;
        let pot = dealer.pot() as u32;
        Ok(Response::new(GetPotResponse { pot }))
    }

    async fn get_event_log(
        &self,
        _request: Request<GetEventLogRequest>,
    ) -> Result<Response<GetEventLogResponse>, Status> {
        let dealer = self.dealer.lock().await;
        let log = dealer.event_log().to_string();
        Ok(Response::new(GetEventLogResponse { log }))
    }

    async fn stream_events(
        &self,
        _request: Request<StreamEventsRequest>,
    ) -> Result<Response<Self::StreamEventsStream>, Status> {
        // TODO: Implement event streaming
        // This would require a broadcast channel to notify all subscribers
        // when events occur. For now, return unimplemented.
        Err(Status::unimplemented("Event streaming not yet implemented"))
    }
}

// ── Helper Functions ──────────────────────────────────────────────────────────

#[cfg(feature = "grpc")]
fn build_table_status(dealer: &Dealer) -> TableStatus {
    let seats = (0..dealer.table.seats.size())
        .filter_map(|i| {
            dealer.table.get_seat(i).and_then(|s| {
                if !s.is_empty() {
                    Some(SeatInfo {
                        seat_number: i as u32,
                        player_name: s.player.handle.clone(),
                        chips: s.player.chips.count() as u32,
                        cards: s.cards.to_string(),
                        state: s.player.state.to_string(),
                    })
                } else {
                    None
                }
            })
        })
        .collect();

    TableStatus {
        seats,
        board: dealer.table.board.to_string(),
        pot: dealer.pot() as u32,
        next_to_act: dealer.next_to_act() as u32,
        hand_in_progress: dealer.is_hand_in_progress(),
        game_over: dealer.table.is_game_over(),
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[cfg(feature = "grpc")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let addr = "0.0.0.0:50051".parse()?;

    // Create a dealer with standard blinds (50/100) and 6 seats
    let dealer_service = DealerServiceImpl::new(ForcedBets::new(50, 100), 6);

    println!("╔══════════════════════════════════════════════════╗");
    println!("║       pkcore Dealer gRPC Server v0.1            ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("  Listening on: {}", addr);
    println!("  Blinds: SB 50 / BB 100");
    println!("  Max seats: 6");
    println!();
    println!("  Ready for client connections...");
    println!();

    Server::builder()
        .add_service(DealerServiceServer::new(dealer_service))
        .serve(addr)
        .await?;

    Ok(())
}
