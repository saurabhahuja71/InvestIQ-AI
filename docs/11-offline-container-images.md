# Offline container images (no docker.io)

InvestIQ local stack needs:

| Image | Purpose |
|-------|---------|
| `docker.io/library/postgres:16-alpine` | PostgreSQL 16 |
| `docker.io/library/redis:7-alpine` | Redis 7 |

If the machine **cannot pull from docker.io**, use the pre-exported images published on this repo’s GitHub Releases.

## Download from GitHub Release

Release tag: **`container-images-v1`**

```bash
cd ~/InvestIQ-AI   # or clone the repo
mkdir -p dist/container-images
gh release download container-images-v1 -D dist/container-images
# Or download the .tar.gz assets from the GitHub UI → Releases
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
