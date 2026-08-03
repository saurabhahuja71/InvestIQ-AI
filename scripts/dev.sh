#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ ! -f .env ]]; then
  cp .env.example .env
  echo "Created .env from .env.example"
fi

docker compose up -d postgres redis
echo "Waiting for Postgres..."
until docker compose exec -T postgres pg_isready -U investiq >/dev/null 2>&1; do sleep 1; done

export $(grep -v '^#' .env | xargs)
cd backend
cargo run
