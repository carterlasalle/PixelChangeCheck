# PixelChangeCheck (PCC)

A highly efficient screen sharing platform using PixelChangeCheck (PCC) for optimized data transmission, written in Rust.

## Features

- **Screen Capture**: Cross-platform screen capture using the `screenshots` crate
- **Pixel Change Detection (PCC)**: Block-based pixel comparison that detects only changed regions between frames
- **JPEG Encoding**: Fast JPEG encoding with configurable quality for frame compression
- **LZ4 Compression**: Additional lossless compression for changed pixel regions
- **QUIC Transport**: Low-latency, reliable network transport using the QUIC protocol
- **Adaptive Quality**: Dynamic quality and frame rate adjustment
- **Frame Buffering**: Server-side frame buffer with partial update support

## Project Structure

```
src/
├── capture/          # Screen capture using screenshots crate
├── encoder/          # JPEG encoding and LZ4 compression
├── network/          # QUIC transport, protocol, and resilience
│   ├── config.rs     # Network and TLS configuration
│   ├── protocol.rs   # Message serialization protocol
│   ├── resilience.rs # Retry logic and connection health
│   └── transport.rs  # QUIC transport layer
├── pcc/              # Pixel Change Check core logic
│   ├── detector.rs   # Block-based change detection
│   └── types.rs      # Frame, PixelChange, and trait definitions
├── server/           # Server-side components
│   ├── network/      # Server network handling
│   └── renderer/     # Frame buffer and rendering
├── lib.rs            # Library exports
└── main.rs           # Binary entry point
```

## Getting Started

### Prerequisites

- Rust (1.70 or higher)
- System dependencies:
  - **Linux**: `libxcb1-dev`, `libxrandr-dev`, `libdbus-1-dev`
  - **macOS/Windows**: No extra dependencies needed

### Installation

```bash
# Clone the repository
git clone https://github.com/carterlasalle/PixelChangeCheck.git
cd PixelChangeCheck

# Install system dependencies (Linux)
# sudo apt-get install -y libxcb1-dev libxrandr-dev libdbus-1-dev

# Or run the setup script
./setup.sh
```

### Building

```bash
cargo build
```

### Running

```bash
# Run the client
cargo run

# Run the screen share example
cargo run --example simple_screen_share

# Run benchmarks
cargo run --example benchmarks
```

### Testing

```bash
cargo test
```

## Architecture

The project uses a client-server architecture:

- **Client**: Captures screen content, detects pixel changes using PCC, encodes changed regions, and transmits them over QUIC
- **Server**: Receives frame updates, maintains a frame buffer, and reconstructs the display
- **PCC Framework**: Compares frames block-by-block, only transmitting regions that have actually changed — dramatically reducing bandwidth for static or mostly-static screens

## How PCC Works

1. The client captures screen frames at the target frame rate
2. Each frame is compared against the previous frame using block-based pixel comparison
3. Only blocks where pixel values have changed beyond a configurable threshold are identified
4. Changed regions are extracted, encoded (JPEG), and compressed (LZ4)
5. The server receives partial updates and applies them to its frame buffer
6. If no pixels have changed, only a keep-alive signal is sent

## License

MIT 