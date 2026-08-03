# Offline container images (no docker.io)

## Why this exists

On some machines (e.g. a second laptop / restricted network) **Podman or Docker cannot pull from `docker.io`**. InvestIQ still needs the same Postgres and Redis images that `compose.yml` references. Those images were exported once on a host that *could* reach docker.io and published as **GitHub Release assets** so you can download them over GitHub (HTTPS/gh) instead of the registry.

> Images are **not** committed into the git tree (too large). They live only on the Release.

InvestIQ local stack needs:

| Image | Purpose | Release asset |
|-------|---------|----------------|
| `docker.io/library/postgres:16-alpine` | PostgreSQL 16 | `postgres-16-alpine.tar.gz` (~118 MB) |
| `docker.io/library/redis:7-alpine` | Redis 7 | `redis-7-alpine.tar.gz` (~17 MB) |

## Download from GitHub Release

| Field | Value |
|-------|--------|
| Release tag | **`container-images-v1`** |
| URL | https://github.com/saurabhahuja71/InvestIQ-AI/releases/tag/container-images-v1 |

```bash
cd ~/InvestIQ-AI   # or clone the repo
mkdir -p dist/container-images
gh release download container-images-v1 -D dist/container-images
# Or download the two .tar.gz files from the GitHub UI → Releases
```

## Load into Podman / Docker

```bash
chmod +x scripts/load-container-images.sh
./scripts/load-container-images.sh dist/container-images
```

This loads images under the same names compose expects (`docker.io/library/postgres:16-alpine`, `docker.io/library/redis:7-alpine`).

## Start Postgres + Redis

```bash
./scripts/compose.sh up -d postgres redis
./scripts/compose.sh ps
```

## Re-export (machine that *can* reach docker.io)

```bash
./scripts/export-container-images.sh
# then upload new tarballs to a new release, e.g. container-images-v2
```

## Notes

- Image tarballs are **not** committed to git (too large). They live only on the GitHub **Release**.
- After load, you do **not** need docker.io for these two images.
- The optional `api` service still **builds from local Dockerfile** (`backend/`); that does not pull Postgres/Redis base images unless the Dockerfile does.
