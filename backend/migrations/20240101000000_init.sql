-- InvestIQ AI initial schema
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

DO $$ BEGIN
    CREATE TYPE user_status AS ENUM ('active', 'suspended', 'deleted');
EXCEPTION WHEN duplicate_object THEN null; END $$;

DO $$ BEGIN
    CREATE TYPE ipo_board AS ENUM ('mainboard', 'sme');
EXCEPTION WHEN duplicate_object THEN null; END $$;

DO $$ BEGIN
    CREATE TYPE ipo_status AS ENUM ('upcoming', 'open', 'closed', 'allotted', 'listed', 'withdrawn');
EXCEPTION WHEN duplicate_object THEN null; END $$;

DO $$ BEGIN
    CREATE TYPE asset_class AS ENUM ('stock', 'etf', 'mutual_fund', 'gold', 'bond', 'cash', 'other');
EXCEPTION WHEN duplicate_object THEN null; END $$;

DO $$ BEGIN
    CREATE TYPE txn_type AS ENUM (
        'buy', 'sell', 'dividend', 'interest', 'deposit', 'withdrawal',
        'split', 'bonus', 'fee', 'transfer_in', 'transfer_out'
    );
EXCEPTION WHEN duplicate_object THEN null; END $$;

DO $$ BEGIN
    CREATE TYPE trade_side AS ENUM ('long', 'short');
EXCEPTION WHEN duplicate_object THEN null; END $$;

DO $$ BEGIN
    CREATE TYPE emotion_tag AS ENUM (
        'confident', 'fearful', 'greedy', 'fomo', 'calm', 'anxious',
        'revenge', 'bored', 'neutral', 'other'
    );
EXCEPTION WHEN duplicate_object THEN null; END $$;

DO $$ BEGIN
    CREATE TYPE notif_type AS ENUM (
        'ipo_open', 'ipo_close', 'allotment', 'listing_day',
        'portfolio_alert', 'price_alert', 'dividend_alert', 'news_alert', 'system'
    );
EXCEPTION WHEN duplicate_object THEN null; END $$;

CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    full_name       TEXT,
    phone           TEXT,
    status          user_status NOT NULL DEFAULT 'active',
    email_verified  BOOLEAN NOT NULL DEFAULT FALSE,
    preferred_currency CHAR(3) NOT NULL DEFAULT 'INR',
    preferred_locale   TEXT NOT NULL DEFAULT 'en',
    theme_preference   TEXT NOT NULL DEFAULT 'system',
    biometric_enabled  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS refresh_tokens (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash      TEXT NOT NULL UNIQUE,
    device_info     TEXT,
    expires_at      TIMESTAMPTZ NOT NULL,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON refresh_tokens(user_id);

CREATE TABLE IF NOT EXISTS companies (
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

CREATE TABLE IF NOT EXISTS ipos (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id      UUID NOT NULL REFERENCES companies(id),
    board           ipo_board NOT NULL,
    status          ipo_status NOT NULL DEFAULT 'upcoming',
    issue_type      TEXT,
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
    exchange        TEXT,
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
    gmp_value       NUMERIC(20, 4),
    gmp_updated_at  TIMESTAMPTZ,
    gmp_disclaimer  TEXT NOT NULL DEFAULT 'Grey Market Premium is unofficial and not endorsed by any exchange or regulator.',
    drhp_url        TEXT,
    rhp_url         TEXT,
    financials      JSONB DEFAULT '{}',
    pros            JSONB DEFAULT '[]',
    risks           JSONB DEFAULT '[]',
    ai_summary      TEXT,
    ai_summary_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ipos_status ON ipos(status);
CREATE INDEX IF NOT EXISTS idx_ipos_board ON ipos(board);

CREATE TABLE IF NOT EXISTS ipo_watchlist (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ipo_id      UUID NOT NULL REFERENCES ipos(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, ipo_id)
);

CREATE TABLE IF NOT EXISTS allotment_checks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ipo_id      UUID NOT NULL REFERENCES ipos(id),
    pan_last4   CHAR(4),
    application_number TEXT,
    status      TEXT,
    shares      INTEGER,
    checked_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS portfolios (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL DEFAULT 'Main',
    base_currency CHAR(3) NOT NULL DEFAULT 'INR',
    is_default  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS holdings (
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

CREATE TABLE IF NOT EXISTS transactions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    holding_id      UUID REFERENCES holdings(id) ON DELETE SET NULL,
    txn_type        txn_type NOT NULL,
    trade_date      DATE NOT NULL,
    quantity        NUMERIC(20, 8),
    price           NUMERIC(20, 8),
    fees            NUMERIC(20, 8) DEFAULT 0,
    amount          NUMERIC(20, 8) NOT NULL,
    currency        CHAR(3) NOT NULL DEFAULT 'INR',
    notes           TEXT,
    external_id     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dividends (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_id    UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    holding_id      UUID REFERENCES holdings(id),
    ex_date         DATE,
    pay_date        DATE,
    amount          NUMERIC(20, 8) NOT NULL,
    currency        CHAR(3) NOT NULL DEFAULT 'INR',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS portfolio_watchlist (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol      TEXT NOT NULL,
    asset_class asset_class NOT NULL DEFAULT 'stock',
    name        TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, symbol, asset_class)
);

CREATE TABLE IF NOT EXISTS journal_trades (
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

CREATE INDEX IF NOT EXISTS idx_journal_user_entry ON journal_trades(user_id, entry_at DESC);

CREATE TABLE IF NOT EXISTS journal_attachments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trade_id    UUID NOT NULL REFERENCES journal_trades(id) ON DELETE CASCADE,
    file_url    TEXT NOT NULL,
    mime_type   TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS journal_ai_insights (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_id    UUID REFERENCES journal_trades(id) ON DELETE CASCADE,
    insight_type TEXT NOT NULL,
    content     TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ai_conversations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ai_messages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content         TEXT NOT NULL,
    context_refs    JSONB DEFAULT '{}',
    model           TEXT,
    tokens_in       INTEGER,
    tokens_out      INTEGER,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS devices (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    fcm_token       TEXT NOT NULL,
    platform        TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, fcm_token)
);

CREATE TABLE IF NOT EXISTS notification_prefs (
    user_id     UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    prefs       JSONB NOT NULL DEFAULT '{}',
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS price_alerts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol      TEXT NOT NULL,
    condition   TEXT NOT NULL,
    threshold   NUMERIC(20, 8) NOT NULL,
    active      BOOLEAN NOT NULL DEFAULT TRUE,
    triggered_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS notifications (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notif_type  notif_type NOT NULL,
    title       TEXT NOT NULL,
    body        TEXT NOT NULL,
    payload     JSONB DEFAULT '{}',
    read_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed sample IPOs for development
INSERT INTO companies (id, name, symbol, sector, description)
VALUES
  ('11111111-1111-1111-1111-111111111111', 'NovaTech Systems Ltd', 'NOVA', 'Technology', 'Enterprise SaaS platform'),
  ('22222222-2222-2222-2222-222222222222', 'GreenGrid Energy Ltd', 'GGRID', 'Energy', 'Renewable power producer'),
  ('33333333-3333-3333-3333-333333333333', 'CraftSME Foods Ltd', 'CRAFT', 'FMCG', 'SME packaged foods brand')
ON CONFLICT DO NOTHING;

INSERT INTO ipos (
  id, company_id, board, status, price_band_low, price_band_high, lot_size,
  issue_size_cr, open_date, close_date, allotment_date, listing_date, exchange,
  subscription_total, gmp_value, gmp_updated_at,
  financials, pros, risks
) VALUES
(
  'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
  '11111111-1111-1111-1111-111111111111',
  'mainboard', 'open', 420, 445, 33,
  2500, CURRENT_DATE - 1, CURRENT_DATE + 1, CURRENT_DATE + 5, CURRENT_DATE + 12, 'NSE/BSE',
  12.4, 48, NOW(),
  '{"revenue_cr": 1800, "pat_cr": 210, "fy": "FY25"}',
  '["Strong recurring revenue", "Experienced management"]',
  '["High valuation", "Client concentration"]'
),
(
  'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
  '22222222-2222-2222-2222-222222222222',
  'mainboard', 'upcoming', 180, 190, 78,
  1200, CURRENT_DATE + 7, CURRENT_DATE + 10, CURRENT_DATE + 16, CURRENT_DATE + 24, 'NSE/BSE',
  NULL, 15, NOW(),
  '{"revenue_cr": 920, "pat_cr": 95, "fy": "FY25"}',
  '["Policy tailwinds for renewables"]',
  '["Regulatory delays", "Commodity input risk"]'
),
(
  'cccccccc-cccc-cccc-cccc-cccccccccccc',
  '33333333-3333-3333-3333-333333333333',
  'sme', 'closed', 95, 100, 1200,
  85, CURRENT_DATE - 14, CURRENT_DATE - 10, CURRENT_DATE - 5, CURRENT_DATE + 2, 'NSE SME',
  48.2, 22, NOW(),
  '{"revenue_cr": 64, "pat_cr": 8, "fy": "FY25"}',
  '["Niche brand loyalty"]',
  '["SME liquidity risk", "Limited track record"]'
)
ON CONFLICT DO NOTHING;
