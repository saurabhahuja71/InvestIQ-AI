# IPO data provider (Milestone 2)

InvestIQ IPO Tracker uses **NSE India** public website JSON feeds as the live data source. Dummy / seeded IPOs were removed.

## Provider

| Field | Value |
|-------|--------|
| **Provider** | National Stock Exchange of India (NSE) |
| **Official commercial API?** | No — website market-data JSON used by NSE’s own IPO pages |
| **Licensing** | Public website content; not a separately licensed market-data product. Do not republish bulk dumps; cache only for app operation. Respect NSE terms of use. |
| **Attribution** | Display “Data source: NSE” / `source=nse` on detail screens |

## Endpoints used

Base: `https://www.nseindia.com/api`

| Purpose | Endpoint |
|---------|----------|
| Open / active issues | `GET /ipo-current-issue` |
| Upcoming + active calendar | `GET /all-upcoming-issues?category=ipo` |
| Past / closed / listed | `GET /public-past-issues?from_date=DD-MM-YYYY&to_date=DD-MM-YYYY` |
| Issue detail + subscription | `GET /ipo-detail?symbol={SYM}&series={EQ\|SME\|…}` |

Session: HTTP client first loads `https://www.nseindia.com/` to obtain cookies, then calls APIs with a browser User-Agent and IPO page Referer (same pattern as community NSE clients).

## Refresh & cache

| Setting | Default | Env var |
|---------|---------|---------|
| Background sync interval | 30 minutes | `IPO_SYNC_INTERVAL_SECS` |
| Redis list cache TTL | 120 seconds | `IPO_LIST_CACHE_TTL_SECS` |
| On-demand sync | `POST /api/v1/ipos/sync` | — |
| Pull-to-refresh | Client calls sync, then `GET /ipos?refresh=true` | — |

Postgres is the durable cache. Redis caches paginated list responses. On NSE failure, the API continues serving the last successful DB snapshot; the Flutter client falls back to Hive offline cache.

## Rate limits

NSE does not publish a formal public rate limit for these endpoints. InvestIQ:

- Serializes detail fetches with ~350 ms delay between calls
- Uses a process-wide sync lock (no overlapping full syncs)
- Defaults to syncing every 30 minutes

Treat aggressive polling as abusive. If NSE returns 403/401, the client refreshes the session cookie once and retries.

## Field availability

| App field | Source | Notes |
|-----------|--------|-------|
| Company name, symbol | NSE list + detail | |
| Board (mainboard / SME) | `series` (EQ→mainboard, SME→sme) | |
| Status open/upcoming/closed/listed | NSE status + dates | Listed when `listingDate` ≤ today |
| Open / close / listing dates | NSE | Exact strings parsed to ISO dates |
| Price band, lot size, face value | NSE detail | |
| Min investment | Computed: lot × upper band | |
| Issue size (₹ Cr) | Estimated from shares × mid price when possible | Text description also stored in `financials` |
| Exchange | Derived (NSE / NSE SME / BSE SME) | |
| Registrar, lead managers | NSE detail | |
| Subscription (total / QIB / NII / retail) | NSE bid details | Category coverage varies by issue |
| Prospectus (RHP) | NSE “Red Herring Prospectus” URL | |
| Ratios ZIP | NSE “Ratios / Basis of Issue Price” | Linked, not parsed |
| **Logo** | — | **Not Available** (NSE feed has no logo) |
| **Industry / sector** | — | **Not Available** unless later enriched |
| **Company website** | — | **Not Available** from NSE IPO APIs |
| **Business description** | — | **Not Available** from NSE IPO APIs |
| **Structured financial highlights** | — | **Not Available** as structured numbers; ratios ZIP link when present |
| **GMP** | — | **Not Available** — NSE does not publish GMP; UI shows Not Available (no invented values) |
| Allotment / refund dates | — | **Not Available** unless present in future feeds |

## InvestIQ API surface

- `GET /api/v1/ipos` — filter `status`, `board`, `q`, paginate `page`/`per_page`, optional `refresh=true`
- `GET /api/v1/ipos/{id}` — full detail; missing fields are JSON `null` (UI: “Not Available”)
- `POST /api/v1/ipos/sync` — force NSE → Postgres sync

## Failure behaviour

1. Sync errors are logged; last good Postgres rows remain queryable.
2. Redis list cache expires independently; miss falls through to DB.
3. Mobile Hive cache used when Dio fails (offline / API down), with a visible offline banner and Retry.
