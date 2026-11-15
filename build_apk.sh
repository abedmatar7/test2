#!/usr/bin/env bash
set -euo pipefail

echo "=== MedPulse Offline Study App — One-Command Android APK Builder ==="

if ! command -v cargo >/dev/null 2>&1; then
  echo "Error: Rust/Cargo is not installed. Install via https://rustup.rs"
  exit 1
fi

echo "[1/4] Adding aarch64-linux-android target..."
rustup target add aarch64-linux-android

echo "[2/4] Running medpulse-core unit & integration tests..."
cargo test -p medpulse-core

if ! command -v cargo-apk2 >/dev/null 2>&1; then
  echo "[3/4] Installing cargo-apk2..."
  cargo install cargo-apk2
fi

echo "[4/4] Building arm64-v8a APK..."
cd app-android
cargo apk2 build --target aarch64-linux-android --lib

APK_OUT="../target/debug/apk/MedPulseStudy.apk"
echo "Build complete: $APK_OUT"

if command -v adb >/dev/null 2>&1 && adb get-state >/dev/null 2>&1; then
  echo "Connected Android device detected! Installing via adb..."
  adb install -r "$APK_OUT"
  adb shell am start -n dev.medpulse.app/android.app.NativeActivity
fi
