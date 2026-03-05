#!/bin/bash

echo "PixelChangeCheck setup"

# Install system dependencies for screen capture
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Installing Linux dependencies..."
    sudo apt-get update
    sudo apt-get install -y \
        libxcb1-dev \
        libxrandr-dev \
        libdbus-1-dev \
        pkg-config
fi

# Build project
echo "Building project..."
cargo build

echo "Setup complete! Run with: cargo run"
echo "Run example with: cargo run --example simple_screen_share"