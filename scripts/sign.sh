#!/usr/bin/env bash
# Signs DustMap binaries or app bundle using Developer ID Application
set -euo pipefail

IDENTITY="${APPLE_IDENTITY:?Set APPLE_IDENTITY to your Developer ID Application identity}"
ENTITLEMENTS="app/src-tauri/entitlements.plist"

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <path-to-binary-or-app>"
  echo "Available identities:"
  security find-identity -p codesigning -v
  exit 1
fi

TARGET="$1"
echo "Signing '$TARGET' with identity '$IDENTITY'..."

codesign --force --verify --verbose --sign "$IDENTITY" \
  --options runtime \
  --timestamp \
  --entitlements "$ENTITLEMENTS" \
  "$TARGET"

echo "Verifying signature..."
codesign --verify --verbose=4 "$TARGET"
spctl --assess --type execute --verbose "$TARGET" || true

echo "Codesigning complete."
