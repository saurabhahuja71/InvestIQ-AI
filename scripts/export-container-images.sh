#!/usr/bin/env bash
# Export Postgres + Redis images used by compose.yml for offline / no-docker.io hosts.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:-$ROOT/dist/container-images}"
mkdir -p "$OUT"

POSTGRES_IMG="${POSTGRES_IMG:-docker.io/library/postgres:16-alpine}"
REDIS_IMG="${REDIS_IMG:-docker.io/library/redis:7-alpine}"

if command -v podman >/dev/null 2>&1; then
  CTR=podman
elif command -v docker >/dev/null 2>&1; then
  CTR=docker
else
  echo "Need podman or docker" >&2
  exit 1
fi

echo "Exporting $POSTGRES_IMG ..."
$CTR save "$POSTGRES_IMG" | gzip -1 > "$OUT/postgres-16-alpine.tar.gz"
echo "Exporting $REDIS_IMG ..."
$CTR save "$REDIS_IMG" | gzip -1 > "$OUT/redis-7-alpine.tar.gz"

cat > "$OUT/MANIFEST.txt" <<EOF
InvestIQ AI — container images for local Postgres + Redis
Exported: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Tools: $CTR

Images:
  $POSTGRES_IMG  -> postgres-16-alpine.tar.gz
  $REDIS_IMG     -> redis-7-alpine.tar.gz

Load on another machine:
  ./scripts/load-container-images.sh $OUT
  # or:
  gunzip -c postgres-16-alpine.tar.gz | podman load
  gunzip -c redis-7-alpine.tar.gz | podman load
EOF

ls -lh "$OUT"
echo "Done. Upload with: gh release upload … (see docs/11-offline-container-images.md)"
