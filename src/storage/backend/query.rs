//! Query operations for SeaOrmStorage
//!
//! This module contains all read-only database operations.

use std::collections::HashMap;

use chrono::Utc;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use tracing::{error, info};

use super::{LinkFilter, SeaOrmStorage};
use crate::storage::ShortLink;
use crate::storage::models::LinkStats;

use migration::entities::short_link;

use super::converters::model_to_shortlink;

/// 用于 SUM 聚合查询的结果结构体
#[derive(Debug, FromQueryResult)]
pub(super) struct ClickSum {
    pub sum: Option<i64>,
}

impl SeaOrmStorage {
    pub async fn get(&self, code: &str) -> Option<ShortLink> {
        let result = short_link::Entity::find_by_id(code).one(&self.db).await;

        match result {
            Ok(Some(model)) => Some(model_to_shortlink(model)),
            Ok(None) => None,
            Err(e) => {
                error!("查询短链接失败: {}", e);
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

    /// 分页加载链接
    pub async fn load_paginated(&self, page: u64, page_size: u64) -> (Vec<ShortLink>, u64) {
        let paginator = short_link::Entity::find()
            .order_by_desc(short_link::Column::CreatedAt)
            .paginate(&self.db, page_size);

        let total = match paginator.num_items().await {
            Ok(count) => count,
            Err(e) => {
                error!("获取总数失败: {}", e);
                return (Vec::new(), 0);
            }
        };

        let models = match paginator.fetch_page(page.saturating_sub(1)).await {
            Ok(models) => models,
            Err(e) => {
                error!("分页查询失败: {}", e);
                return (Vec::new(), total);
            }
        };

        let links: Vec<ShortLink> = models.into_iter().map(model_to_shortlink).collect();

        (links, total)
    }

    /// 带过滤条件的分页加载链接
    pub async fn load_paginated_filtered(
        &self,
        page: u64,
        page_size: u64,
        filter: LinkFilter,
    ) -> (Vec<ShortLink>, u64) {
        let now = Utc::now();

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

        let paginator = short_link::Entity::find()
            .filter(condition)
            .order_by_desc(short_link::Column::CreatedAt)
            .paginate(&self.db, page_size);

        let total = match paginator.num_items().await {
            Ok(count) => count,
            Err(e) => {
                error!("获取总数失败: {}", e);
                return (Vec::new(), 0);
            }
        };

        let models = match paginator.fetch_page(page.saturating_sub(1)).await {
            Ok(models) => models,
            Err(e) => {
                error!("分页查询失败: {}", e);
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

        let result = short_link::Entity::find()
            .filter(short_link::Column::ShortCode.is_in(codes.iter().map(|s| s.to_string())))
            .all(&self.db)
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
                error!("批量查询失败: {}", e);
                HashMap::new()
            }
        }
    }

    /// 获取链接统计信息
    pub async fn get_stats(&self) -> LinkStats {
        let now = Utc::now();

        // 获取总链接数
        let total_links = short_link::Entity::find()
            .count(&self.db)
            .await
            .unwrap_or(0) as usize;

        // 获取总点击数（使用数据库聚合）
        let all_clicks: i64 = match short_link::Entity::find()
            .select_only()
            .column_as(short_link::Column::ClickCount.sum(), "sum")
            .into_model::<ClickSum>()
            .one(&self.db)
            .await
        {
            Ok(Some(result)) => result.sum.unwrap_or(0),
            Ok(None) | Err(_) => 0,
        };

        // 获取活跃链接数（未过期的）
        let active_links = short_link::Entity::find()
            .filter(
                Condition::any()
                    .add(short_link::Column::ExpiresAt.is_null())
                    .add(short_link::Column::ExpiresAt.gt(now)),
            )
            .count(&self.db)
            .await
            .unwrap_or(0) as usize;

        LinkStats {
            total_links,
            total_clicks: all_clicks as usize,
            active_links,
        }
    }
}
