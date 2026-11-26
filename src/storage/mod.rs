use std::sync::Arc;

use crate::errors::Result;
use crate::metrics::MetricsRecorder;

/// SeaORM 存储后端：连接管理、查询与写入实现
pub mod backend;
/// 运行时配置的持久化存储与历史记录
pub mod config_store;
/// 短链接等实体的数据模型
pub mod models;

pub use backend::{LinkFilter, SeaOrmStorage};
pub use config_store::{ConfigHistoryEntry, ConfigItem, ConfigStore, ConfigUpdateResult};
pub use models::{LinkStats, ShortLink};

/// 存储工厂：根据配置的 database URL 创建对应的存储实例
pub struct StorageFactory;

impl StorageFactory {
    /// 从配置读取 database URL 并推断后端类型，建立带迁移的存储连接
    pub async fn create(metrics: Arc<dyn MetricsRecorder>) -> Result<Arc<SeaOrmStorage>> {
        let config = crate::config::get_config();
        let database_url = &config.database.database_url;

        // 从 URL 自动推断数据库类型
        let backend_type = backend::infer_backend_from_url(database_url)?;

        let storage = backend::SeaOrmStorage::new(database_url, &backend_type, metrics).await?;
        Ok(Arc::new(storage))
    }
}
