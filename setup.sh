#!/bin/bash

# Check if Homebrew is installed (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    if ! command -v brew &> /dev/null; then
        echo "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    # Remove any existing FFmpeg installation
    brew uninstall ffmpeg || true
    
    # Install FFmpeg with VideoToolbox support
    echo "Installing FFmpeg..."
    brew install ffmpeg

    # Install pkg-config
    brew install pkg-config

    # Create local include directory
    mkdir -p include
    
    # Set up environment variables for FFmpeg
    FFMPEG_PREFIX="$(brew --prefix ffmpeg)"
    export PKG_CONFIG_PATH="${FFMPEG_PREFIX}/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
    export LIBRARY_PATH="${FFMPEG_PREFIX}/lib:${LIBRARY_PATH:-}"
    export CPATH="${FFMPEG_PREFIX}/include:${CPATH:-}"
    export MACOSX_DEPLOYMENT_TARGET=11.0
    
    # Create symbolic links for headers
    echo "Setting up header files..."
    ln -sf "${FFMPEG_PREFIX}/include/libavcodec" include/
    ln -sf "${FFMPEG_PREFIX}/include/libavformat" include/
    ln -sf "${FFMPEG_PREFIX}/include/libavutil" include/
    ln -sf "${FFMPEG_PREFIX}/include/libswscale" include/
    
    # Verify FFmpeg installation and capabilities
    echo "Verifying FFmpeg capabilities..."
    ffmpeg -encoders | grep videotoolbox
    
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

# Set environment variables for build
export FFMPEG_NO_VAAPI=1    # Disable VAAPI dependency
export FFMPEG_NO_VDPAU=1    # Disable VDPAU dependency

# Build project
cargo clean
cargo build --example simple_screen_share