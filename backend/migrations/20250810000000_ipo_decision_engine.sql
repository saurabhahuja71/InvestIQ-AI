-- Milestone 5 — IPO Investment Decision Engine.
--
-- Stores the latest deterministic InvestIQ Analysis for each IPO as an audit
-- trail / cache. The payload JSONB mirrors the GET /ipos/{id}/analysis
-- response (scores, views, confidence, factors, missing data). The
-- methodology_version is stored alongside so old rows remain attributable to
-- the exact scoring rules that produced them.
CREATE TABLE IF NOT EXISTS ipo_analysis (
    ipo_id              UUID PRIMARY KEY REFERENCES ipos(id) ON DELETE CASCADE,
    payload             JSONB NOT NULL,
    methodology_version TEXT NOT NULL,
    computed_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
