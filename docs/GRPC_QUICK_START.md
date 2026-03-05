# gRPC Quick Reference

## Start Server

```bash
cargo run --example dealer_grpc_server --features grpc
```

Server starts on `0.0.0.0:50051`

## Connect with Rust Client

```bash
cargo run --example dealer_grpc_client --features grpc
```

## Connect with Python Client

### First time only:
```bash
./scripts/setup_python_grpc.sh
```

### Every time:
```bash
python3 examples/dealer_grpc_client.py
```

## Environment Variables

```bash
# Change server address (default: localhost:50051)
export DEALER_SERVER=http://example.com:50051
cargo run --example dealer_grpc_client --features grpc
```

## Commands (Same as dealer_repl)

| Command | Alias | Example |
|---------|-------|---------|
| seat NAME [CHIPS] | s | `seat Alice 10000` |
| seat-at SEAT NAME [CHIPS] | sa | `seat-at 2 Bob 5000` |
| remove SEAT | rm | `remove 2` |
| start | st | `start` |
| street | sv | `street` |
| end | e | `end` |
| bet SEAT AMOUNT | b | `bet 0 400` |
| call SEAT | c | `call 1` |
| check SEAT | ck | `check 2` |
| raise SEAT AMOUNT | r | `raise 0 900` |
| allin SEAT | ai | `allin 3` |
| fold SEAT | f | `fold 0` |
| status | sh | `status` |
| next | n | `next` |
| board | bo | `board` |
| chips | ch | `chips` |
| pot | p | `pot` |
| log | l | `log` |
| quit | q | `quit` |

## Example Session

```bash
# Terminal 1: Start server
cargo run --example dealer_grpc_server --features grpc

# Terminal 2: Run client
cargo run --example dealer_grpc_client --features grpc

# In the client:
dealer❯ seat Alice 10000
dealer❯ seat Bob 10000
dealer❯ seat Carol 10000
dealer❯ start
dealer❯ call 0
dealer❯ call 1
dealer❯ check 2
dealer❯ street
dealer❯ check 1
dealer❯ bet 2 200
dealer❯ fold 0
dealer❯ call 1
dealer❯ street
dealer❯ check 1
dealer❯ check 2
dealer❯ street
dealer❯ check 1
dealer❯ check 2
dealer❯ end
```

## Troubleshooting

### "Failed to connect"
- Make sure the server is running
- Check the server address/port
- Verify firewall settings

### "Permission denied"
- Port 50051 may be in use
- Try a different port (modify the examples)

### Python: "Module not found"
- Run `./scripts/setup_python_grpc.sh`
- Make sure you're in the project root directory

### Build errors
- Ensure you're using `--features grpc`
- Check that `build.rs` is present
- Verify proto file exists at `proto/dealer.proto`

## Files Created

```
pkcore/
├── build.rs                           # Proto compilation config
├── proto/
│   ├── dealer.proto                   # Service definition
│   └── README.md                      # Proto documentation
├── examples/
│   ├── dealer_grpc_server.rs          # Server implementation
│   ├── dealer_grpc_client.rs          # Rust client
│   └── dealer_grpc_client.py          # Python client
├── scripts/
│   └── setup_python_grpc.sh           # Python setup script
└── docs/
    ├── GRPC_DEALER.md                 # High-level guide
    └── GRPC_IMPLEMENTATION_SUMMARY.md # Implementation details
```

## More Information

- **High-level overview**: `docs/GRPC_DEALER.md`
- **Implementation details**: `docs/GRPC_IMPLEMENTATION_SUMMARY.md`
- **Proto documentation**: `proto/README.md`

