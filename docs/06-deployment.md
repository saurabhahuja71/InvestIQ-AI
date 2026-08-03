# Deployment Guide

## Prerequisites
- Docker & Docker Compose
- Domain + TLS certificates (or Caddy automatic HTTPS)
- Managed PostgreSQL 16+ and Redis 7+ (or Compose for staging)
- FCM project for push
- LLM API key (OpenAI-compatible / xAI / etc.) stored in secrets manager

## Local development

```bash
# From repo root
cp .env.example .env
docker compose up -d postgres redis
cd backend && cargo run
cd mobile && flutter pub get && flutter run
```

API: `http://localhost:8080`  
Postgres: `localhost:5432`  
Redis: `localhost:6379`

## Staging / production with Compose

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build
```

Services: `api`, `postgres`, `redis`, `migrate`, `caddy` (TLS).

## Environment variables

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | Postgres connection string |
| `REDIS_URL` | Redis URL |
| `JWT_SECRET` | ≥32 byte secret |
| `JWT_ACCESS_TTL_SECS` | default 900 |
| `JWT_REFRESH_TTL_SECS` | default 2592000 |
| `AES_KEY_BASE64` | 32-byte key base64 |
| `AI_API_KEY` | LLM provider key |
| `AI_BASE_URL` | Provider base URL |
| `AI_MODEL` | Model name |
| `RUST_LOG` | `info,tower_http=debug` |
| `CORS_ORIGINS` | Comma-separated |
| `RATE_LIMIT_RPS` | e.g. 20 |

## Migrations

```bash
cd backend
sqlx migrate run
# or container: migrate job in Compose
```

## Health checks
- `GET /health` — process up
- `GET /ready` — DB + Redis reachable

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
