#!/bin/bash

# Check if Homebrew is installed (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    if ! command -v brew &> /dev/null; then
        echo "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    echo "Removing existing FFmpeg installation..."
    brew uninstall ffmpeg || true

    echo "Installing FFmpeg..."
    brew install ffmpeg

    echo "Creating cargo config..."
    mkdir -p ~/.cargo
    cat > ~/.cargo/config.toml << EOF
[target.aarch64-apple-darwin]
rustflags = [
    "-C", "link-args=-L/opt/homebrew/lib",
    "-C", "link-args=-framework",
    "-C", "link-args=VideoToolbox",
    "-C", "link-args=-framework",
    "-C", "link-args=CoreMedia",
    "-C", "link-args=-framework",
    "-C", "link-args=CoreFoundation"
]
EOF

    echo "Setting up environment variables..."
    export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH"
    export FFMPEG_LIB_DIR="/opt/homebrew/lib"
    export FFMPEG_INCLUDE_DIR="/opt/homebrew/include"
    
    echo "Verifying FFmpeg capabilities..."
    ffmpeg -encoders 2>/dev/null | grep videotoolbox
    
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Install FFmpeg on Linux with minimal configuration
    echo "Installing FFmpeg..."
    sudo apt-get update
    sudo apt-get install -y \
        ffmpeg \
        libavcodec-dev \
        libavformat-dev \
        libavutil-dev \
        libswscale-dev \
        pkg-config
fi

# Clean and build
echo "Building project..."

# Remove target directory to ensure clean build
rm -rf target/

# Build project
cargo clean
cargo build --example simple_screen_share