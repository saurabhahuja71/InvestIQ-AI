#!/usr/bin/env bash
# Run Flutter web against local API with Firebase dart-defines if present.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="${HOME}/development/flutter/bin:${PATH}"

API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"
DEFINE_FILE="$ROOT/mobile/config/firebase.dart-define.json"

cd "$ROOT/mobile"

args=(
  run -d chrome
  --dart-define="API_BASE_URL=${API_BASE_URL}"
)

if [[ -f "$DEFINE_FILE" ]]; then
  echo "Using $DEFINE_FILE"
  args+=(--dart-define-from-file="$DEFINE_FILE")
else
  echo "WARNING: $DEFINE_FILE not found."
  echo "Copy mobile/config/firebase.dart-define.json.example and fill values."
  echo "See CONFIGURATION_REQUIRED.md"
  echo "Continuing without Firebase dart-defines (Google Sign-In will show not configured)."
fi

exec flutter "${args[@]}"
