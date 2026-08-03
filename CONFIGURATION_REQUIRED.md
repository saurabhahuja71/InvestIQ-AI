# Google / Firebase Authentication — Manual Configuration Required

InvestIQ AI **implements** Google Sign-In end-to-end in code (Flutter + Firebase Auth + Rust JWT + Postgres).  
The following values **cannot be invented or committed as secrets**. You must create them in Google Cloud / Firebase and place them as documented below.

Until these are set, the app correctly shows:

> Google Sign-In is not configured…

Email/password auth continues to work without Firebase.

---

## Architecture (what the code already does)

```
Flutter  →  Firebase Auth (Google provider)  →  Firebase ID token
       →  POST /api/v1/auth/google { "id_token": "..." }
       →  Rust verifies token (JWKS: Firebase or Google)
       →  Upsert user in PostgreSQL
       →  Issue app JWT access + refresh
       →  FlutterSecureStorage persists tokens
       →  App restart → /auth/me with access token (auto-login)
       →  Logout clears storage + Firebase + Google session
```

---

## Prerequisites

- Google account with access to [Firebase Console](https://console.firebase.google.com/)
- Browser for Web testing; Android Studio / SDK for Android (optional on this host)
- Local API running (`cargo run` in `backend/`) with Postgres + Redis

---

## Step 1 — Create a Firebase project

1. Open https://console.firebase.google.com/
2. **Add project** → name e.g. `investiq-ai` → disable Google Analytics if you want (optional).
3. Note the **Project ID** (e.g. `investiq-ai-12345`).

| Value | Where to find | Where to place |
|-------|----------------|----------------|
| **Firebase Project ID** | Project settings (gear) → **General** → Project ID | Backend `.env`: `FIREBASE_PROJECT_ID=...` **and** Flutter dart-define `FIREBASE_PROJECT_ID` |

---

## Step 2 — Enable Google Sign-In (Authentication)

1. Firebase Console → **Build** → **Authentication** → **Get started**
2. **Sign-in method** → **Google** → **Enable**
3. Set a **Project support email** → **Save**

No secret is copied in this step; enabling the provider is required.

---

## Step 3 — Register a **Web** app (required for Chrome / Flutter web)

1. Project settings → **Your apps** → **Add app** → **Web** (`</>`)
2. Nickname e.g. `investiq-web` → Register
3. Copy the `firebaseConfig` object fields:

| Firebase field | Flutter dart-define / JSON key |
|----------------|--------------------------------|
| `apiKey` | `FIREBASE_API_KEY` |
| `authDomain` | `FIREBASE_AUTH_DOMAIN` (usually `{projectId}.firebaseapp.com`) |
| `projectId` | `FIREBASE_PROJECT_ID` |
| `messagingSenderId` | `FIREBASE_MESSAGING_SENDER_ID` |
| `appId` | `FIREBASE_APP_ID` (Web app ID) |

4. **Authorized domains** (Authentication → Settings → Authorized domains):
   - `localhost` (default)
   - Add any LAN host you use if needed

---

## Step 4 — Google OAuth **Web Client ID** (ID token audience)

1. Open https://console.cloud.google.com/apis/credentials?project=YOUR_PROJECT_ID  
   (or Firebase → Project settings → **Service accounts** → link to Google Cloud)
2. Under **OAuth 2.0 Client IDs**, open the client of type **Web application**  
   (often auto-created as “Web client (auto created by Google Service)”)
3. Copy **Client ID**  
   Format: `….apps.googleusercontent.com`

| Value | Where to place |
|-------|----------------|
| **Web Client ID** | Flutter: `GOOGLE_WEB_CLIENT_ID` (dart-define or JSON file) |
| Same + any Android client IDs | Backend `.env`: `GOOGLE_CLIENT_IDS=web-id,android-id` (comma-separated) |

**Backend:** set at least:

```bash
FIREBASE_PROJECT_ID=your-project-id
GOOGLE_CLIENT_IDS=123456789-xxxx.apps.googleusercontent.com
```

`FIREBASE_PROJECT_ID` is enough for **Firebase ID tokens** (issuer `https://securetoken.google.com/{projectId}`).  
`GOOGLE_CLIENT_IDS` is required if the client ever sends a **raw Google** ID token (`iss` = `accounts.google.com`).  
**Recommended: set both.**

Restart the API after editing `.env`:

```bash
# free port if needed
fuser -k 8080/tcp 2>/dev/null || true
cd ~/InvestIQ-AI/backend
set -a && source ../.env && set +a
cargo run
```

Check:

```bash
curl -s http://127.0.0.1:8080/api/v1/auth/providers
# expect: "google": true
```

---

## Step 5 — Flutter config file (recommended)

Do **not** commit real secrets. Copy the example and fill values:

```bash
cp mobile/config/firebase.dart-define.json.example \
   mobile/config/firebase.dart-define.json
# edit firebase.dart-define.json with real values
```

File is gitignored. Run web with:

```bash
export PATH="$HOME/development/flutter/bin:$PATH"
cd ~/InvestIQ-AI/mobile
flutter run -d chrome \
  --dart-define=API_BASE_URL=http://127.0.0.1:8080 \
  --dart-define-from-file=config/firebase.dart-define.json
```

Or use helper:

```bash
./scripts/run-mobile-chrome.sh
```

### Alternative: pass dart-defines one by one

```bash
flutter run -d chrome \
  --dart-define=API_BASE_URL=http://127.0.0.1:8080 \
  --dart-define=FIREBASE_API_KEY=... \
  --dart-define=FIREBASE_APP_ID=... \
  --dart-define=FIREBASE_MESSAGING_SENDER_ID=... \
  --dart-define=FIREBASE_PROJECT_ID=... \
  --dart-define=FIREBASE_AUTH_DOMAIN=your-project.firebaseapp.com \
  --dart-define=GOOGLE_WEB_CLIENT_ID=....apps.googleusercontent.com
```

Optional Android-specific (if Web and Android app IDs differ):

- `FIREBASE_ANDROID_API_KEY`
- `FIREBASE_ANDROID_APP_ID`
- `FIREBASE_ANDROID_MESSAGING_SENDER_ID`

---

## Step 6 — Register an **Android** app (for device / emulator)

1. Firebase → Add app → **Android**
2. **Android package name** must match:

   ```
   ai.investiq.investiq_ai
   ```

   (see `mobile/android/app/build.gradle.kts` → `applicationId`)

3. Download **`google-services.json`**
4. Place it at:

   ```
   mobile/android/app/google-services.json
   ```

   (gitignored; template: `mobile/android/app/google-services.json.example`)

5. **SHA-1 fingerprint** (required for Google Sign-In on Android):

   ```bash
   # Debug keystore SHA-1
   keytool -list -v \
     -keystore ~/.android/debug.keystore \
     -alias androiddebugkey \
     -storepass android -keypass android
   ```

   Copy **SHA1** → Firebase Android app settings → **Add fingerprint**.

6. Rebuild the app after adding `google-services.json` and SHA-1.

| Value | Where obtained | Where placed |
|-------|----------------|--------------|
| Package name | Already `ai.investiq.investiq_ai` | Firebase Android app registration |
| `google-services.json` | Firebase download | `mobile/android/app/google-services.json` |
| SHA-1 | `keytool` on debug/release keystore | Firebase Console → Android app → SHA certificate fingerprints |
| Android OAuth client ID | Auto-created after SHA-1 | Optionally append to backend `GOOGLE_CLIENT_IDS` |

Gradle: if `google-services.json` is present, the Google Services plugin is applied automatically.

---

## Step 7 — Web OAuth authorized origins (Chrome)

In Google Cloud → Credentials → your **Web client**:

**Authorized JavaScript origins** (examples):

- `http://localhost`
- `http://localhost:XXXX` (port shown by `flutter run -d chrome`)
- `http://127.0.0.1:XXXX`

Flutter web often uses a random port; add the exact origin from the browser address bar if sign-in fails with origin errors.

**Authorized redirect URIs** (if prompted):

- `http://localhost`
- `https://your-project.firebaseapp.com/__/auth/handler`

---

## Step 8 — Verification checklist

After config is filled:

| # | Check | How |
|---|--------|-----|
| 1 | Backend sees Google enabled | `curl -s http://127.0.0.1:8080/api/v1/auth/providers` → `"google":true` |
| 2 | Flutter has config | App starts without “not configured” on Google button |
| 3 | Google UI works | Tap **Continue with Google** → account picker |
| 4 | Backend accepts token | Network: `POST /api/v1/auth/google` → `200` + `access_token` |
| 5 | User in Postgres | `podman exec -it investiq-ai_postgres_1 psql -U investiq -c "SELECT email, auth_provider, firebase_uid FROM users;"` |
| 6 | JWT works | `GET /api/v1/auth/me` with `Authorization: Bearer <access>` |
| 7 | Persistence | Kill app → reopen → still logged in |
| 8 | Logout | Settings/logout → cleared session; must sign in again |

Automated without secrets:

```bash
./scripts/check-auth-config.sh          # reports missing config
cd backend && cargo test
cd mobile && flutter analyze
curl -s -X POST http://127.0.0.1:8080/api/v1/auth/google \
  -H 'Content-Type: application/json' \
  -d '{"id_token":"invalid"}'           # 401 after config; 400/503 before
```

---

## File map (quick reference)

| Secret / file | Location |
|---------------|----------|
| Backend Firebase project + OAuth audiences | `InvestIQ-AI/.env` (from `.env.example`) |
| Flutter dart-defines (local) | `mobile/config/firebase.dart-define.json` |
| Example (safe to commit) | `mobile/config/firebase.dart-define.json.example` |
| Android Firebase file | `mobile/android/app/google-services.json` |
| Android example | `mobile/android/app/google-services.json.example` |
| Generated-style options (code) | `mobile/lib/firebase_options.dart` (reads env defines only) |
| This guide | `CONFIGURATION_REQUIRED.md` |

---

## Security rules

- **Never** commit `.env`, `firebase.dart-define.json`, or `google-services.json` with production keys to a public repo if avoidable (Android `google-services.json` is often committed in private apps; treat as semi-sensitive).
- Use a **strong** `JWT_SECRET` (≥32 chars) in any shared/staging environment.
- Prefer separate Firebase projects for dev vs production.

---

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| “Google Sign-In is not configured” in Flutter | Missing dart-defines / `firebase.dart-define.json` not passed |
| Backend `google: false` | Empty `FIREBASE_PROJECT_ID` and `GOOGLE_CLIENT_IDS` in `.env`; restart API |
| `invalid or expired id token` | Wrong project ID; token from different Firebase project; clock skew |
| `audience mismatch` | Add Web client ID to `GOOGLE_CLIENT_IDS`; ensure `FIREBASE_PROJECT_ID` matches token `aud` |
| Web popup blocked / origin error | Add Flutter web origin to OAuth client + Firebase authorized domains |
| Android `ApiException: 10` | Missing SHA-1 in Firebase; package name mismatch |
| Android no `idToken` | Set `GOOGLE_WEB_CLIENT_ID` as `serverClientId` (Web client ID, not Android client) |

---

## What is already implemented (no secrets needed)

- Flutter: `GoogleAuthService`, login/register Google buttons, secure token storage, bootstrap `/auth/me`, logout
- Backend: `POST /api/v1/auth/google`, JWKS verification, user upsert, JWT issue, refresh/logout
- Migrations: nullable `password_hash`, `google_sub`, `firebase_uid`, `auth_provider`
- Android Gradle: optional Google Services plugin when `google-services.json` exists
- Scripts: `check-auth-config.sh`, `run-mobile-chrome.sh`

**Blocker for “fully functional Google login” on any machine:** completing Steps 1–5 (and 6 for Android) with **your** Firebase project values.
