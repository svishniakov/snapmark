#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="SnapMark"
APP_BUNDLE="${ROOT_DIR}/build/${APP_NAME}.app"
DMG_STAGING="${ROOT_DIR}/build/dmg-staging"
VERSION=""

usage() {
  cat <<'EOF'
Usage:
  ./scripts/build_release_dmg.sh [version]

Examples:
  ./scripts/build_release_dmg.sh
  ./scripts/build_release_dmg.sh 0.1.0
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script requires macOS (hdiutil)." >&2
  exit 1
fi

VERSION="${1:-$(awk -F '\"' '/^version = \"/ { print $2; exit }' "${ROOT_DIR}/Cargo.toml")}"
if [[ -z "${VERSION}" ]]; then
  echo "Cannot resolve version from Cargo.toml." >&2
  exit 1
fi

echo "Building universal app bundle..."
"${ROOT_DIR}/scripts/build_macos_app.sh"

if [[ ! -d "${APP_BUNDLE}" ]]; then
  echo "App bundle not found: ${APP_BUNDLE}" >&2
  exit 1
fi

DMG_NAME="${APP_NAME}-${VERSION}.dmg"
DMG_PATH="${ROOT_DIR}/build/${DMG_NAME}"

rm -rf "${DMG_STAGING}" "${DMG_PATH}"
mkdir -p "${DMG_STAGING}"
cp -R "${APP_BUNDLE}" "${DMG_STAGING}/"
ln -s /Applications "${DMG_STAGING}/Applications"

echo "Creating DMG: ${DMG_PATH}"
hdiutil create \
  -volname "${APP_NAME}" \
  -srcfolder "${DMG_STAGING}" \
  -ov \
  -format UDZO \
  "${DMG_PATH}" >/dev/null

rm -rf "${DMG_STAGING}"

SHA256="$(shasum -a 256 "${DMG_PATH}" | awk '{print $1}')"
echo "Done."
echo "DMG: ${DMG_PATH}"
echo "SHA256: ${SHA256}"
