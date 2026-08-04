# Android — debug APK & Google Sign-In on a physical device

## Package & signing

| Item | Value |
|------|--------|
| Application ID | `ai.investiq.investiq_ai` |
| Debug keystore | `~/.android/debug.keystore` (Android debug standard) |
| Debug SHA-1 (this machine) | `AC:4E:85:AB:10:45:54:96:A7:26:D2:69:D7:70:1A:25:FB:2D:9E:F4` |
| Debug SHA-256 | `BE:5C:92:47:BF:64:20:49:F2:F3:D5:96:87:7E:D5:31:30:12:69:98:4D:17:30:AF:E9:A3:F8:B1:95:88:B2:DC` |

Re-print fingerprints anytime:

```bash
keytool -list -v \
  -keystore ~/.android/debug.keystore \
  -alias androiddebugkey -storepass android -keypass android \
  | grep -E 'SHA1:|SHA256:'
```

## Firebase Console (required for Google Sign-In on Android)

Client config alone is not enough on Android. You must register the app and SHA-1.

### 1. Add Android app

1. https://console.firebase.google.com/project/investiq-ai-a514e/settings/general  
2. **Add app** → **Android**  
3. **Android package name:** `ai.investiq.investiq_ai` (exact)  
4. App nickname: `investiq-android`  
5. **Register app**

### 2. Add debug SHA-1

1. Same Android app → **Add fingerprint**  
2. Paste:

   ```text
   AC:4E:85:AB:10:45:54:96:A7:26:D2:69:D7:70:1A:25:FB:2D:9E:F4
   ```

3. Save (Firebase creates an Android OAuth client automatically).

### 3. Download `google-services.json`

1. Download **google-services.json**  
2. Place at:

   ```text
   mobile/android/app/google-services.json
   ```

   (gitignored; enables Google Services Gradle plugin automatically)

3. Optionally copy Android `mobilesdk_app_id` / apiKey into  
   `mobile/config/firebase.dart-define.json` as:

   - `FIREBASE_ANDROID_APP_ID`
   - `FIREBASE_ANDROID_API_KEY`
   - `FIREBASE_ANDROID_MESSAGING_SENDER_ID` (usually same as web)

Without SHA-1, Google Sign-In typically fails with **ApiException: 10** (DEVELOPER_ERROR).

### 4. Google provider

Authentication → Sign-in method → **Google** must stay **Enabled** (already done for web).

### 5. Web client ID (already set for this machine)

`GOOGLE_WEB_CLIENT_ID` / `serverClientId` must be the **Web** OAuth client  
(`….apps.googleusercontent.com`) so `google_sign_in` returns an ID token.  
This is already in `firebase.dart-define.json` on the build machine.

---

## Build signed debug APK

```bash
# Host API must be reachable from the phone (not 127.0.0.1)
# Optional: API_HOST=192.168.x.x ./scripts/build-android-debug-apk.sh

cd ~/InvestIQ-AI
./scripts/build-android-debug-apk.sh
```

Output:

```text
dist/android/investiq-ai-debug.apk
# also:
mobile/build/app/outputs/flutter-apk/app-debug.apk
```

The APK is **signed with the Android debug keystore** (normal for debug builds).

---

## Install on a physical device

1. Enable **Developer options** → **USB debugging** on the phone.  
2. USB cable → trust computer.  
3. On PC:

```bash
export PATH="$HOME/Android/Sdk/platform-tools:$PATH"
adb devices
# must show device as "device", not "unauthorized"

adb install -r dist/android/investiq-ai-debug.apk
```

4. **API on PC** must listen on all interfaces (already `HOST=0.0.0.0` in `.env`):

```bash
cd ~/InvestIQ-AI/backend
set -a && source ../.env && set +a
cargo run
```

5. Phone and PC on **same Wi‑Fi**. Open host firewall for the API **once** (permanent):

```bash
# Allows TCP 8080 through firewalld/ufw/iptables (needs sudo password)
./scripts/open-api-firewall.sh
```

Verify from the PC:

```bash
curl -s "http://$(hostname -I | awk '{print $1}'):8080/health"
```

The phone must also reach that LAN IP (not blocked by AP client isolation).  
USB alternative without firewall: `USE_ADB_REVERSE=1 ./scripts/build-android-debug-apk.sh` and keep `adb reverse tcp:8080 tcp:8080`.  
6. APK is built with `API_BASE_URL=http://<PC_LAN_IP>:8080` by the build script.

Check which URL was baked in by rebuilding after setting:

```bash
API_HOST=192.168.1.20 ./scripts/build-android-debug-apk.sh
```

---

## Android-specific app settings (already in repo)

| Item | Status |
|------|--------|
| `INTERNET` + `ACCESS_NETWORK_STATE` | AndroidManifest |
| Cleartext HTTP to local API | `usesCleartextTraffic=true` (dev) |
| minSdk ≥ 23 | `build.gradle.kts` |
| Debug signing | debug + release use debug keystore locally |
| Google Services plugin | Applied when `google-services.json` exists |
| FlutterFire options | dart-defines + optional Android overrides |

---

## Verify Google Sign-In on device

1. Open **InvestIQ AI** on the phone.  
2. **Continue with Google** → account picker.  
3. Should land on home after backend issues JWT.  
4. Confirm user row:

```bash
podman exec -it investiq-ai_postgres_1 \
  psql -U investiq -c \
  "SELECT email, auth_provider, firebase_uid IS NOT NULL FROM users ORDER BY created_at DESC LIMIT 5;"
```

5. Kill app → reopen → still logged in (secure storage).  
6. Logout → must sign in again.

---

## Troubleshooting

| Error | Fix |
|-------|-----|
| `ApiException: 10` | Add debug SHA-1 to Firebase Android app; package name must match |
| `configuration-not-found` | Enable Authentication + Google provider |
| Network / login fails | Wrong `API_BASE_URL` (use PC LAN IP, not 127.0.0.1); firewall; API `HOST=0.0.0.0` |
| `adb devices` empty | USB debugging, cable, `adb kill-server && adb start-server` |
| No Google ID token | Set `GOOGLE_WEB_CLIENT_ID` (Web client) in dart-define file |

---

## Related

- [CONFIGURATION_REQUIRED.md](../CONFIGURATION_REQUIRED.md)  
- [Offline container images](11-offline-container-images.md) (other laptop)  

## Task completion status

See [14-android-task-status.md](14-android-task-status.md).

