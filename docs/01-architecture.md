# InvestIQ AI — Complete App Architecture

## 1. Product Vision

**InvestIQ AI** is a cross-platform fintech mobile app that unifies IPO discovery, portfolio tracking, trading journaling, and an AI investment assistant — with strong security, offline-first UX, and production-grade backend services.

**Non-negotiables**
- AI never guarantees returns; every AI surface shows an investment disclaimer.
- Grey Market Premium (GMP) is always labeled **unofficial**.
- Secrets never live in client binaries; tokens in secure storage only.
- Clean Architecture + feature modules on both client and server.

---

## 2. High-Level System Context

```
┌─────────────────────────────────────────────────────────────────┐
│                     Flutter App (iOS / Android)                  │
│  Presentation → Domain → Data | Riverpod | Hive/SQLite offline   │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTPS / REST + JWT
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                     API Gateway (Axum)                           │
│  Auth · Rate limit · CORS · Tracing · Request ID                 │
└──────┬──────────────┬──────────────┬──────────────┬─────────────┘
       │              │              │              │
       ▼              ▼              ▼              ▼
  PostgreSQL       Redis         AI Worker       Push (FCM)
  (source of      (cache,        (queue +        APNs via
   truth)          sessions,      LLM client)     provider
                   rate limit)
```

---

## 3. Clean Architecture Layers

### 3.1 Flutter (mobile)

| Layer | Responsibility | Depends on |
|-------|----------------|------------|
| **Presentation** | Screens, widgets, Riverpod UI state | Domain |
| **Domain** | Entities, use cases, repository interfaces | Nothing |
| **Data** | DTOs, API clients, local DB, repository impls | Domain |
| **Core** | Theme, DI, routing, errors, secure storage | — |

### 3.2 Rust backend

| Layer | Responsibility |
|-------|----------------|
| **API (handlers)** | HTTP mapping, validation, auth extraction |
| **Application** | Use cases / services |
| **Domain** | Entities, value objects, domain errors |
| **Infrastructure** | Postgres (sqlx), Redis, JWT, AI providers, email/push |

Feature modules: `auth`, `ipo`, `portfolio`, `journal`, `ai`, `notifications`, `users`, `watchlist`.

---

## 4. Feature Module Map

```
mobile/lib/
├── core/                 # shared: theme, router, network, di, storage
├── features/
│   ├── auth/
│   ├── onboarding/
│   ├── home/
│   ├── ipo/
│   ├── portfolio/
│   ├── journal/
│   ├── ai_assistant/
│   ├── notifications/
│   ├── watchlist/
│   └── settings/
└── main.dart

backend/src/
├── main.rs
├── config.rs
├── error.rs
├── middleware/
├── routes/
├── modules/
│   ├── auth/
│   ├── users/
│   ├── ipo/
│   ├── portfolio/
│   ├── journal/
│   ├── ai/
│   ├── notifications/
│   └── watchlist/
└── infra/
    ├── db.rs
    ├── redis.rs
    ├── jwt.rs
    └── crypto.rs
```

---

## 5. Cross-Cutting Concerns

### Offline support
- Critical reads cached in Hive / SQLite (portfolio snapshots, IPO lists, journal drafts).
- Write queue with conflict policy: last-write-wins for drafts; server truth for balances after sync.
- Connectivity listener triggers sync.

### Security
- TLS everywhere; HSTS at reverse proxy.
- JWT access (short-lived) + refresh (rotated, Redis denylist on logout).
- Passwords: Argon2id.
- Sensitive fields at rest: AES-256-GCM (app-level for journal notes optional; DB encryption at rest via cloud).
- Biometric unlock gates local secure storage (flutter_secure_storage + local_auth).
- Rate limiting: Redis token bucket per IP + per user.
- Input validation with `validator` / serde; SQL via parameterized sqlx.

### Observability
- Structured tracing (`tracing` + OpenTelemetry optional).
- Health `/health`, readiness `/ready`.
- Metrics: Prometheus scrape endpoint (optional phase 2).

### AI safety
- System prompt enforces no guaranteed returns, no personalized financial advice as “surety”.
- Response post-filter for prohibited phrases.
- Disclaimer banner on all AI chat UIs and API `disclaimer` field.

---

## 6. Data Flow Examples

### IPO list (cached)
1. Client → `GET /api/v1/ipos?status=open`
2. API checks Redis key `ipos:open:v1`
3. Miss → Postgres → cache 5–15 min → response
4. Client stores in local cache for offline

### Portfolio XIRR
1. Client loads holdings + transactions from local DB if offline
2. Online: `GET /api/v1/portfolio/analytics`
3. Server computes XIRR/CAGR from cash flows (transactions)
4. Returns metrics + series for charts

### AI chat
1. Client `POST /api/v1/ai/chat` with message + context refs (portfolio_id, ipo_id)
2. Server assembles grounded context from DB (never invents holdings)
3. LLM call with safety system prompt
4. Stream or JSON reply + disclaimer

---

## 7. State Management (Flutter)

**Riverpod 2** (codegen optional):
- `AsyncNotifier` for remote features
- `Notifier` for UI-only state
- `Provider` for pure deps (ApiClient, repositories)
- Feature-scoped providers keep rebuilds local

Persistence:
- `flutter_secure_storage` — tokens, biometric gate
- `hive` / `isar` — offline cache
- `shared_preferences` — theme, currency, language flags

---

## 8. Navigation

**go_router** with shell route for bottom nav:

| Tab | Routes |
|-----|--------|
| Home | `/`, dashboard widgets |
| IPOs | `/ipos`, `/ipos/:id`, calendar, allotment |
| Portfolio | `/portfolio`, holdings, transactions, analytics |
| Journal | `/journal`, entry, calendar, reports |
| AI | `/ai` chat |
| Settings | `/settings/*` (drawer or profile) |

Auth guard: redirect unauthenticated users to `/login`.

---

## 9. Dependency Injection

- **Flutter**: Riverpod providers as composition root.
- **Rust**: `AppState` Arc with pool, redis, config, services; extract via Axum `State`.

---

## 10. Deployment Topology

```
Internet → CDN/WAF → Nginx/Caddy → Axum (n replicas)
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
           Postgres    Redis     Object storage
           (primary)   (cache)   (screenshots)
```

Docker Compose for local/staging; Kubernetes optional for production scale.

---

## 11. Quality Gates

- Unit tests: domain math (XIRR, P&L), auth
- Integration: sqlx tests against Postgres
- Widget/golden tests: key screens
- API contract tests
- CI: lint, test, build, Docker image, security scan
