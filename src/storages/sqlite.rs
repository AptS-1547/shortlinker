use bloomfilter::Bloom;
use moka::future::Cache;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::{ShortLink, Storage};
use crate::errors::{Result, ShortlinkerError};
use async_trait::async_trait;

static CLI_MODE: OnceLock<bool> = OnceLock::new();

pub struct SqliteStorage {
    pool: Pool<SqliteConnectionManager>,
    cache: Cache<String, ShortLink>,
    bloom_filter: Arc<RwLock<Bloom<str>>>,
}

impl SqliteStorage {
    pub async fn new_async() -> Result<Self> {
        let cli_mode = *CLI_MODE.get_or_init(|| {
                env::var("CLI_MODE")
                    .map(|v| v == "true")
                    .unwrap_or(false)
        });

        let db_path = env::var("DB_FILE_NAME").unwrap_or_else(|_| "links.db".to_string());

        let manager = SqliteConnectionManager::file(&db_path).with_init(|c| {
            // 启用 WAL 模式以支持并发读取
            c.execute_batch(
                "PRAGMA synchronous = NORMAL;
                 PRAGMA cache_size = -64000;
                 PRAGMA temp_store = memory;
                 PRAGMA mmap_size = 536870912;
                 PRAGMA journal_mode = WAL;
                 PRAGMA wal_autocheckpoint = 1000;
                 PRAGMA busy_timeout = 5000;
                 PRAGMA optimize;",
            )?;

            // 检查并设置 WAL 模式 - 使用 query 而不是 execute
            let mut stmt = c.prepare("PRAGMA journal_mode = WAL")?;
            let current_mode: String = stmt.query_row([], |row| row.get::<_, String>(0))?;

            if current_mode.to_lowercase() != "wal" {
                return Err(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
                    Some(format!("无法设置WAL模式，当前模式: {}", current_mode)),
                ));
            }

            Ok(())
        });

        let pool = Pool::builder()
            .max_size(20)
            .min_idle(Some(8))
            .max_lifetime(Some(std::time::Duration::from_secs(1800)))
            .connection_timeout(std::time::Duration::from_secs(10))
            .build(manager)
            .map_err(|e| ShortlinkerError::database_connection(format!("无法创建连接池: {}", e)))?;

        let cache = Cache::builder()
            .max_capacity(10000) // 设置缓存容量
            .time_to_live(std::time::Duration::from_secs(60 * 15))
            .time_to_idle(std::time::Duration::from_secs(60 * 5))
            .build();

        // 先创建一个临时的 bloom filter，稍后会根据实际数据量重新创建
        let bloom_filter = Arc::new(RwLock::new(
            Bloom::new_for_fp_rate(1000, 0.01)
                .unwrap_or_else(|_| panic!("Failed to create temporary bloom filter")),
        ));

        let storage = SqliteStorage {
            pool,
            cache,
            bloom_filter,
        };

        // Initialize database tables
        storage.init_db()?;

        // 在同步构造函数中初始化异步 bloom filter：
        // 因为此方法只在程序启动时调用一次，不存在并发或 runtime 嵌套问题
        if cli_mode {
            warn!("CLI mode detected, skipping bloom filter initialization");
            return Ok(storage);
        }
        
        storage.init_bloom_filter_with_count().await?;

        warn!("SqliteStorage initialized, database path: {}", db_path);
        Ok(storage)
    }

    fn get_connection(&self) -> Result<PooledConnection<SqliteConnectionManager>> {
        self.pool.get().map_err(|e| {
            ShortlinkerError::database_connection(format!("获取数据库连接失败: {}", e))
        })
    }

    fn init_db(&self) -> Result<()> {
        let conn = self.get_connection()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS short_links (
                short_code TEXT PRIMARY KEY,
                target_url TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT
            )",
            [],
        )
        .map_err(|e| ShortlinkerError::database_operation(format!("创建表失败: {}", e)))?;

        // 为过期时间添加索引，用于快速查找过期链接
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_expires_at ON short_links(expires_at)",
            [],
        )
        .map_err(|e| {
            ShortlinkerError::database_operation(format!("创建过期时间索引失败: {}", e))
        })?;

        // 为创建时间添加索引，用于按时间排序查询
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON short_links(created_at)",
            [],
        )
        .map_err(|e| ShortlinkerError::database_operation(format!("创建时间索引失败: {}", e)))?;

        Ok(())
    }

    fn shortlink_from_row(
        short_code: String,
        target_url: String,
        created_at: String,
        expires_at: Option<String>,
    ) -> Result<ShortLink> {
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| ShortlinkerError::date_parse(format!("创建时间解析失败: {}", e)))?
            .with_timezone(&chrono::Utc);

        let expires_at = expires_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
        });

        Ok(ShortLink {
            code: short_code,
            target: target_url,
            created_at,
            expires_at,
        })
    }

    async fn init_bloom_filter_with_count(&self) -> Result<()> {
        let conn = self.get_connection()?;
        let bloom_filter = Arc::clone(&self.bloom_filter);

        let result = {
            // 先查询记录总数
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM short_links", [], |row| row.get(0))
                .map_err(|e| {
                    ShortlinkerError::database_operation(format!("查询记录总数失败: {}", e))
                })?;

            // 根据实际记录数量确定 bloom filter 容量
            // 至少设置为 1000，为未来增长预留空间
            let capacity = std::cmp::max(count as usize * 2, 1000);

            info!(
                "Database has {} records, creating bloom filter with capacity {}",
                count, capacity
            );

            // 查询所有短码
            let mut stmt = conn
                .prepare("SELECT short_code FROM short_links")
                .map_err(|e| {
                    ShortlinkerError::database_operation(format!("准备查询语句失败: {}", e))
                })?;
            let rows = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| {
                    ShortlinkerError::database_operation(format!("查询短码失败: {}", e))
                })?;

            let mut codes = Vec::new();
            for row in rows {
                match row {
                    Ok(code) => codes.push(code),
                    Err(e) => error!("Failed to read short code: {}", e),
                }
            }

            Ok((capacity, codes))
        };

        match result {
            Ok((capacity, codes)) => {
                // 创建新的 bloom filter
                let new_bloom = Bloom::new_for_fp_rate(capacity, 0.01).unwrap_or_else(|_| {
                    warn!(
                        "Failed to create bloom filter with capacity {}, using default",
                        capacity
                    );
                    Bloom::new_for_fp_rate(10000, 0.01)
                        .unwrap_or_else(|_| panic!("Failed to create fallback bloom filter"))
                });

                // 替换原有的 bloom filter 并加载数据
                {
                    let mut bloom = bloom_filter.write().await;
                    *bloom = new_bloom;

                    for code in &codes {
                        bloom.set(code);
                    }
                }

                info!(
                    "Bloom filter initialized with {} short codes, capacity: {}",
                    codes.len(),
                    capacity
                );
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        // 先检查缓存
        if let Some(link) = self.cache.get(code).await {
            return Some(link);
        }

        // 使用 bloom filter 进行快速预检查
        {
            if *CLI_MODE.get().unwrap_or(&false) {
                warn!("CLI mode detected, skipping bloom filter check");
                return None;
            } else {
                let bloom = self.bloom_filter.read().await;
                if !bloom.check(code) {
                    // bloom filter 确定不存在，直接返回 None
                    return None;
                }
            }
        }

        // Cache miss, query database
        let conn = match self.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get database connection: {}", e);
                return None;
            }
        };

        let code = code.to_string();
        let cache = self.cache.clone();

        let result = actix_web::web::block(move || {
            let mut stmt = conn.prepare(
                "SELECT short_code, target_url, created_at, expires_at FROM short_links WHERE short_code = ?1"
            )?;

            stmt.query_row(params![code], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })
        }).await;

        match result {
            Ok(Ok((short_code, target_url, created_at, expires_at))) => {
                match Self::shortlink_from_row(
                    short_code.clone(),
                    target_url,
                    created_at,
                    expires_at,
                ) {
                    Ok(link) => {
                        // Store result into cache
                        cache.insert(short_code, link.clone()).await;
                        Some(link)
                    }
                    Err(e) => {
                        error!("Failed to parse short link data: {}", e);
                        None
                    }
                }
            }
            Ok(Err(rusqlite::Error::QueryReturnedNoRows)) => None,
            Ok(Err(e)) => {
                error!("Query short link failed: {}", e);
                None
            }
            Err(e) => {
                error!("Async query failed: {:?}", e);
                None
            }
        }
    }

    // NOTE: load_all 不更新 bloom filter，因为主要用于非 HTTP 路径（例如 CLI export）
    // 如果未来在 get() 中依赖它，需要手动触发 reload() 逻辑
    async fn load_all(&self) -> HashMap<String, ShortLink> {
        let conn = match self.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get database connection: {}", e);
                return HashMap::new();
            }
        };

        let mut links = HashMap::new();

        let mut stmt = match conn
            .prepare("SELECT short_code, target_url, created_at, expires_at FROM short_links")
        {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("Failed to prepare query statement: {}", e);
                return links;
            }
        };

        let rows = match stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        }) {
            Ok(rows) => rows,
            Err(e) => {
                error!("Failed to query all short links: {}", e);
                return links;
            }
        };

        for row in rows {
            match row {
                Ok((short_code, target_url, created_at, expires_at)) => {
                    match Self::shortlink_from_row(
                        short_code.clone(),
                        target_url,
                        created_at,
                        expires_at,
                    ) {
                        Ok(link) => {
                            links.insert(short_code, link);
                        }
                        Err(e) => {
                            error!("Failed to parse short link data: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read row data: {}", e);
                }
            }
        }

        info!("Loaded {} short links", links.len());
        links
    }

    async fn set(&self, link: ShortLink) -> Result<()> {
        let pool = self.pool.clone();
        let cache = self.cache.clone();
        let bloom_filter = Arc::clone(&self.bloom_filter);
        let link_clone = link.clone();

        let result = actix_web::web::block(move || {
            let mut conn = pool.get().map_err(|e| {
                ShortlinkerError::database_connection(format!("获取数据库连接失败: {}", e))
            })?;

            let created_at = link_clone.created_at.to_rfc3339();
            let expires_at = link_clone.expires_at.map(|dt| dt.to_rfc3339());

            // 开始事务
            let transaction = conn.transaction().map_err(|e| {
                ShortlinkerError::database_operation(format!("开始事务失败: {}", e))
            })?;

            // 先检查记录是否存在
            let exists = {
                let mut stmt = transaction
                    .prepare("SELECT 1 FROM short_links WHERE short_code = ?1")
                    .map_err(|e| {
                        ShortlinkerError::database_operation(format!("准备查询语句失败: {}", e))
                    })?;

                stmt.exists(params![link_clone.code]).map_err(|e| {
                    ShortlinkerError::database_operation(format!("检查记录存在性失败: {}", e))
                })?
            };

            let result = if exists {
                transaction
                    .execute(
                        "UPDATE short_links SET target_url = ?2, expires_at = ?3 
                     WHERE short_code = ?1",
                        params![link_clone.code, link_clone.target, expires_at],
                    )
                    .map_err(|e| {
                        ShortlinkerError::database_operation(format!("更新短链接失败: {}", e))
                    })
                    .map(|_| {
                        info!("Short link updated: {}", link_clone.code);
                    })
            } else {
                transaction
                    .execute(
                        "INSERT INTO short_links (short_code, target_url, created_at, expires_at) 
                     VALUES (?1, ?2, ?3, ?4)",
                        params![link_clone.code, link_clone.target, created_at, expires_at],
                    )
                    .map_err(|e| {
                        ShortlinkerError::database_operation(format!("插入短链接失败: {}", e))
                    })
                    .map(|_| {
                        info!("Short link created: {}", link_clone.code);
                    })
            };

            // 处理操作结果
            match result {
                Ok(_) => {
                    // 提交事务
                    transaction.commit().map_err(|e| {
                        ShortlinkerError::database_operation(format!("提交事务失败: {}", e))
                    })?;
                    Ok(())
                }
                Err(e) => {
                    // 事务会自动回滚
                    Err(e)
                }
            }
        })
        .await;

        match result {
            Ok(Ok(_)) => {
                // 更新缓存
                cache.insert(link.code.clone(), link.clone()).await;

                // 将新的短码添加到 bloom filter
                {
                    let mut bloom = bloom_filter.write().await;
                    bloom.set(link.code.as_str());
                }

                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(e) => Err(ShortlinkerError::database_operation(format!(
                "执行异步操作失败: {:?}",
                e
            ))),
        }
    }

    async fn remove(&self, code: &str) -> Result<()> {
        let pool = self.pool.clone();
        let cache = self.cache.clone();
        let code = code.to_string();

        let result = actix_web::web::block(move || {
            let mut conn = pool.get().map_err(|e| {
                ShortlinkerError::database_connection(format!("获取数据库连接失败: {}", e))
            })?;

            // 开始事务
            let transaction = conn.transaction().map_err(|e| {
                ShortlinkerError::database_operation(format!("开始事务失败: {}", e))
            })?;

            let rows_affected = transaction
                .execute(
                    "DELETE FROM short_links WHERE short_code = ?1",
                    params![code],
                )
                .map_err(|e| {
                    ShortlinkerError::database_operation(format!("删除短链接失败: {}", e))
                })?;

            if rows_affected == 0 {
                // 事务会自动回滚
                return Err(ShortlinkerError::not_found(format!(
                    "短链接不存在: {}",
                    code
                )));
            }

            // 提交事务
            transaction.commit().map_err(|e| {
                ShortlinkerError::database_operation(format!("提交事务失败: {}", e))
            })?;

            info!("Short link deleted: {}", code);
            Ok(code)
        })
        .await;

        match result {
            Ok(Ok(deleted_code)) => {
                // 从缓存中移除
                cache.remove(&deleted_code).await;
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(e) => Err(ShortlinkerError::database_operation(format!(
                "执行异步操作失败: {:?}",
                e
            ))),
        }
    }

    async fn reload(&self) -> Result<()> {
        // Clear cache to force reload from database
        self.cache.invalidate_all();

        // 重新初始化 bloom filter
        self.init_bloom_filter_with_count().await?;

        info!("SQLite cache and bloom filter cleared, data reloaded from database");
        Ok(())
    }

    async fn get_backend_name(&self) -> String {
        "sqlite".to_string()
    }

    async fn increment_click(&self, _code: &str) -> Result<()> {
        Ok(())
    }
}
