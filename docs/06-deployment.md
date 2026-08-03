# Deployment Guide

## Prerequisites

### Local / self-hosted (RHEL, Oracle Linux, Fedora)
- **Podman** + **Podman Compose** (`dnf install podman podman-compose`)
- See full package list: [09-podman-dnf-setup.md](09-podman-dnf-setup.md)
- Domain + TLS certificates (or Caddy automatic HTTPS)
- Managed PostgreSQL 16+ and Redis 7+ (or Compose for staging)
- FCM project for push (optional for MVP)
- LLM API key stored in secrets manager (optional; local AI engine works without it)

### Install host deps

```bash
./scripts/install-deps-dnf.sh
```

---

## Local development

**Full guide:** [10-local-run.md](10-local-run.md)

```bash
# From repo root
cp .env.example .env
./scripts/compose.sh up -d postgres redis
# or: podman compose -f compose.yml up -d postgres redis

cd backend && cargo run
# Mobile
export PATH="$HOME/development/flutter/bin:$PATH"
cd mobile && flutter pub get && flutter run -d chrome --dart-define=API_BASE_URL=http://127.0.0.1:8080
```

API: `http://localhost:8080`  
Postgres: `localhost:5432`  
Redis: `localhost:6379`

---

## Staging / production with Podman Compose

```bash
# Prefer compose.yml
podman compose -f compose.yml up -d --build

# Or wrapper
./scripts/compose.sh up -d --build
```

Services: `api`, `postgres`, `redis`.

For production:
- Set `APP_ENV=production`
- Set a strong `JWT_SECRET` (≥32 chars)
- Set explicit `CORS_ORIGINS` (not `*`)
- Put Postgres/Redis on private network; expose only reverse proxy → API

---

## Environment variables

| Variable | Description |
|----------|-------------|
| `APP_ENV` | `development` \| `production` |
| `DATABASE_URL` | Postgres connection string |
| `REDIS_URL` | Redis URL |
| `JWT_SECRET` | ≥32 byte secret (required in production) |
| `JWT_ACCESS_TTL_SECS` | default 900 |
| `JWT_REFRESH_TTL_SECS` | default 2592000 |
| `AES_KEY_BASE64` | 32-byte key base64 (optional field encryption) |
| `AI_API_KEY` | LLM provider key (optional) |
| `AI_BASE_URL` | Provider base URL |
| `AI_MODEL` | Model name |
| `RUST_LOG` | `info,tower_http=info` |
| `CORS_ORIGINS` | Comma-separated origins |
| `RATE_LIMIT_RPS` | e.g. 30 |

---

## Migrations

```bash
cd backend
# Automatic on API startup via sqlx::migrate!
# Or with sqlx-cli if installed:
# sqlx migrate run
```

---

## Health checks
- `GET /health` — process up
- `GET /ready` — DB + Redis reachable

```bash
curl -s http://localhost:8080/health
curl -s http://localhost:8080/ready
```

---

## Useful Podman commands

```bash
./scripts/compose.sh ps
./scripts/compose.sh logs -f api
./scripts/compose.sh exec postgres psql -U investiq -d investiq
./scripts/compose.sh down
podman volume ls
```

---

## Backups
- Postgres: daily `pg_dump` to object storage, retain 30 days
- Redis: AOF optional (cache-only OK to lose)
- Test restore quarterly

## Rollback
1. Redeploy previous image tag
2. Avoid destructive migrations without expand/contract pattern

## Mobile release
```bash
flutter build appbundle --release   # Android
flutter build ipa --release         # iOS (macOS)
```
Sign with Play App Signing / Apple certificates. Point `API_BASE_URL` via `--dart-define`.

## Security checklist
- [ ] Secrets not in git
- [ ] TLS only public
- [ ] DB not publicly reachable
- [ ] Rate limits enabled
- [ ] Dependency audit in CI
- [ ] JWT secret rotated procedure documented
- [ ] `APP_ENV=production` and non-wildcard CORS
