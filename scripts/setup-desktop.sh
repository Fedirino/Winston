#!/usr/bin/env bash
# Winston Desktop — setup and build script for Windows
# Run this from git-bash to install dependencies and build

set -e

echo "=== Winston Desktop Setup ==="

# 1. Install MSYS2 if not present
if [ ! -d "/c/msys64" ]; then
    echo "Installing MSYS2 (needed for Rust build tools)..."
    curl -sLo /tmp/msys2-installer.exe https://github.com/msys2/msys2-installer/releases/download/2025-01-21/msys2-x86_64.exe
    /tmp/msys2-installer.exe install --root /c/msys64 --quiet --confirm-command
    echo "MSYS2 installed."
fi

# 2. Install MinGW-w64 tools via MSYS2
echo "Installing MinGW-w64 build tools..."
/c/msys64/usr/bin/bash -lc "pacman -S --noconfirm mingw-w64-x86_64-binutils mingw-w64-x86_64-toolchain make"

# 3. Add MSYS2 mingw64 to PATH
echo 'export PATH="$PATH:/c/msys64/mingw64/bin"' >> ~/.bashrc
export PATH="$PATH:/c/msys64/mingw64/bin"

# 4. Ensure Rust toolchain is up to date
echo "Updating Rust toolchain..."
export PATH="$PATH:$HOME/.cargo/bin"
rustup update stable

# 5. Build
echo ""
echo "=== Building Winston Desktop ==="
cd "$(dirname "$0")/src-tauri"
cargo build --release

echo ""
echo "=== Done! ==="
echo "Binary at: src-tauri/target/release/winston.exe"
echo "Or run with: npx tauri dev (from project root)"