//! Query operations for SeaOrmStorage
//!
//! This module contains all read-only database operations.

use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::time::Instant;

#[cfg(feature = "metrics")]
use std::sync::Arc;

use chrono::Utc;
use futures_util::stream::Stream;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, ExprTrait, FromQueryResult, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, sea_query::Expr,
};
use tracing::{debug, info};

use super::{LinkFilter, SeaOrmStorage, retry};
use crate::errors::{Result, ShortlinkerError};
use crate::storage::ShortLink;
use crate::storage::models::LinkStats;

use migration::entities::short_link;

use super::converters::model_to_shortlink;

#[cfg(feature = "metrics")]
use crate::metrics::MetricsRecorder;

/// Record database query metrics
#[cfg(feature = "metrics")]
fn record_db_metrics(metrics: &Arc<dyn MetricsRecorder>, operation: &str, start: Instant) {
    let duration = start.elapsed().as_secs_f64();
    metrics.observe_db_query(operation, duration);
    metrics.inc_db_query(operation);
}

#[cfg(not(feature = "metrics"))]
fn record_db_metrics(_operation: &str, _start: Instant) {}

/// 根据 LinkFilter 构建 SeaORM 查询条件
fn build_filter_condition(filter: &LinkFilter, now: chrono::DateTime<Utc>) -> Condition {
    let mut condition = Condition::all();

    // search: 模糊匹配 code 或 target
    // 注意：SeaORM 的 contains() 会自动添加 %，不要手动拼接
    if let Some(ref search) = filter.search {
        condition = condition.add(
            Condition::any()
                .add(short_link::Column::ShortCode.contains(search))
                .add(short_link::Column::TargetUrl.contains(search)),
        );
    }

    // created_after
    if let Some(ref after) = filter.created_after {
        condition = condition.add(short_link::Column::CreatedAt.gte(*after));
    }

    // created_before
    if let Some(ref before) = filter.created_before {
        condition = condition.add(short_link::Column::CreatedAt.lte(*before));
    }

    // only_expired: 只返回已过期的
    if filter.only_expired {
        condition = condition.add(short_link::Column::ExpiresAt.is_not_null());
        condition = condition.add(short_link::Column::ExpiresAt.lt(now));
    }

    // only_active: 只返回未过期的（expires_at 为 null 或 > now）
    if filter.only_active {
        condition = condition.add(
            Condition::any()
                .add(short_link::Column::ExpiresAt.is_null())
                .add(short_link::Column::ExpiresAt.gt(now)),
        );
    }

    condition
}

/// 用于统计查询的结果结构体（DSL 聚合查询）
#[derive(Debug, FromQueryResult)]
struct StatsResult {
    total_links: i64,
    total_clicks: Option<i64>,
    active_links: Option<i64>,
}

/// Links 游标分页返回类型: (数据, 下一个游标)
pub type LinkCursorStream =
    Pin<Box<dyn Stream<Item = Result<(Vec<ShortLink>, Option<String>)>> + Send + 'static>>;

impl SeaOrmStorage {
    pub async fn get(&self, code: &str) -> Result<Option<ShortLink>> {
        let start = Instant::now();
        let db = &self.db;
        let code_owned = code.to_string();

        let result = retry::with_retry(&format!("get({})", code), self.retry_config, || async {
            short_link::Entity::find_by_id(&code_owned).one(db).await
        })
        .await
        .map_err(|e| {
            ShortlinkerError::database_operation(format!(
                "Failed to query short link (still failed after retries): {}",
                e
            ))
        })?;

        #[cfg(feature = "metrics")]
        record_db_metrics(&self.metrics, "get", start);
        #[cfg(not(feature = "metrics"))]
        record_db_metrics("get", start);
        Ok(result.map(model_to_shortlink))
    }

    pub async fn load_all(&self) -> Result<HashMap<String, ShortLink>> {
        let start = Instant::now();
        let models = short_link::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!(
                    "Failed to load all short links: {}",
                    e
                ))
            })?;

        #[cfg(feature = "metrics")]
        record_db_metrics(&self.metrics, "load_all", start);
        #[cfg(not(feature = "metrics"))]
        record_db_metrics("load_all", start);
        let count = models.len();
        let links: HashMap<String, ShortLink> = models
            .into_iter()
            .map(|model| {
                let link = model_to_shortlink(model);
                (link.code.clone(), link)
            })
            .collect();
        info!("Loaded {} short links", count);
        Ok(links)
    }

    /// 只加载所有短码（用于 Bloom Filter 初始化，内存占用小）
    pub async fn load_all_codes(&self) -> Result<Vec<String>> {
        let start = Instant::now();
        let codes = short_link::Entity::find()
            .select_only()
            .column(short_link::Column::ShortCode)
            .into_tuple::<String>()
            .all(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!(
                    "Failed to load short code list: {}",
                    e
                ))
            })?;

        #[cfg(feature = "metrics")]
        record_db_metrics(&self.metrics, "load_all_codes", start);
        #[cfg(not(feature = "metrics"))]
        record_db_metrics("load_all_codes", start);
        info!("Loaded {} short codes for Bloom filter", codes.len());
        Ok(codes)
    }

    /// 批量检查短码是否已存在（只返回已存在的短码）
    /// 用于 CSV 导入冲突检测，避免全量加载所有短码
    pub async fn batch_check_codes_exist(&self, codes: &[String]) -> Result<HashSet<String>> {
        if codes.is_empty() {
            return Ok(HashSet::new());
        }

        let mut existing = HashSet::new();
        let db = &self.db;

        // 分批查询，每批 500 个，避免 SQL IN 子句过长
        for chunk in codes.chunks(500) {
            let chunk_owned: Vec<String> = chunk.to_vec();
            let result =
                retry::with_retry("batch_check_codes_exist", self.retry_config, || async {
                    short_link::Entity::find()
                        .select_only()
                        .column(short_link::Column::ShortCode)
                        .filter(short_link::Column::ShortCode.is_in(chunk_owned.clone()))
                        .into_tuple::<String>()
                        .all(db)
                        .await
                })
                .await
                .map_err(|e| {
                    ShortlinkerError::database_operation(format!(
                        "Failed to batch-check short code existence: {}",
                        e
                    ))
                })?;

            existing.extend(result);
        }

        debug!(
            "Checked {} codes, {} already exist",
            codes.len(),
            existing.len()
        );
        Ok(existing)
    }

    /// 获取链接总数（轻量查询，用于健康检查）
    pub async fn count(&self) -> Result<u64> {
        let start = Instant::now();
        let db = &self.db;
        let result = retry::with_retry("count", self.retry_config, || async {
            short_link::Entity::find().count(db).await
        })
        .await
        .map_err(|e| ShortlinkerError::database_operation(format!("Failed to count links: {}", e)));

        #[cfg(feature = "metrics")]
        record_db_metrics(&self.metrics, "count", start);
        #[cfg(not(feature = "metrics"))]
        record_db_metrics("count", start);
        result
    }

    /// 带过滤条件的分页加载链接（带 COUNT 缓存）
    pub async fn load_paginated_filtered(
        &self,
        page: u64,
        page_size: u64,
        filter: LinkFilter,
    ) -> Result<(Vec<ShortLink>, u64)> {
        let start = Instant::now();
        let now = Utc::now();

        // 生成缓存 key（基于过滤条件）
        let cache_key = format!(
            "count:s={:?}:a={:?}:b={:?}:e={}:v={}",
            filter.search,
            filter.created_after.map(|d| d.timestamp()),
            filter.created_before.map(|d| d.timestamp()),
            filter.only_expired,
            filter.only_active
        );

        // 构建查询条件
        let condition = build_filter_condition(&filter, now);

        // 尝试从缓存获取总数
        let total = if let Some(cached) = self.count_cache.get(&cache_key) {
            debug!("count cache hit: key={}, value={}", cache_key, cached);
            cached
        } else {
            // 缓存未命中，执行 COUNT 查询（带重试）
            let db = &self.db;
            let cond = condition.clone();
            let count = retry::with_retry(
                "load_paginated_filtered(count)",
                self.retry_config,
                || async {
                    short_link::Entity::find()
                        .filter(cond.clone())
                        .count(db)
                        .await
                },
            )
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!(
                    "Pagination COUNT query failed: {}",
                    e
                ))
            })?;

            self.count_cache.insert(cache_key, count);
            count
        };

        // 执行分页数据查询（带重试）
        let db = &self.db;
        let page_offset = page.saturating_sub(1);
        let models = retry::with_retry(
            "load_paginated_filtered(data)",
            self.retry_config,
            || async {
                short_link::Entity::find()
                    .filter(condition.clone())
                    .order_by_desc(short_link::Column::CreatedAt)
                    .paginate(db, page_size)
                    .fetch_page(page_offset)
                    .await
            },
        )
        .await
        .map_err(|e| {
            ShortlinkerError::database_operation(format!("Pagination data query failed: {}", e))
        })?;

        let links: Vec<ShortLink> = models.into_iter().map(model_to_shortlink).collect();
        #[cfg(feature = "metrics")]
        record_db_metrics(&self.metrics, "paginated_query", start);
        #[cfg(not(feature = "metrics"))]
        record_db_metrics("paginated_query", start);
        Ok((links, total))
    }

    /// 批量获取链接
    pub async fn batch_get(&self, codes: &[&str]) -> Result<HashMap<String, ShortLink>> {
        if codes.is_empty() {
            return Ok(HashMap::new());
        }

        let start = Instant::now();
        let db = &self.db;
        let codes_owned: Vec<String> = codes.iter().map(|s| s.to_string()).collect();

        let models = retry::with_retry("batch_get", self.retry_config, || async {
            short_link::Entity::find()
                .filter(short_link::Column::ShortCode.is_in(codes_owned.clone()))
                .all(db)
                .await
        })
        .await
        .map_err(|e| {
            ShortlinkerError::database_operation(format!(
                "Batch query failed (still failed after retries): {}",
                e
            ))
        })?;

        #[cfg(feature = "metrics")]
        record_db_metrics(&self.metrics, "batch_get", start);
        #[cfg(not(feature = "metrics"))]
        record_db_metrics("batch_get", start);
        Ok(models
            .into_iter()
            .map(|m| {
                let link = model_to_shortlink(m);
                (link.code.clone(), link)
            })
            .collect())
    }

    /// 分页流式加载所有符合条件的链接（用于导出等大数据量场景）
    ///
    /// 使用 SeaORM paginate API 分批查询，避免一次性加载全部数据到内存。
    /// 返回 boxed stream 产生分批数据。
    pub fn stream_all_filtered_paginated(
        &self,
        filter: LinkFilter,
        page_size: u64,
    ) -> Pin<Box<dyn Stream<Item = Result<Vec<ShortLink>>> + Send + 'static>> {
        let now = Utc::now();
        let condition = build_filter_condition(&filter, now);
        let db = self.db.clone();

        use futures_util::stream;

        // 手动实现分页流
        Box::pin(stream::unfold(
            (0u64, db, condition, page_size),
            |(page, db, condition, page_size)| async move {
                // 构建查询
                let models = short_link::Entity::find()
                    .filter(condition.clone())
                    .order_by_desc(short_link::Column::CreatedAt)
                    .limit(page_size)
                    .offset(page * page_size)
                    .all(&db)
                    .await;

                match models {
                    Ok(models) if models.is_empty() => None, // 没有更多数据
                    Ok(models) => {
                        let links: Vec<ShortLink> =
                            models.into_iter().map(model_to_shortlink).collect();
                        Some((Ok(links), (page + 1, db, condition, page_size)))
                    }
                    Err(e) => {
                        // 查询失败时记录错误并终止 stream，避免跳过失败页继续下一页
                        tracing::error!("Paginated query failed at page {}: {}", page, e);
                        None
                    }
                }
            },
        ))
    }

    /// 流式导出链接（游标分页，性能更好）
    ///
    /// 使用 `short_code` 作为游标，避免 OFFSET 在大数据量下的性能问题。
    /// 返回 `(Vec<ShortLink>, Option<String>)` 的流，其中游标是最后一条记录的 `short_code`。
    pub fn stream_all_filtered_cursor(
        &self,
        filter: LinkFilter,
        page_size: u64,
    ) -> LinkCursorStream {
        let db = self.db.clone();
        let now = Utc::now();
        let condition = build_filter_condition(&filter, now);

        use futures_util::stream;

        Box::pin(stream::unfold(
            (None::<String>, db, condition, page_size, false),
            |(cursor, db, condition, page_size, done)| async move {
                if done {
                    return None;
                }

                let mut query = short_link::Entity::find().filter(condition.clone());

                // 如果有游标，从游标位置开始（不包含游标本身）
                if let Some(ref last_code) = cursor {
                    query = query.filter(short_link::Column::ShortCode.gt(last_code.clone()));
                }

                let models = query
                    .order_by_asc(short_link::Column::ShortCode)
                    .limit(page_size)
                    .all(&db)
                    .await;

                match models {
                    Ok(models) if models.is_empty() => None,
                    Ok(models) => {
                        let next_cursor = models.last().map(|m| m.short_code.clone());
                        let is_last = (models.len() as u64) < page_size;
                        let links: Vec<ShortLink> =
                            models.into_iter().map(model_to_shortlink).collect();
                        Some((
                            Ok((links, next_cursor.clone())),
                            (next_cursor, db, condition, page_size, is_last),
                        ))
                    }
                    Err(e) => {
                        tracing::error!("Cursor query failed: {}", e);
                        Some((
                            Err(ShortlinkerError::database_operation(format!(
                                "Cursor query failed: {}",
                                e
                            ))),
                            (cursor, db, condition, page_size, true),
                        ))
                    }
                }
            },
        ))
    }

    /// 获取链接统计信息（SeaORM DSL 聚合查询）
    pub async fn get_stats(&self) -> Result<LinkStats> {
        let start = Instant::now();
        let now = Utc::now();

        let result = short_link::Entity::find()
            .select_only()
            // COUNT(*) - 总链接数
            .column_as(short_link::Column::ShortCode.count(), "total_links")
            // SUM(click_count) - 总点击数
            .column_as(short_link::Column::ClickCount.sum(), "total_clicks")
            // SUM(CASE WHEN expires_at IS NULL OR expires_at > now THEN 1 ELSE 0 END) - 活跃链接数
            .column_as(
                Expr::case(
                    Condition::any()
                        .add(short_link::Column::ExpiresAt.is_null())
                        .add(short_link::Column::ExpiresAt.gt(now)),
                    1,
                )
                .finally(0)
                .sum(),
                "active_links",
            )
            .into_model::<StatsResult>()
            .one(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("Stats query failed: {}", e))
            })?;

        #[cfg(feature = "metrics")]
        record_db_metrics(&self.metrics, "get_stats", start);
        #[cfg(not(feature = "metrics"))]
        record_db_metrics("get_stats", start);
        match result {
            Some(stats) => Ok(LinkStats {
                total_links: stats.total_links as usize,
                total_clicks: stats.total_clicks.unwrap_or(0) as usize,
                active_links: stats.active_links.unwrap_or(0) as usize,
            }),
            None => Ok(LinkStats::default()),
        }
    }
}
