# Rust Backend Structure

```
backend/
├── Cargo.toml
├── Dockerfile
├── migrations/
│   └── 20240101000000_init.sql
└── src/
    ├── main.rs                 # Bootstrap, tracing, CORS, graceful shutdown
    ├── config.rs               # Env config
    ├── error.rs                # AppError → JSON envelope
    ├── state.rs                # AppState (PgPool, Redis, Jwt, Ai)
    ├── routes/mod.rs           # /health, /ready, /api/v1/*
    ├── middleware/auth.rs      # Bearer JWT extractor
    ├── infra/
    │   ├── jwt.rs
    │   ├── password.rs         # Argon2id
    │   ├── crypto.rs           # AES-256-GCM
    │   └── ai.rs               # LLM client + safety system prompt
    └── modules/
        ├── common.rs           # ApiResponse envelope
        ├── health.rs
        ├── auth/               # register, login, refresh, logout, me
        ├── ipo/                # list, detail, watchlist, GMP, AI summary
        ├── portfolio/          # holdings, txns, XIRR analytics, AI review
        ├── journal/            # trades, analytics, AI mistakes
        └── ai/                 # conversations + chat
```

## Request path

`HTTP → Tower middleware (trace, request-id, cors) → Axum route → handler → sqlx/redis/ai → ApiResponse JSON`

## Auth

Handlers extract `AuthUser` via `FromRequestParts` (JWT access token). Refresh tokens stored as SHA-256 hashes with rotation on refresh.
