//! Query operations for SeaOrmStorage
//!
//! This module contains all read-only database operations.

use std::collections::HashMap;

use chrono::Utc;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, ExprTrait, FromQueryResult, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, sea_query::Expr,
};
use tracing::{debug, error, info};

use super::{LinkFilter, SeaOrmStorage, retry};
use crate::storage::ShortLink;
use crate::storage::models::LinkStats;

use migration::entities::short_link;

use super::converters::model_to_shortlink;

/// 用于统计查询的结果结构体（DSL 聚合查询）
#[derive(Debug, FromQueryResult)]
struct StatsResult {
    total_links: i64,
    total_clicks: Option<i64>,
    active_links: Option<i64>,
}

impl SeaOrmStorage {
    pub async fn get(&self, code: &str) -> Option<ShortLink> {
        let db = &self.db;
        let code_owned = code.to_string();

        let result = retry::with_retry(&format!("get({})", code), self.retry_config, || async {
            short_link::Entity::find_by_id(&code_owned).one(db).await
        })
        .await;

        match result {
            Ok(Some(model)) => Some(model_to_shortlink(model)),
            Ok(None) => None,
            Err(e) => {
                error!("查询短链接失败（重试后仍失败）: {}", e);
                None
            }
        }
    }

    pub async fn load_all(&self) -> HashMap<String, ShortLink> {
        match short_link::Entity::find().all(&self.db).await {
            Ok(models) => {
                let count = models.len();
                let links: HashMap<String, ShortLink> = models
                    .into_iter()
                    .map(|model| {
                        let link = model_to_shortlink(model);
                        (link.code.clone(), link)
                    })
                    .collect();
                info!("Loaded {} short links", count);
                links
            }
            Err(e) => {
                error!("加载所有短链接失败: {}", e);
                HashMap::new()
            }
        }
    }

    /// 只加载所有短码（用于 Bloom Filter 初始化，内存占用小）
    pub async fn load_all_codes(&self) -> Vec<String> {
        match short_link::Entity::find()
            .select_only()
            .column(short_link::Column::ShortCode)
            .into_tuple::<String>()
            .all(&self.db)
            .await
        {
            Ok(codes) => {
                info!("Loaded {} short codes for Bloom filter", codes.len());
                codes
            }
            Err(e) => {
                error!("加载短码列表失败: {}", e);
                Vec::new()
            }
        }
    }

    /// 带过滤条件的分页加载链接（带 COUNT 缓存）
    pub async fn load_paginated_filtered(
        &self,
        page: u64,
        page_size: u64,
        filter: LinkFilter,
    ) -> (Vec<ShortLink>, u64) {
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
        let mut condition = Condition::all();

        // search: 模糊匹配 code 或 target
        if let Some(ref search) = filter.search {
            let pattern = format!("%{}%", search);
            condition = condition.add(
                Condition::any()
                    .add(short_link::Column::ShortCode.contains(&pattern))
                    .add(short_link::Column::TargetUrl.contains(&pattern)),
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

        // 尝试从缓存获取总数
        let total = if let Some(cached) = self.count_cache.get(&cache_key) {
            debug!("count cache hit: key={}, value={}", cache_key, cached);
            cached
        } else {
            // 缓存未命中，执行 COUNT 查询（带重试）
            let db = &self.db;
            let cond = condition.clone();
            let count_result = retry::with_retry(
                "load_paginated_filtered(count)",
                self.retry_config,
                || async {
                    short_link::Entity::find()
                        .filter(cond.clone())
                        .count(db)
                        .await
                },
            )
            .await;

            let count = count_result.unwrap_or(0);
            self.count_cache.insert(cache_key, count);
            count
        };

        // 执行分页数据查询（带重试）
        let db = &self.db;
        let page_offset = page.saturating_sub(1);
        let models_result = retry::with_retry(
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
        .await;

        let models = match models_result {
            Ok(models) => models,
            Err(e) => {
                error!("分页查询失败（重试后仍失败）: {}", e);
                return (Vec::new(), total);
            }
        };

        let links: Vec<ShortLink> = models.into_iter().map(model_to_shortlink).collect();
        (links, total)
    }

    /// 批量获取链接
    pub async fn batch_get(&self, codes: &[&str]) -> HashMap<String, ShortLink> {
        if codes.is_empty() {
            return HashMap::new();
        }

        let db = &self.db;
        let codes_owned: Vec<String> = codes.iter().map(|s| s.to_string()).collect();

        let result = retry::with_retry("batch_get", self.retry_config, || async {
            short_link::Entity::find()
                .filter(short_link::Column::ShortCode.is_in(codes_owned.clone()))
                .all(db)
                .await
        })
        .await;

        match result {
            Ok(models) => models
                .into_iter()
                .map(|m| {
                    let link = model_to_shortlink(m);
                    (link.code.clone(), link)
                })
                .collect(),
            Err(e) => {
                error!("批量查询失败（重试后仍失败）: {}", e);
                HashMap::new()
            }
        }
    }

    /// 带过滤条件加载所有链接（不分页，用于导出）
    pub async fn load_all_filtered(&self, filter: LinkFilter) -> Vec<ShortLink> {
        let now = Utc::now();

        // 构建查询条件（复用 load_paginated_filtered 的逻辑）
        let mut condition = Condition::all();

        if let Some(ref search) = filter.search {
            let pattern = format!("%{}%", search);
            condition = condition.add(
                Condition::any()
                    .add(short_link::Column::ShortCode.contains(&pattern))
                    .add(short_link::Column::TargetUrl.contains(&pattern)),
            );
        }

        if let Some(ref after) = filter.created_after {
            condition = condition.add(short_link::Column::CreatedAt.gte(*after));
        }

        if let Some(ref before) = filter.created_before {
            condition = condition.add(short_link::Column::CreatedAt.lte(*before));
        }

        if filter.only_expired {
            condition = condition.add(short_link::Column::ExpiresAt.is_not_null());
            condition = condition.add(short_link::Column::ExpiresAt.lt(now));
        }

        if filter.only_active {
            condition = condition.add(
                Condition::any()
                    .add(short_link::Column::ExpiresAt.is_null())
                    .add(short_link::Column::ExpiresAt.gt(now)),
            );
        }

        match short_link::Entity::find()
            .filter(condition)
            .order_by_desc(short_link::Column::CreatedAt)
            .all(&self.db)
            .await
        {
            Ok(models) => models.into_iter().map(model_to_shortlink).collect(),
            Err(e) => {
                error!("加载过滤链接失败: {}", e);
                Vec::new()
            }
        }
    }

    /// 获取链接统计信息（SeaORM DSL 聚合查询）
    pub async fn get_stats(&self) -> LinkStats {
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
            .await;

        match result {
            Ok(Some(stats)) => LinkStats {
                total_links: stats.total_links as usize,
                total_clicks: stats.total_clicks.unwrap_or(0) as usize,
                active_links: stats.active_links.unwrap_or(0) as usize,
            },
            Ok(None) => {
                error!("统计查询返回空结果");
                LinkStats::default()
            }
            Err(e) => {
                error!("统计查询失败: {}", e);
                LinkStats::default()
            }
        }
    }
}
