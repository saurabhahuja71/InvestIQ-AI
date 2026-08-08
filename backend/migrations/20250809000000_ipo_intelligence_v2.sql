-- Milestone 4 revision: fundamentals-first IPO Intelligence.
--
-- Adds source_type (data-quality contract) to financial and subscription
-- tables and re-aligns the score cache table with the revised 8-component
-- schema. No dummy production data is introduced.

-- Data-quality contract: every financial/subscription value carries its
-- provider (source), official-vs-unofficial classification (source_type),
-- the period it refers to (financials) and its retrieval timestamp.
ALTER TABLE ipo_financials
    ADD COLUMN IF NOT EXISTS source_type TEXT NOT NULL DEFAULT 'official';

ALTER TABLE ipo_subscription_snapshots
    ADD COLUMN IF NOT EXISTS source_type TEXT NOT NULL DEFAULT 'official';

ALTER TABLE ipo_subscription_history
    ADD COLUMN IF NOT EXISTS source_type TEXT NOT NULL DEFAULT 'official';

-- Score cache: the total may be NULL when nothing is scorable (all components
-- have insufficient data), and the components/data-quality snapshots are stored
-- as JSON so the exact methodology output is auditable after the fact.
ALTER TABLE ipo_scores ALTER COLUMN total DROP NOT NULL;

ALTER TABLE ipo_scores
    ADD COLUMN IF NOT EXISTS components_json JSONB NOT NULL DEFAULT '[]';

ALTER TABLE ipo_scores
    ADD COLUMN IF NOT EXISTS data_quality_json JSONB NOT NULL DEFAULT '{}';

ALTER TABLE ipo_scores
    ADD COLUMN IF NOT EXISTS disclaimer TEXT NOT NULL DEFAULT '';

-- Drop the legacy flat component columns if any were created by an earlier
-- revision of the milestone; the components are now stored in components_json.
ALTER TABLE ipo_scores DROP COLUMN IF EXISTS financial_strength;
ALTER TABLE ipo_scores DROP COLUMN IF EXISTS growth;
ALTER TABLE ipo_scores DROP COLUMN IF EXISTS profitability;
ALTER TABLE ipo_scores DROP COLUMN IF EXISTS valuation;
ALTER TABLE ipo_scores DROP COLUMN IF EXISTS subscription;
ALTER TABLE ipo_scores DROP COLUMN IF EXISTS industry_business;
ALTER TABLE ipo_scores DROP COLUMN IF EXISTS risk;
ALTER TABLE ipo_scores DROP COLUMN IF EXISTS included_max;
