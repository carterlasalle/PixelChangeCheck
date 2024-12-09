#!/bin/bash

# Set up environment variables for libvpx
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig"
export VPX_LIB_DIR="/opt/homebrew/lib"
export VPX_INCLUDE_DIR="/opt/homebrew/include"

# Clean and build
cargo clean
cargo build --example simple_screen_share 