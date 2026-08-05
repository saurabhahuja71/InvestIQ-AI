//! Sync IPO rows from NSE into Postgres and invalidate Redis list caches.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use redis::AsyncCommands;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::ipo::nse::{
    exchange_label, map_board, map_status, NseClient, NseIpoDetail, NseIpoSummary,
};

const REDIS_LAST_SYNC: &str = "ipos:nse:last_sync";
const REDIS_SYNC_LOCK: &str = "ipos:nse:sync_lock";
const REDIS_LIST_PREFIX: &str = "ipos:list:";

#[derive(Clone)]
pub struct IpoSyncService {
    db: PgPool,
    redis: redis::aio::ConnectionManager,
    nse: NseClient,
    running: Arc<AtomicBool>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncReport {
    pub synced: usize,
    pub details_enriched: usize,
    pub errors: Vec<String>,
    pub duration_ms: u128,
    pub source: &'static str,
}

impl IpoSyncService {
    pub fn new(db: PgPool, redis: redis::aio::ConnectionManager) -> anyhow::Result<Self> {
        Ok(Self {
            db,
            redis,
            nse: NseClient::new()?,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub async fn last_sync_at(&self) -> Option<String> {
        let mut conn = self.redis.clone();
        conn.get::<_, Option<String>>(REDIS_LAST_SYNC)
            .await
            .ok()
            .flatten()
    }

    pub async fn sync_if_stale(&self, max_age_secs: u64) -> anyhow::Result<Option<SyncReport>> {
        let mut conn = self.redis.clone();
        let last: Option<String> = conn.get(REDIS_LAST_SYNC).await.unwrap_or(None);
        let stale = match last.as_deref() {
            None => true,
            Some(ts) => chrono::DateTime::parse_from_rfc3339(ts)
                .map(|t| {
                    Utc::now()
                        .signed_duration_since(t.with_timezone(&Utc))
                        .num_seconds()
                        > max_age_secs as i64
                })
                .unwrap_or(true),
        };
        if !stale {
            return Ok(None);
        }
        Ok(Some(self.sync_now().await?))
    }

    pub async fn sync_now(&self) -> anyhow::Result<SyncReport> {
        if self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            anyhow::bail!("IPO sync already in progress");
        }

        let started = std::time::Instant::now();
        let mut errors = Vec::new();
        let result = self.sync_inner(&mut errors).await;
        self.running.store(false, Ordering::SeqCst);

        let (synced, details_enriched) = result?;
        let _ = self.invalidate_list_cache().await;
        let mut conn = self.redis.clone();
        let _: Result<(), _> = conn
            .set(REDIS_LAST_SYNC, Utc::now().to_rfc3339())
            .await;
        let _: Result<(), _> = conn
            .set_ex(REDIS_SYNC_LOCK, "0", 1u64)
            .await;

        Ok(SyncReport {
            synced,
            details_enriched,
            errors,
            duration_ms: started.elapsed().as_millis(),
            source: "nse",
        })
    }

    async fn invalidate_list_cache(&self) -> anyhow::Result<()> {
        let mut conn = self.redis.clone();
        let keys: Vec<String> = conn.keys(format!("{REDIS_LIST_PREFIX}*")).await?;
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }
        Ok(())
    }

    async fn sync_inner(&self, errors: &mut Vec<String>) -> anyhow::Result<(usize, usize)> {
        let today = Utc::now().date_naive();
        let mut summaries: Vec<(NseIpoSummary, &'static str)> = Vec::new();

        match self.nse.list_current().await {
            Ok(list) => {
                for s in list {
                    summaries.push((s, "current"));
                }
            }
            Err(e) => errors.push(format!("current: {e}")),
        }

        match self.nse.list_upcoming().await {
            Ok(list) => {
                for s in list {
                    summaries.push((s, "upcoming"));
                }
            }
            Err(e) => errors.push(format!("upcoming: {e}")),
        }

        let from = today - ChronoDuration::days(120);
        match self.nse.list_past(from, today).await {
            Ok(list) => {
                for s in list {
                    summaries.push((s, "past"));
                }
            }
            Err(e) => errors.push(format!("past: {e}")),
        }

        if summaries.is_empty() && !errors.is_empty() {
            anyhow::bail!("NSE IPO sync failed: {}", errors.join("; "));
        }

        // Deduplicate by symbol+series, preferring current > upcoming > past
        let mut seen = HashSet::new();
        let mut ordered = Vec::new();
        for bucket in ["current", "upcoming", "past"] {
            for (s, b) in &summaries {
                if *b != bucket {
                    continue;
                }
                let key = format!("{}:{}", s.symbol, s.series);
                if seen.insert(key) {
                    ordered.push((s.clone(), *b));
                }
            }
        }

        let mut synced = 0usize;
        let mut details_enriched = 0usize;

        for (summary, bucket) in ordered {
            match self.upsert_summary(&summary, bucket, today).await {
                Ok(ipo_id) => {
                    synced += 1;
                    // Enrich open/upcoming and recently closed with full detail
                    let should_detail = matches!(bucket, "current" | "upcoming")
                        || summary
                            .close_date
                            .map(|d| (today - d).num_days().abs() <= 45)
                            .unwrap_or(false);
                    if should_detail {
                        match self.nse.get_detail(&summary.symbol, &summary.series).await {
                            Ok(detail) => {
                                if let Err(e) =
                                    self.apply_detail(ipo_id, &summary, &detail).await
                                {
                                    errors.push(format!(
                                        "detail upsert {}:{}: {e}",
                                        summary.symbol, summary.series
                                    ));
                                } else {
                                    details_enriched += 1;
                                }
                            }
                            Err(e) => {
                                errors.push(format!(
                                    "detail fetch {}:{}: {e}",
                                    summary.symbol, summary.series
                                ));
                            }
                        }
                        // Be polite to NSE (~3 req/sec guidance from community clients)
                        tokio::time::sleep(Duration::from_millis(350)).await;
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "upsert {}:{}: {e}",
                        summary.symbol, summary.series
                    ));
                }
            }
        }

        Ok((synced, details_enriched))
    }

    async fn upsert_summary(
        &self,
        s: &NseIpoSummary,
        bucket: &str,
        today: chrono::NaiveDate,
    ) -> anyhow::Result<Uuid> {
        let board = map_board(&s.series);
        let status = map_status(
            &s.status_raw,
            s.open_date,
            s.close_date,
            s.listing_date,
            today,
            bucket,
        );
        let exchange = exchange_label(&s.series, s.is_bse);

        let company_id = self.ensure_company(&s.company_name, &s.symbol).await?;

        let issue_size_cr = s.issue_size_shares.and_then(|shares| {
            // Prefer computing from shares * mid price / 1e7 when prices exist
            let mid = match (s.price_band_low, s.price_band_high) {
                (Some(l), Some(h)) => Some((l + h) / Decimal::from(2)),
                (Some(l), None) | (None, Some(l)) => Some(l),
                _ => s.issue_price,
            };
            mid.map(|p| (shares * p) / Decimal::from(10_000_000i64))
        });

        let min_investment = match (s.lot_size, s.price_band_high.or(s.price_band_low).or(s.issue_price))
        {
            (Some(lot), Some(px)) => Some(Decimal::from(lot) * px),
            _ => None,
        };

        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO ipos (
                company_id, board, status, price_band_low, price_band_high, issue_price,
                lot_size, issue_size_cr, open_date, close_date, listing_date, exchange,
                subscription_total, nse_symbol, nse_series, min_investment, source,
                source_synced_at, updated_at
            ) VALUES (
                $1, $2::ipo_board, $3::ipo_status, $4, $5, $6,
                $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, 'nse',
                NOW(), NOW()
            )
            ON CONFLICT (nse_symbol, nse_series)
            DO UPDATE SET
                company_id = EXCLUDED.company_id,
                board = EXCLUDED.board,
                status = EXCLUDED.status,
                price_band_low = COALESCE(EXCLUDED.price_band_low, ipos.price_band_low),
                price_band_high = COALESCE(EXCLUDED.price_band_high, ipos.price_band_high),
                issue_price = COALESCE(EXCLUDED.issue_price, ipos.issue_price),
                lot_size = COALESCE(EXCLUDED.lot_size, ipos.lot_size),
                issue_size_cr = COALESCE(EXCLUDED.issue_size_cr, ipos.issue_size_cr),
                open_date = COALESCE(EXCLUDED.open_date, ipos.open_date),
                close_date = COALESCE(EXCLUDED.close_date, ipos.close_date),
                listing_date = COALESCE(EXCLUDED.listing_date, ipos.listing_date),
                exchange = COALESCE(EXCLUDED.exchange, ipos.exchange),
                subscription_total = COALESCE(EXCLUDED.subscription_total, ipos.subscription_total),
                min_investment = COALESCE(EXCLUDED.min_investment, ipos.min_investment),
                source_synced_at = NOW(),
                updated_at = NOW()
            RETURNING id
            "#,
        )
        .bind(company_id)
        .bind(board)
        .bind(status)
        .bind(s.price_band_low)
        .bind(s.price_band_high)
        .bind(s.issue_price)
        .bind(s.lot_size)
        .bind(issue_size_cr)
        .bind(s.open_date)
        .bind(s.close_date)
        .bind(s.listing_date)
        .bind(&exchange)
        .bind(s.subscription_total)
        .bind(&s.symbol)
        .bind(&s.series)
        .bind(min_investment)
        .fetch_one(&self.db)
        .await?;

        Ok(id)
    }

    async fn ensure_company(&self, name: &str, symbol: &str) -> anyhow::Result<Uuid> {
        let existing: Option<Uuid> = sqlx::query_scalar(
            r#"SELECT id FROM companies WHERE symbol = $1 ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(symbol)
        .fetch_optional(&self.db)
        .await?;

        if let Some(id) = existing {
            sqlx::query(r#"UPDATE companies SET name = $2 WHERE id = $1"#)
                .bind(id)
                .bind(name)
                .execute(&self.db)
                .await?;
            return Ok(id);
        }

        let id = sqlx::query_scalar(
            r#"INSERT INTO companies (name, symbol) VALUES ($1, $2) RETURNING id"#,
        )
        .bind(name)
        .bind(symbol)
        .fetch_one(&self.db)
        .await?;
        Ok(id)
    }

    async fn apply_detail(
        &self,
        ipo_id: Uuid,
        summary: &NseIpoSummary,
        d: &NseIpoDetail,
    ) -> anyhow::Result<()> {
        let company_name = d
            .company_name
            .as_ref()
            .filter(|n| {
                n.contains(' ')
                    && !n.eq_ignore_ascii_case(&summary.symbol)
                    && n.len() > summary.symbol.len()
            })
            .cloned()
            .unwrap_or_else(|| summary.company_name.clone());

        let lot = d.lot_size.or(d.min_order_qty).or(summary.lot_size);
        let low = d.price_band_low.or(summary.price_band_low);
        let high = d.price_band_high.or(summary.price_band_high);
        let min_investment = match (lot, high.or(low)) {
            (Some(l), Some(px)) => Some(Decimal::from(l) * px),
            _ => None,
        };

        let lead_managers = serde_json::Value::Array(
            d.lead_managers
                .iter()
                .map(|m| serde_json::Value::String(m.clone()))
                .collect(),
        );

        let mut financials = serde_json::Map::new();
        if let Some(url) = &d.ratios_url {
            financials.insert(
                "ratios_basis_of_issue_price_url".into(),
                serde_json::Value::String(url.clone()),
            );
        }
        if let Some(text) = &d.issue_size_text {
            financials.insert(
                "issue_size_description".into(),
                serde_json::Value::String(text.clone()),
            );
        }
        let financials = serde_json::Value::Object(financials);
        let issue_info = serde_json::Value::Object(d.issue_info.clone());

        sqlx::query(
            r#"
            UPDATE companies SET name = $2 WHERE id = (
                SELECT company_id FROM ipos WHERE id = $1
            )
            "#,
        )
        .bind(ipo_id)
        .bind(&company_name)
        .execute(&self.db)
        .await?;

        sqlx::query(
            r#"
            UPDATE ipos SET
                issue_type = COALESCE($2, issue_type),
                price_band_low = COALESCE($3, price_band_low),
                price_band_high = COALESCE($4, price_band_high),
                lot_size = COALESCE($5, lot_size),
                face_value = COALESCE($6, face_value),
                min_investment = COALESCE($7, min_investment),
                open_date = COALESCE($8, open_date),
                close_date = COALESCE($9, close_date),
                registrar = COALESCE($10, registrar),
                lead_managers = $11,
                rhp_url = COALESCE($12, rhp_url),
                subscription_total = COALESCE($13, subscription_total),
                subscription_retail = COALESCE($14, subscription_retail),
                subscription_qib = COALESCE($15, subscription_qib),
                subscription_nii = COALESCE($16, subscription_nii),
                financials = CASE
                    WHEN $17::jsonb = '{}'::jsonb THEN financials
                    ELSE financials || $17::jsonb
                END,
                issue_info = $18,
                source_synced_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(ipo_id)
        .bind(&d.issue_type)
        .bind(low)
        .bind(high)
        .bind(lot)
        .bind(d.face_value)
        .bind(min_investment)
        .bind(d.open_date)
        .bind(d.close_date)
        .bind(&d.registrar)
        .bind(lead_managers)
        .bind(&d.rhp_url)
        .bind(d.subscription_total.or(summary.subscription_total))
        .bind(d.subscription_retail)
        .bind(d.subscription_qib)
        .bind(d.subscription_nii)
        .bind(financials)
        .bind(issue_info)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

/// Spawn periodic background sync. Failures are logged; API continues serving DB cache.
pub fn spawn_background_sync(service: IpoSyncService, interval_secs: u64) {
    tokio::spawn(async move {
        // Initial sync shortly after boot
        tokio::time::sleep(Duration::from_secs(2)).await;
        match service.sync_now().await {
            Ok(r) => tracing::info!(
                synced = r.synced,
                details = r.details_enriched,
                ms = r.duration_ms,
                errors = r.errors.len(),
                "initial NSE IPO sync complete"
            ),
            Err(e) => tracing::warn!(error = %e, "initial NSE IPO sync failed"),
        }

        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs.max(60)));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            match service.sync_now().await {
                Ok(r) => tracing::info!(
                    synced = r.synced,
                    details = r.details_enriched,
                    ms = r.duration_ms,
                    errors = r.errors.len(),
                    "periodic NSE IPO sync complete"
                ),
                Err(e) => tracing::warn!(error = %e, "periodic NSE IPO sync failed"),
            }
        }
    });
}
