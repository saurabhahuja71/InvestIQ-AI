# Run InvestIQ AI locally

Complete local development guide using **Podman Compose**, **dnf** host packages, **Rust**, and **Flutter**.

Related docs:
- [Podman + dnf package setup](09-podman-dnf-setup.md)
- [Deployment](06-deployment.md)
- [Project status dashboard](../PROJECT_STATUS.md)

---

## Prerequisites

| Tool | Notes |
|------|--------|
| RHEL / Oracle Linux / Fedora | `dnf` package manager |
| Podman + Podman Compose | Containers for Postgres & Redis |
| Rust stable | `cargo` / `rustc` via rustup recommended |
| Flutter stable | e.g. `$HOME/development/flutter` |
| Optional | `AI_API_KEY` for remote LLM (local AI engine works without it) |

---

## 1. One-time host setup

```bash
cd ~/InvestIQ-AI

# Install Podman, build tools, CLIs (needs sudo)
./scripts/install-deps-dnf.sh
```

### Rust (if not installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

### Flutter (if not installed)

Install the Flutter SDK, then:

```bash
export PATH="$HOME/development/flutter/bin:$PATH"
flutter --version
flutter doctor
```

Add the `export PATH=...` line to `~/.bashrc` so it persists.

Full package lists: [09-podman-dnf-setup.md](09-podman-dnf-setup.md).

---

## 2. Environment file

```bash
cd ~/InvestIQ-AI
cp -n .env.example .env
```

Edit `.env` as needed:

| Variable | Local default / tip |
|----------|---------------------|
| `APP_ENV` | `development` |
| `JWT_SECRET` | Change from the example value |
| `DATABASE_URL` | `postgres://investiq:investiq@localhost:5432/investiq` |
| `REDIS_URL` | `redis://127.0.0.1:6379` |
| `AI_API_KEY` | Optional; leave empty for local educational AI |
| `CORS_ORIGINS` | `*` is OK in development only |
| `FIREBASE_PROJECT_ID` | Required for Google Sign-In (Firebase project ID) |
| `GOOGLE_CLIENT_IDS` | Comma-separated OAuth client IDs (Web + Android) for ID token audience |
| `IPO_SYNC_INTERVAL_SECS` | NSE pull interval (default `900`) |

---

## 3. Start Postgres and Redis (Podman)

```bash
cd ~/InvestIQ-AI
./scripts/compose.sh up -d postgres redis
```

Equivalent:

```bash
podman compose -f compose.yml up -d postgres redis
# or, if only the standalone binary exists:
# podman-compose -f compose.yml up -d postgres redis
```

Check status:

```bash
./scripts/compose.sh ps
./scripts/compose.sh logs postgres
```

Wait until Postgres is ready:

```bash
./scripts/compose.sh exec postgres pg_isready -U investiq
```

---

## 4. Run the API (Rust)

### Option A — Dev helper (recommended)

Starts (or expects) Compose DBs, loads `.env`, runs the API on the host:

```bash
cd ~/InvestIQ-AI
./scripts/dev.sh
```

### Option B — Manual

```bash
source "$HOME/.cargo/env"
cd ~/InvestIQ-AI/backend
cargo run
```

Migrations run automatically on startup (`sqlx::migrate!`).

### Option C — Full stack in Podman (API container + DB + Redis)

```bash
cd ~/InvestIQ-AI
./scripts/compose.sh up -d --build
```

### Verify API

```bash
curl -s http://127.0.0.1:8080/health
curl -s http://127.0.0.1:8080/ready
```

Expected: JSON with `"status":"ok"` / `"status":"ready"`.

**API base URL:** `http://127.0.0.1:8080`  
**REST prefix:** `http://127.0.0.1:8080/api/v1`

---

## 5. Run the Flutter app

Open a **second terminal**:

```bash
export PATH="$HOME/development/flutter/bin:$PATH"
cd ~/InvestIQ-AI/mobile
flutter pub get
```

### Chrome (easiest without Android SDK)

```bash
flutter run -d chrome \
  --dart-define=API_BASE_URL=http://127.0.0.1:8080 \
  --dart-define=FIREBASE_API_KEY=YOUR_KEY \
  --dart-define=FIREBASE_APP_ID=YOUR_APP_ID \
  --dart-define=FIREBASE_MESSAGING_SENDER_ID=YOUR_SENDER \
  --dart-define=FIREBASE_PROJECT_ID=YOUR_PROJECT \
  --dart-define=FIREBASE_AUTH_DOMAIN=YOUR_PROJECT.firebaseapp.com \
  --dart-define=GOOGLE_WEB_CLIENT_ID=YOUR_WEB_CLIENT_ID.apps.googleusercontent.com
```

Email/password still works without Firebase. **Continue with Google** requires the dart-defines above **and** matching backend `FIREBASE_PROJECT_ID` / `GOOGLE_CLIENT_IDS`.

**IPO feed:** API syncs from NSE on startup and every `IPO_SYNC_INTERVAL_SECS`. Manual: `curl -X POST http://127.0.0.1:8080/api/v1/ipos/sync`.

### Android emulator

```bash
flutter run --dart-define=API_BASE_URL=http://10.0.2.2:8080
```

(`10.0.2.2` is the emulator’s alias for the host machine.)

### Physical Android device (same LAN)

```bash
# Use your machine’s LAN IP, e.g. 192.168.1.20
flutter run --dart-define=API_BASE_URL=http://192.168.1.20:8080
```

Ensure the device can reach that IP and that firewall allows port `8080`.

---

## 6. Smoke test checklist

1. **Register** a new account (email + password ≥ 8 characters).  
2. **Home** — app shell loads; open IPOs from chips/tabs.  
3. **IPOs** — live NSE IPOs appear after sync (~few seconds on API boot); open detail; missing fields show **Not Available**; GMP is **Not Available** unless a real unofficial source is configured (NSE does not publish GMP).  
4. **Allotment** — enter PAN last 4 and/or application number; get pending/allotted/not_allotted.  
5. **Portfolio** — add a holding; see value, today P&L, allocation.  
6. **Journal** — log a trade; check stats strip.  
7. **AI** — send a message (works without API key).  
8. **Settings** — theme cycle; optional export / notification prefs.  
9. **Notifications** — open inbox (syncs IPO events).

---

## 7. Ports and services

| Service | Host address |
|---------|----------------|
| API | `http://127.0.0.1:8080` |
| Postgres | `localhost:5432` (user/db/password: `investiq`) |
| Redis | `localhost:6379` |

Compose services: `postgres`, `redis`, `api` (optional).

---

## 8. Stop and clean up

```bash
# API: Ctrl+C in the cargo / dev.sh terminal

# Stop containers (keeps volume data)
./scripts/compose.sh down

# Optional: remove named volume (wipes DB)
podman volume rm investiq-ai_pgdata 2>/dev/null || true
# volume name may vary; list with:
podman volume ls
```

---

## 9. Common issues

| Problem | Fix |
|---------|-----|
| `podman compose` not found | `sudo dnf -y install podman podman-compose` or use `./scripts/compose.sh` |
| Port 5432/6379/8080 in use | Stop conflicting service or change ports in `compose.yml` |
| `/ready` fails | Ensure Postgres + Redis containers are healthy |
| Flutter can’t reach API | Check `API_BASE_URL`; use `10.0.2.2` on Android emulator |
| `cargo` build fails on OpenSSL | `sudo dnf -y install openssl-devel gcc pkgconf-pkg-config` |
| CORS errors in browser | Dev allows `CORS_ORIGINS=*`; ensure API is on `127.0.0.1:8080` |
| Empty IPO list | Wait for NSE sync on API boot (`initial NSE IPO sync complete` in logs) or `curl -X POST http://127.0.0.1:8080/api/v1/ipos/sync`; see [docs/11-ipo-data-provider.md](11-ipo-data-provider.md) |

---

## 10. Useful commands cheat sheet

```bash
# Deps
./scripts/install-deps-dnf.sh

# Infra
./scripts/compose.sh up -d postgres redis
./scripts/compose.sh ps
./scripts/compose.sh logs -f postgres
./scripts/compose.sh down

# API
./scripts/dev.sh
# or
cd backend && cargo run && cargo test && cargo clippy --all-targets -- -D warnings

# App
export PATH="$HOME/development/flutter/bin:$PATH"
cd mobile && flutter analyze && flutter run -d chrome \
  --dart-define=API_BASE_URL=http://127.0.0.1:8080
```

---

## 11. Quick path (minimal copy-paste)

```bash
cd ~/InvestIQ-AI
./scripts/install-deps-dnf.sh          # once
cp -n .env.example .env
./scripts/compose.sh up -d postgres redis
source "$HOME/.cargo/env"
cd backend && cargo run                # terminal 1

# terminal 2
export PATH="$HOME/development/flutter/bin:$PATH"
cd ~/InvestIQ-AI/mobile
flutter run -d chrome --dart-define=API_BASE_URL=http://127.0.0.1:8080
```

---

*Keep this file updated when local run steps change.*

---

## Offline images (no docker.io)

If Podman/Docker cannot pull from `docker.io`, use the GitHub Release image tarballs:

See **[11-offline-container-images.md](11-offline-container-images.md)**.

```bash
mkdir -p dist/container-images
gh release download container-images-v1 -D dist/container-images
./scripts/load-container-images.sh dist/container-images
./scripts/compose.sh up -d postgres redis
```
