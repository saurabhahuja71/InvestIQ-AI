#!/usr/bin/env bash
# Sync local secret files → GitHub Actions secrets for this repo.
# Usage:
#   ./scripts/sync-github-secrets.sh
# Requires: gh auth login, jq
#
# Sources (gitignored locally):
#   mobile/config/firebase.dart-define.json
#   mobile/android/app/google-services.json  (optional)
#   .env  (selected keys)
#
# Does NOT print secret values.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v gh >/dev/null; then
  echo "ERROR: gh CLI required (https://cli.github.com/)" >&2
  exit 1
fi
if ! command -v jq >/dev/null; then
  echo "ERROR: jq required" >&2
  exit 1
fi

REPO="${GITHUB_REPOSITORY:-}"
if [[ -z "$REPO" ]]; then
  REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || true)"
fi
if [[ -z "$REPO" ]]; then
  REPO="saurabhahuja71/InvestIQ-AI"
fi

set_secret() {
  local name="$1"
  local value="$2"
  if [[ -z "$value" ]]; then
    echo "  skip $name (empty)"
    return 0
  fi
  printf '%s' "$value" | gh secret set "$name" --repo "$REPO"
  echo "  set  $name"
}

echo "Syncing secrets → GitHub repo: $REPO"
echo

DEFINE="$ROOT/mobile/config/firebase.dart-define.json"
if [[ -f "$DEFINE" ]]; then
  echo "[firebase.dart-define.json]"
  # Individual keys
  for key in \
    FIREBASE_API_KEY \
    FIREBASE_APP_ID \
    FIREBASE_MESSAGING_SENDER_ID \
    FIREBASE_PROJECT_ID \
    FIREBASE_AUTH_DOMAIN \
    FIREBASE_STORAGE_BUCKET \
    FIREBASE_MEASUREMENT_ID \
    GOOGLE_WEB_CLIENT_ID \
    FIREBASE_ANDROID_API_KEY \
    FIREBASE_ANDROID_APP_ID \
    FIREBASE_ANDROID_MESSAGING_SENDER_ID
  do
    val="$(jq -r --arg k "$key" '.[$k] // empty' "$DEFINE")"
    set_secret "$key" "$val"
  done
  # Full bundle for CI restore
  set_secret FIREBASE_CONFIG_JSON "$(cat "$DEFINE")"
else
  echo "WARN: missing $DEFINE — skip Flutter Firebase keys"
fi

echo
GSERVICES="$ROOT/mobile/android/app/google-services.json"
if [[ -f "$GSERVICES" ]]; then
  echo "[google-services.json]"
  set_secret GOOGLE_SERVICES_JSON "$(cat "$GSERVICES")"
else
  echo "WARN: missing $GSERVICES — skip GOOGLE_SERVICES_JSON"
fi

echo
ENV_FILE="$ROOT/.env"
if [[ -f "$ENV_FILE" ]]; then
  echo "[.env selected keys]"
  # shellcheck disable=SC1090
  set -a
  # Only export known keys (avoid running arbitrary .env as shell)
  while IFS= read -r line; do
    [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
    [[ "$line" != *=* ]] && continue
    key="${line%%=*}"
    val="${line#*=}"
    case "$key" in
      FIREBASE_PROJECT_ID|GOOGLE_CLIENT_IDS|AI_API_KEY|AI_BASE_URL|AI_MODEL|JWT_SECRET|AES_KEY_BASE64|CORS_ORIGINS)
        # Skip obvious placeholders
        if [[ "$val" == replace-* || "$val" == REPLACE* ]]; then
          echo "  skip $key (placeholder)"
          continue
        fi
        set_secret "$key" "$val"
        ;;
    esac
  done < "$ENV_FILE"
  set +a
else
  echo "WARN: missing $ENV_FILE — skip backend keys"
fi

echo
echo "Done. Current secret names:"
gh secret list --repo "$REPO"
echo
echo "UI: https://github.com/${REPO}/settings/secrets/actions"
