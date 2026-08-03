# Podman Compose & dnf setup (RHEL / Oracle Linux / Fedora)

InvestIQ AI local infra is run with **Podman** and **Podman Compose** (not Docker).  
Host packages are installed with **`dnf`**.

**After packages are installed, run the app using:** [10-local-run.md](10-local-run.md).

---

## 1. Install packages with dnf

### Core (required for API + Postgres + Redis)

```bash
sudo dnf -y update
sudo dnf -y install \
  podman \
  podman-compose \
  git \
  curl \
  ca-certificates \
  gcc \
  gcc-c++ \
  make \
  openssl-devel \
  pkgconf-pkg-config \
  postgresql \
  redis
```

| Package | Why |
|---------|-----|
| `podman` | Rootless containers |
| `podman-compose` | Compose file orchestration |
| `gcc` / `openssl-devel` / `pkgconf-pkg-config` | Build Rust crates (`sqlx`, `ring`, etc.) |
| `postgresql` | Optional host client (`psql`) |
| `redis` | Optional host CLI (`redis-cli`) |

### Optional: treat `docker` CLI as Podman

```bash
sudo dnf -y install podman-docker
# Provides a docker-compatible CLI that talks to Podman
```

### Optional: Linux desktop Flutter tooling

```bash
sudo dnf -y install \
  clang \
  cmake \
  ninja-build \
  gtk3-devel \
  xz \
  unzip \
  which \
  mesa-libGL-devel
```

### Optional: Android build (large)

Install Android Studio or command-line tools separately; then:

```bash
# Example packages that help native builds
sudo dnf -y install java-17-openjdk-devel unzip wget
```

### Rust (not always in dnf as latest)

```bash
# Prefer rustup for current stable
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustc --version
```

### Flutter

Install from [flutter.dev](https://docs.flutter.dev/get-started/install/linux) (or your path  
`$HOME/development/flutter`), then:

```bash
export PATH="$HOME/development/flutter/bin:$PATH"
flutter doctor
```

---

## 2. Podman Compose commands

Compose file: **`compose.yml`** (same content as classic docker-compose; Podman-compatible).

```bash
cd /path/to/InvestIQ-AI

# Start only databases
podman compose up -d postgres redis

# Full stack (API image + DB + Redis)
podman compose up -d --build

# Logs
podman compose logs -f api

# Stop
podman compose down

# Shell into Postgres
podman compose exec postgres pg_isready -U investiq
```

If your distro only ships the standalone binary:

```bash
podman-compose up -d postgres redis
```

---

## 3. Rootless tips

```bash
# Enable lingering so user containers survive logout (optional)
loginctl enable-linger "$USER"

# Check Podman
podman info
podman compose version   # or: podman-compose version
```

Port bindings (`5432`, `6379`, `8080`) use the same host ports as documented for local dev.

---

## 4. One-shot helper scripts

```bash
# Install dnf packages (needs sudo)
./scripts/install-deps-dnf.sh

# Start Postgres/Redis via Podman and run the API
./scripts/dev.sh
```

---

## 5. CI note

GitHub Actions still uses Docker service containers on `ubuntu-latest` runners.  
Local development and self-hosted RHEL/Oracle agents should use **Podman Compose**.
