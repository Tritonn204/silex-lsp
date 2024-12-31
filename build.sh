#!/bin/bash
set -e

declare -A targets=(
    ["x86_64-unknown-linux-gnu"]="silex-lsp-linux"
    ["aarch64-unknown-linux-gnu"]="silex-lsp-linux-arm64"
    ["x86_64-pc-windows-gnu"]="silex-lsp-win.exe"
)

for target in "${!targets[@]}"; do
    output_name="${targets[$target]}"
    echo "Building for target: $target -> $output_name"
    cross build --release --target "$target"
    
    # Copy the binary to the server directory with the desired name
    mkdir -p server
    cp "target/$target/release/silex-lsp" "server/$output_name" || \
    cp "target/$target/release/silex-lsp.exe" "server/$output_name"

    echo "Binary placed at server/$output_name"
done
