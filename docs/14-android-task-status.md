# Android task status — debug APK & device Google Sign-In

**Date:** 2026-08-03  
**Host:** this development machine (Oracle Linux)  
**Package:** `ai.investiq.investiq_ai`

---

## Completed

| Item | Status | Notes |
|------|--------|--------|
| Android SDK toolchain | Done | `$HOME/Android/Sdk`, Flutter doctor Android ✓ |
| App Android config | Done | INTERNET, cleartext HTTP (dev), minSdk 23, debug signing, Kotlin plugin, shrinkResources off |
| Gradle build fixes | Done | Fixed Kotlin plugin / shrink-resources AGP 9 issues |
| Firebase dart-defines for APK | Done | Baked from `mobile/config/firebase.dart-define.json` |
| API base URL for devices | Done | Debug APK uses `http://192.168.0.9:8080` (this host LAN IP at build time) |
| **Signed debug APK** | **Done** | See paths below |
| Debug keystore + SHA-1 | Done | Generated / printed for Firebase |
| Docs & build script | Done | `docs/12-android-run.md`, `scripts/build-android-debug-apk.sh` |
| Backend API on host | Running | `http://127.0.0.1:8080/health` OK (`HOST=0.0.0.0`) |

### APK paths (local machine — **not** in git; `dist/` is gitignored)

```text
dist/android/investiq-ai-debug.apk          (~149 MB)
mobile/build/app/outputs/flutter-apk/app-debug.apk
```

Signed with Android **debug** keystore (`~/.android/debug.keystore`).

Rebuild:

```bash
API_HOST=192.168.0.9 ./scripts/build-android-debug-apk.sh
# or your current LAN IP
```

### Debug fingerprints (add to Firebase Android app)

```text
SHA1:   AC:4E:85:AB:10:45:54:96:A7:26:D2:69:D7:70:1A:25:FB:2D:9E:F4
SHA256: BE:5C:92:47:BF:64:20:49:F2:F3:D5:96:87:7E:D5:31:30:12:69:98:4D:17:30:AF:E9:A3:F8:B1:95:88:B2:DC
```

---

## Not completed (blocked on this host)

### 1. Physical Android device — **not connected**

```text
adb devices  → empty
flutter devices → only Linux desktop + Chrome
```

**Cannot verify Google Sign-In on a physical device** until a phone is attached with USB debugging (or wireless debugging) authorized.

**Your steps when phone is ready:**

```bash
export PATH="$HOME/Android/Sdk/platform-tools:$PATH"
adb devices   # must show "device"
adb install -r dist/android/investiq-ai-debug.apk
# Phone + PC same Wi-Fi; API: cargo run with HOST=0.0.0.0
# Open app → Continue with Google
```

### 2. Firebase **Android** app registration — **manual (Console)**

Not done from this environment (requires your Firebase Console):

1. Project **investiq-ai-a514e** → Add Android app  
2. Package name: **`ai.investiq.investiq_ai`**  
3. Add **debug SHA-1** (above)  
4. Download **`google-services.json`** →  
   `mobile/android/app/google-services.json`  
5. Rebuild APK after that file is present  

Without SHA-1 + Android app, Google Sign-In on device often fails with **`ApiException: 10`**.

`google-services.json` was **missing** at build time. FlutterFire still embeds web-based options via dart-defines (works for many cases); official Android file is still required for production-grade Google Sign-In.

### 3. Device Google Sign-In E2E — **not verified**

Blocked by (1) and likely (2).

### 4. APK not uploaded to GitHub

Binary ~149 MB is gitignored (`dist/`). Keep it locally or attach to a **GitHub Release** if you want remote download (same pattern as container images).

### 5. NDK version warning

Plugins prefer NDK `28.2.13676358`. `build.gradle.kts` now pins that version; first rebuild may download NDK 28 if not installed yet.

### 6. Android Studio

Not installed (optional; CLI SDK is enough for APK builds).

---

## Checklist for “fully done” on your side

- [ ] Phone USB debugging on → `adb devices` shows device  
- [ ] Firebase Android app + SHA-1 + `google-services.json`  
- [ ] Rebuild: `./scripts/build-android-debug-apk.sh`  
- [ ] `adb install -r dist/android/investiq-ai-debug.apk`  
- [ ] API running; phone can reach `http://<PC_LAN_IP>:8080/health` (browser on phone)  
- [ ] Google Sign-In → JWT → user row in Postgres  

---

## Related docs

- [12-android-run.md](12-android-run.md) — full Android + Firebase steps  
- [CONFIGURATION_REQUIRED.md](../CONFIGURATION_REQUIRED.md) — Firebase Auth  
- [13-github-secrets.md](13-github-secrets.md) — secrets for CI  
