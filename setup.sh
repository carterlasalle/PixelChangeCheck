#!/bin/bash

echo "PixelChangeCheck setup"

# Install system dependencies for screen capture
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Installing Linux dependencies..."
    sudo apt-get update
    # libasound2-dev and libudev-dev are not optional: alsa-sys, which
    # cpal pulls in for audio capture, runs pkg-config at build time and
    # panics without the headers, so a missing package is a build error
    # rather than a feature that quietly turns off.
    sudo apt-get install -y \
        libxcb1-dev \
        libxrandr-dev \
        libdbus-1-dev \
        libasound2-dev \
        libudev-dev \
        pkg-config
fi

# Build project
echo "Building project..."
cargo build --release

echo "Setup complete."
echo
echo "Share your screen:   ./target/release/pcc share"
echo "Watch from a viewer: ./target/release/pcc view --connect <host> --token <token> --pin <pin>"
echo "Check connectivity:   ./target/release/pcc diagnose"