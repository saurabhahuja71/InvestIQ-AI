# InvestIQ AI — Project Status Dashboard

> **How to use this file**  
> At the start of every coding session: read this file, update the relevant section after your work, then commit.  
> Keep status honest — prefer “partial” over “done” when something compiles but is not production-ready.

| Field | Value |
|-------|--------|
| **Last updated** | 2026-08-08 |
| **Product phase** | Milestone 4 — IPO Intelligence (research gate) — M1/M2/M3 done |
| **Overall completion** | ~78% toward Play Store MVP |
| **Release readiness** | **~65%** |
| **Default branch** | `main` |
| **Primary stack** | Flutter 3.35 · Rust/Axum · PostgreSQL 16 · Redis 7 · Firebase Auth |
| **IPO data source** | **NSE India public APIs** (`/api/ipo-current-issue`, `/api/public-past-issues`) |

---

## 1. Executive summary

**Milestones 1–3 are in code:**

- **M1 — Google Sign-In** (Firebase Auth + Google provider; backend JWKS → app JWT)
- **M2 — Production IPO Tracker** on live NSE data (sync, cache, list/detail UX)
- **M3 — Watchlist & IPO Alerts** (Postgres-backed multi-device watchlist, configurable IPO event alerts, offline Hive cache, Material 3 star UI + badge)

Other modules (portfolio, journal, AI) remain as before — **not** modified for M3.

**Still not store-ready:** Firebase/Google OAuth credentials; Android SDK/signing; hosted privacy policy; **FCM push delivery** (in-app notifications only for IPO alerts).

**Milestone 4 (IPO Intelligence) is at a research gate:** backend scaffolding (subscription history, GMP, financials, score endpoints) exists but **provider integration is deliberately frozen** until provider selection is approved. Research completed → `docs/IPO_DATA_PROVIDER_RESEARCH.md`. **Confirmed (2026-08-08): IPO Guru free API** (300 req/day, 15/min) covers IPO details + subscription + GMP at ₹0 with commercial use + attribution — selected as the launch primary for the free app. Key compliance note: NSE public feeds are **not** compliant for a commercial Play Store app without a formal NSE Data & Analytics license; GMP must come from the unofficial provider (IPO Guru); financials primary source is the official RHP.

---

## 2. Completed features

### Documentation & ops
- [x] Architecture, schema, API, wireframes, roadmap, deploy docs
- [x] **Podman Compose** (`compose.yml`) + dnf install script, CI workflow, `.env.example`
- [x] Migrations: init + MVP + **google_auth** + **`20240805000000_ipo_live_source`** (drop seeds, NSE sync columns) + **`20250808000000_ipo_intelligence`** (M4 subscription/GMP/financials/score tables)
- [x] **IPO data provider doc** (`docs/11-ipo-data-provider.md`)
- [x] **Milestone 4 provider research** (`docs/IPO_DATA_PROVIDER_RESEARCH.md`) — IPO Guru / IPOAlerts / IPONotify + official NSE/BSE/SEBI/registrar sources; comparison table; dev/production/fallback recommendations; proposed architecture. **Integration pending approval.**
- [x] **API design** updated for `/watchlist` and `/alerts` (Milestone 3)
- [x] **Offline container images** on GitHub Release `container-images-v1` + load/export scripts

### Backend
- [x] Auth: register/login/refresh/logout/me/update, **change-password**, **export**, **delete account**
- [x] **Google/Firebase**: `POST /auth/google` with JWKS verification, user upsert, JWT session
- [x] Suspended/deleted → **403 Forbidden**
- [x] **NSE IPO sync** (`IpoSyncService`): boot + periodic refresh, Redis lock, list cache TTL
- [x] `GET /ipos` filters: status, board, q, page, per_page, refresh; `POST /ipos/sync`
- [x] IPO detail: dates, band, lot, min investment, registrar, lead managers, subscription, RHP URL, ratios link
- [x] **Watchlist (M3):** `GET/POST /watchlist`, `DELETE /watchlist/{ipo_id}` — authz to current user; Postgres `ipo_watchlist`
- [x] **Alerts (M3):** `GET /alerts`, `GET|PUT /alerts/preferences`, `POST /alerts/sync`
- [x] Alert evaluation for **watched IPOs only**: open, closes today, allotment, listing tomorrow, listing today (prefs toggles)
- [x] Legacy aliases: `GET /ipos/watchlist`, `POST|DELETE /ipos/{id}/watch`, `POST /notifications/sync-ipo-events`
- [x] Portfolio / journal / AI / price alerts unchanged
- [x] Redis **rate limit** middleware on `/api/v1`
- [x] Unit tests: XIRR, CAGR, allotment, crypto, AI, NSE parsers, **IPO alert logic**, **M4 score + parsers** (**37 tests**)

### Milestone 4 backend (scaffolded, integration frozen)
- [x] Migration `20250808000000_ipo_intelligence`: subscription snapshots + history, GMP + history, financials, score cache, `data_sources` metadata (empty tables are the honest default)
- [x] `ipo_intel` module: subscription/history, gmp/history, financials, score endpoints; transparent scoring methodology (`logic.rs`) + tests
- [x] NSE sync captures **official subscription bid-details** to daily history + latest snapshot (skips all-NULL rows)
- [x] `investment_requirements` on IPO detail (lot × price; NSE max caps; SEBI ₹2L NII floor as labelled regulatory constant)
- [x] `/data-sources` endpoint with per-IPO freshness timestamps
- [ ] **Provider integration — frozen pending approval** (see `docs/IPO_DATA_PROVIDER_RESEARCH.md`)

### Mobile
- [x] Material 3 shell, auth, IPO, portfolio, journal, AI, settings
- [x] **Continue with Google** on login/register
- [x] IPO tabs + board filter + search + pull-to-refresh + pagination + skeleton/empty/error
- [x] **★ on IPO cards / detail** — add/remove watchlist
- [x] **Watchlist page** (`/watchlist`): name, status, open/close/listing dates, subscription; offline cache banner
- [x] **Badge** on IPO Tracker app bar with watched count
- [x] **IPO alert settings** page (enable/disable each alert type) → `PUT /alerts/preferences`
- [x] **IPO alerts inbox** → `GET /alerts` (+ sync)
- [x] Offline: Hive cache for watchlist + alert prefs; offline write queue for watchlist mutations
- [x] Settings links: watchlist, alert settings, alerts inbox
- [x] `flutter analyze` clean on changed modules

---

## 3. Missing features

### Remaining for Play Store / ops
| Item | Status |
|------|--------|
| **Firebase project + OAuth client IDs in `.env` / dart-defines** | **Required for live Google login** |
| Android SDK + signed AAB | Missing on this host |
| Hosted privacy policy / terms URLs | Missing |
| Real FCM push send (device tokens stored only) | Partial — M3 uses **in-app** notifications |
| Live equity price feed (portfolio MTM) | Deferred |
| Tests ≥ 80% coverage | Not met (~unit core only) |
| Integration tests (HTTP+DB) | Missing |
| iOS TestFlight packaging | Needs macOS |

### Explicitly post-MVP
- Broker import, premium, multi-currency FX
- Official registrar allotment APIs
- Paid IPO data vendors for GMP / logos / financials
- Background server-wide alert worker (today: on-demand `/alerts/sync` + client open of inbox)

---

## 4. Known bugs & risks

| Severity | Issue |
|----------|--------|
| High | Google login needs real `FIREBASE_PROJECT_ID` + client IDs |
| High | **NSE public feeds not compliant for commercial Play Store app** without NSE Data & Analytics license or a licensed aggregator (see `docs/IPO_DATA_PROVIDER_RESEARCH.md`) |
| Medium | NSE may 403; sync fails soft and serves last DB snapshot |
| Medium | Allotment engine is **indicative**, not registrar-authoritative |
| Medium | Logo, website, industry, description, structured financials, GMP often **Not Available** from NSE |
| Medium | IPO alerts are **in-app only** until FCM send is wired; require user/session to call sync |
| Medium | Alerts only fire for **watchlist** IPOs (by design for M3) |
| Low | Company names on some upcoming rows may stay symbol-like until detail enrich |
| Low | Flutter test runner may fail under corporate proxy WebSocket 502 |
| Low | Default CORS `*` only allowed in non-production |

---

## 5. Technical debt

1. Handlers still SQL-heavy — extract repositories for larger coverage.  
2. No OpenAPI contract between Flutter and Rust.  
3. Price snapshots table underused vs holdings columns.  
4. FCM: register device only — need worker to send push.  
5. Optional: cron/worker to run watchlist alert evaluation without client open.  
6. Expand widget/integration tests (Flutter env may block `flutter test`).

---

## 6. Security status

| Area | Status |
|------|--------|
| JWT | Access + rotating refresh; production secret enforced |
| SQL | sqlx parameterized |
| AuthN/Z | Ownership checks on watchlist / alerts / notifications |
| Rate limit | Redis fixed window on API |
| CORS | Configurable; `*` blocked in production |
| Secrets | `.env` / env vars; no secrets in git |
| IPO source | NSE public feeds; polite pacing; no fabricated market data |

---

## 7. Build status

| Target | Status |
|--------|--------|
| `cargo check` | Pass |
| `cargo test` | Pass (**37 tests**) |
| `flutter analyze` (M3 modules) | **Pass** |
| `flutter test` | Environment-sensitive (proxy WebSocket 502 on this host) |
| Android debug/release | Blocked without SDK |

---

## 8. Test status

| Suite | Est. coverage | Status |
|-------|---------------|--------|
| Rust unit (analytics, allotment, crypto, AI, NSE, **alert logic**, **M4 score/parsers**) | Core paths | Pass (37) |
| Manual IPO API smoke (2026-08-05) | List/search/detail/sync | Pass |
| Milestone 3 unit (alert evaluate + prefs merge) | Pure logic | Pass |
| Milestone 4 unit (score, investment requirements, GMP parsing) | Pure logic | Pass |
| Rust integration | ~0% | Missing |
| Flutter unit/widget | Low | Env-blocked on host |
| **Target ≥ 80%** | Not met | Next priority |

### Milestone 3 checklist

- [x] Watchlist add/remove/list (Postgres, auth)
- [x] Multi-device sync via server state
- [x] Offline Hive cache + write queue for watchlist
- [x] Alert prefs save (`ipo_open`, `ipo_close`, `allotment`, `listing_tomorrow`, `listing_day`)
- [x] Alert evaluation for watched IPOs (pure logic unit-tested)
- [x] UI: ★ cards, watchlist page, badge, settings
- [ ] End-to-end HTTP+DB integration test (not automated yet)
- [ ] FCM push delivery (deferred)

---

## 9. Release readiness

| Gate | Ready? |
|------|--------|
| IPO Tracker production data path | **Yes (NSE-backed)** ⚠️ NSE license posture TBD for commercial store |
| Watchlist + IPO alerts | **Yes (in-app)** |
| Milestone 4 data providers | **Research done — integration frozen pending approval** |
| Four MVP modules usable | **Yes (local full stack)** |
| Google Sign-In E2E | **Code ready; needs Firebase credentials** |
| Backend hardened enough for staging | Mostly |
| Android signed AAB | No |
| Store legal assets | No |
| **Weighted readiness** | **~65%** |

---

## 10. Next priorities

1. **Operator:** Firebase project + secrets for Google login E2E  
2. Install Android SDK → debug APK + release AAB  
3. **Milestone 4 decision:** ✅ IPO Guru free confirmed as launch provider (GMP + subscription + IPO details, commercial use + attribution). Remaining: **NSE posture** (license vs aggregator), **financials path** (RHP ingestion vs defer), written IPO Guru license confirmation — per `docs/IPO_DATA_PROVIDER_RESEARCH.md`  
4. Integration tests (auth, watchlist, alerts/sync)  
5. FCM send path for real push on IPO events  
6. Privacy policy page + in-app links  
7. Staging deploy smoke script  

---

## 11. Session changelog

| Date | Change |
|------|--------|
| 2026-08-03 | Initial scaffold + dashboard |
| 2026-08-03 | **Phase 1 MVP completion:** portfolio calc, allotment, AI local+remote, settings, notifications, errors, offline, clippy clean |
| 2026-08-03 | Switched local ops docs/scripts to **Podman Compose** + **dnf** install helpers |
| 2026-08-03 | Added **docs/10-local-run.md** (full local run guide) and linked from README |
| 2026-08-03 | **Milestone 1:** Google/Firebase auth, NSE real IPO sync, seed removal, Flutter Google button + IPO UX |
| 2026-08-03 | **Offline images:** Postgres 16 + Redis 7 on GitHub Release `container-images-v1` |
| 2026-08-03 | **Google Auth complete (code):** FlutterFire scaffolding; blocked on operator Firebase secrets |
| 2026-08-05 | **Milestone 2:** live NSE IPO sync, Redis list cache, production Flutter IPO list/detail, provider docs |
| 2026-08-06 | **Milestone 3:** `/watchlist` + `/alerts` APIs, watchlist-scoped IPO alert evaluation, Flutter star/badge/watchlist page/alert prefs, offline cache |
| 2026-08-08 | **Milestone 4 scaffolding:** subscription/GMP/financials/score endpoints + scoring logic, `investment_requirements`, `data-sources` freshness, 37 tests. **Provider research done** (`docs/IPO_DATA_PROVIDER_RESEARCH.md`); integration frozen pending provider + NSE-license approval |

---

## 12. How to test this build

**Canonical guide:** [docs/10-local-run.md](docs/10-local-run.md)  
**IPO source:** [docs/11-ipo-data-provider.md](docs/11-ipo-data-provider.md)  
**API surface:** [docs/03-api-design.md](docs/03-api-design.md)

### Milestone 3 API smoke

```bash
# after login → ACCESS token
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:8080/api/v1/watchlist
curl -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"ipo_id":"<UUID>"}' http://127.0.0.1:8080/api/v1/watchlist
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:8080/api/v1/watchlist/<UUID>
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:8080/api/v1/alerts/preferences
curl -X PUT -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"preferences":{"ipo_open":true,"ipo_close":true,"allotment":true,"listing_tomorrow":true,"listing_day":false}}' \
  http://127.0.0.1:8080/api/v1/alerts/preferences
curl -X POST -H "Authorization: Bearer $TOKEN" http://127.0.0.1:8080/api/v1/alerts/sync
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:8080/api/v1/alerts
```

### Flutter checklist (M3)

1. Sign in → **IPOs** → tap ★ on a card → badge increments.  
2. Open **Watchlist** (star icon in app bar or Home chip) → fields + pull-to-refresh.  
3. **Settings → IPO alert settings** → toggle prefs → Save.  
4. Open **IPO alerts inbox** → sync creates events when watchlist dates match today/tomorrow.  
5. Airplane mode → watchlist still shows Hive cache; mutations queue until online.

---

## 13. Definition of done (MVP store)

- [x] Clippy / analyze clean for current modules  
- [x] IPO Tracker on live exchange data (no dummy seeds)  
- [x] Watchlist + configurable IPO alerts (in-app)  
- [ ] Tests ≥ 80%  
- [ ] Signed AAB + privacy policy  
- [ ] This file **Release readiness ≥ 85%**

---

*End of dashboard.*
