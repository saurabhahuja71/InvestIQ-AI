# IPO Data Provider Research — Milestone 4

**Status:** Research only — no provider integration implemented.
**Date:** 2026-08-08
**Scope:** Identify real, licenseable data providers for Milestone 4 IPO Intelligence (subscription, GMP, financials, company information) for a **public Google Play Store app**.

> ⚠️ **Implementation is frozen until provider selection is approved.** Nothing below changes the current NSE-based implementation.

---

## 1. Executive summary

| Data category | Preferred source | Why |
|---------------|------------------|-----|
| IPO dates, price band, issue size, lot size, subscription | **Official** — NSE/BSE exchange feeds (or a licensed aggregator that normalizes them) | These are exchange-published facts; aggregators republish them from the same public sources |
| GMP / Kostak / SS | **Unofficial provider only** — no official source exists | NSE/BSE/SEBI explicitly do **not** publish GMP |
| Company financials | **Official RHP/prospectus** (audited, company-filed) + optional structured feed | Audited figures come from the company's own filing; no free official structured API exists |
| Allotment status | **Registrar (official)** — deep-link + indicative engine | Registrars are authoritative but only expose PAN/application lookups, not bulk feeds |
| Company info (name, sector, description, logo) | **Aggregator enrichment** (IPONotify / IPO Guru) | NSE feeds lack logo/sector/description — these come from aggregators or the RHP |

### Headline findings

1. **The three aggregator providers named for evaluation (IPO Guru, IPOAlerts, IPONotify) all exist and offer real JSON APIs.** Two are India-focussed fintech aggregators; IPO Guru is currently free.
2. **GMP is only available from unofficial providers.** This is expected — there is no "official GMP". All three providers carry GMP (IPO Guru free, IPONotify included in paid plans, IPOAlerts as a paid add-on).
3. **Critical compliance finding — current NSE approach is a production risk.** NSE's Terms of Use explicitly prohibit systematic/automated data collection (scraping, data harvesting), caching website content in proxy servers, and redistribution of market data without a signed agreement with NSE Data & Analytics (a paid, licensed feed). The current cookie/UA/referer sync pattern is common community practice but **violates NSE's written terms** for a commercial, Play-Store-distributed app. For production this must be resolved by (a) a formal NSE Data & Analytics license, and/or (b) routing display data through a licensed aggregator that has already taken on normalization/compliance, while treating NSE as a *direct source only for official facts* and keeping usage low-profile and non-commercial until licensed.
4. **No provider publishes explicit written rights for caching-in-Postgres + redistribution-to-end-users.** All are "use in your own application" style licenses with "no resale of raw data". Displaying the data inside your own app is the intended use; **getting a written license confirmation from the chosen provider is a hard prerequisite before Play Store production** (and before any paid plan).
5. **Financials have no free, official, structured API.** The official source is the RHP/prospectus PDF (audited, company-filed, and freely publishable facts). Chittorgarh/IPOMatrix sells structured data only via enterprise inquiry. Plan: primary = own RHP ingestion (official facts); optional paid structured feed later.

---

## 2. Do-not-scrape / do-not-seed guardrails (confirmed)

- ❌ No scraping of **IPO Trend, IPO Watch, InvestorGain, IPO Central** (or any competitor site).
- ❌ No scraping at all — including of the official NSE/BSE/SEBI/registrar **HTML pages**.
- ❌ No scraping-as-a-service middlemen (e.g. Parse.bot's "InvestorGain API" or "Chittorgarh API" are crawlers of those sites and are **excluded**).
- ❌ No inventing/seeding GMP, subscription, or financial data. Empty tables remain the honest default.
- ✅ Only *officially documented JSON APIs* (NSE public JSON feeds, BSE feeds) and *published developer APIs* of the aggregator providers are candidates.

---

## 3. Data category → official vs unofficial

### OFFICIAL sources exist (use official where possible)

| Field | Official source | Notes |
|-------|-----------------|-------|
| IPO open/close/allotment/listing dates | NSE/BSE public issue feeds; registrar announcements | Exchange-published |
| Price band, issue price, face value, lot size | NSE/BSE issue detail; RHP | Exchange-published |
| Issue size, fresh issue vs OFS | NSE/BSE; RHP | |
| Subscription multiples (QIB/NII/Retail/Total; Employee/Shareholder when applicable) | **NSE bid-details API** (current impl) / BSE bidding data | Official; published live during issue window |
| Listing price / listing day data | NSE/BSE equity feed (post-listing) | Official exchange data (requires license for redistribution — see §6) |
| Registrar, lead managers | NSE/BSE issue detail | Official |
| RHP / DRHP URLs | NSE/BSE; SEBI filings portal | Official |
| Allotment status (individual) | **Registrar** portals: KFintech (`ipostatus.kfintech.com`), Link Intime / MUFG Intime (`linkintime.co.in`), Bigshare, Cameo, Maashitla | Authoritative but per-PAN/application only — no bulk feed |
| Company financials (audited) | **RHP/prospectus** (official company filing) | Primary official source; parse or link, do not invent |
| Company identity (name, CIN, registered office) | RHP; MCA records | |

### UNOFFICIAL (no official source exists — clearly label)

| Field | Source |
|-------|--------|
| GMP (Grey Market Premium) | Unofficial aggregator only (IPO Guru, IPONotify, IPOAlerts GMP add-on). Always labelled "unofficial" |
| Kostak, Subject-to-Sauda | Same unofficial channels |

---

## 4. Provider deep-dives

### 4.1 IPO Guru — `ipoguru.in`

> ✅ **Verified against the live API page on 2026-08-08.** All advertised claims confirmed: free (no credit card); 300 req/day + 15 req/min (429 with `retry_after`/`resets_at`); IPO details; subscription QIB/NII/Retail/Total + `updated_at`; GMP `price`/`percentage`/`updated_at`; price band; lot size; issue size; dates (open/close/allotment/listing); registrar; Mainboard + SME; REST/JSON; **commercial use permitted with attribution** (FAQ).

| Dimension | Finding |
|-----------|---------|
| **API availability** | ✅ Yes. `GET https://www.ipoguru.in/api/v1/ipos` (+ `type`, `status` filters). One aggregate call returns open/upcoming/recently-listed issues. |
| **API documentation** | ✅ Public page: `ipoguru.in/ipo-gmp-details-developer-api` — base URL, auth, rate limits, full field reference, code samples (curl/Python/PHP/JS/Go). |
| **GMP availability** | ✅ Included free. `gmp.price`, `gmp.percentage`, `gmp.updated_at` per IPO. Updated multiple times/day. |
| **Subscription availability** | ✅ Included free. `subscription.qib/nii/retail/total/updated_at`. |
| **Financial data availability** | ❌ None (issue-level fields only: price band, lot, issue size, registrar, dates, listing price). |
| **Company info availability** | Partial — full company name, type (Mainboard/SME), sub-type, listing exchange. No logo/sector/description. |
| **Update frequency** | GMP: multiple times/day. Subscription: "as per exchange releases". Timestamps returned per field. |
| **API rate limits** | 15 req/min + 300 req/day per key. 429 with `retry_after` / `resets_at`. |
| **Free tier** | ✅ Entirely free. No credit card. Key issued manually by emailing `ipoguru.in@gmail.com` / contact form (name, app, use case). |
| **Paid pricing** | None published. Free-only today → supply/scale risk for production. |
| **API key requirements** | Yes — `X-API-KEY` header (or query param). Manual issuance. |
| **Commercial usage rights** | API page states: *"Yes, commercial use is permitted. We ask that you attribute IPO data to IPO Guru in your application where appropriate."* |
| **Mobile app usage rights** | Not explicitly stated → treat as covered by the commercial-use statement; **confirm in writing** (their Android app exists). |
| **Attribution requirements** | ✅ Yes — "attribute IPO data to IPO Guru where appropriate". |
| **Play Store distribution permitted** | Not explicitly stated. Website ToS ("do not republish/redistribute material") **contradicts** the API page's commercial-use grant. **Must obtain written confirmation.** |
| **Caching in PostgreSQL permitted** | Not explicitly stated. Treat as OK for app operation under the commercial-use grant; confirm in writing. |
| **Redistribution through app permitted** | The API exists to serve IPO data to apps → intended use, but the ToS wording conflict means **written confirmation required**. |
| **Reliability / freshness** | No SLA published. Community-sized aggregator; GMP freshness is timestamped. For dev/startup this is fine; for production, pair with a fallback. |
| **Verdict** | ✅ **Best for development** — free, single call, GMP + subscription included, no credit card. ⚠️ Production needs written license + lacks paid/SLA tier. |

### 4.2 IPOAlerts — `ipoalerts.in`

| Dimension | Finding |
|-----------|---------|
| **API availability** | ✅ Yes. `https://api.ipoalerts.in` — `GET /ipos` (paged, filterable), `GET /ipos/{id}`. |
| **API documentation** | ✅ Public: `ipoalerts.in/docs` — auth, rate limits, data sources, IPO object reference, endpoints, pagination. TypeScript interfaces. |
| **GMP availability** | ✅ Yes, but **paid add-on** (Pro plan + GMP addon). Rich object: `min/max/mean/median/mode` aggregations + per-source breakdowns + `lastUpdatedAt`. |
| **Subscription availability** | ✅ Yes (QIB/NII/Retail + Total; from NSE/BSE bid data). |
| **Financial data availability** | ⚠️ Partial — structured `about`, `strengths`, `risks` per IPO, but **no P&L/balance-sheet numbers**. |
| **Company info availability** | ⚠️ Partial — name, symbol, logo, prospectus URL, about, strengths, risks, media coverage. No sector per the IPO object shown. |
| **Update frequency** | "High-frequency updates with slight delays depending on source availability" (aggregates registrar/broker/issuer/exchange disclosures). |
| **API rate limits** | Documented per plan: Free 6/min, 360/hr, 8640/day; Pro higher; Enterprise custom. 429 responses include quota detail. |
| **Free tier** | ✅ Free: 6 req/min, 750 req/month, **25 req/day cap**, 1 API key, max 1 IPO/request, community support. |
| **Paid pricing** | **Pro ₹499/mo** (1,000–5,000 req/mo tiers, ₹0.50/req overage, 60 req/min, unlimited keys, historical access). **Enterprise** custom. GMP is a paid add-on. (Older "Hobby" ₹99/mo plan existed.) |
| **API key requirements** | Yes — `x-api-key` header, issued from dashboard. |
| **Commercial usage rights** | Positioned for "mobile and web applications"; disclaimer states data is aggregated from publicly accessible sources and not affiliated with NSE/BSE/SEBI. |
| **Mobile app usage rights** | ✅ Explicitly marketed for mobile apps ("Ideal for mobile and web applications"). |
| **Attribution requirements** | Not found in public docs — confirm. |
| **Play Store distribution permitted** | Strongly implied by "ideal for mobile apps"; not written in detail → confirm in the paid agreement. |
| **Caching in PostgreSQL permitted** | Not explicitly documented → confirm in paid agreement. |
| **Redistribution through app permitted** | Aggregator data meant to power apps → intended; confirm in paid agreement. |
| **Reliability / freshness** | No SLA on free; Pro/Enterprise with SLA-like terms (priority support on Enterprise). Data source-disclaimed. |
| **Verdict** | ✅ Strong **production candidate** for subscription + GMP (with add-on) + company narrative. Cheapest paid option (₹499/mo + GMP add-on). |

### 4.3 IPONotify — `iponotify.me`

| Dimension | Finding |
|-----------|---------|
| **API availability** | ✅ Yes. `https://iponotify.me/api/ipo/{status}` (open/upcoming/closed), `GET /api/ipo/id/{searchId}`, plus GMP, listing, subscription, allotment, NCD/bond/buyback endpoints. Cursor pagination. |
| **API documentation** | ✅ Public: `iponotify.me/docs` — auth, rate limits, endpoints, full sample payloads. |
| **GMP availability** | ✅ Yes — included in paid plans (`/api/ipos/gmp`), plus per-IPO GMP. |
| **Subscription availability** | ✅ Yes — per-category `subscriptionRates`: QIB, NII, RETAIL, EMPLOYEES, SHA (shareholder), TOTAL. Covers the exact columns InvestIQ stores. |
| **Financial data availability** | ✅ **Best in class** — Pro plan includes "detailed IPO financial analysis"; payloads already carry `aboutCompany` (yearFounded, MD, description), `pros`, `cons`, RHP `documentUrl`, `logoUrl`, `sector`. |
| **Company info availability** | ✅ **Best in class** — sector, company description, year founded, MD, pros, cons, logo, registrar, RTA link, scrip codes (BSE/NSE), listing data. This directly fills every "Not Available" gap in the current NSE app. |
| **Update frequency** | Real-time during market hours (GMP intraday, subscription per exchange). 99.9% uptime claim. |
| **API rate limits** | Free: 4/min, 250 req/mo, open-only, 5 IPOs/response. Starter: 10/min, 1500 req/mo. Pro: 25/min, 5000 req/mo, 50 IPOs/response. 429 with monthly-quota message. |
| **Free tier** | ✅ Free: 250 req/month, 1 key, **open IPOs only**. |
| **Paid pricing** | **Starter ₹1,999/mo** (1,500 req/mo, open+upcoming+closed, GMP). **Pro ₹4,999/mo** (5,000 req/mo, all data + GMP + financial analysis + bonds, priority lane, analytics). **Enterprise** custom. |
| **API key requirements** | Yes — `X-API-KEY` header. |
| **Commercial usage rights** | API ToS: non-exclusive license to use data for **your own applications**; **may not resell raw data**; **no scraping or redistribution**. |
| **Mobile app usage rights** | Built for "fintech apps, brokers, market platforms" — display in your own app is the intended use. |
| **Attribution requirements** | Not explicitly stated → confirm. |
| **Play Store distribution permitted** | Implied by "own applications"/fintech positioning; "no redistribution" wording needs written clarification that serving your own app's users is in scope. |
| **Caching in PostgreSQL permitted** | Not explicit. Caching for app operation is standard; "no resale of raw data" is the clear boundary. **Confirm in writing.** |
| **Redistribution through app permitted** | Intended (your own app) but "no redistribution" boilerplate must be clarified for the paid license. |
| **Reliability / freshness** | Production positioning, 99.9% uptime claim, priority lane on Pro, webhooks for status changes & allotment. |
| **Verdict** | ✅ **Best for public Play Store production** — richest structured dataset (GMP + full subscription incl. Employee/Shareholder + financial analysis + company info), webhooks, production-grade claims. ₹4,999/mo. Requires written license confirmation on redistribution/caching. |

### 4.4 NSE India (official — current source)

| Dimension | Finding |
|-----------|---------|
| **API availability** | Public website JSON feeds: `/api/ipo-current-issue`, `/api/all-upcoming-issues`, `/api/public-past-issues`, `/api/ipo-detail?symbol=&series=`. No official developer program/keys. |
| **API documentation** | ❌ Not documented for third parties; community-derived (cookie + UA + Referer session pattern). |
| **GMP availability** | ❌ **No** — NSE does not publish GMP (documented in `docs/11-ipo-data-provider.md`). |
| **Subscription availability** | ✅ **Official** — bid-details subscription multiples (total/QIB/NII/retail; employee/shareholder when present). Current sync already captures this. |
| **Financial data availability** | ❌ Not structured. Only a "Ratios / Basis of Issue Price" ZIP link when present. |
| **Company info availability** | ❌ No logo, industry, description, or website (documented gap). |
| **Update frequency** | Live during market hours (subscription); issue list refreshed by NSE. |
| **API rate limits** | None published; InvestIQ self-paces ~3 req/s with a sync lock. |
| **Free tier** | ✅ Free to access (public website). |
| **Paid pricing** | NSE Data & Analytics sells licensed market-data feeds (Corporate Data, real-time/snapshot/EOD). |
| **API key requirements** | None (public feeds). |
| **Commercial usage rights** | ⚠️ **Restricted.** NSE ToU prohibit systematic/automated collection, caching in proxy servers, and redistribution of market data without a signed agreement with NSE Data & Analytics. Commercial use is expressly not granted by the website ToU. |
| **Mobile app usage rights** | Not granted by public ToU for commercial apps. |
| **Attribution requirements** | NSE asks attribution where data is displayed. |
| **Play Store distribution permitted** | ❌ **Not without a formal NSE Data & Analytics license.** This is the single biggest compliance item for M4 production. |
| **Caching in PostgreSQL permitted** | ⚠️ ToU prohibit caching website content in proxy servers; a signed data agreement is the correct path for durable caching + display. |
| **Redistribution through app permitted** | ❌ Only per a signed Relevant Agreement. |
| **Reliability / freshness** | Good when up; 403/rate-limit risk documented; app already fails soft to last DB snapshot. |
| **Verdict** | ✅ Best **official** source for issue metadata + subscription facts. ⚠️ For a commercial Play Store app, use under a formal license OR display via a licensed aggregator while treating direct NSE use as non-commercial/low-volume. |

### 4.5 BSE India (official)

| Dimension | Finding |
|-----------|---------|
| **API availability** | Public issue listings on `beta.bseindia.com/markets/PublicIssues/IPOIssues_new.aspx` (HTML). Commercial feeds via BSE **Self Data Feed** / BSE StAR / Corporate data products. |
| **API documentation** | No public free JSON API for IPO subscription; commercial feed agreements required for structured data. |
| **GMP availability** | ❌ No (exchange — GMP not published). |
| **Subscription availability** | ✅ Official (combined NSE+BSE bidding); exposed via HTML/public-issue pages and commercial feeds. |
| **Financial data availability** | Company filings via BSE corporate announcements (HTML/PDF). |
| **Verdict** | Cross-check official dates/subscription for issues listed only on BSE (some IPOs list on BSE only). **Do not scrape the HTML** — use commercial feed or aggregator instead. |

### 4.6 SEBI (official)

| Dimension | Finding |
|-----------|---------|
| **API availability** | `sebi.gov.in` filings portal (DRHP pipeline, issue status). No public structured API. |
| **Data value** | DRHP/RHP filing dates + PDF URLs; regulatory status (approved/withdrawn). Official `Curation_Links_for_Securities_Market_Data` page links the canonical NSE/BSE IPO pages. |
| **Verdict** | Official for the **DRHP pipeline** (pre-announcement tracking). Integrate only as a document/catalog source, not as a live data feed; do not scrape bulk HTML. |

### 4.7 Registrars — KFintech, Link Intime (MUFG Intime), Bigshare, Cameo, Maashitla (official)

| Dimension | Finding |
|-----------|---------|
| **API availability** | ❌ No public API. Portals: `ipostatus.kfintech.com`, `linkintime.co.in/MIPO/Ipoallotment.html`, `bigshareonline.com/IPOAllotment.aspx`. |
| **Data value** | **Authoritative allotment status** — but only per PAN/application number (individual lookup, CAPTCHA). No bulk feed. |
| **Verdict** | Keep current approach: **indicative allotment engine** + **deep-links** to the correct registrar portal per IPO. Do not scrape registrar portals. Registrar name is already captured from NSE detail. |

### 4.8 Chittorgarh / IPOMatrix (financials & deep analytics)

| Dimension | Finding |
|-----------|---------|
| **API availability** | ❌ No self-serve public API. IPOMatrix offers "media / API access" **by inquiry** (enterprise, paid). |
| **Data value** | Structured subscription, **financial statements** (revenue, PAT, EBITDA, net worth, debt), GMP trends, allotment ratios, 20+ yr history. |
| **Verdict** | ⚠️ Strong data but enterprise-negotiated. **Fallback/optional** financial-data path later. The Parse.bot middleman API for Chittorgarh is a scraper and is **excluded**. |

### 4.9 Other options (brief)

| Provider | Notes |
|----------|-------|
| **Tijori** | Paid research platform; public API limited, enterprise custom. Not self-serve. |
| **APIDataFeed** | Paid IPO API (BSE/NSE IPOs), subscription required, pricing not public. |
| **Global Datafeeds** | Paid fundamental-data APIs (financial results) for listed companies — post-listing, not IPO-RHP specific. |
| **Finnhub** | IPO calendar API — global/US-centric, no Indian GMP, no Indian IPO financials. Not suitable for this feature. |
| **IPO Grid / IPOTracker / IPO.AI / Renaissance IPO Pro** | India GMP/specific coverage absent or US-focussed. Not suitable. |
| **Parse.bot marketplaces (InvestorGain, Chittorgarh)** | Scraping-as-a-service of sites we must not scrape. **Excluded.** |

---

## 5. Comparison table

| Criteria | IPO Guru | IPOAlerts | IPONotify | NSE (official) | Registrars (official) |
|----------|----------|-----------|-----------|----------------|------------------------|
| Official JSON API | ✅ | ✅ | ✅ | ⚠️ public feed (undocumented) | ❌ |
| GMP | ✅ free | ✅ add-on (paid) | ✅ paid plans | ❌ | ❌ |
| Subscription (QIB/NII/Retail/Total) | ✅ | ✅ | ✅ + Employee/Shareholder | ✅ | — |
| Financial data | ❌ | ⚠️ narrative only | ✅ financial analysis (Pro) | ❌ | ❌ |
| Company info (logo/sector/about) | ⚠️ partial | ⚠️ partial | ✅ rich | ❌ | ❌ |
| Webhooks | ❌ | ❌ | ✅ (status/allotment) | ❌ | ❌ |
| Free tier | ✅ 300 req/day | ✅ 750 req/mo (25/day cap) | ✅ 250 req/mo (open only) | ✅ | — |
| Paid pricing | none published | ₹499/mo + GMP add-on | ₹1,999–₹4,999/mo | licensed (NSE Data & Analytics) | — |
| Rate limits (free→paid) | 15/min, 300/day | 6/min → 60/min | 4/min → 25/min | self-paced | — |
| Commercial rights (written) | ⚠️ API page yes, ToS conflict | ⚠️ implied, confirm | ⚠️ implied, confirm | ❌ needs agreement | ❌ |
| Play Store redistribution rights | ⚠️ confirm | ⚠️ confirm | ⚠️ confirm | ❌ license needed | ❌ |
| Attribution required | ✅ | TBC | TBC | ✅ | — |
| Caching in Postgres | ⚠️ confirm | ⚠️ confirm | ⚠️ confirm | ⚠️ license needed | n/a |
| SLA / reliability | ❌ | ⚠️ (Enterprise SLA) | ✅ (99.9% claim, priority) | ✅ (but 403 risk) | — |
| Cost for M4 scope | **₹0** | **~₹499+/mo** | **~₹4,999/mo** | license cost TBD | ₹0 (deep-link) |

---

## 6. Legal & Play Store compliance checklist

For a **public, commercial Play Store app**, before integrating any provider, obtain and keep on file:

1. **Written commercial/data license** from the chosen aggregator (IPONotify or IPOAlerts) explicitly permitting:
   - display of the data to your app's end users,
   - durable caching of the data in PostgreSQL/Redis for app operation,
   - commercial/Play Store distribution of the app itself,
   - attribution requirements (if any).
2. **NSE decision**: either (a) sign NSE Data & Analytics agreement for official data redistribution, or (b) keep direct NSE use minimal/non-commercial and serve subscription/issue facts through the licensed aggregator. **Document this decision.**
3. **GMP disclaimer** on every GMP screen: unofficial, not from NSE/BSE/SEBI, not indicative of listing price. (Already designed: `GMP_DISCLAIMER` in backend.)
4. **Attribution** strings per provider ("Data source: IPO Guru", "GMP: IPO Guru", etc.).
5. **Registrar deep-links** instead of scraping registrar portals for allotment.
6. **No fabricated data**: keep the empty-table-is-honest default for anything not yet covered (already the design in `20250808000000_ipo_intelligence.sql`).

---

## 7. Recommendations

### A. Best provider for development
**IPO Guru** — ✅ **confirmed (2026-08-08)**: free, no credit card, **one call** returns IPO details + subscription + GMP + registrar + dates (exactly the M4 fields). 300 req/day is ample for a dev/staging key. Add **IPOAlerts free tier** as a secondary dev feed for cross-validation.

### A′. Best fit for the free InvestIQ app at launch
**IPO Guru (free)** is a very good fit for the free, ad-supported app:
- It is the only evaluated provider with **GMP included at zero cost** (IPOAlerts charges an add-on; IPONotify only on paid plans).
- Subscription (QIB/NII/Retail/Total) + issue metadata + registrar come in the same single call.
- Commercial use + attribution is granted on the API page.

**Watch-outs for production on the free tier:**
- **300 req/day budget:** a 30-min full sync (open+upcoming+closed) ≈ 144 req/day + detail fetches. Fits only because InvestIQ caches durably in PostgreSQL (already designed). Consider syncing open/upcoming frequently and `closed` less often, and/or caching detail responses with a TTL.
- **No paid tier / no SLA:** if the app outgrows the quota or the API degrades, the fallback path (NSE → Postgres → Redis, plus IPOAlerts free tier) must engage automatically.
- **ToS conflict:** the API page grants commercial use, but the website ToS still says "do not republish/redistribute". Obtain written confirmation before Play Store production (see §6).

### B. Best provider for public Play Store production
**IPONotify Pro (₹4,999/mo)** — richest structured coverage — is the production *scale-up* path once the app has a revenue model. For a **free app at launch**, however, **IPO Guru (free) + attribution** is the recommended primary (see A′), with IPONotify/IPOAlerts as the paid upgrade when traffic or reliability requirements demand it.

When scaling up to IPONotify Pro:
- GMP (unofficial) ✅
- Subscription incl. **Employee & Shareholder** categories (matches the InvestIQ schema exactly) ✅
- **Financial analysis** + RHP/DRHP links, logo, sector, about, pros/cons ✅ (fills every current "Not Available" gap)
- Webhooks (status/allotment) ✅
- Production claims (99.9% uptime, priority lane) ✅

**Primary architecture for production:**
- **Official facts** (dates, price band, lot, issue size, subscription): from the licensed aggregator feed (already normalized from NSE/BSE), with NSE kept as a direct non-commercial fallback while licensing is pending.
- **GMP**: IPONotify GMP (or IPO Guru as a low-cost/secondary GMP source with attribution).
- **Financials**: primary = own ingestion of the official RHP (audited company facts → `ipo_financials`), optional enrichment from a paid structured feed (Chittorgarh/IPOMatrix enterprise or IPONotify's financial analysis) when licensed.
- **Allotment**: registrar deep-links + indicative engine (unchanged).

**Budget alternative:** IPOAlerts **Pro (₹499/mo) + GMP add-on** — cheapest compliant paid path; less company/financial enrichment than IPONotify.

### C. Best fallback provider
**IPO Guru** (free, attribution) as the GMP/subscription fallback feed, plus the existing **NSE → Postgres → Redis** path that already fails soft to the last good snapshot when a provider is down. **IPOAlerts free tier** as a second fallback. Keep the backend provider abstraction so fallback is automatic.

---

## 8. Proposed architecture

```
                    OFFICIAL SOURCES                    UNOFFICIAL
   ┌──────────┬───────────┬──────────┬──────────┐
   │ NSE      │ BSE       │ SEBI     │ Registrar│      GMP provider
   │ (feeds/  │ (feeds/   │ (DRHP    │ (KFin,   │      (IPO Guru /
   │  license)│  license) │  catalog)│  Intime) │       IPONotify)
   └────┬─────┴─────┬─────┴────┬─────┴────┬─────┘            │
        │           │          │          │                  │
        └───────────┴────┬─────┴──────────┘                  │
                         │                                   │
                 ┌───────▼────────┐            ┌─────────────▼──────┐
                 │  Licensed      │            │  Unofficial GMP    │
                 │  aggregator    │            │  + enrichment      │
                 │  (IPONotify /  │            │  (sub/fin/company) │
                 │   IPOAlerts)   │            │                    │
                 └───────┬────────┘            └──────────┬─────────┘
                         │                                │
                         ▼                                ▼
              ┌─────────────────────────────────────────────────┐
              │              Rust / Axum backend               │
              │  sync workers · provider adapters · scoring     │
              └───────────────────────┬─────────────────────────┘
                                      ▼
                              ┌──────────────┐
                              │ PostgreSQL   │   durable cache / history
                              │ (Postgres)   │
                              └──────┬───────┘
                                     ▼
                              ┌──────────────┐
                              │ Redis        │   response cache / locks
                              └──────┬───────┘
                                     ▼
                              ┌──────────────┐
                              │ Flutter app  │   (display + disclaimers)
                              └──────────────┘
```

Layering rules:
- **Official → official sources / licensed aggregator.** Never scraped.
- **Unofficial (GMP) → explicitly-labelled unofficial provider only.**
- Every row carries `source` + `updated_at` so the "Data source & data quality" page stays auditable (already designed).
- Sync is rate-limited, idempotent, and fails soft to last good DB state.

---

## 9. Open decisions (blockers before implementation)

| # | Decision | Status / Options |
|---|----------|------------------|
| 1 | **NSE production posture** | (a) sign NSE Data & Analytics license, (b) serve official facts via licensed aggregator and keep NSE direct use non-commercial, (c) continue current approach (risk) — **recommend b now, a later** |
| 2 | **Primary production aggregator** | ✅ **IPO Guru (free) confirmed as launch primary** for the free app (verified 2026-08-08). Paid scale-up: IPONotify Pro ₹4,999/mo (rich) or IPOAlerts Pro ₹499/mo + GMP add-on (budget). Written license needed before Play Store production. |
| 3 | **GMP source** | ✅ **IPO Guru (free, attribution)** for launch; IPONotify/IPOAlerts as paid upgrade. |
| 4 | **Financial data** | Own RHP ingestion (official, free, manual/PDF-parsing effort) vs paid structured feed (Chittorgarh/IPOMatrix enterprise or IPONotify analysis) vs defer — **undecided** |
| 5 | **Dev-only scope** | ✅ Free tiers (IPO Guru + IPOAlerts) are enough for Milestone 4 development without any paid plan |

> **Action required:** confirm provider choice(s) + written license checks before any integration work begins.

---

## 10. Sources

- IPO Guru — Developer API page `ipoguru.in/ipo-gmp-details-developer-api`; website ToS `ipoguru.in/terms-conditions` (2026-08-08).
- IPOAlerts — `ipoalerts.in` homepage, `/docs`, `/docs/api-reference/*`, `/pricing`, blog (Hobby plan) (2026-08-08).
- IPONotify — `iponotify.me` homepage, `/developers`, `/docs`, `/pricing`, `/apiterms`, `/terms` (2026-08-08).
- NSE — Data Usage & Sharing Policy `nseindia.com/static/market-data/nse-data-policy`; Terms of Use `nseindia.com/static/nse-terms-of-use`; Data & Info Vending page (2026-08-08).
- BSE — `beta.bseindia.com` public issues pages; `marketdata.bseindia.com` Self Data Feed (2026-08-08).
- SEBI — `sebi.gov.in` Curation Links for Securities Market Data (2026-08-08).
- Registrars — KFintech `ipostatus.kfintech.com`, Link Intime `linkintime.co.in`, Bigshare `bigshareonline.com` (2026-08-08).
- Chittorgarh / IPOMatrix — `ipomatrix.com` (About, Contact, Data sources); `chittorgarh.com` FAQ (2026-08-08).
