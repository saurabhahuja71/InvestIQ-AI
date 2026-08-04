#!/usr/bin/env bash
# Build a signed debug APK with Firebase dart-defines for physical devices.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="${HOME}/development/flutter/bin:${HOME}/Android/Sdk/platform-tools:${PATH}"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$ANDROID_HOME}"

# API base URL for the phone (LAN by default — open host firewall TCP 8080 once):
#   ./scripts/open-api-firewall.sh
# Overrides:
#   API_BASE_URL=http://192.168.x.x:8080
#   USE_ADB_REVERSE=1  → http://127.0.0.1:8080 + adb reverse (USB)
DEFINE_FILE="${DEFINE_FILE:-$ROOT/mobile/config/firebase.dart-define.json}"
OUT_DIR="$ROOT/dist/android"

DEFAULT_IP="$(hostname -I 2>/dev/null | awk '{print $1}')"
API_HOST="${API_HOST:-${DEFAULT_IP:-10.0.2.2}}"

if [[ -z "${API_BASE_URL:-}" ]]; then
  if [[ "${USE_ADB_REVERSE:-}" == "1" ]]; then
    API_BASE_URL="http://127.0.0.1:8080"
  else
    API_BASE_URL="http://${API_HOST}:8080"
  fi
fi

# Always include common local fallbacks unless caller sets API_FALLBACK_BASE_URLS
if [[ -z "${API_FALLBACK_BASE_URLS:-}" ]]; then
  API_FALLBACK_BASE_URLS="http://127.0.0.1:8080,http://${API_HOST}:8080,http://10.0.2.2:8080"
fi

cd "$ROOT/mobile"

if [[ ! -f "$DEFINE_FILE" ]]; then
  echo "ERROR: Missing $DEFINE_FILE" >&2
  echo "Copy mobile/config/firebase.dart-define.json.example and fill Firebase values." >&2
  exit 1
fi

echo "API_BASE_URL=$API_BASE_URL"
echo "Firebase defines: $DEFINE_FILE"
echo "Debug SHA-1 (must be in Firebase Android app):"
keytool -list -v \
  -keystore "${HOME}/.android/debug.keystore" \
  -alias androiddebugkey -storepass android -keypass android 2>/dev/null \
  | grep 'SHA1:' || true

# Optional comma-separated fallbacks (tunnel, LAN, reverse, etc.)
API_FALLBACK_BASE_URLS="${API_FALLBACK_BASE_URLS:-}"

flutter pub get
FLUTTER_ARGS=(
  build apk --debug
  --dart-define="API_BASE_URL=${API_BASE_URL}"
  --dart-define-from-file="$DEFINE_FILE"
)
if [[ -n "$API_FALLBACK_BASE_URLS" ]]; then
  FLUTTER_ARGS+=(--dart-define="API_FALLBACK_BASE_URLS=${API_FALLBACK_BASE_URLS}")
  echo "API_FALLBACK_BASE_URLS=$API_FALLBACK_BASE_URLS"
fi
flutter "${FLUTTER_ARGS[@]}"

mkdir -p "$OUT_DIR"
APK_SRC="$ROOT/mobile/build/app/outputs/flutter-apk/app-debug.apk"
APK_DST="$OUT_DIR/investiq-ai-debug.apk"
cp -f "$APK_SRC" "$APK_DST"
ls -lh "$APK_SRC" "$APK_DST"

echo
echo "Installed APK path: $APK_DST"
echo "Install on device: adb install -r $APK_DST"
if [[ "${USE_ADB_REVERSE:-}" == "1" ]] || [[ "$API_BASE_URL" == *"127.0.0.1"* ]]; then
  if command -v adb >/dev/null 2>&1; then
    adb reverse tcp:8080 tcp:8080 || true
    echo "adb reverse tcp:8080 → host :8080 (keep USB connected)"
  fi
  echo "API via USB reverse: $API_BASE_URL"
else
  echo "API via LAN: $API_BASE_URL (open host firewall TCP 8080 if needed)"
fi
echo "Firebase Android: package ai.investiq.investiq_ai + debug SHA-1 + google-services.json"
echo "See docs/12-android-run.md"
