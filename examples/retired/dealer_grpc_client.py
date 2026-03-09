#!/usr/bin/env python3
"""
Python gRPC client for the pkcore Dealer service.

This demonstrates how to interact with the Dealer service from Python,
showing that gRPC enables cross-language communication.

Prerequisites:
    pip install grpcio grpcio-tools

Generate the Python stubs:
    python -m grpc_tools.protoc -I./proto --python_out=. --grpc_python_out=. proto/dealer.proto

Usage:
    python examples/dealer_grpc_client.py
"""

import grpc
import sys

# These imports assume you've generated the Python stubs
# python -m grpc_tools.protoc -I./proto --python_out=. --grpc_python_out=. proto/dealer.proto
try:
    import dealer_pb2
    import dealer_pb2_grpc
except ImportError:
    print("Error: Generated proto files not found!")
    print("Run: python -m grpc_tools.protoc -I./proto --python_out=. --grpc_python_out=. proto/dealer.proto")
    sys.exit(1)


class DealerClient:
    """Simple wrapper around the gRPC client."""

    def __init__(self, server_address="localhost:50051"):
        self.channel = grpc.insecure_channel(server_address)
        self.stub = dealer_pb2_grpc.DealerServiceStub(self.channel)

    def seat_player(self, name, chips=10000):
        """Seat a new player."""
        request = dealer_pb2.SeatPlayerRequest(name=name, chips=chips)
        response = self.stub.SeatPlayer(request)

        if response.HasField("seat_number"):
            print(f"✓ {name} seated at seat {response.seat_number} with {chips} chips")
            return response.seat_number
        else:
            print(f"✗ {response.error}")
            return None

    def start_hand(self):
        """Start a new hand."""
        request = dealer_pb2.StartHandRequest()
        response = self.stub.StartHand(request)

        if response.HasField("status"):
            print("✓ Hand started — blinds posted and hole cards dealt")
            self.print_status(response.status)
        else:
            print(f"✗ {response.error}")

    def bet(self, seat, amount):
        """Player bets."""
        action = dealer_pb2.PlayerAction(
            seat=seat,
            action_type=dealer_pb2.BET,
            amount=amount
        )
        request = dealer_pb2.ActRequest(action=action)
        response = self.stub.Act(request)

        if response.HasField("action_result"):
            print(f"✓ Seat {seat} bets {amount}")
            result = response.action_result
            if not result.hand_complete:
                print(f"  Action to seat {result.next_to_act}  pot: {result.pot}")
        else:
            print(f"✗ {response.error}")

    def call(self, seat):
        """Player calls."""
        action = dealer_pb2.PlayerAction(
            seat=seat,
            action_type=dealer_pb2.CALL,
            amount=0
        )
        request = dealer_pb2.ActRequest(action=action)
        response = self.stub.Act(request)

        if response.HasField("action_result"):
            print(f"✓ Seat {seat} calls")
            result = response.action_result
            if not result.hand_complete:
                print(f"  Action to seat {result.next_to_act}  pot: {result.pot}")
        else:
            print(f"✗ {response.error}")

    def check(self, seat):
        """Player checks."""
        action = dealer_pb2.PlayerAction(
            seat=seat,
            action_type=dealer_pb2.CHECK,
            amount=0
        )
        request = dealer_pb2.ActRequest(action=action)
        response = self.stub.Act(request)

        if response.HasField("action_result"):
            print(f"✓ Seat {seat} checks")
            result = response.action_result
            if not result.hand_complete:
                print(f"  Action to seat {result.next_to_act}  pot: {result.pot}")
        else:
            print(f"✗ {response.error}")

    def fold(self, seat):
        """Player folds."""
        action = dealer_pb2.PlayerAction(
            seat=seat,
            action_type=dealer_pb2.FOLD,
            amount=0
        )
        request = dealer_pb2.ActRequest(action=action)
        response = self.stub.Act(request)

        if response.HasField("action_result"):
            print(f"✓ Seat {seat} folds")
            result = response.action_result
            if not result.hand_complete:
                print(f"  Action to seat {result.next_to_act}  pot: {result.pot}")
        else:
            print(f"✗ {response.error}")

    def advance_street(self):
        """Advance to the next street."""
        request = dealer_pb2.AdvanceStreetRequest()
        response = self.stub.AdvanceStreet(request)

        if response.HasField("street_result"):
            result = response.street_result
            if result.board.strip():
                print(f"✓ Board: {result.board}")
            else:
                print("✓ Bets consolidated")
            print(f"  Action to seat {result.next_to_act}  pot: {result.pot}")
        else:
            print(f"✗ {response.error}")

    def end_hand(self):
        """End the current hand."""
        request = dealer_pb2.EndHandRequest()
        response = self.stub.EndHand(request)

        if response.HasField("hand_result"):
            print("✓ Hand complete")
            print(response.hand_result.result_text)
            print()
            self.print_chips(response.hand_result.final_chips)
        else:
            print(f"✗ {response.error}")

    def get_status(self):
        """Get the current table status."""
        request = dealer_pb2.GetStatusRequest()
        response = self.stub.GetStatus(request)
        self.print_status(response.status)

    def get_board(self):
        """Get the board."""
        request = dealer_pb2.GetBoardRequest()
        response = self.stub.GetBoard(request)
        if response.board.strip():
            print(f"Board: {response.board}")
        else:
            print("Board: (no community cards yet)")

    def get_pot(self):
        """Get the pot."""
        request = dealer_pb2.GetPotRequest()
        response = self.stub.GetPot(request)
        print(f"Pot: {response.pot}")

    def print_status(self, status):
        """Print the table status."""
        print("=" * 60)
        print("Table Status:")
        print()
        for seat in status.seats:
            print(f"  Seat {seat.seat_number}  {seat.player_name}  →  {seat.chips} chips  [{seat.state}]")
        print()
        if status.board.strip():
            print(f"  Board: {status.board}")
        print(f"  Pot: {status.pot}")
        if status.hand_in_progress and not status.game_over:
            print(f"  Next to act: seat {status.next_to_act}")
        print("=" * 60)

    def print_chips(self, chips):
        """Print chip counts."""
        print("─" * 40)
        for pc in chips:
            print(f"  Seat {pc.seat}  {pc.player_name}  →  {pc.chips} chips")
        print("─" * 40)

    def close(self):
        """Close the channel."""
        self.channel.close()


def run_example():
    """Run a simple example game."""
    print("╔══════════════════════════════════════════════════╗")
    print("║     pkcore Dealer Python gRPC Client v0.1       ║")
    print("╚══════════════════════════════════════════════════╝")
    print()

    client = DealerClient()

    try:
        print("Seating players...")
        client.seat_player("Alice", 10000)
        client.seat_player("Bob", 10000)
        client.seat_player("Carol", 10000)
        print()

        print("Starting hand...")
        client.start_hand()
        print()

        print("Player actions...")
        # Seat 0 (button) acts first preflop after blinds
        client.call(0)  # button calls
        client.call(1)  # small blind completes
        client.check(2) # big blind checks
        print()

        print("Advancing to flop...")
        client.advance_street()
        print()

        print("Flop action...")
        client.check(1)  # SB checks
        client.bet(2, 200)  # BB bets
        client.fold(0)  # button folds
        client.call(1)  # SB calls
        print()

        print("Advancing to turn...")
        client.advance_street()
        print()

        print("Turn action...")
        client.check(1)  # SB checks
        client.check(2)  # BB checks
        print()

        print("Advancing to river...")
        client.advance_street()
        print()

        print("River action...")
        client.check(1)  # SB checks
        client.check(2)  # BB checks
        print()

        print("Ending hand...")
        client.end_hand()
        print()

    except grpc.RpcError as e:
        print(f"RPC error: {e.code()}: {e.details()}")
    finally:
        client.close()


if __name__ == "__main__":
    run_example()

