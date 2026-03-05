# gRPC Dealer Service

This directory contains the Protocol Buffer definitions and implementation for
exposing the pkcore `Dealer` as a remote gRPC service.

## Quick Start

### 1. Build with gRPC support

```bash
cargo build --features grpc
```

### 2. Run the server

```bash
cargo run --example dealer_grpc_server --features grpc
```

The server will start on `0.0.0.0:50051`.

### 3. Run the Rust client

In another terminal:

```bash
cargo run --example dealer_grpc_client --features grpc
```

You'll get an interactive REPL that communicates with the remote server.

## Python Client

### Setup

Install the required packages:

```bash
pip install grpcio grpcio-tools
```

Generate the Python stubs from the proto file:

```bash
python -m grpc_tools.protoc \
  -I./proto \
  --python_out=. \
  --grpc_python_out=. \
  proto/dealer.proto
```

This will generate:
- `dealer_pb2.py` - Message definitions
- `dealer_pb2_grpc.py` - Service stubs

### Run the Python example

```bash
python examples/dealer_grpc_client.py
```

## Architecture

```
                                   Network (TCP/HTTP2)
                                           │
         ┌─────────────────────────────────┼─────────────────────────────────┐
         │                                 │                                 │
    ┌────▼─────┐                     ┌────▼─────┐                     ┌─────▼────┐
    │  Client  │                     │  Client  │                     │  Client  │
    │  (Rust)  │                     │ (Python) │                     │   (Go)   │
    └──────────┘                     └──────────┘                     └──────────┘
         │                                 │                                 │
         │         gRPC Protocol (Protobuf)│                                 │
         └─────────────────────────────────┼─────────────────────────────────┘
                                           │
                                    ┌──────▼───────┐
                                    │    Server    │
                                    │    (Rust)    │
                                    └──────┬───────┘
                                           │
                                    ┌──────▼───────┐
                                    │    Dealer    │
                                    │   + Table    │
                                    └──────────────┘
```

## Service Definition

The service is defined in `proto/dealer.proto`. Key RPC methods:

### Player Management
- `SeatPlayer` - Seat a player at the next available seat
- `SeatPlayerAt` - Seat a player at a specific seat
- `RemovePlayer` - Remove a player from their seat

### Game Flow
- `StartHand` - Start a new hand (shuffle, post blinds, deal)
- `AdvanceStreet` - Move to the next street (flop/turn/river)
- `EndHand` - Resolve the hand and pay winners

### Player Actions
- `Act` - Execute a player action (bet, call, check, raise, fold, all-in)

### Information
- `GetStatus` - Get complete table state
- `GetNextToAct` - Get who should act next
- `GetBoard` - Get community cards
- `GetChips` - Get all player chip counts
- `GetPot` - Get current pot size
- `GetEventLog` - Get full event history

### Advanced (TODO)
- `StreamEvents` - Subscribe to real-time table updates

## Writing Clients in Other Languages

### Go

```bash
# Install protoc compiler
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

# Generate Go code
protoc --go_out=. --go-grpc_out=. proto/dealer.proto
```

### JavaScript/TypeScript

```bash
npm install @grpc/grpc-js @grpc/proto-loader
# or for static code generation:
npm install -g grpc-tools
grpc_tools_node_protoc --js_out=import_style=commonjs,binary:. \
  --grpc_out=grpc_js:. --plugin=protoc-gen-grpc=`which grpc_tools_node_protoc_plugin` \
  proto/dealer.proto
```

### Java

Use the `protobuf-maven-plugin` or `protobuf-gradle-plugin` in your build config.

## Security

The current implementation uses **insecure channels** for development.

For production, you should:

1. **Enable TLS encryption**
```rust
let tls = ServerTlsConfig::new()
    .identity(Identity::from_pem(cert, key));

Server::builder()
    .tls_config(tls)?
    .add_service(service)
    .serve(addr)
    .await?;
```

2. **Add authentication**
   - JWT tokens in metadata
   - mTLS (mutual TLS)
   - API keys

3. **Rate limiting**
   - Prevent abuse
   - Protect against DoS

4. **Input validation**
   - Already handled by the Dealer/Table logic
   - But add request-level validation too

## Deployment

### Docker

Create a `Dockerfile`:

```dockerfile
FROM rust:1.85 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features grpc --example dealer_grpc_server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/examples/dealer_grpc_server /usr/local/bin/
CMD ["dealer_grpc_server"]
EXPOSE 50051
```

Build and run:

```bash
docker build -t pkcore-dealer .
docker run -p 50051:50051 pkcore-dealer
```

### Kubernetes

Example deployment manifest:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pkcore-dealer
spec:
  replicas: 3
  selector:
    matchLabels:
      app: pkcore-dealer
  template:
    metadata:
      labels:
        app: pkcore-dealer
    spec:
      containers:
      - name: dealer
        image: pkcore-dealer:latest
        ports:
        - containerPort: 50051
---
apiVersion: v1
kind: Service
metadata:
  name: pkcore-dealer
spec:
  selector:
    app: pkcore-dealer
  ports:
  - protocol: TCP
    port: 50051
    targetPort: 50051
  type: LoadBalancer
```

## Multi-Table Support

The current implementation manages a single table per server instance. For
multi-table support, modify the server to maintain a map of tables:

```rust
struct DealerServiceImpl {
    tables: Arc<Mutex<HashMap<String, Arc<Mutex<Dealer>>>>>,
}

// Add table_id to each request
message SeatPlayerRequest {
    string table_id = 1;
    string name = 2;
    uint32 chips = 3;
}
```

## Performance

gRPC uses HTTP/2 and Protocol Buffers, providing:
- Binary serialization (smaller than JSON)
- Multiplexing (multiple requests on one connection)
- Flow control
- Header compression

Typical latency: **1-5ms** on localhost, **10-50ms** over WAN.

## Monitoring

Add observability with:

1. **Logging** - Already using `log` crate
2. **Metrics** - Use `prometheus` crate
3. **Tracing** - Use `tracing` and `opentelemetry`

Example with metrics:

```rust
use prometheus::{Encoder, TextEncoder, Counter, register_counter};

let requests = register_counter!("dealer_requests_total", "Total requests").unwrap();

// In each handler
requests.inc();
```

## Testing

gRPC services are easy to test:

```rust
#[tokio::test]
async fn test_full_game_flow() {
    let addr = "127.0.0.1:50052".parse().unwrap();
    let service = DealerServiceImpl::new(ForcedBets::new(50, 100), 6);
    
    tokio::spawn(async move {
        Server::builder()
            .add_service(DealerServiceServer::new(service))
            .serve(addr)
            .await
            .unwrap();
    });
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let mut client = DealerServiceClient::connect("http://127.0.0.1:50052")
        .await
        .unwrap();
    
    // Seat players
    let response = client.seat_player(SeatPlayerRequest {
        name: "Alice".to_string(),
        chips: 10000,
    }).await.unwrap();
    
    // ... rest of test
}
```

## Comparison: REPL vs gRPC

| Feature | REPL | gRPC |
|---------|------|------|
| Local only | ✓ | ✗ |
| Remote access | ✗ | ✓ |
| Multi-language | ✗ | ✓ |
| Scriptable | Limited | ✓ |
| Web/mobile clients | ✗ | ✓ |
| Real-time updates | Manual | Streaming |
| Type safety | ✓ | ✓ |
| Horizontal scaling | ✗ | ✓ |

## Further Reading

- [gRPC Official Docs](https://grpc.io/docs/)
- [Tonic Documentation](https://docs.rs/tonic/)
- [Protocol Buffers Guide](https://protobuf.dev/)
- [gRPC Best Practices](https://grpc.io/docs/guides/performance/)

