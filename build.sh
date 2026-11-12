#!/usr/bin/env bash
# Build do APK do Pet Clawd — só funciona em máquina Linux x86_64 com
# toolchain Android (Rust + JDK 17+ + SDK/NDK + tauri-cli).
set -e
cd "$(dirname "$0")/src-tauri"

: "${JAVA_HOME:?export JAVA_HOME=…}"
: "${ANDROID_HOME:?export ANDROID_HOME=…}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/27.0.12077973}"

command -v cargo-tauri >/dev/null 2>&1 || cargo install tauri-cli --version "^2" --locked

[ -d gen/android ] || cargo tauri android init
cargo tauri icon ../app-icon.png || true
cargo tauri android build --target aarch64 --debug --apk

echo "APK: $(find gen/android -name '*.apk' -type f | head -1)"
