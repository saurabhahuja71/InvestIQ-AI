-- Milestone 2: live NSE IPO source — remove dummy seeds, add sync metadata

DELETE FROM allotment_checks
WHERE ipo_id IN (
  'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
  'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
  'cccccccc-cccc-cccc-cccc-cccccccccccc'
);

DELETE FROM ipo_watchlist
WHERE ipo_id IN (
  'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
  'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
  'cccccccc-cccc-cccc-cccc-cccccccccccc'
);

DELETE FROM notifications
WHERE payload->>'ipo_id' IN (
  'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
  'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
  'cccccccc-cccc-cccc-cccc-cccccccccccc'
);

DELETE FROM ipos
WHERE id IN (
  'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
  'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
  'cccccccc-cccc-cccc-cccc-cccccccccccc'
);

DELETE FROM companies
WHERE id IN (
  '11111111-1111-1111-1111-111111111111',
  '22222222-2222-2222-2222-222222222222',
  '33333333-3333-3333-3333-333333333333'
);

ALTER TABLE ipos
    ADD COLUMN IF NOT EXISTS nse_symbol TEXT,
    ADD COLUMN IF NOT EXISTS nse_series TEXT,
    ADD COLUMN IF NOT EXISTS face_value NUMERIC(20, 4),
    ADD COLUMN IF NOT EXISTS min_investment NUMERIC(20, 4),
    ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'nse',
    ADD COLUMN IF NOT EXISTS source_synced_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS issue_info JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE UNIQUE INDEX IF NOT EXISTS idx_ipos_nse_symbol_series
    ON ipos (nse_symbol, nse_series);

CREATE INDEX IF NOT EXISTS idx_companies_symbol ON companies (symbol);
