# InvestIQ AI — REST API Design

**Base URL:** `https://api.investiq.ai/api/v1`  
**Auth:** `Authorization: Bearer <access_token>`  
**Content-Type:** `application/json`  
**Rate limit headers:** `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-Request-Id`

## Standard envelope

```json
{
  "success": true,
  "data": {},
  "error": null,
  "meta": { "page": 1, "per_page": 20, "total": 100 }
}
```

Error:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "email is required",
    "details": []
  }
}
```

---

## Auth

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/register` | No | Create account |
| POST | `/auth/login` | No | Email/password → tokens |
| POST | `/auth/refresh` | No | Rotate refresh token |
| POST | `/auth/logout` | Yes | Revoke refresh |
| GET | `/auth/me` | Yes | Current user |
| PATCH | `/auth/me` | Yes | Update profile prefs |
| POST | `/auth/change-password` | Yes | Change password |

### POST `/auth/register`
```json
{ "email": "user@example.com", "password": "...", "full_name": "Ada" }
```
Response: `{ user, access_token, refresh_token, expires_in }`

### POST `/auth/login`
```json
{ "email": "...", "password": "..." }
```

---

## IPOs

| Method | Path | Description |
|--------|------|-------------|
| GET | `/ipos` | List/filter/search |
| POST | `/ipos/sync` | Force NSE → Postgres sync |
| GET | `/ipos/calendar` | Calendar range |
| GET | `/ipos/:id` | Detail + financials + GMP |
| GET | `/ipos/:id/ai-summary` | Cached AI DRHP summary |
| POST | `/ipos/:id/watch` | Add watchlist |
| DELETE | `/ipos/:id/watch` | Remove watchlist |
| GET | `/ipos/watchlist` | User IPO watchlist |
| POST | `/ipos/:id/allotment-check` | Check allotment |

### Query params `GET /ipos`
- `status`: upcoming|open|closed|listed
- `board`: mainboard|sme
- `q`: search
- `from`, `to`: dates
- `page`, `per_page`
- `refresh`: `true` to sync from NSE before listing (best-effort)
- `sort`: open_date|close_date|gmp|subscription

### IPO detail notes
- `gmp` object always includes `unofficial: true`, `disclaimer`, and `available` (false when NSE has no GMP — do not invent values).
- Live data provider: see [11-ipo-data-provider.md](11-ipo-data-provider.md).

---

## Portfolio

| Method | Path | Description |
|--------|------|-------------|
| GET | `/portfolios` | List portfolios |
| POST | `/portfolios` | Create |
| GET | `/portfolios/:id` | Dashboard summary |
| GET | `/portfolios/:id/holdings` | Holdings |
| POST | `/portfolios/:id/holdings` | Add holding |
| PATCH | `/portfolios/:id/holdings/:hid` | Update |
| DELETE | `/portfolios/:id/holdings/:hid` | Remove |
| GET | `/portfolios/:id/transactions` | History |
| POST | `/portfolios/:id/transactions` | Record txn |
| GET | `/portfolios/:id/analytics` | XIRR, CAGR, allocation |
| GET | `/portfolios/:id/dividends` | Dividend tracker |
| GET | `/portfolios/:id/performance` | Time series |
| POST | `/portfolios/:id/ai-review` | AI portfolio review |
| GET | `/watchlist/symbols` | Symbol watchlist |
| POST | `/watchlist/symbols` | Add symbol |
| DELETE | `/watchlist/symbols/:symbol` | Remove |

### Analytics response (excerpt)
```json
{
  "total_value": 1250000.50,
  "today_pnl": 3200.00,
  "today_pnl_pct": 0.26,
  "overall_return_pct": 18.4,
  "xirr": 0.142,
  "cagr": 0.128,
  "allocation_by_class": [{ "asset_class": "stock", "pct": 62.5, "value": 781250 }],
  "allocation_by_sector": []
}
```

---

## Journal

| Method | Path | Description |
|--------|------|-------------|
| GET | `/journal/trades` | List + filters |
| POST | `/journal/trades` | Manual entry |
| GET | `/journal/trades/:id` | Detail |
| PATCH | `/journal/trades/:id` | Update |
| DELETE | `/journal/trades/:id` | Soft delete |
| POST | `/journal/trades/:id/attachments` | Upload URL / multipart |
| POST | `/journal/import` | Broker CSV import |
| GET | `/journal/analytics` | Win rate, avg win/loss, etc. |
| GET | `/journal/reports/monthly` | Monthly report |
| GET | `/journal/calendar` | Calendar heatmap data |
| GET | `/journal/strategies` | Strategy performance |
| POST | `/journal/ai/mistakes` | AI mistake detection |

---

## AI Assistant

| Method | Path | Description |
|--------|------|-------------|
| GET | `/ai/conversations` | List |
| POST | `/ai/conversations` | Create |
| GET | `/ai/conversations/:id` | Messages |
| POST | `/ai/chat` | Send message (optionally stream) |
| DELETE | `/ai/conversations/:id` | Delete |

### POST `/ai/chat`
```json
{
  "conversation_id": "uuid-or-null",
  "message": "Summarize this IPO",
  "context": { "ipo_id": "..." }
}
```
Response always includes:
```json
{
  "reply": "...",
  "disclaimer": "This is not financial advice. Past performance does not guarantee future results. InvestIQ AI does not provide guaranteed returns.",
  "conversation_id": "..."
}
```

---

## Notifications & Settings

| Method | Path | Description |
|--------|------|-------------|
| GET | `/notifications` | In-app feed |
| POST | `/notifications/:id/read` | Mark read |
| POST | `/notifications/read-all` | Mark all |
| POST | `/devices` | Register FCM token |
| GET | `/settings/notification-prefs` | Prefs |
| PUT | `/settings/notification-prefs` | Update |
| POST | `/alerts/price` | Create price alert |
| GET | `/alerts/price` | List |
| DELETE | `/alerts/price/:id` | Delete |
| POST | `/data/export` | Export user data (async job) |
| DELETE | `/account` | Delete account |

---

## Health

| Method | Path |
|--------|------|
| GET | `/health` |
| GET | `/ready` |

---

## Auth flow (sequence)

1. Register/Login → access (15m) + refresh (30d)
2. API calls with access token
3. 401 → client refreshes with refresh token
4. Refresh rotates; old refresh revoked
5. Logout revokes refresh + optional Redis denylist for jti
