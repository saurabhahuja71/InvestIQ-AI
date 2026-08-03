#!/usr/bin/env bash
# Report whether Google/Firebase auth is configured for API + Flutter.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ok=0
warn=0

echo "=== InvestIQ Google Auth configuration check ==="
echo

# Backend .env
ENV_FILE="$ROOT/.env"
if [[ ! -f "$ENV_FILE" ]]; then
  echo "[MISSING] $ENV_FILE — copy from .env.example"
  warn=$((warn + 1))
else
  # shellcheck disable=SC1090
  set -a
  # shellcheck disable=SC1091
  source <(grep -E '^(FIREBASE_PROJECT_ID|GOOGLE_CLIENT_IDS)=' "$ENV_FILE" | sed 's/\r$//' || true)
  set +a
  if [[ -n "${FIREBASE_PROJECT_ID:-}" ]]; then
    echo "[OK] FIREBASE_PROJECT_ID is set"
    ok=$((ok + 1))
  else
    echo "[MISSING] FIREBASE_PROJECT_ID in .env"
    warn=$((warn + 1))
  fi
  if [[ -n "${GOOGLE_CLIENT_IDS:-}" ]]; then
    echo "[OK] GOOGLE_CLIENT_IDS is set"
    ok=$((ok + 1))
  else
    echo "[WARN] GOOGLE_CLIENT_IDS empty (Firebase ID tokens still work if FIREBASE_PROJECT_ID is set)"
  fi
fi

# Flutter dart-define file
DEFINE="$ROOT/mobile/config/firebase.dart-define.json"
if [[ -f "$DEFINE" ]]; then
  echo "[OK] $DEFINE exists"
  ok=$((ok + 1))
  if grep -q 'REPLACE_WITH' "$DEFINE" 2>/dev/null; then
    echo "[WARN] $DEFINE still contains REPLACE_WITH placeholders"
    warn=$((warn + 1))
  fi
else
  echo "[MISSING] $DEFINE — copy from mobile/config/firebase.dart-define.json.example"
  warn=$((warn + 1))
fi

# Android google-services
GSERVICES="$ROOT/mobile/android/app/google-services.json"
if [[ -f "$GSERVICES" ]]; then
  echo "[OK] google-services.json present (Android Firebase)"
  ok=$((ok + 1))
else
  echo "[INFO] google-services.json not present (required only for Android builds)"
fi

# Live API if up
if curl -sf http://127.0.0.1:8080/health >/dev/null 2>&1; then
  echo
  echo "API /auth/providers:"
  curl -s http://127.0.0.1:8080/api/v1/auth/providers || true
  echo
else
  echo
  echo "[INFO] API not reachable on :8080 — start with: cd backend && cargo run"
fi

echo
echo "Docs: CONFIGURATION_REQUIRED.md"
echo "Summary: $ok ok signal(s), $warn missing/warn item(s)"
if [[ "$warn" -gt 0 ]]; then
  echo "Google Sign-In will not work until missing items are fixed."
  exit 1
fi
echo "Config files look filled. Run the app and complete the verification checklist in CONFIGURATION_REQUIRED.md."
exit 0
