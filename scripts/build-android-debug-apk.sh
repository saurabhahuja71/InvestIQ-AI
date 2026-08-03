#!/usr/bin/env bash
# Build a signed debug APK with Firebase dart-defines for physical devices.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="${HOME}/development/flutter/bin:${HOME}/Android/Sdk/platform-tools:${PATH}"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$ANDROID_HOME}"

# Device cannot use 127.0.0.1 for host API — default to primary LAN IP.
DEFAULT_IP="$(hostname -I 2>/dev/null | awk '{print $1}')"
API_HOST="${API_HOST:-${DEFAULT_IP:-10.0.2.2}}"
API_BASE_URL="${API_BASE_URL:-http://${API_HOST}:8080}"
DEFINE_FILE="${DEFINE_FILE:-$ROOT/mobile/config/firebase.dart-define.json}"
OUT_DIR="$ROOT/dist/android"

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

flutter pub get
flutter build apk --debug \
  --dart-define="API_BASE_URL=${API_BASE_URL}" \
  --dart-define-from-file="$DEFINE_FILE"

mkdir -p "$OUT_DIR"
APK_SRC="$ROOT/mobile/build/app/outputs/flutter-apk/app-debug.apk"
APK_DST="$OUT_DIR/investiq-ai-debug.apk"
cp -f "$APK_SRC" "$APK_DST"
ls -lh "$APK_SRC" "$APK_DST"

echo
echo "Installed APK path: $APK_DST"
echo "Install on device: adb install -r $APK_DST"
echo "Ensure phone and PC share the same Wi‑Fi and API listens on 0.0.0.0:8080"
echo "Firebase Android: package ai.investiq.investiq_ai + debug SHA-1 + google-services.json"
echo "See docs/12-android-run.md"
