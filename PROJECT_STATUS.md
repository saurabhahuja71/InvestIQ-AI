# InvestIQ AI — Project Status Dashboard

> **How to use this file**  
> At the start of every coding session: read this file, update the relevant section after your work, then commit.  
> Keep status honest — prefer “partial” over “done” when something compiles but is not production-ready.

| Field | Value |
|-------|--------|
| **Last updated** | 2026-08-03 |
| **Product phase** | Milestone 1 — Google Auth + Real IPO Tracker |
| **Overall completion** | ~75% toward Play Store MVP |
| **Release readiness** | **~60%** |
| **Default branch** | `main` |
| **Primary stack** | Flutter 3.35 · Rust/Axum · PostgreSQL 16 · Redis 7 · Firebase Auth |
| **IPO data source** | **NSE India public APIs** (`/api/ipo-current-issue`, `/api/public-past-issues`) |

---

## 1. Executive summary

**Milestone 1 (Auth + IPO Tracker) is implemented end-to-end in code:**

- **Google Sign-In** (Firebase Auth + Google provider on Flutter; backend verifies Firebase/Google ID tokens via JWKS; issues app JWT + refresh)
- **Real IPO feed** from NSE (dummy seeds removed; ~1300+ issues synced into Postgres; Redis lock + periodic refresh)
- Email/password auth still available as secondary path

Other modules (portfolio, journal, AI) remain as before — not the focus of this milestone.

**Still not store-ready:** Firebase/Google OAuth credentials must be filled by the operator; Android SDK/signing; hosted privacy policy; FCM push delivery.

---

## 2. Completed features

### Documentation & ops
- [x] Architecture, schema, API, wireframes, roadmap, deploy docs
- [x] **Podman Compose** (`compose.yml`) + dnf install script, CI workflow, `.env.example`
- [x] Migrations: init + mvp_complete + **google_auth** + **real_ipos** (seed removal + external IDs)

### Backend
- [x] Auth: register/login/refresh/logout/me/update, **change-password**, **export**, **delete account**
- [x] **Google/Firebase**: `POST /auth/google` with JWKS verification, user upsert, JWT session
- [x] Suspended/deleted → **403 Forbidden**
- [x] IPO list/detail/watchlist/AI summary + **allotment engine** (pending/allotted/not_allotted)
- [x] **NSE IPO sync** worker + `POST /ipos/sync`; Redis lock `ipo:sync:lock`
- [x] Portfolio holdings with **last_price / prev_close**, price update API, txn avg-cost rollup
- [x] Analytics: total value/cost, **today P&L + %**, unrealized, XIRR, **CAGR**, allocations
- [x] Journal CRUD + analytics + AI mistakes
- [x] AI: remote chat when `AI_API_KEY` set; **local grounded engine** otherwise; remote failure fallback
- [x] Notifications module: list, read, prefs, devices, price alerts, **sync-ipo-events**
- [x] Redis **rate limit** middleware on `/api/v1`
- [x] AES seal on data export when `AES_KEY_BASE64` set
- [x] Structured JSON logging; production JWT/CORS checks
- [x] Unit tests: XIRR, CAGR, allotment, crypto, AI local disclaimer (**10 tests**)

### Mobile
- [x] Material 3 shell, auth, IPO, portfolio, journal, AI, settings
- [x] **Continue with Google** on login/register (Firebase + google_sign_in)
- [x] IPO list: pull-to-refresh (triggers NSE sync), search, filters, loading/error/empty
- [x] IPO detail: real fields; honest “Not available” for lot size / GMP / docs when NSE omits them
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
| Lot size / ₹ issue size / official prospectus URLs from exchange | Not in NSE public IPO JSON |
| Tests ≥ 80% coverage | Not met (~unit core only) |
| Integration tests (HTTP+DB) | Missing |
| iOS TestFlight packaging | Needs macOS |

### Explicitly post-MVP
- Broker import, premium, multi-currency FX, advanced rebalance engine

---

## 4. Known bugs & risks

| Severity | Issue |
|----------|--------|
| High | Google login needs real `FIREBASE_PROJECT_ID` + client IDs; unconfigured → clear server/app errors |
| Medium | NSE may 403 without cookies / block changes; sync fails soft and serves last DB snapshot |
| Medium | Allotment is **indicative** (hash-based), not registrar-authoritative |
| Medium | Today P&L depends on last_price/prev_close (defaults from cost if unset) |
| Medium | Rate limit fail-open if Redis down |
| Medium | Offline queue only covers journal create path extensively |
| Low | Flutter test runner may fail under corporate proxy WebSocket 502 |
| Low | Default CORS `*` only allowed in non-production |
| Low | Delete-account / change-password require password; Google-only accounts get a clear validation error |

---

## 5. Technical debt

1. Handlers still SQL-heavy — extract repositories for larger coverage.  
2. No OpenAPI contract between Flutter and Rust.  
3. Price snapshots table underused vs holdings columns.  
4. FCM: register device only — need worker to send.  
5. Expand widget/integration tests next.

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
| Encryption | Optional AES on export seal |
| AI | No guaranteed returns; disclaimer always |
| GMP | Unofficial labeling retained |

---

## 7. Build status

| Target | Status |
|--------|--------|
| `cargo check` | Pass |
| `cargo test` | Pass (12 tests, incl. NSE parsers) |
| `cargo clippy -D warnings` | Not re-run this session (build clean) |
| `flutter analyze` | **Pass** |
| `flutter test` | Environment-sensitive |
| Android debug/release | Blocked without SDK |
| Web build | Google button present; needs Firebase config |

---

## 8. Test status

| Suite | Est. coverage | Status |
|-------|---------------|--------|
| Rust unit (analytics, allotment, crypto, AI) | Core paths | Pass |
| Rust integration | ~0% | Missing |
| Flutter unit/widget | Low | Minimal |
| **Target ≥ 80%** | Not met | Next priority |

---

## 9. Release readiness

| Gate | Ready? |
|------|--------|
| Four MVP modules usable | **Yes (local full stack)** |
| Real NSE IPO list/detail | **Yes** |
| Google Sign-In E2E | **Code ready; needs Firebase credentials** |
| Backend hardened enough for staging | Mostly |
| Android signed AAB | No |
| Store legal assets | No |
| Tests/CI green for Android | No |
| **Weighted readiness** | **~60%** |

---

## 10. Next priorities

1. **Operator:** create Firebase project + set `FIREBASE_*` / `GOOGLE_CLIENT_IDS` and re-test Google login E2E  
2. Install Android SDK → debug APK + release AAB recipe  
3. Integration tests (Google auth, IPO sync)  
4. Privacy policy page + in-app links  
5. FCM send path for notifications  
6. Staging deploy smoke script  

---

## 11. Session changelog

| Date | Change |
|------|--------|
| 2026-08-03 | Initial scaffold + dashboard |
| 2026-08-03 | **Phase 1 MVP completion:** portfolio calc, allotment, AI local+remote, settings, notifications, errors, offline, clippy clean |
| 2026-08-03 | Switched local ops docs/scripts to **Podman Compose** + **dnf** install helpers |
| 2026-08-03 | Added **docs/10-local-run.md** (full local run guide) and linked from README |
| 2026-08-03 | **Milestone 1:** Google/Firebase auth (`POST /auth/google`), NSE real IPO sync, seed removal, Flutter Google button + IPO UX |

---

## 12. How to test this build

**Canonical guide:** [docs/10-local-run.md](docs/10-local-run.md)

### Infra + API

```bash
cd InvestIQ-AI && cp -n .env.example .env
# Set FIREBASE_PROJECT_ID and GOOGLE_CLIENT_IDS for Google login
./scripts/compose.sh up -d postgres redis
cd backend && cargo run
# curl http://127.0.0.1:8080/health
# curl -X POST http://127.0.0.1:8080/api/v1/ipos/sync
# curl 'http://127.0.0.1:8080/api/v1/ipos?status=open&per_page=5'
```

### Flutter (Chrome)

```bash
export PATH="$HOME/development/flutter/bin:$PATH"
cd mobile
flutter run -d chrome \
  --dart-define=API_BASE_URL=http://127.0.0.1:8080 \
  --dart-define=FIREBASE_API_KEY=... \
  --dart-define=FIREBASE_APP_ID=... \
  --dart-define=FIREBASE_MESSAGING_SENDER_ID=... \
  --dart-define=FIREBASE_PROJECT_ID=... \
  --dart-define=FIREBASE_AUTH_DOMAIN=... \
  --dart-define=GOOGLE_WEB_CLIENT_ID=...
```

**IPO data source:** NSE India public JSON (`ipo-current-issue`, `public-past-issues`), cached in Postgres + Redis sync lock.

Checklist: Google sign-in (or email) → real IPO list → open detail → pull-to-refresh.

---

## 13. Definition of done (MVP store)

- [x] Clippy clean  
- [x] Allotment / portfolio / AI / settings / notifications / offline not stubs  
- [ ] Tests ≥ 80%  
- [ ] Signed AAB + privacy policy  
- [ ] This file **Release readiness ≥ 85%**

---

*End of dashboard.*
