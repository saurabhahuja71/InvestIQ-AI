# InvestIQ AI

Cross-platform fintech app built **MVP-first** so you can reach the Play Store and App Store faster.

## Product strategy: four modules only (MVP)

| MVP (ship first) | Later (do not block launch) |
|------------------|-----------------------------|
| **IPO Tracker** | Broker integrations |
| **Portfolio Tracker** (manual) | Advanced analytics (deep XIRR/risk/rebalance) |
| **AI Chat** (educational + disclaimers) | Premium / subscriptions |
| **Trading Journal** (manual) | Live prices, push matrix, news, multi-currency |

Full phased plan: **[docs/05-roadmap.md](docs/05-roadmap.md)**.  
**Session dashboard (start here every time):** **[PROJECT_STATUS.md](PROJECT_STATUS.md)**.

| Layer | Stack |
|-------|--------|
| Mobile | Flutter (Material 3), Riverpod, go_router |
| API | Rust + Axum, JWT, Argon2 |
| Data | PostgreSQL 16, Redis 7 |
| Ops | **Podman Compose**, dnf host packages, GitHub Actions |

---

## Repository layout

```
InvestIQ-AI/
├── docs/                 # Architecture, schema, API, wireframes, roadmap, deploy
├── backend/              # Rust Axum API
│   ├── migrations/       # SQL migrations + seed IPOs
│   └── src/modules/      # auth, ipo, portfolio, journal, ai
├── mobile/               # Flutter application
├── compose.yml           # Podman Compose (preferred)
├── docker-compose.yml    # Compatibility alias
├── scripts/
│   ├── install-deps-dnf.sh
│   ├── compose.sh
│   └── dev.sh
├── .github/workflows/ci.yml
└── .env.example
```

---

## Documentation index

1. [Architecture](docs/01-architecture.md)
2. [Database schema](docs/02-database-schema.md)
3. [API design](docs/03-api-design.md)
4. [Navigation & wireframes](docs/04-navigation-and-wireframes.md)
5. [Roadmap MVP → production](docs/05-roadmap.md)
6. [Deployment](docs/06-deployment.md)
7. [Podman + dnf setup](docs/09-podman-dnf-setup.md)
8. **[Run locally (full guide)](docs/10-local-run.md)** ← start here for day-to-day dev
9. **[Offline container images (no docker.io)](docs/11-offline-container-images.md)** ← second laptop / blocked registry
10. **[Google / Firebase Auth setup](CONFIGURATION_REQUIRED.md)** ← required for Continue with Google

---

## Notes — Google Sign-In

Code for Google Authentication is **fully implemented** (Flutter Firebase Auth → `POST /api/v1/auth/google` → JWT → Postgres → secure storage).

**You must create a Firebase project and paste real IDs** — nothing is hardcoded:

1. Follow **[CONFIGURATION_REQUIRED.md](CONFIGURATION_REQUIRED.md)**
2. Fill `InvestIQ-AI/.env` (`FIREBASE_PROJECT_ID`, `GOOGLE_CLIENT_IDS`)
3. Copy `mobile/config/firebase.dart-define.json.example` → `firebase.dart-define.json` and fill values
4. Restart API + run: `./scripts/run-mobile-chrome.sh`
5. Check: `./scripts/check-auth-config.sh` and `curl -s http://127.0.0.1:8080/api/v1/auth/providers`

Until then, email/password login works; Google shows a clear configuration error.

---

## Notes — offline Postgres & Redis images

Compose needs `postgres:16-alpine` and `redis:7-alpine`. Those images are **exported and published on GitHub Releases** (not in git) so a machine that **cannot reach docker.io** can still start the stack.

| Item | Value |
|------|--------|
| Release | [`container-images-v1`](https://github.com/saurabhahuja71/InvestIQ-AI/releases/tag/container-images-v1) |
| Assets | `postgres-16-alpine.tar.gz` (~118 MB), `redis-7-alpine.tar.gz` (~17 MB) |
| Load | `./scripts/load-container-images.sh dist/container-images` |
| Re-export | `./scripts/export-container-images.sh` (host that *can* pull docker.io) |

**Second laptop / no docker.io:**

```bash
git clone git@github.com:saurabhahuja71/InvestIQ-AI.git && cd InvestIQ-AI
mkdir -p dist/container-images
gh release download container-images-v1 -D dist/container-images
# Or download the two .tar.gz files from the Releases page
./scripts/load-container-images.sh dist/container-images
./scripts/compose.sh up -d postgres redis
```

Full write-up: [docs/11-offline-container-images.md](docs/11-offline-container-images.md).

---

## Quick start

**Full step-by-step:** [docs/10-local-run.md](docs/10-local-run.md)

### Minimal path

```bash
# Once: host packages
./scripts/install-deps-dnf.sh

cp -n .env.example .env
# If docker.io is blocked, load images from the GitHub Release first (see Notes above)
./scripts/compose.sh up -d postgres redis   # Podman: Postgres + Redis

# Terminal 1 — API
source "$HOME/.cargo/env"
cd backend && cargo run
# curl http://127.0.0.1:8080/health

# Terminal 2 — Flutter (Chrome)
export PATH="$HOME/development/flutter/bin:$PATH"
cd mobile
flutter pub get
flutter run -d chrome --dart-define=API_BASE_URL=http://127.0.0.1:8080
```

Helpers:

```bash
./scripts/dev.sh                 # compose DBs + cargo run
./scripts/compose.sh up -d --build   # full stack in Podman
```

---

## MVP modules (implemented skeleton)

Scaffold exists for all four. Harden these before any “Phase 2” work.

### 1. IPO Tracker
- List/filter by status & board (mainboard / SME)
- Detail: issue band, lot, dates, subscription
- **GMP always marked unofficial** with disclaimer
- Watchlist (allotment = stub / deep-link only for MVP)

### 2. Portfolio Tracker
- Manual holdings: stock, ETF, MF, gold, bond, cash
- Transactions + dashboard value + simple allocation chart
- Basic return display (advanced XIRR UI can stay secondary)

### 3. AI Chat
- Chat UI with persistent **investment disclaimer**
- No guaranteed returns in system prompt
- Works with educational stub if `AI_API_KEY` is unset

### 4. Trading Journal
- Manual trade entry (side, strategy, R:R, emotions, notes)
- Basic stats: win rate, avg win/loss, largest winner/loser
- Defer broker import and heavy AI “mistake coach” to post-launch

### Auth & security
- Register / login / refresh / logout
- Argon2id passwords, JWT access + rotating refresh (hashed in DB)
- Secure token storage on device
- AES-256-GCM helper for field encryption
- HTTPS + rate limiting planned at edge (Compose-ready)

---

## Authentication flow

```
Register/Login → access_token (short) + refresh_token (long)
API calls: Authorization: Bearer <access>
401 → POST /auth/refresh → new pair (old refresh revoked)
Logout → revoke refresh
```

---

## State management

- **Riverpod** `StateNotifier` / `FutureProvider` for auth, IPO, portfolio, journal
- **go_router** shell navigation (5 tabs)
- **flutter_secure_storage** for tokens
- **Hive** cache box initialized for offline (expand per feature)

---

## CI/CD

GitHub Actions (`.github/workflows/ci.yml`):
- Backend: `fmt`, `clippy`, `test` with Postgres + Redis services
- Mobile: `flutter analyze` + `flutter test`
- Docker image build for API

---

## Important disclaimers

- **Not financial advice.** InvestIQ AI does not provide guaranteed returns.
- **Grey Market Premium is unofficial** and not endorsed by exchanges or regulators.
- Always do your own research.

---

## License

Proprietary — all rights reserved.
