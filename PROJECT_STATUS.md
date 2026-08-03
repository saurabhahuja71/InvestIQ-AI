# InvestIQ AI — Project Status Dashboard

> **How to use this file**  
> At the start of every coding session: read this file, update the relevant section after your work, then commit.  
> Keep status honest — prefer “partial” over “done” when something compiles but is not production-ready.

| Field | Value |
|-------|--------|
| **Last updated** | 2026-08-03 |
| **Product phase** | Phase 1 — MVP hardening (four modules) |
| **Overall completion** | ~45% toward Play Store MVP |
| **Release readiness** | **~35%** |
| **Default branch** | `main` |
| **Primary stack** | Flutter 3.44 · Rust/Axum · PostgreSQL 16 · Redis 7 |

---

## 1. Executive summary

InvestIQ AI is a monorepo with architecture docs, a compiling Rust API skeleton, and a Flutter app shell covering the four MVP modules (IPO, Portfolio, Journal, AI Chat).  

It is **not** yet ready for Play Store / production at 1M-user scale. Core APIs and UI exist as an integrated scaffold; security hardening, Android release tooling, comprehensive tests, notifications, and store compliance assets are incomplete.

**MVP product promise (unchanged):**  
IPO Tracker · Portfolio Tracker · Trading Journal · AI Chat — then broker/advanced/premium later.

---

## 2. Completed features

### Documentation & ops scaffolding
- [x] Architecture, DB schema, API design, wireframes, roadmap, deployment docs (`docs/`)
- [x] Docker Compose for Postgres + Redis + API image
- [x] GitHub Actions CI workflow (backend + Flutter + Docker) — may fail until warnings/tests/Android SDK fixed
- [x] `.env.example`, root README, Flutter/Rust structure docs
- [x] SQL init migration with enums, tables, seed IPOs

### Backend (Rust / Axum) — scaffold compiles
- [x] Config from env, structured errors, health `/health` + readiness `/ready`
- [x] JWT issue/decode (access + refresh concept)
- [x] Argon2 password hashing
- [x] Auth: register, login, refresh, logout, me, profile update
- [x] Default portfolio created on register
- [x] IPO: list/filter/search, detail, watchlist, AI summary, allotment stub endpoint
- [x] Portfolio: list/create, holdings, transactions, analytics (allocation, XIRR helper), AI review
- [x] Journal: trade CRUD (soft delete), analytics, AI mistakes endpoint
- [x] AI: conversations, chat with disclaimer + safety system prompt; stub without API key
- [x] AES-GCM helper, optional rate-limit middleware file (wiring may be incomplete)
- [x] Unit test for XIRR; crypto unit tests present in source

### Mobile (Flutter) — analyze clean
- [x] Project platforms: android / ios / web generated
- [x] Material 3 theme (light/dark/system), glass cards, bottom nav shell
- [x] Auth UI: login, register, secure token storage, JWT refresh interceptor
- [x] Home, IPO list/detail, Portfolio, Journal + trade entry, AI chat, Settings
- [x] GMP + investment disclaimer copy in UI
- [x] `flutter analyze` — no issues (last check)
- [x] Web release build previously succeeded

---

## 3. Missing features (MVP blockers & deferred)

### MVP blockers (must finish before store)
| Item | Status | Notes |
|------|--------|--------|
| End-to-end API verification against real Postgres/Redis | Missing | Full stack not proven in CI with green clippy |
| Clippy clean (`-D warnings`) | Failing | Dead-code / unused infra still warns as errors under `-D warnings` |
| Rate limiting fully wired on all public routes | Partial | Middleware exists; confirm attached in router/main |
| Account deletion API + UI | Missing | Play / privacy requirement |
| Data export | Stub only | Privacy / user rights |
| Privacy policy & Terms hosted URLs | Missing | Store listing |
| Production CORS (not `Any`) | Missing | Security |
| HTTPS-only production config + HSTS | Missing | Deployment |
| Android SDK + signed release AAB | Missing | No Android SDK on build host |
| Play signing, ProGuard/R8 review | Missing | |
| Push notifications | Out of MVP? / Missing | Phase 7 |
| Comprehensive unit + integration + widget tests | Missing | Far below 80% coverage |
| Offline sync | Partial | Hive box only initialized |
| Live market prices | Deferred | Portfolio mark-to-cost only |
| Broker import | Deferred | Post-MVP |
| Premium / paywall | Deferred | Post-MVP |

### Deferred (post-MVP, do not block launch)
- Broker integrations  
- Advanced analytics UI / rebalancing  
- Full allotment partner API  
- News alerts, multi-currency FX  
- iOS App Store packaging (macOS required)

---

## 4. Known bugs & risks

| Severity | Issue | Impact |
|----------|--------|--------|
| High | Android toolchain missing on CI/dev host | Cannot produce debug/release APK/AAB |
| High | `cargo clippy -- -D warnings` fails | CI “strict” quality gate broken |
| High | Default JWT secret in env example / code fallback | Unsafe if deployed without override |
| High | CORS allows any origin | CSRF/token theft risk in browser clients |
| Medium | Allotment check always returns unknown | Misleading if presented as final |
| Medium | Portfolio “today P&L” is zero without prices | Incorrect user expectation |
| Medium | AI stub / `unwrap` on API key path after empty check | Edge-case panics if logic drifts |
| Medium | Flutter `flutter test` failed with WebSocket 502 (proxy) | Unreliable local test runner |
| Medium | Rate limit fail-open on Redis errors | Abuse possible during Redis outage |
| Low | `DropdownButtonFormField` `initialValue` vs stateful updates | Emotion/side may not rebind as expected on rebuild |
| Low | No request body size limits / strict validation everywhere | DoS / junk data risk |

---

## 5. Technical debt

1. **Dead / half-wired security infra** — AES cipher, rate limit, config fields not consistently used → clippy noise and false sense of completeness.  
2. **No shared domain layer on Flutter** — maps of `dynamic` JSON instead of typed models + repositories under test.  
3. **Handlers are large** — auth/ipo/portfolio/journal mix SQL + HTTP; need repository split for testability.  
4. **Single fat migration** — expand/contract migrations for production evolve-ability.  
5. **No OpenAPI / contract tests** — Flutter and Rust can drift.  
6. **Logging** — basic tracing only; no request metrics, no PII redaction policy.  
7. **Secrets** — need secrets manager path (no long-lived secrets in env files in prod).  
8. **Feature branch discipline** — process requested; not yet enforced by docs/automation.  
9. **PROJECT_STATUS** (this file) — must be updated every session (new process).  
10. **Incomplete production refactor** — rate_limit module may not be fully integrated; security audit/test/release prep work was requested but not fully landed.

---

## 6. Security status (audit snapshot)

| Area | Status | Notes |
|------|--------|--------|
| JWT | Partial | Access/refresh present; need short TTL enforcement in prod, refresh rotation verified, denylist optional |
| SQL injection | Good baseline | sqlx parameterized queries |
| XSS | N/A mobile-primary | Web build: avoid raw HTML; Flutter Text widgets default-safe |
| Authentication | Partial | Argon2 + JWT; need lockout, email verify optional, biometric fully wired |
| Authorization | Partial | Resource ownership checks on portfolio/journal; audit all routes |
| Rate limiting | Partial | Implementation exists; must be on auth routes especially |
| HTTPS | Deploy concern | Not enforced in app code; reverse proxy required |
| Secret management | Weak | `.env` pattern only; change JWT_SECRET before any deploy |
| Encryption | Partial | AES helper optional; TLS + disk encryption at infra still required |
| Password policy | Minimal | Length ≥ 8 only |
| GMP / AI disclaimers | Present | Keep non-negotiable for compliance |

**Security release bar for 1M users (not met):**  
WAF, rate limits on auth, secret rotation, no open CORS, audit logs, backup/restore drills, dependency scanning, pen test.

---

## 7. Build status

| Target | Status | Last known |
|--------|--------|------------|
| `cargo check` | Pass (with warnings) | 2026-08-03 |
| `cargo test` | Pass (1 XIRR test + crypto tests if compiled) | Minimal coverage |
| `cargo clippy -D warnings` | **Fail** | Dead code errors |
| `flutter analyze` | **Pass** | Clean |
| `flutter test` | **Unreliable / fail** | Proxy WebSocket 502 environment |
| `flutter build web` | Pass (historical) | |
| `flutter build apk` (debug) | **Blocked** | No Android SDK |
| `flutter build appbundle` (release) | **Blocked** | No Android SDK + signing |
| Docker API image | Defined | Not continuously verified |

---

## 8. Test status

| Suite | Coverage (est.) | Status |
|-------|-----------------|--------|
| Rust unit | &lt; 10% | XIRR + crypto only |
| Rust integration (HTTP + DB) | ~0% | Not present |
| Flutter unit | ~0% | Trivial disclaimer test only |
| Flutter widget | ~0% | Not present |
| E2E | 0% | Not present |
| **Target** | **≥ 80%** | **Not met** |

---

## 9. Release readiness (Play Store)

| Gate | Ready? | Weight |
|------|--------|--------|
| Four MVP modules usable end-to-end | Partial | 25% |
| Backend production-hardened | No | 20% |
| Android signed AAB | No | 15% |
| Store listing + privacy policy | No | 10% |
| Tests ≥ 80% / CI green | No | 15% |
| Security bar for public launch | No | 15% |
| **Weighted readiness** | | **~35%** |

---

## 10. Next priorities (ordered)

Work **only** on the next item until green; update this file after each.

### P0 — Make the tree trustworthy (current focus)
1. Fix all `cargo clippy --all-targets -- -D warnings` issues; wire or remove dead code.  
2. Wire rate limiting + production CORS + reject default JWT secret in non-dev.  
3. Install/configure Android SDK; green `flutter build apk --debug` and release AAB recipe.  
4. Add Rust integration tests (auth + one IPO + portfolio ownership).  
5. Add Flutter unit/widget tests for auth and IPO list.  
6. Account deletion + privacy policy template + store listing docs.  
7. Update CI so main stays green.

### P1 — MVP feature completion
8. Harden auth UX and session edge cases.  
9. IPO data path reliability + empty/error states.  
10. Portfolio CRUD consistency + honest “no live prices” copy.  
11. Journal edit/delete UX + analytics accuracy.  
12. AI chat grounding + disclaimer always visible.

### P2 — Store submission
13. Screenshots, description, content rating, data safety form.  
14. Staging deploy + smoke test.  
15. Closed testing track → production rollout.

### P3 — Post-MVP
16. Notifications, live prices, broker import, advanced analytics, premium.

---

## 11. Session changelog

| Date | Change | Author |
|------|--------|--------|
| 2026-08-03 | Initial monorepo, docs, backend modules, Flutter shell, MVP roadmap | Build session |
| 2026-08-03 | Flutter platforms + analyze clean; web build | Build session |
| 2026-08-03 | Created `PROJECT_STATUS.md` dashboard | Build session |

---

## 12. Commands cheat sheet

```bash
# Backend
cd backend && cargo check && cargo test && cargo clippy --all-targets -- -D warnings

# Mobile
export PATH="$HOME/development/flutter/bin:$PATH"
cd mobile && flutter analyze && flutter test
# After Android SDK:
flutter build apk --debug
flutter build appbundle --release

# Infra
docker compose up -d postgres redis
```

---

## 13. Definition of “done” for next release (MVP)

- [ ] Clippy clean, tests ≥ 80% on critical packages  
- [ ] Auth + 4 modules demo script passes on staging  
- [ ] No default secrets; HTTPS; CORS locked  
- [ ] Account delete + privacy policy URL  
- [ ] Signed AAB uploaded to Play Console internal testing  
- [ ] Crash-free and disclaimer/GMP compliance reviewed  
- [ ] This file shows **Release readiness ≥ 85%**

---

*End of dashboard. Update before and after every coding session.*
