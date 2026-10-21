#!/usr/bin/env bash
set -euo pipefail

FILE="$1"
AUTH=(--key "$APPLE_API_KEY_PATH" --key-id "$APPLE_API_KEY" --issuer "$APPLE_API_ISSUER")

ID="$(xcrun notarytool submit "$FILE" "${AUTH[@]}" --output-format json | jq -r .id)"
echo "Notary submission $ID for $(basename "$FILE")"

if ! xcrun notarytool wait "$ID" "${AUTH[@]}" --timeout 30m; then
  echo "Notarization did not finish within 30 minutes; fetching the log."
  xcrun notarytool log "$ID" "${AUTH[@]}" || true
  exit 1
fi

STATUS="$(xcrun notarytool info "$ID" "${AUTH[@]}" --output-format json | jq -r .status)"
xcrun notarytool log "$ID" "${AUTH[@]}" || true
if [ "$STATUS" != "Accepted" ]; then
  echo "Notarization finished with status: $STATUS"
  exit 1
fi
