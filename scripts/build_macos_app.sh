#!/usr/bin/env bash
set -euo pipefail

APP_NAME="SnapMark"
BIN_NAME="snapmark"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${ROOT_DIR}/build/${APP_NAME}.app/Contents"

echo "Generating icons from assets/snapmark-icon.svg..."
"${ROOT_DIR}/scripts/generate_icons_from_svg.sh"

mkdir -p "${BUILD_DIR}/MacOS" "${BUILD_DIR}/Resources"

echo "Building arm64..."
cargo build --release --target aarch64-apple-darwin

echo "Building x86_64..."
cargo build --release --target x86_64-apple-darwin

echo "Creating universal binary..."
lipo -create \
  "${ROOT_DIR}/target/aarch64-apple-darwin/release/${BIN_NAME}" \
  "${ROOT_DIR}/target/x86_64-apple-darwin/release/${BIN_NAME}" \
  -output "${BUILD_DIR}/MacOS/${BIN_NAME}"

cp "${ROOT_DIR}/Info.plist" "${BUILD_DIR}/Info.plist"
if [[ -f "${ROOT_DIR}/assets/icon.icns" ]]; then
  cp "${ROOT_DIR}/assets/icon.icns" "${BUILD_DIR}/Resources/AppIcon.icns"
fi
if [[ -f "${ROOT_DIR}/assets/status_icon_template.png" ]]; then
  cp "${ROOT_DIR}/assets/status_icon_template.png" "${BUILD_DIR}/Resources/status_icon_template.png"
fi

chmod +x "${BUILD_DIR}/MacOS/${BIN_NAME}"

echo "Built app bundle: ${ROOT_DIR}/build/${APP_NAME}.app"
