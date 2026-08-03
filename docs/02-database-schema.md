# InvestIQ AI — Database Schema (PostgreSQL)

## Conventions
- UUIDs as primary keys (`gen_random_uuid()`)
- `created_at` / `updated_at` timestamptz
- Soft delete where user content matters (`deleted_at`)
- Money as `NUMERIC(20, 8)` with explicit `currency` (ISO 4217)
- Enums as PostgreSQL ENUM or TEXT CHECK

---

## Extensions

```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
```

---

## Auth & Users

```sql
CREATE TYPE user_status AS ENUM ('active', 'suspended', 'deleted');

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           CITEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    full_name       TEXT,
    phone           TEXT,
    status          user_status NOT NULL DEFAULT 'active',
    email_verified  BOOLEAN NOT NULL DEFAULT FALSE,
    preferred_currency CHAR(3) NOT NULL DEFAULT 'INR',
    preferred_locale   TEXT NOT NULL DEFAULT 'en',
    theme_preference   TEXT NOT NULL DEFAULT 'system', -- light|dark|system
    biometric_enabled  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE refresh_tokens (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash      TEXT NOT NULL UNIQUE,
    device_info     TEXT,
    expires_at      TIMESTAMPTZ NOT NULL,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
```

---

## IPO Module

```sql
CREATE TYPE ipo_board AS ENUM ('mainboard', 'sme');
CREATE TYPE ipo_status AS ENUM (
    'upcoming', 'open', 'closed', 'allotted', 'listed', 'withdrawn'
);

CREATE TABLE companies (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    symbol          TEXT,
    sector          TEXT,
    industry        TEXT,
    description     TEXT,
    logo_url        TEXT,
    website         TEXT,
    cin             TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE ipos (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id      UUID NOT NULL REFERENCES companies(id),
    board           ipo_board NOT NULL,
    status          ipo_status NOT NULL DEFAULT 'upcoming',
    issue_type      TEXT, -- fresh|ofs|both
    price_band_low  NUMERIC(20, 4),
    price_band_high NUMERIC(20, 4),
    issue_price     NUMERIC(20, 4),
    lot_size        INTEGER,
    issue_size_cr   NUMERIC(20, 4),
    fresh_issue_cr  NUMERIC(20, 4),
    ofs_cr          NUMERIC(20, 4),
    open_date       DATE,
    close_date      DATE,
    allotment_date  DATE,
    refund_date     DATE,
    listing_date    DATE,
    exchange        TEXT, -- NSE|BSE|both
    registrar       TEXT,
    lead_managers   JSONB DEFAULT '[]',
    subscription_total NUMERIC(12, 4),
    subscription_retail NUMERIC(12, 4),
    subscription_qib    NUMERIC(12, 4),
    subscription_nii    NUMERIC(12, 4),
    listing_open    NUMERIC(20, 4),
    listing_close   NUMERIC(20, 4),
    listing_high    NUMERIC(20, 4),
    listing_low     NUMERIC(20, 4),
    gmp_value       NUMERIC(20, 4), -- unofficial
    gmp_updated_at  TIMESTAMPTZ,
    gmp_disclaimer  TEXT NOT NULL DEFAULT 'Grey Market Premium is unofficial and not endorsed by any exchange or regulator.',
    drhp_url        TEXT,
    rhp_url         TEXT,
    financials      JSONB DEFAULT '{}', -- revenue, pat, etc.
    pros            JSONB DEFAULT '[]',
    risks           JSONB DEFAULT '[]',
    ai_summary      TEXT,
    ai_summary_at   TIMESTAMPTZ,
    search_vector   tsvector,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ipos_status ON ipos(status);
CREATE INDEX idx_ipos_board ON ipos(board);
CREATE INDEX idx_ipos_open_close ON ipos(open_date, close_date);
CREATE INDEX idx_ipos_search ON ipos USING GIN(search_vector);

CREATE TABLE ipo_watchlist (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ipo_id      UUID NOT NULL REFERENCES ipos(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, ipo_id)
);

CREATE TABLE allotment_checks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ipo_id      UUID NOT NULL REFERENCES ipos(id),
    pan_last4   CHAR(4), -- store minimal PII
    application_number TEXT,
    status      TEXT, -- pending|allotted|not_allotted|unknown
    shares      INTEGER,
    checked_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## Portfolio Module

```sql
CREATE TYPE asset_class AS ENUM (
    'stock', 'etf', 'mutual_fund', 'gold', 'bond', 'cash', 'other'
);

CREATE TABLE portfolios (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL DEFAULT 'Main',
    base_currency CHAR(3) NOT NULL DEFAULT 'INR',
    is_default  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE holdings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    asset_class     asset_class NOT NULL,
    symbol          TEXT,
    name            TEXT NOT NULL,
    isin            TEXT,
    quantity        NUMERIC(20, 8) NOT NULL DEFAULT 0,
    avg_cost        NUMERIC(20, 8) NOT NULL DEFAULT 0,
    currency        CHAR(3) NOT NULL DEFAULT 'INR',
    sector          TEXT,
    exchange        TEXT,
    metadata        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_holdings_portfolio ON holdings(portfolio_id);

CREATE TYPE txn_type AS ENUM (
    'buy', 'sell', 'dividend', 'interest', 'deposit', 'withdrawal',
    'split', 'bonus', 'fee', 'transfer_in', 'transfer_out'
);

CREATE TABLE transactions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    holding_id      UUID REFERENCES holdings(id) ON DELETE SET NULL,
    txn_type        txn_type NOT NULL,
    trade_date      DATE NOT NULL,
    quantity        NUMERIC(20, 8),
    price           NUMERIC(20, 8),
    fees            NUMERIC(20, 8) DEFAULT 0,
    amount          NUMERIC(20, 8) NOT NULL, -- signed cash impact
    currency        CHAR(3) NOT NULL DEFAULT 'INR',
    notes           TEXT,
    external_id     TEXT, -- broker import idempotency
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_txn_portfolio_date ON transactions(portfolio_id, trade_date);

CREATE TABLE dividends (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    holding_id      UUID REFERENCES holdings(id),
    ex_date         DATE,
    pay_date        DATE,
    amount          NUMERIC(20, 8) NOT NULL,
    currency        CHAR(3) NOT NULL DEFAULT 'INR',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE price_snapshots (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol      TEXT NOT NULL,
    asset_class asset_class NOT NULL,
    price       NUMERIC(20, 8) NOT NULL,
    currency    CHAR(3) NOT NULL DEFAULT 'INR',
    as_of       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (symbol, asset_class, as_of)
);

CREATE TABLE portfolio_watchlist (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol      TEXT NOT NULL,
    asset_class asset_class NOT NULL DEFAULT 'stock',
    name        TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, symbol, asset_class)
);
```

---

## Trading Journal

```sql
CREATE TYPE trade_side AS ENUM ('long', 'short');
CREATE TYPE emotion_tag AS ENUM (
    'confident', 'fearful', 'greedy', 'fomo', 'calm', 'anxious',
    'revenge', 'bored', 'neutral', 'other'
);

CREATE TABLE journal_trades (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    side            trade_side NOT NULL,
    strategy_name   TEXT,
    entry_price     NUMERIC(20, 8) NOT NULL,
    exit_price      NUMERIC(20, 8),
    quantity        NUMERIC(20, 8) NOT NULL,
    entry_at        TIMESTAMPTZ NOT NULL,
    exit_at         TIMESTAMPTZ,
    stop_loss       NUMERIC(20, 8),
    take_profit     NUMERIC(20, 8),
    risk_reward     NUMERIC(12, 4),
    fees            NUMERIC(20, 8) DEFAULT 0,
    pnl             NUMERIC(20, 8),
    emotion_before  emotion_tag,
    emotion_after   emotion_tag,
    notes           TEXT,
    tags            TEXT[] DEFAULT '{}',
    broker_import_id TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_journal_user_entry ON journal_trades(user_id, entry_at DESC);

CREATE TABLE journal_attachments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trade_id    UUID NOT NULL REFERENCES journal_trades(id) ON DELETE CASCADE,
    file_url    TEXT NOT NULL,
    mime_type   TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE journal_ai_insights (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_id    UUID REFERENCES journal_trades(id) ON DELETE CASCADE,
    insight_type TEXT NOT NULL, -- mistake|pattern|summary
    content     TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## AI Assistant

```sql
CREATE TABLE ai_conversations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE ai_messages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content         TEXT NOT NULL,
    context_refs    JSONB DEFAULT '{}', -- {ipo_id, portfolio_id, trade_ids}
    model           TEXT,
    tokens_in       INTEGER,
    tokens_out      INTEGER,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_messages_conv ON ai_messages(conversation_id, created_at);
```

---

## Notifications & Alerts

```sql
CREATE TYPE notif_channel AS ENUM ('push', 'email', 'in_app');
CREATE TYPE notif_type AS ENUM (
    'ipo_open', 'ipo_close', 'allotment', 'listing_day',
    'portfolio_alert', 'price_alert', 'dividend_alert', 'news_alert', 'system'
);

CREATE TABLE devices (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    fcm_token       TEXT NOT NULL,
    platform        TEXT NOT NULL, -- ios|android
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, fcm_token)
);

CREATE TABLE notification_prefs (
    user_id     UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    prefs       JSONB NOT NULL DEFAULT '{}',
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE price_alerts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol      TEXT NOT NULL,
    condition   TEXT NOT NULL, -- above|below
    threshold   NUMERIC(20, 8) NOT NULL,
    active      BOOLEAN NOT NULL DEFAULT TRUE,
    triggered_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE notifications (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notif_type  notif_type NOT NULL,
    title       TEXT NOT NULL,
    body        TEXT NOT NULL,
    payload     JSONB DEFAULT '{}',
    read_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_user ON notifications(user_id, created_at DESC);
```

---

## Audit (optional production)

```sql
CREATE TABLE audit_log (
    id          BIGSERIAL PRIMARY KEY,
    user_id     UUID,
    action      TEXT NOT NULL,
    resource    TEXT,
    ip          INET,
    meta        JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## ER Summary

- `users` 1—N `portfolios` 1—N `holdings` / `transactions`
- `companies` 1—N `ipos` N—M `users` via `ipo_watchlist`
- `users` 1—N `journal_trades` 1—N `journal_attachments`
- `users` 1—N `ai_conversations` 1—N `ai_messages`
