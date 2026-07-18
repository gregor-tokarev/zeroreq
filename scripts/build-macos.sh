#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${VERSION:-$(sed -n '/^name = "zeroreq"$/{n;s/version = "\\([^"]*\\)"/\\1/p;q;}' "$ROOT/crates/zeroreq/Cargo.toml")}"
BUILD_VERSION="${BUILD_VERSION:-${VERSION//[^0-9]/}}"
BUILD_VERSION="${BUILD_VERSION:-1}"
DIST_DIR="${DIST_DIR:-$ROOT/dist}"
APP="$DIST_DIR/Zeroreq.app"
SIGN_IDENTITY="${SIGN_IDENTITY:--}"

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

if [[ "${UNIVERSAL:-1}" == "1" ]]; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  cargo build --locked --release -p zeroreq --target aarch64-apple-darwin
  cargo build --locked --release -p zeroreq --target x86_64-apple-darwin
  /usr/bin/lipo -create \
    "$ROOT/target/aarch64-apple-darwin/release/zeroreq" \
    "$ROOT/target/x86_64-apple-darwin/release/zeroreq" \
    -output "$APP/Contents/MacOS/zeroreq"
else
  cargo build --locked --release -p zeroreq
  cp "$ROOT/target/release/zeroreq" "$APP/Contents/MacOS/zeroreq"
fi

chmod 755 "$APP/Contents/MacOS/zeroreq"
sed \
  -e "s/__VERSION__/$VERSION/g" \
  -e "s/__BUILD_VERSION__/$BUILD_VERSION/g" \
  "$ROOT/packaging/macos/Info.plist" > "$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"

/usr/bin/codesign \
  --force \
  --options runtime \
  --timestamp \
  --entitlements "$ROOT/packaging/macos/entitlements.plist" \
  --sign "$SIGN_IDENTITY" \
  "$APP"
/usr/bin/codesign --verify --deep --strict --verbose=2 "$APP"

echo "Built $APP"
