# Navigation Flow & UI Wireframes

## App shell (authenticated)

```
┌──────────────────────────────────────┐
│  AppBar (context title + actions)    │
├──────────────────────────────────────┤
│                                      │
│           Content area               │
│     (glass cards, charts, lists)     │
│                                      │
├──────────────────────────────────────┤
│ Home │ IPOs │ Portfolio │ Journal │ AI│
└──────────────────────────────────────┘
```

## Navigation graph

```
Splash
  ├─ (no token) → Login ⇄ Register
  └─ (token) → MainShell
                 ├─ HomeDashboard
                 │     └─ deep links to IPO/Portfolio/Journal/AI
                 ├─ IPO
                 │     ├─ IPOList (tabs: Open/Upcoming/Closed/SME)
                 │     ├─ IPOCalendar
                 │     ├─ IPODetail
                 │     │     ├─ Financials / Pros-Risks / GMP / AI Summary
                 │     │     └─ AllotmentChecker
                 │     └─ IPOWatchlist
                 ├─ Portfolio
                 │     ├─ Dashboard (value, P&L, charts)
                 │     ├─ HoldingsList → HoldingDetail
                 │     ├─ Transactions
                 │     ├─ Dividends
                 │     ├─ Analytics (XIRR/CAGR/Allocation)
                 │     └─ AI Portfolio Review
                 ├─ Journal
                 │     ├─ TradeList
                 │     ├─ TradeEntry / TradeDetail
                 │     ├─ CalendarView
                 │     ├─ Analytics & Monthly Report
                 │     └─ AI Mistake Detection
                 ├─ AI Chat
                 └─ Settings (from profile icon)
                       ├─ Appearance (theme)
                       ├─ Currency / Language
                       ├─ Biometric
                       ├─ Notifications
                       ├─ Backup / Export
                       └─ Privacy / About / Disclaimer
```

---

## Wireframes (textual, production UI intent)

### Login
- Logo + wordmark “InvestIQ AI”
- Email, password fields (Material 3 filled)
- Primary CTA “Sign in”
- Secondary “Create account”
- Biometric button if enrolled
- Footer microcopy: security / encrypted

### Home dashboard
- Greeting + portfolio pulse card (total value, today %)
- Horizontal chips: Open IPOs · Alerts · AI tip
- Section “Open IPOs” carousel (glass cards)
- Section “Recent trades” mini list
- FAB or card “Ask AI”

### IPO list
- Search bar + filter sheet (board, status, date)
- Segmented: Open | Upcoming | Closed | SME
- Card: logo, name, price band, dates, subscription badge, unofficial GMP pill
- Pull-to-refresh

### IPO detail
- Hero: company name, board chip, status
- Key metrics row: issue price, lot size, dates
- GMP card (amber border, “Unofficial” badge + disclaimer)
- Tabs: Overview | Financials | Pros & Risks | Timeline | Listing
- Sticky CTA: Add to watchlist · Check allotment · AI Summary

### Portfolio dashboard
- Large value + today P&L (green/red)
- Sparkline performance
- Donut: asset allocation
- Horizontal sector bars
- Quick actions: Add transaction · AI review
- Holdings list with mini P&L

### Journal
- Stats strip: Win rate · Expectancy · Avg R
- Calendar heat strip
- Trade cards: symbol, side chip, P&L, tags, emotions
- FAB: New trade
- Import broker (overflow)

### AI chat
- Message bubbles (user right, assistant left)
- Context chips when grounded (IPO X, Portfolio)
- Persistent disclaimer banner (collapsed after first scroll, always accessible)
- Suggestions: “Summarize open IPOs”, “Review portfolio risk”, “Find trade mistakes”
- Input bar + send; no “guaranteed returns” language in UI copy

### Settings
- Grouped lists Material 3
- Theme toggle
- Currency picker
- Biometric switch
- Export data / Delete account (destructive)

---

## Visual system

| Token | Light | Dark |
|-------|-------|------|
| Primary | Deep teal `#0D9488` | Teal accent `#2DD4BF` |
| Surface | `#F8FAFC` | `#0B1220` |
| Card | white 90% + blur | `#111827` 80% + blur |
| Profit | `#059669` | `#34D399` |
| Loss | `#DC2626` | `#F87171` |
| Warning (GMP) | `#D97706` | `#FBBF24` |

**Glassmorphism:** semi-transparent surface + 12–20 blur + 1px border `white@8%`  
**Motion:** 200–300ms ease-out page transitions; chart load fade  
**Typography:** Material 3 type scale; tabular nums for money  
