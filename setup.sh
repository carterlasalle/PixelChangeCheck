#!/bin/bash

# Check if Homebrew is installed (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    if ! command -v brew &> /dev/null; then
        echo "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    # Install FFmpeg and dependencies
    echo "Installing FFmpeg and dependencies..."
   # brew install ffmpeg pkg-config

    # Set up environment variables for FFmpeg
    #export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig"
    #export FFMPEG_LIB_DIR="/opt/homebrew/lib"
    #export FFMPEG_INCLUDE_DIR="/opt/homebrew/include"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Install FFmpeg on Linux
    echo "Installing FFmpeg and dependencies..."
    sudo apt-get update
    sudo apt-get install -y ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswscale-dev pkg-config
fi

# Clean and build
echo "Building project..."
cargo clean
cargo build --example simple_screen_share 