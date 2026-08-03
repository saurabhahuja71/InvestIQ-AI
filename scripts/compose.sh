#!/usr/bin/env bash
# Wrapper: prefer `podman compose`, then `podman-compose`.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

compose() {
  if command -v podman >/dev/null 2>&1 && podman compose version >/dev/null 2>&1; then
    podman compose "$@"
  elif command -v podman-compose >/dev/null 2>&1; then
    podman-compose "$@"
  else
    echo "Neither 'podman compose' nor 'podman-compose' found." >&2
    echo "Install with: sudo dnf -y install podman podman-compose" >&2
    echo "See docs/09-podman-dnf-setup.md" >&2
    exit 1
  fi
}

# Default compose file
if [[ -f compose.yml ]]; then
  compose -f compose.yml "$@"
else
  compose "$@"
fi
