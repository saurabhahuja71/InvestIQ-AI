-- MVP completion: market prices on holdings, notification defaults

ALTER TABLE holdings
    ADD COLUMN IF NOT EXISTS last_price NUMERIC(20, 8),
    ADD COLUMN IF NOT EXISTS prev_close NUMERIC(20, 8),
    ADD COLUMN IF NOT EXISTS price_as_of TIMESTAMPTZ;

-- Backfill last/prev from avg_cost for existing rows (demo-ready MTM)
UPDATE holdings
SET last_price = COALESCE(last_price, avg_cost),
    prev_close = COALESCE(prev_close, avg_cost * 0.995),
    price_as_of = COALESCE(price_as_of, NOW())
WHERE last_price IS NULL OR prev_close IS NULL;

CREATE TABLE IF NOT EXISTS data_export_jobs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'completed',
    payload     JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Ensure every user has notification prefs
INSERT INTO notification_prefs (user_id, prefs)
SELECT id, '{
  "ipo_open": true,
  "ipo_close": true,
  "allotment": true,
  "listing_day": true,
  "portfolio_alert": true,
  "price_alert": true,
  "dividend_alert": true,
  "news_alert": false
}'::jsonb
FROM users
ON CONFLICT (user_id) DO NOTHING;
