#!/bin/bash
set -e

# 检查必要的工具
echo "Checking build tools..."
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust."
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo "Error: npm not found. Please install Node.js."
    exit 1
fi

FRONTEND_DIR="frontend"
BACKEND_DIR="backend"
BUILD_DIR="build"
RELEASE_DIR="release"

echo "Building Frontend..."
cd "$FRONTEND_DIR"
npm install
npm run build
cd ..

echo "Building Backend..."
mkdir -p "$BUILD_DIR"
ABS_BUILD_DIR="$(pwd)/$BUILD_DIR"

cd "$BACKEND_DIR"
# 构建amd64版本
echo "Building backend for amd64..."
cargo build --release --target-dir "$ABS_BUILD_DIR"

# 构建aarch64版本
echo "Building backend for aarch64..."
if cargo build --release --target aarch64-unknown-linux-gnu --target-dir "$ABS_BUILD_DIR"; then
    echo "Aarch64 build successful"
    AARCH64_BUILT=true
else
    echo "Warning: Aarch64 build failed. Make sure you have the cross-compilation toolchain installed."
    echo "Try: rustup target add aarch64-unknown-linux-gnu"
    AARCH64_BUILT=false
fi

# 构建windows-mingw-w64版本
echo "Building backend for Windows..."
if cargo build --release --target x86_64-pc-windows-gnu --target-dir "$ABS_BUILD_DIR"; then
    echo "Windows build successful"
    WINDOWS_BUILT=true
else
    echo "Warning: Windows build failed. Make sure you have the mingw-w64 toolchain installed."
    echo "Try: rustup target add x86_64-pc-windows-gnu"
    WINDOWS_BUILT=false
fi
cd ..

echo "Preparing Release..."
mkdir -p "$RELEASE_DIR"

# 复制构建产物到对应架构目录
echo "Copying build artifacts..."

# 复制amd64版本（总是尝试构建的）
if [ -f "$ABS_BUILD_DIR/release/zher" ]; then
    cp "$ABS_BUILD_DIR/release/zher" "$RELEASE_DIR/zher-linux-x86_64"
    echo "✓ Copied amd64 build"
else
    echo "✗ Amd64 build file not found"
fi

if [ "$AARCH64_BUILT" = true ] && [ -f "$ABS_BUILD_DIR/aarch64-unknown-linux-gnu/release/zher" ]; then
    cp "$ABS_BUILD_DIR/aarch64-unknown-linux-gnu/release/zher" "$RELEASE_DIR/zher-linux-arm64"
    echo "✓ Copied aarch64 build"
fi

if [ "$WINDOWS_BUILT" = true ] && [ -f "$ABS_BUILD_DIR/x86_64-pc-windows-gnu/release/zher.exe" ]; then
    cp "$ABS_BUILD_DIR/x86_64-pc-windows-gnu/release/zher.exe" "$RELEASE_DIR/zher-windows-x86_64.exe"
    echo "✓ Copied Windows build"
fi

echo "Build process completed."

