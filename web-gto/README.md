# Poker GTO Web Application

A web-based interface for the pkcore poker library's GTO (Game Theory Optimal) calculations, built with Rust using warp, tokio, and reqwest.

## Features

- **Web Interface**: Simple, responsive HTML interface for GTO calculations
- **Real-time Analysis**: Calculate poker hand equity vs ranges
- **Board Support**: Optional board cards for flop/turn analysis
- **Fast Processing**: Asynchronous processing with detailed timing information
- **Error Handling**: Graceful error handling with user-friendly messages

## Prerequisites

1. Rust (latest stable version)
2. The pkcore library (parent directory)
3. Database file: `generated/hups.db` (generated from pkcore examples)

## Installation & Setup

1. Navigate to the web-gto directory:
   ```bash
   cd web-gto
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. Make sure the database exists (optional, but recommended for full functionality):
   ```bash
   # From the pkcore root directory
   cargo run --example hup  # This may take some time to generate the database
   ```

## Running the Application

Start the web server:
```bash
cargo run
```

The server will start on `http://localhost:3030`

## Usage

### Web Interface

Open your browser and navigate to `http://localhost:3030`. The interface provides:

- **Player Hand**: Enter a specific hand (e.g., "K♠ K♥" or "KsKh")
- **Villain Range**: Enter a range string (e.g., "66+,AJs+,KQs,AJo+,KQo")
- **Board** (optional): Enter board cards (e.g., "Kc 7h 2d")
- **Calculate Nuts**: Checkbox for nuts calculations

### API Endpoint

You can also make direct API calls to `/api/gto` with POST requests:

```bash
curl -X POST http://localhost:3030/api/gto \
  -H "Content-Type: application/json" \
  -d '{
    "player": "K♠ K♥",
    "villain": "66+,AJs+,KQs,AJo+,KQo",
    "board": "Kc 7h 2d",
    "nuts": false
  }'
```

### Example Inputs

**Player Hands:**
- `K♠ K♥` or `KsKh` (pocket kings)
- `A♠ K♠` or `AsKs` (ace-king suited)
- `7♥ 7♦` or `7h7d` (pocket sevens)

**Villain Ranges:**
- `66+,AJs+,KQs,AJo+,KQo` (medium-strong range)
- `22+` (all pocket pairs)
- `ATs+,KJs+,QJs,AJo+,KQo` (tight range)

**Board Cards:**
- `Kc 7h 2d` (king-high rainbow)
- `As Kh Qd` (ace-high straight potential)
- `9s 8s 7c` (coordinated board)

## Response Format

The API returns JSON with the following structure:

```json
{
  "player": "K♠ K♥",
  "villain": "66+,AJs+,KQs,AJo+,KQo",
  "board": "Kc 7h 2d",
  "combo_pairs": "...",
  "villain_combo_pairs": "...",
  "results": {
    "win_lose_draw": "...",
    "hup_results": ["..."]
  },
  "flop_results": "FLOP: ...",
  "turn_results": "TURN: ...",
  "elapsed_ms": 42,
  "error": null
}
```

## Architecture

- **Backend**: Rust with warp web framework
- **Frontend**: Vanilla HTML/CSS/JavaScript
- **Async Processing**: tokio runtime for non-blocking operations
- **Poker Logic**: pkcore library for all GTO calculations
- **Database**: SQLite for precomputed hand vs hand results

## Performance Notes

- Initial calculation may take longer if the HUP database needs to be built
- Subsequent calculations should be fast (typically under 100ms)
- Complex ranges may take longer to process
- The application is single-threaded but uses async I/O for responsiveness

## Error Handling

The application handles various error cases:
- Invalid hand notation
- Invalid range strings
- Invalid board cards
- Database connection issues
- Missing database files

Errors are returned as JSON with descriptive messages.

## Development

To run in development mode with logging:

```bash
RUST_LOG=info cargo run
```

This will show request logs and processing information.

## Extending the Application

The code is structured to make it easy to add new features:

- Add new endpoints by extending the `routes` in `main.rs`
- Modify the calculation logic in `process_gto_calculation`
- Extend the frontend by modifying the HTML template
- Add new API parameters by extending the `GTORequest` struct

## Dependencies

- `warp`: Web framework
- `tokio`: Async runtime
- `reqwest`: HTTP client (for future extensions)
- `serde`: JSON serialization
- `pkcore`: Poker calculation library
- `anyhow`: Error handling
- `log` + `env_logger`: Logging

## License

This project follows the same license as the parent pkcore library (GPL-3.0-or-later).