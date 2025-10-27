#!/bin/bash

# Format script using nightly rustfmt
# This script uses nightly rustfmt for advanced formatting features

# Check if nightly toolchain is installed
if ! rustup toolchain list | grep -q nightly; then
    echo "Installing nightly toolchain..."
    rustup toolchain install nightly
fi

# Run cargo fmt with nightly toolchain
echo "Running cargo +nightly fmt..."
cargo +nightly fmt

echo "Formatting complete!"

