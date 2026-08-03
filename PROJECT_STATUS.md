# InvestIQ AI — Project Status Dashboard

> **How to use this file**  
> At the start of every coding session: read this file, update the relevant section after your work, then commit.  
> Keep status honest — prefer “partial” over “done” when something compiles but is not production-ready.

| Field | Value |
|-------|--------|
| **Last updated** | 2026-08-03 |
| **Product phase** | Phase 1 MVP — unfinished architecture items completed |
| **Overall completion** | ~70% toward Play Store MVP |
| **Release readiness** | **~55%** |
| **Default branch** | `main` |
| **Primary stack** | Flutter 3.44 · Rust/Axum · PostgreSQL 16 · Redis 7 |

---

## 1. Executive summary

InvestIQ AI monorepo now implements **working** versions of previously stubbed MVP surfaces:

- Portfolio mark-to-market math (value, cost, today P&L, unrealized, XIRR, CAGR)
- Deterministic allotment status (indicative, registrar disclaimer)
- AI local educational engine + remote LLM with fallback
- Settings: currency, biometric flag, change password, export JSON, delete account
- Notifications: inbox, prefs, devices, price alerts, IPO event sync
- Error mapping + offline Hive cache + write queue sync
- Rate limiting wired; production CORS/JWT secret guards

**Still not store-ready:** Android SDK/signing, deep automated test coverage, hosted privacy policy, FCM push delivery to devices.

---

## 2. Completed features

### Documentation & ops
- [x] Architecture, schema, API, wireframes, roadmap, deploy docs
- [x] **Podman Compose** (`compose.yml`) + dnf install script, CI workflow, `.env.example`
- [x] Migrations: init + `20240102000000_mvp_complete` (holding prices, export jobs)

### Backend
- [x] Auth: register/login/refresh/logout/me/update, **change-password**, **export**, **delete account**
- [x] Suspended/deleted → **403 Forbidden**
- [x] IPO list/detail/watchlist/AI summary + **allotment engine** (pending/allotted/not_allotted)
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
- [x] Settings wired to backend (currency, biometric, password, export clipboard, delete)
- [x] Notifications inbox + prefs screens
- [x] Allotment form (PAN last4 + application number)
- [x] Portfolio shows XIRR/CAGR/today/unrealized
- [x] Offline: Hive cache for IPOs/portfolios; journal write queue; connectivity flush
- [x] Global error widget + `AppException` mapping
- [x] `flutter analyze` clean

---

## 3. Missing features

### Remaining for Play Store
| Item | Status |
|------|--------|
| Android SDK + signed AAB | Missing on this host |
| Hosted privacy policy / terms URLs | Missing |
| Real FCM push send (device tokens stored only) | Partial |
| Live exchange price feed | Deferred (manual last_price works) |
| Tests ≥ 80% coverage | Not met (~unit core only) |
| Integration tests (HTTP+DB) | Missing |
| iOS TestFlight packaging | Needs macOS |

### Explicitly post-MVP
- Broker import, premium, multi-currency FX, advanced rebalance engine

---

## 4. Known bugs & risks

| Severity | Issue |
|----------|--------|
| Medium | Allotment is **indicative** (hash-based), not registrar-authoritative |
| Medium | Today P&L depends on last_price/prev_close (defaults from cost if unset) |
| Medium | Rate limit fail-open if Redis down |
| Medium | Offline queue only covers journal create path extensively |
| Low | Flutter test runner may fail under corporate proxy WebSocket 502 |
| Low | Default CORS `*` only allowed in non-production |

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
| `cargo test` | Pass (10 tests) |
| `cargo clippy -D warnings` | **Pass** |
| `flutter analyze` | **Pass** |
| `flutter test` | Environment-sensitive |
| Android debug/release | Blocked without SDK |
| Web build | Previously pass |

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
| Backend hardened enough for staging | Mostly |
| Android signed AAB | No |
| Store legal assets | No |
| Tests/CI green for Android | No |
| **Weighted readiness** | **~55%** |

---

## 10. Next priorities

1. Install Android SDK → debug APK + release AAB recipe  
2. Integration tests (auth, IPO, portfolio ownership)  
3. Flutter widget tests (login, IPO list)  
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

---

## 12. How to test this build

**Canonical guide:** [docs/10-local-run.md](docs/10-local-run.md)

```bash
cd InvestIQ-AI && cp -n .env.example .env
./scripts/install-deps-dnf.sh              # once (dnf + Podman)
./scripts/compose.sh up -d postgres redis  # Podman Compose
cd backend && cargo run

export PATH="$HOME/development/flutter/bin:$PATH"
cd mobile
flutter run -d chrome --dart-define=API_BASE_URL=http://127.0.0.1:8080
```

Checklist: register → IPOs + allotment form → add holding (see today P&L) → journal trade → AI chat → settings export → notifications sync.

---

## 13. Definition of done (MVP store)

- [x] Clippy clean  
- [x] Allotment / portfolio / AI / settings / notifications / offline not stubs  
- [ ] Tests ≥ 80%  
- [ ] Signed AAB + privacy policy  
- [ ] This file **Release readiness ≥ 85%**

---

*End of dashboard.*
