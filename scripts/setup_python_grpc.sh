#!/bin/bash
# Setup script for Python gRPC client

set -e

echo "╔══════════════════════════════════════════════════╗"
echo "║   Python gRPC Client Setup for pkcore Dealer    ║"
echo "╚══════════════════════════════════════════════════╝"
echo

# Check if Python 3 is installed
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is not installed. Please install Python 3 first."
    exit 1
fi

echo "✓ Python 3 found: $(python3 --version)"

# Check if pip is installed
if ! command -v pip3 &> /dev/null; then
    echo "❌ pip3 is not installed. Please install pip3 first."
    exit 1
fi

echo "✓ pip3 found"
echo

# Install required packages
echo "📦 Installing required Python packages..."
pip3 install grpcio grpcio-tools

echo
echo "✓ Packages installed"
echo

# Generate Python stubs from proto file
echo "🔨 Generating Python code from proto file..."
python3 -m grpc_tools.protoc \
    -I./proto \
    --python_out=. \
    --grpc_python_out=. \
    proto/dealer.proto

echo "✓ Generated dealer_pb2.py and dealer_pb2_grpc.py"
echo

echo "╔══════════════════════════════════════════════════╗"
echo "║                Setup Complete! ✓                 ║"
echo "╚══════════════════════════════════════════════════╝"
echo
echo "Next steps:"
echo "  1. Start the server in one terminal:"
echo "     cargo run --example dealer_grpc_server --features grpc"
echo
echo "  2. Run the Python client in another terminal:"
echo "     python3 examples/dealer_grpc_client.py"
echo

