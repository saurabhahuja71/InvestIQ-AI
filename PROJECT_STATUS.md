# InvestIQ AI — Project Status Dashboard

> **How to use this file**  
> At the start of every coding session: read this file, update the relevant section after your work, then commit.  
> Keep status honest — prefer “partial” over “done” when something compiles but is not production-ready.

| Field | Value |
|-------|--------|
| **Last updated** | 2026-08-05 |
| **Product phase** | Milestone 2 — Production-quality IPO Tracker (+ Google Auth from M1) |
| **Overall completion** | ~75% toward Play Store MVP |
| **Release readiness** | **~62%** |
| **Default branch** | `main` |
| **Primary stack** | Flutter 3.35 · Rust/Axum · PostgreSQL 16 · Redis 7 · Firebase Auth |
| **IPO data source** | **NSE India public APIs** (`/api/ipo-current-issue`, `/api/public-past-issues`) |

---

## 1. Executive summary

**Milestone 1 (Google Auth + real NSE IPO) and Milestone 2 (production IPO UX/sync) are both in code:**

- **Google Sign-In** (Firebase Auth + Google provider on Flutter; backend verifies Firebase/Google ID tokens via JWKS; issues app JWT + refresh)
- **Production IPO Tracker** on live NSE data: background sync, Redis list cache, `POST /ipos/sync`, refresh query param, expanded models, production Flutter list/detail UX
- Email/password auth still available as secondary path

Other modules (portfolio, journal, AI) remain as before — not the focus of recent milestones.

**Still not store-ready:** Firebase/Google OAuth credentials must be filled by the operator; Android SDK/signing; hosted privacy policy; FCM push delivery.

---

## 2. Completed features

### Documentation & ops
- [x] Architecture, schema, API, wireframes, roadmap, deploy docs
- [x] **Podman Compose** (`compose.yml`) + dnf install script, CI workflow, `.env.example`
- [x] Migrations: init + MVP + **google_auth** + **`20240805000000_ipo_live_source`** (drop seeds, NSE sync columns)
- [x] **IPO data provider doc** (`docs/11-ipo-data-provider.md`)
- [x] **Offline container images** on GitHub Release `container-images-v1` + load/export scripts

### Backend
- [x] Auth: register/login/refresh/logout/me/update, **change-password**, **export**, **delete account**
- [x] **Google/Firebase**: `POST /auth/google` with JWKS verification, user upsert, JWT session
- [x] Suspended/deleted → **403 Forbidden**
- [x] **NSE IPO sync** (`IpoSyncService`): boot + periodic refresh, Redis lock, list cache TTL
- [x] `GET /ipos` filters: status, board, q, page, per_page, refresh; `POST /ipos/sync`
- [x] IPO detail: dates, band, lot, min investment, registrar, lead managers, subscription, RHP URL, ratios link
- [x] IPO list/detail/watchlist/AI summary + **allotment engine** (pending/allotted/not_allotted)
- [x] No invented GMP / logos / websites / structured financials
- [x] Portfolio holdings with **last_price / prev_close**, price update API, txn avg-cost rollup
- [x] Analytics: total value/cost, **today P&L + %**, unrealized, XIRR, **CAGR**, allocations
- [x] Journal CRUD + analytics + AI mistakes
- [x] AI: remote chat when `AI_API_KEY` set; **local grounded engine** otherwise; remote failure fallback
- [x] Notifications module: list, read, prefs, devices, price alerts, **sync-ipo-events**
- [x] Redis **rate limit** middleware on `/api/v1`
- [x] AES seal on data export when `AES_KEY_BASE64` set
- [x] Structured JSON logging; production JWT/CORS checks
- [x] Unit tests: XIRR, CAGR, allotment, crypto, AI local disclaimer, **NSE parsers** (**13 tests**)

### Mobile
- [x] Material 3 shell, auth, IPO, portfolio, journal, AI, settings
- [x] **Continue with Google** on login/register (Firebase + google_sign_in)
- [x] IPO tabs: Open / Upcoming / Closed / Listed; board filter, debounced search
- [x] Pull-to-refresh (triggers NSE sync), pagination / infinite scroll
- [x] Loading skeleton, empty state, error state + Retry; offline Hive cache + banner
- [x] IPO detail: all fields with API values or **Not Available**; prospectus/website links via `url_launcher`
- [x] Settings wired to backend (currency, biometric, password, export clipboard, delete)
- [x] Notifications inbox + prefs screens
- [x] Allotment form (PAN last4 + application number)
- [x] Portfolio shows XIRR/CAGR/today/unrealized
- [x] Offline: Hive cache for IPOs/portfolios; journal write queue; connectivity flush
- [x] Global error widget + `AppException` mapping
- [x] `flutter analyze` clean

---

## 3. Missing features

### Remaining for Play Store / ops
| Item | Status |
|------|--------|
| **Firebase project + OAuth client IDs in `.env` / dart-defines** | **Required for live Google login** |
| Android SDK + signed AAB | Missing on this host |
| Hosted privacy policy / terms URLs | Missing |
| Real FCM push send (device tokens stored only) | Partial |
| Live equity price feed (portfolio MTM) | Deferred |
| Tests ≥ 80% coverage | Not met (~unit core only) |
| Integration tests (HTTP+DB) | Missing |
| iOS TestFlight packaging | Needs macOS |

### Explicitly post-MVP / out of Milestone 2
- Broker import, premium, multi-currency FX
- Official registrar allotment APIs
- Paid IPO data vendors for GMP / logos / financials

---

## 4. Known bugs & risks

| Severity | Issue |
|----------|--------|
| High | Google login needs real `FIREBASE_PROJECT_ID` + client IDs; unconfigured → clear server/app errors |
| Medium | NSE may 403 without cookies / block changes; sync fails soft and serves last DB snapshot |
| Medium | Allotment is **indicative** (hash-based), not registrar-authoritative |
| Medium | Logo, website, industry, description, structured financials, GMP are **Not Available** from NSE (documented) |
| Medium | Issue size ₹ Cr is estimated from shares × mid band when NSE gives share count |
| Low | Company names on some upcoming rows may stay symbol-like until detail enrich runs |
| Low | Flutter test runner may fail under corporate proxy WebSocket 502 |
| Low | Default CORS `*` only allowed in non-production |
| Low | Delete-account / change-password require password; Google-only accounts get a clear validation error |

---

## 5. Technical debt

1. Handlers still SQL-heavy — extract repositories for larger coverage.  
2. No OpenAPI contract between Flutter and Rust.  
3. Price snapshots table underused vs holdings columns.  
4. FCM: register device only — need worker to send.  
5. Expand widget/integration tests next (IPO list widget coverage still thin).

---

## 6. Security status

| Area | Status |
|------|--------|
| JWT | Access + rotating refresh; production secret enforced |
| SQL | sqlx parameterized |
| AuthN/Z | Ownership checks; Forbidden for suspended |
| Rate limit | Redis fixed window on API |
| CORS | Configurable; `*` blocked in production |
| Secrets | `.env` / env vars; no secrets in git |
| IPO source | NSE public feeds; polite pacing; no fabricated market data |

---

## 7. Build status

| Target | Status |
|--------|--------|
| `cargo check` | Pass |
| `cargo test` | Pass (**13 tests**) |
| `cargo clippy -D warnings` | Pass |
| `flutter analyze` | **Pass** |
| `flutter test` | Environment-sensitive |
| Android debug/release | Blocked without SDK |
| Web build | Google button present; needs Firebase config |

---

## 8. Test status

| Suite | Est. coverage | Status |
|-------|---------------|--------|
| Rust unit (analytics, allotment, crypto, AI, **NSE parsers**) | Core paths | Pass |
| Manual IPO API smoke (2026-08-05) | List/search/detail/sync | Pass — live NSE open/upcoming/closed/listed |
| Rust integration | ~0% | Missing |
| Flutter unit/widget | Low | Minimal |
| **Target ≥ 80%** | Not met | Next priority |

### Milestone 2 manual checklist (verified)

- [x] Search works (`q=ardee`)
- [x] Filters work (status + board=sme)
- [x] Refresh / sync works (`POST /ipos/sync` + boot sync)
- [x] API failures → DB/Hive cache path implemented
- [x] Offline cache path in Flutter providers
- [x] Details page loads with registrar, lead managers, RHP, subscription; N/A for logo/GMP/website

---

## 9. Release readiness

| Gate | Ready? |
|------|--------|
| IPO Tracker production data path | **Yes (NSE-backed)** |
| Four MVP modules usable | **Yes (local full stack)** |
| Real NSE IPO list/detail | **Yes** |
| Google Sign-In E2E | **Code ready; needs Firebase credentials** |
| Backend hardened enough for staging | Mostly |
| Android signed AAB | No |
| Store legal assets | No |
| Tests/CI green for Android | No |
| **Weighted readiness** | **~62%** |

---

## 10. Next priorities

1. **Operator:** create Firebase project + set `FIREBASE_*` / `GOOGLE_CLIENT_IDS` and re-test Google login E2E  
2. Install Android SDK → debug APK + release AAB recipe  
3. Integration tests (Google auth, IPO sync)  
4. Flutter widget tests for IPO list empty/error  
5. Privacy policy page + in-app links  
6. FCM send path for notifications  
7. Staging deploy smoke script  

---

## 11. Session changelog

| Date | Change |
|------|--------|
| 2026-08-03 | Initial scaffold + dashboard |
| 2026-08-03 | **Phase 1 MVP completion:** portfolio calc, allotment, AI local+remote, settings, notifications, errors, offline, clippy clean |
| 2026-08-03 | Switched local ops docs/scripts to **Podman Compose** + **dnf** install helpers |
| 2026-08-03 | Added **docs/10-local-run.md** (full local run guide) and linked from README |
| 2026-08-03 | **Milestone 1:** Google/Firebase auth (`POST /auth/google`), NSE real IPO sync, seed removal, Flutter Google button + IPO UX |
| 2026-08-03 | **Offline images:** exported Postgres 16 + Redis 7 Alpine to GitHub Release `container-images-v1`; load/export scripts + docs |
| 2026-08-03 | **Google Auth complete (code):** FlutterFire + Android/Web scaffolding, `GET /auth/providers`, `CONFIGURATION_REQUIRED.md`; blocked only on operator Firebase secrets |
| 2026-08-05 | **Milestone 2:** live NSE IPO sync service, Redis list cache, production Flutter IPO list/detail, provider docs |

---

## 12. How to test this build

**Canonical guide:** [docs/10-local-run.md](docs/10-local-run.md)  
**No docker.io:** [docs/11-offline-container-images.md](docs/11-offline-container-images.md)  
**IPO source:** [docs/11-ipo-data-provider.md](docs/11-ipo-data-provider.md)

### Notes — container images on GitHub

| Note | Detail |
|------|--------|
| Why | Second laptop / environments cannot pull from `docker.io` |
| What | `postgres:16-alpine` + `redis:7-alpine` saved as gzipped `podman save` tarballs |
| Where | GitHub Release **[container-images-v1](https://github.com/saurabhahuja71/InvestIQ-AI/releases/tag/container-images-v1)** (not in git history) |
| Assets | `postgres-16-alpine.tar.gz`, `redis-7-alpine.tar.gz` |
| Load | `./scripts/load-container-images.sh dist/container-images` |
| Re-export | `./scripts/export-container-images.sh` then upload a new release if tags change |

### Infra + API (normal path — docker.io OK)

```bash
cd InvestIQ-AI && cp -n .env.example .env
# Set FIREBASE_PROJECT_ID and GOOGLE_CLIENT_IDS for Google login
./scripts/compose.sh up -d postgres redis
cd backend && cargo run
# wait for: initial NSE IPO sync complete
# curl http://127.0.0.1:8080/health
# curl -X POST http://127.0.0.1:8080/api/v1/ipos/sync
# curl 'http://127.0.0.1:8080/api/v1/ipos?status=open&per_page=5'
```

### Infra when docker.io is blocked

```bash
mkdir -p dist/container-images
gh release download container-images-v1 -D dist/container-images
./scripts/load-container-images.sh dist/container-images
./scripts/compose.sh up -d postgres redis
```

### Flutter (Chrome)

```bash
export PATH="$HOME/development/flutter/bin:$PATH"
cd mobile
flutter pub get
flutter run -d chrome \
  --dart-define=API_BASE_URL=http://127.0.0.1:8080 \
  --dart-define=FIREBASE_API_KEY=... \
  --dart-define=FIREBASE_APP_ID=... \
  --dart-define=FIREBASE_MESSAGING_SENDER_ID=... \
  --dart-define=FIREBASE_PROJECT_ID=... \
  --dart-define=FIREBASE_AUTH_DOMAIN=... \
  --dart-define=GOOGLE_WEB_CLIENT_ID=...
```

Checklist: Google sign-in (or email) → **IPOs (live NSE)** → filters/search/refresh → detail N/A fields → (other modules unchanged).

---

## 13. Definition of done (MVP store)

- [x] Clippy clean  
- [x] IPO Tracker on live exchange data (no dummy seeds)  
- [ ] Tests ≥ 80%  
- [ ] Signed AAB + privacy policy  
- [ ] This file **Release readiness ≥ 85%**

---

*End of dashboard.*
