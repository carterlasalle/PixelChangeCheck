#!/bin/bash

# Check if Homebrew is installed (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    if ! command -v brew &> /dev/null; then
        echo "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    # Install FFmpeg with VideoToolbox support
    echo "Installing FFmpeg with VideoToolbox support..."
    brew install ffmpeg --with-videotoolbox
    
    # Install pkg-config for build configuration
    brew install pkg-config

    # Set up environment variables for FFmpeg
    export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig"
    export FFMPEG_LIB_DIR="/opt/homebrew/lib"
    export FFMPEG_INCLUDE_DIR="/opt/homebrew/include"
    
    # Verify VideoToolbox support
    echo "Verifying FFmpeg VideoToolbox support..."
    ffmpeg -encoders 2>/dev/null | grep videotoolbox
    
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Install FFmpeg on Linux with VAAPI support
    echo "Installing FFmpeg with VAAPI support..."
    sudo apt-get update
    sudo apt-get install -y ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswscale-dev pkg-config vainfo
    
    # Verify VAAPI support
    echo "Verifying VAAPI support..."
    vainfo
fi

# Clean and build
echo "Building project..."
cargo clean
cargo build --example simple_screen_share