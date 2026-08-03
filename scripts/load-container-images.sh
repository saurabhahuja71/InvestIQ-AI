#!/usr/bin/env bash
# Load Postgres + Redis images from tarballs (offline / no docker.io).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR="${1:-}"

if [[ -z "$DIR" ]]; then
  # Prefer release download dir, then local dist
  if [[ -d "$ROOT/dist/container-images" ]]; then
    DIR="$ROOT/dist/container-images"
  else
    echo "Usage: $0 /path/to/dir-with-tar.gz" >&2
    echo "Or download the GitHub release assets into dist/container-images first." >&2
    exit 1
  fi
fi

if command -v podman >/dev/null 2>&1; then
  CTR=podman
elif command -v docker >/dev/null 2>&1; then
  CTR=docker
else
  echo "Need podman or docker" >&2
  exit 1
fi

load_one() {
  local file="$1"
  if [[ ! -f "$file" ]]; then
    echo "Missing: $file" >&2
    exit 1
  fi
  echo "Loading $file ..."
  gunzip -c "$file" | $CTR load
}

load_one "$DIR/postgres-16-alpine.tar.gz"
load_one "$DIR/redis-7-alpine.tar.gz"

echo
echo "Loaded. Tag check:"
$CTR images | grep -E 'postgres|redis' || true
echo
echo "Start stack:"
echo "  cd $ROOT && ./scripts/compose.sh up -d postgres redis"
