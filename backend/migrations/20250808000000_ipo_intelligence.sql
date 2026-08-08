-- Milestone 4: IPO Intelligence
-- Subscription snapshots + history, GMP snapshots + history, financials,
-- InvestIQ IPO Score cache, and data-source metadata.
-- No dummy production data. Empty tables are the honest default for fields
-- that no configured provider can legally supply.

-- Current/latest subscription snapshot per IPO.
CREATE TABLE IF NOT EXISTS ipo_subscription_snapshots (
    ipo_id         UUID PRIMARY KEY REFERENCES ipos(id) ON DELETE CASCADE,
    retail         NUMERIC(12, 4),
    nii            NUMERIC(12, 4),
    qib            NUMERIC(12, 4),
    employee       NUMERIC(12, 4),
    shareholder    NUMERIC(12, 4),
    overall        NUMERIC(12, 4),
    is_final       BOOLEAN NOT NULL DEFAULT FALSE,
    source         TEXT NOT NULL DEFAULT 'nse',
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Subscription by day (capture day during the issue window) per IPO.
CREATE TABLE IF NOT EXISTS ipo_subscription_history (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ipo_id         UUID NOT NULL REFERENCES ipos(id) ON DELETE CASCADE,
    day            DATE NOT NULL,
    retail         NUMERIC(12, 4),
    nii            NUMERIC(12, 4),
    qib            NUMERIC(12, 4),
    employee       NUMERIC(12, 4),
    shareholder    NUMERIC(12, 4),
    overall        NUMERIC(12, 4),
    is_final       BOOLEAN NOT NULL DEFAULT FALSE,
    source         TEXT NOT NULL DEFAULT 'nse',
    captured_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (ipo_id, day, source)
);

CREATE INDEX IF NOT EXISTS idx_sub_history_ipo_day
    ON ipo_subscription_history (ipo_id, day DESC);

-- Current/latest unofficial GMP snapshot per IPO.
CREATE TABLE IF NOT EXISTS ipo_gmp (
    ipo_id         UUID PRIMARY KEY REFERENCES ipos(id) ON DELETE CASCADE,
    value          NUMERIC(20, 4),
    gmp_percent    NUMERIC(12, 4),
    source         TEXT,
    updated_at     TIMESTAMPTZ,
    disclaimer     TEXT NOT NULL DEFAULT 'Grey Market Premium is unofficial market information and is not guaranteed to reflect the actual listing price.'
);

-- GMP history.
CREATE TABLE IF NOT EXISTS ipo_gmp_history (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ipo_id         UUID NOT NULL REFERENCES ipos(id) ON DELETE CASCADE,
    value          NUMERIC(20, 4),
    gmp_percent    NUMERIC(12, 4),
    source         TEXT,
    captured_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_gmp_history_ipo_time
    ON ipo_gmp_history (ipo_id, captured_at DESC);

-- Financial metrics per fiscal period (audited/company-provided only).
CREATE TABLE IF NOT EXISTS ipo_financials (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ipo_id             UUID NOT NULL REFERENCES ipos(id) ON DELETE CASCADE,
    period             TEXT NOT NULL,
    period_start       DATE,
    period_end         DATE,
    revenue            NUMERIC(20, 4),
    revenue_growth_pct NUMERIC(12, 4),
    ebitda             NUMERIC(20, 4),
    ebitda_margin_pct  NUMERIC(12, 4),
    pat                NUMERIC(20, 4),
    pat_growth_pct     NUMERIC(12, 4),
    eps                NUMERIC(20, 4),
    pe_ratio           NUMERIC(20, 4),
    roe_pct            NUMERIC(12, 4),
    roce_pct           NUMERIC(12, 4),
    debt               NUMERIC(20, 4),
    debt_to_equity     NUMERIC(12, 4),
    audited            BOOLEAN NOT NULL DEFAULT TRUE,
    source             TEXT NOT NULL DEFAULT 'company_prospectus',
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (ipo_id, period)
);

CREATE INDEX IF NOT EXISTS idx_financials_ipo_period
    ON ipo_financials (ipo_id, period DESC);

-- InvestIQ IPO Score cache. The score is computed server-side from real data
-- and stored here so the breakdown is reproducible/auditable.
CREATE TABLE IF NOT EXISTS ipo_scores (
    ipo_id               UUID PRIMARY KEY REFERENCES ipos(id) ON DELETE CASCADE,
    total                NUMERIC(6, 2),
    financial_strength   NUMERIC(6, 2),
    growth               NUMERIC(6, 2),
    profitability        NUMERIC(6, 2),
    valuation            NUMERIC(6, 2),
    subscription         NUMERIC(6, 2),
    industry_business    NUMERIC(6, 2),
    risk                 NUMERIC(6, 2),
    included_max         NUMERIC(6, 2),
    positive_factors     JSONB NOT NULL DEFAULT '[]'::jsonb,
    concerns             JSONB NOT NULL DEFAULT '[]'::jsonb,
    methodology_version  TEXT NOT NULL,
    computed_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Data-source metadata. Enables the app to show provider / freshness /
-- official-vs-unofficial / licensing for every externally sourced category.
CREATE TABLE IF NOT EXISTS data_sources (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider               TEXT NOT NULL,
    category               TEXT NOT NULL,
    official               BOOLEAN NOT NULL DEFAULT FALSE,
    api_url                TEXT,
    refresh_frequency_secs INT,
    licensing              TEXT,
    rate_limits            TEXT,
    enabled                BOOLEAN NOT NULL DEFAULT TRUE,
    notes                  TEXT,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Make the seed below idempotent (one provider+category row).
CREATE UNIQUE INDEX IF NOT EXISTS idx_data_sources_provider_category
    ON data_sources (provider, category);

INSERT INTO data_sources (provider, category, official, api_url, refresh_frequency_secs, licensing, rate_limits, enabled, notes) VALUES
  ('NSE India public feeds', 'ipo_details', TRUE, 'https://www.nseindia.com/api/ipo-detail', NULL,
   'Public website JSON used by NSE IPO pages; not a separately licensed market-data product. Do not republish bulk dumps.',
   'No formal public rate limit; InvestIQ paces at ~3 req/sec with a sync lock.', TRUE,
   'Issue metadata, dates, registrar, lead managers, prospectus URLs, subscription bid details.'),
  ('NSE India public feeds', 'subscription', TRUE, 'https://www.nseindia.com/api/ipo-detail', NULL,
   'Public website JSON used by NSE IPO pages; not a separately licensed market-data product.',
   'No formal public rate limit; InvestIQ paces at ~3 req/sec with a sync lock.', TRUE,
   'Bid-details subscription multiples (total/QIB/NII/retail; employee/shareholder when present).'),
  ('NSE India public feeds', 'gmp', FALSE, NULL, NULL,
   'NSE does NOT publish GMP. InvestIQ never fabricates grey-market values.',
   NULL, TRUE,
   'Unavailable. An unofficial GMP vendor (e.g. IPO Guru, IPOAlerts, IPONotify) with an API key is required to populate this.'),
  ('Company prospectus / RHP', 'financials', TRUE, NULL, NULL,
   'Company filing, audited. InvestIQ only stores audited/company-provided figures.',
   NULL, TRUE,
   'Revenue/EBITDA/PAT/EPS/ratios per fiscal period. Structured ingestion requires a financial-data provider (e.g. Chittorgarh, Tijori) or manual RHP ingestion.')
ON CONFLICT DO NOTHING;
