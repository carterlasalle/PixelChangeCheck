# PixelChangeCheck (PCC)

A highly efficient screen sharing platform using PixelChangeCheck (PCC) for optimized data transmission.

## Features

- Efficient screen sharing with up to 60fps (targeting stable 30fps)
- Dynamic quality and bitrate adjustment
- Pixel-level change detection to minimize data transmission
- Real-time streaming with low latency
- Localhost testing support

## Project Structure

```
src/
├── client/           # Client-side code
│   ├── capture/      # Screen capture functionality
│   └── network/      # Client network handling
├── server/           # Server-side code
│   ├── network/      # Server network handling
│   └── renderer/     # Frame rendering logic
└── shared/           # Shared code
    ├── types/        # TypeScript type definitions
    └── utils/        # Shared utilities
```

## Getting Started

### Prerequisites

- Node.js (v14 or higher)
- Yarn package manager

### Installation

```bash
# Clone the repository
git clone https://github.com/carterlasalle/PixelChangeCheck.git

# Install dependencies
yarn install
```

### Development

```bash
# Start the server in development mode
yarn dev:server

# Start the client in development mode
yarn dev:client
```

### Building

```bash
# Build the project
yarn build
```

### Running

```bash
# Start the server
yarn start:server

# Start the client
yarn start:client
```

## Architecture

The project uses a client-server architecture where:

- **Client**: Captures screen content and detects pixel changes
- **Server**: Receives updates and renders the screen content
- **PCC Framework**: Optimizes data transmission by only sending changed pixels

## License

MIT 