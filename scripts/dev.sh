#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ ! -f .env ]]; then
  cp .env.example .env
  echo "Created .env from .env.example"
fi

COMPOSE="$ROOT/scripts/compose.sh"
chmod +x "$COMPOSE" 2>/dev/null || true

echo "Starting Postgres + Redis with Podman Compose..."
"$COMPOSE" up -d postgres redis

echo "Waiting for Postgres..."
for _ in $(seq 1 60); do
  if "$COMPOSE" exec -T postgres pg_isready -U investiq >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if ! "$COMPOSE" exec -T postgres pg_isready -U investiq >/dev/null 2>&1; then
  echo "Postgres did not become ready. Check: $COMPOSE logs postgres" >&2
  exit 1
fi

set -a
# shellcheck disable=SC1091
source <(grep -v '^#' .env | sed '/^$/d' | sed 's/^/export /')
set +a

export PATH="${HOME}/.cargo/bin:${PATH}"
cd backend
cargo run
