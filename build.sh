#!/bin/bash
set -e

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
cargo build --release --target-dir "$ABS_BUILD_DIR"
cd ..

echo "Preparing Release..."
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"
cp "$ABS_BUILD_DIR/release/zher" "$RELEASE_DIR/zher"

