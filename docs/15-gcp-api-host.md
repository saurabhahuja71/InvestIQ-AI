# InvestIQ API host — GCP static runner

Production API for mobile / Play Store should **not** depend on a developer laptop.

## Validated host (2026-08-07)

| Setting | Value |
|--------|--------|
| Shared infra | Same GCP Always Free runner as Covered Call Bot |
| Hostname | `github-runner-free` |
| Public IP | **`136.67.97.86`** |
| GitHub runner | `fyers-gcp-free` (repo `Tradebots71/covered_call_bot`) |
| SSH (corp laptop) | `ssh sauahuja@136.67.97.86` via corkscrew proxy |
| SSH Host alias | `github-runner-free` (see `~/.ssh/config`) |

Old Mumbai IP `35.200.243.249` is **decommissioned**. Do not document or whitelist it.

## Target layout on the runner

```text
/home/sauahuja/investiq-ai/          # git checkout of InvestIQ-AI
  backend/                          # Rust API
  .env                              # secrets (not in git)
docker/podman: postgres + redis
systemd or compose: investiq-api on 0.0.0.0:8080
optional: Caddy/nginx TLS on 443 → 8080
```

## Mobile `API_BASE_URL`

After the API is listening publicly (or via HTTPS):

```bash
# rebuild APK pointed at permanent host
API_BASE_URL=http://136.67.97.86:8080 \
  ./scripts/build-android-debug-apk.sh
```

Prefer HTTPS in production (`https://api.<your-domain>`).

## Related

- Covered Call Bot runner docs: sibling repo `covered_call_bot` → `docs/self-hosted-runner.md`
- Local Android run: `docs/12-android-run.md`
- Firebase / Google Sign-In: `CONFIGURATION_REQUIRED.md`
