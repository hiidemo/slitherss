# Slitherss - Rust Implementation

This is a complete rewrite of the Slither.io server in Rust, providing better performance, memory safety, and modern async I/O.

## Original Project

The original C++ implementation can be found in this repository. This Rust version maintains compatibility with the Slither.io protocol while leveraging Rust's features for improved reliability and performance.

## Features

- ✅ WebSocket-based multiplayer server
- ✅ Snake movement and physics
- ✅ Collision detection with spatial partitioning (sectors)
- ✅ Food spawning and consumption
- ✅ Bot support with basic AI
- ✅ Async I/O with Tokio
- ✅ Protocol-compatible with original Slither.io clients

## Project Structure

```
src/
├── main.rs              # Entry point
├── config.rs            # Configuration and CLI arguments
├── game/
│   ├── mod.rs          # Game module exports
│   ├── food.rs         # Food structures
│   ├── math.rs         # Math utilities
│   ├── sector.rs       # Spatial partitioning
│   ├── snake.rs        # Snake logic and movement
│   └── world.rs        # Game world management
├── packet/
│   └── mod.rs          # Network protocol packets
└── server/
    ├── mod.rs          # Server module exports
    └── game_server.rs  # WebSocket server and session management
```

## Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

## Running the Server

```bash
# Run with default settings (port 8080, no bots)
cargo run --release

# Run with bots
cargo run --release -- --bots 50 --avg-len 50

# Run on custom port
cargo run --release -- --port 9000

# Show help
cargo run --release -- --help
```

## Command-line Options

```
Options:
  -p, --port <PORT>         Server port [default: 8080]
      --bots <BOTS>         Number of bots to spawn [default: 0]
      --avg-len <AVG_LEN>   Average snake length [default: 2]
      --min-len <MIN_LEN>   Minimum snake length [default: 2]
  -v, --verbose             Verbose logging
  -d, --debug               Debug logging
  -h, --help                Print help
```

## Connecting to the Server

Use the same method as the C++ version:

1. Open Slither.io in your browser
2. Open browser console (F12)
3. Run: `window.bso = { ip: "127.0.0.1", po: 8080 }; window.forcing = true; window.want_play = true;`

Or use the debug client from [Slither.io Protocol](https://github.com/sitano/Slither.io-Protocol).

## Performance

The Rust implementation offers:

- **Memory Safety**: No buffer overflows or use-after-free bugs
- **Async I/O**: Non-blocking WebSocket handling with Tokio
- **Zero-cost Abstractions**: Performance comparable to C++ with safer code
- **Better Concurrency**: Safe multi-threading with Rust's ownership system

## Differences from C++ Version

- Async/await instead of callback-based I/O
- Stronger type safety
- Automatic memory management without garbage collection
- Modern error handling with Result types

## Dependencies

- `tokio`: Async runtime
- `tokio-tungstenite`: WebSocket server
- `futures-util`: Async utilities
- `serde`: Serialization framework
- `rand`: Random number generation
- `clap`: Command-line argument parsing
- `log` & `env_logger`: Logging
- `bytes`: Byte buffer manipulation

## Development

```bash
# Run with logging
RUST_LOG=info cargo run --release

# Run with debug logging
RUST_LOG=debug cargo run --release

# Format code
cargo fmt

# Lint code
cargo clippy

# Run tests
cargo test
```

## License

Same as original project - see LICENSE file.

## Credits

- Original C++ implementation by the original authors
- Rust rewrite maintains protocol compatibility
- Uses the Slither.io protocol documentation
