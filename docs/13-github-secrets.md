# GitHub Secrets — InvestIQ AI

All sensitive configuration for **InvestIQ AI** must live in **GitHub Actions secrets**  
(repo: `saurabhahuja71/InvestIQ-AI` → **Settings → Secrets and variables → Actions**).

Local files (`.env`, `firebase.dart-define.json`, `google-services.json`) stay **gitignored**.  
Sync them to GitHub with:

```bash
cd ~/InvestIQ-AI
./scripts/sync-github-secrets.sh
```

## Important limitation

GitHub secrets are **write-only**:

- You **cannot** open a secret later and read its value in the UI.
- You can only **use** them in Actions, or **overwrite** them.
- Keep a copy in a password manager for human recovery.

UI: https://github.com/saurabhahuja71/InvestIQ-AI/settings/secrets/actions

---

## Secret inventory

### Flutter / Firebase client (from `firebase.dart-define.json`)

| Secret name | Purpose |
|-------------|---------|
| `FIREBASE_API_KEY` | Web API key |
| `FIREBASE_APP_ID` | Web app ID |
| `FIREBASE_MESSAGING_SENDER_ID` | Messaging sender ID |
| `FIREBASE_PROJECT_ID` | Firebase project ID |
| `FIREBASE_AUTH_DOMAIN` | Auth domain |
| `FIREBASE_STORAGE_BUCKET` | Storage bucket |
| `FIREBASE_MEASUREMENT_ID` | Analytics measurement ID (optional) |
| `GOOGLE_WEB_CLIENT_ID` | OAuth Web client (`serverClientId` / ID token) |
| `FIREBASE_ANDROID_API_KEY` | Android app API key |
| `FIREBASE_ANDROID_APP_ID` | Android app ID |
| `FIREBASE_ANDROID_MESSAGING_SENDER_ID` | Android sender ID (usually same as web) |
| `FIREBASE_CONFIG_JSON` | Full dart-define JSON (CI restore) |

### Android Firebase file

| Secret name | Purpose |
|-------------|---------|
| `GOOGLE_SERVICES_JSON` | Full contents of `mobile/android/app/google-services.json` |

### Backend (from `.env`)

| Secret name | Purpose |
|-------------|---------|
| `FIREBASE_PROJECT_ID` | Same project ID (JWT audience for Firebase tokens) |
| `GOOGLE_CLIENT_IDS` | Comma-separated OAuth client IDs (Web + Android) |
| `JWT_SECRET` | App JWT signing secret (≥32 chars; not a placeholder) |
| `AI_API_KEY` | xAI / OpenAI-compatible key (optional) |
| `AI_BASE_URL` | AI API base URL (optional) |
| `AI_MODEL` | Model name (optional) |
| `AES_KEY_BASE64` | Optional field encryption key |
| `CORS_ORIGINS` | CORS allow list (optional) |

Do **not** put local Postgres/Redis passwords into GitHub unless you use managed cloud DBs for deploy. CI uses ephemeral service containers with fixed test credentials.

---

## Sync from this machine

```bash
# Requires: gh auth login, jq
# Reads local gitignored files and overwrites GitHub secrets
./scripts/sync-github-secrets.sh
```

Single secret:

```bash
printf '%s' "$(jq -r .FIREBASE_API_KEY mobile/config/firebase.dart-define.json)" \
  | gh secret set FIREBASE_API_KEY

gh secret set FIREBASE_CONFIG_JSON < mobile/config/firebase.dart-define.json
gh secret set GOOGLE_SERVICES_JSON < mobile/android/app/google-services.json
```

Restore locally from GitHub (only works if you still have values elsewhere — GitHub cannot print them back).  
After clone on a new machine, re-create local files from your password manager, then re-run the sync script.

---

## Use in GitHub Actions

CI writes Flutter dart-defines and `google-services.json` from secrets before mobile build steps:

```yaml
env:
  FIREBASE_PROJECT_ID: ${{ secrets.FIREBASE_PROJECT_ID }}
  FIREBASE_API_KEY: ${{ secrets.FIREBASE_API_KEY }}
  # …
```

Or:

```yaml
- name: Write Firebase dart-defines
  run: |
    echo '${{ secrets.FIREBASE_CONFIG_JSON }}' > mobile/config/firebase.dart-define.json
- name: Write google-services.json
  if: ${{ secrets.GOOGLE_SERVICES_JSON != '' }}
  run: |
    echo '${{ secrets.GOOGLE_SERVICES_JSON }}' > mobile/android/app/google-services.json
```

Backend deploy (when added) should inject:

```yaml
env:
  JWT_SECRET: ${{ secrets.JWT_SECRET }}
  FIREBASE_PROJECT_ID: ${{ secrets.FIREBASE_PROJECT_ID }}
  GOOGLE_CLIENT_IDS: ${{ secrets.GOOGLE_CLIENT_IDS }}
  AI_API_KEY: ${{ secrets.AI_API_KEY }}
```

---

## Policy

1. **Never commit** real `.env`, `firebase.dart-define.json`, or `google-services.json`.
2. **Always** store production/shared secrets in GitHub Secrets (or another vault).
3. After rotating a key in Firebase/Google Cloud, re-run `./scripts/sync-github-secrets.sh`.
4. Placeholders (`REPLACE_*`, `replace-with-*`) are **not** synced.
