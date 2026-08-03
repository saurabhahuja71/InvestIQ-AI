# Roadmap: MVP → Stores → Growth

**Principle:** Ship a focused MVP with four modules only. Defer broker integrations, advanced analytics, and premium upsells until after Play Store / App Store launch.

---

## MVP scope (launch target)

Build **only** these four products end-to-end:

| # | Module | In MVP | Explicitly out of MVP |
|---|--------|--------|------------------------|
| 1 | **IPO Tracker** | Open / upcoming / closed lists, SME + mainboard filter, search, detail (price band, lot, dates, subscription), **unofficial GMP** label, watchlist, basic allotment status stub or registrar deep-link | Live registrar APIs, push for every IPO event, news feed |
| 2 | **Portfolio Tracker** | Manual holdings (stock, ETF, MF, gold, bond, cash), transactions, dashboard value, simple allocation chart, basic overall return | Live market prices feed, XIRR polish as “advanced”, rebalancing engine, dividends automation |
| 3 | **AI Chat** | Chat UI, disclaimer always visible, prompts for IPO summary / portfolio Q&A / journal Q&A, no guaranteed returns | Streaming polish, multi-agent tools, paid “Pro” AI tiers |
| 4 | **Trading Journal** | Manual trade entry, notes, strategy, tags, emotions, P&L list, win rate / basic stats | Broker CSV/API import, screenshot cloud, calendar heatmaps, AI mistake deep-dives as premium |

### MVP platform & quality bar
- Android + iOS (Flutter)
- Auth (email/password + JWT)
- Dark / light theme
- REST API (Rust) + Postgres
- App Store / Play Store compliance: privacy policy, investment disclaimer, account deletion path
- Crash-free enough for first reviews; p95 non-AI API &lt; 500ms

### MVP non-goals (do not block launch)
- Broker integrations (Zerodha, Groww, etc.)
- Advanced analytics (full XIRR suite, sector deep-dives, risk models)
- Premium subscriptions / paywall
- Push notification matrix (nice-to-have if time; not required)
- Multi-currency FX, family portfolios, web companion

---

## Phase map

### Phase 0 — Foundations (done / in progress)
- [x] Monorepo, architecture docs
- [x] Axum skeleton: auth, IPO, portfolio, journal, AI modules
- [x] Flutter shell + four feature UIs
- [ ] Stable local Docker runbook used by whole team
- [ ] Store listing assets (icon, screenshots, privacy policy URL)

### Phase 1 — MVP hardening (next)
**Goal:** trustable beta → store submission

1. **Auth polish** — error UX, password rules, session restore, logout
2. **IPO** — reliable list/detail from DB or partner feed; GMP always unofficial; watchlist sync
3. **Portfolio** — add/edit/delete holdings + buy/sell; dashboard numbers always consistent
4. **Journal** — create/edit/delete trades; closed-trade P&L; simple analytics strip
5. **AI Chat** — grounded answers when context exists; stub OK without key; permanent disclaimer
6. **Settings** — theme, currency display, disclaimer, delete account / export stub
7. **QA** — empty states, offline “no connection” banner, basic widget tests
8. **Legal** — privacy policy, terms, investment disclaimer in-app + store text
9. **Release** — Android App Bundle + iOS build; staged rollout 10% → 100%

**Exit criteria:** new user can register → browse IPOs → log a holding → journal a trade → ask AI → sign out, without crashes.

### Phase 2 — Post-launch (after store approval)
Ship only if metrics justify effort:

| Item | Why later |
|------|-----------|
| Push: IPO open/close | Retention, needs FCM ops |
| Live prices for portfolio | Vendor cost + reliability |
| Allotment partner API | Integration + PII handling |
| Journal calendar / monthly PDF | Nice analytics, not core loop |
| Offline cache expansion | Complexity after online path is solid |

### Phase 3 — Growth & premium
- **Broker import** (CSV first, then official APIs where available)
- **Advanced analytics** (XIRR/CAGR dashboards, risk, rebalancing suggestions)
- **Premium** (deeper AI reviews, unlimited history, attachments, priority support)
- News alerts, multi-currency, shared portfolios, web companion

---

## Recommended build order (execution)

```
Week 1–2   Auth + app shell + empty states
Week 2–3   IPO list/detail/watchlist
Week 3–4   Portfolio manual CRUD + dashboard
Week 4–5   Trading journal manual + basic stats
Week 5–6   AI chat + disclaimers + grounding
Week 6–7   Polish, legal, store builds, beta
Week 7–8   Store review + hotfixes
```

Do **not** start broker work or premium paywalls before Phase 1 exit criteria are met.

---

## Success metrics (MVP)

| Metric | Target |
|--------|--------|
| Time-to-first-value (register → first useful screen) | &lt; 2 minutes |
| Crash-free sessions | ≥ 99% |
| Store rejection risk | Zero for missing privacy/disclaimer |
| Core loop completion (IPO or portfolio or journal action) | &gt; 40% of new users day 1 |

---

## What already exists vs MVP focus

The monorepo may contain stubs for post-MVP ideas (price alerts tables, XIRR helpers, allotment endpoints). Treat those as **optional scaffolding**. For MVP release branches:

- Feature-flag or hide unfinished UI
- Document APIs as “beta” if incomplete
- Prioritize polish on the four modules above

---

## One-line product promise (store)

> Track IPOs, manage a simple portfolio, journal your trades, and ask InvestIQ AI educational questions — with clear disclaimers and no guaranteed returns.
