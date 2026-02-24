#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_PATH="${ROOT_DIR}/packaging/homebrew/Casks/snapmark.rb"
VERSION="$(awk -F '\"' '/^version = \"/ { print $2; exit }' "${ROOT_DIR}/Cargo.toml")"
SHA256=""
REPO=""
DMG_PATH=""

usage() {
  cat <<'EOF'
Usage:
  ./scripts/generate_cask.sh --repo OWNER/REPO [--version X.Y.Z] [--dmg PATH | --sha256 HASH] [--output PATH]

Examples:
  ./scripts/build_release_dmg.sh 0.1.0
  ./scripts/generate_cask.sh --repo yourname/snapmark --dmg build/SnapMark-0.1.0.dmg

Notes:
  - If --dmg is provided, SHA256 is calculated automatically.
  - If --sha256 is provided, --dmg is optional.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      REPO="${2:-}"
      shift 2
      ;;
    --version)
      VERSION="${2:-}"
      shift 2
      ;;
    --sha256)
      SHA256="${2:-}"
      shift 2
      ;;
    --dmg)
      DMG_PATH="${2:-}"
      shift 2
      ;;
    --output)
      OUTPUT_PATH="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "${REPO}" ]]; then
  echo "--repo OWNER/REPO is required." >&2
  usage
  exit 1
fi

if [[ -n "${DMG_PATH}" && -z "${SHA256}" ]]; then
  if [[ ! -f "${DMG_PATH}" ]]; then
    echo "DMG not found: ${DMG_PATH}" >&2
    exit 1
  fi
  SHA256="$(shasum -a 256 "${DMG_PATH}" | awk '{print $1}')"
fi

if [[ -z "${SHA256}" ]]; then
  echo "Either --dmg PATH or --sha256 HASH is required." >&2
  usage
  exit 1
fi

mkdir -p "$(dirname "${OUTPUT_PATH}")"

cat > "${OUTPUT_PATH}" <<EOF
cask "snapmark" do
  version "${VERSION}"
  sha256 "${SHA256}"

  url "https://github.com/${REPO}/releases/download/v\#{version}/SnapMark-\#{version}.dmg"
  name "SnapMark"
  desc "Screenshot annotation tool for macOS"
  homepage "https://github.com/${REPO}"

  depends_on macos: ">= :ventura"

  app "SnapMark.app"

  zap trash: [
    "~/Library/Application Support/snapmark/settings.json",
    "~/Library/Preferences/com.snapmark.app.plist",
  ]
end
EOF

echo "Cask generated: ${OUTPUT_PATH}"
