use log::{error, info};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};

use super::{ShortLink, Storage};
use async_trait::async_trait;

pub struct SqliteStorage {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteStorage {
    pub fn new() -> Result<Self, String> {
        let db_path = env::var("LINKS_FILE").unwrap_or_else(|_| "links.db".to_string());

        let conn = Connection::open(&db_path).map_err(|e| format!("无法打开数据库: {}", e))?;

        let storage = SqliteStorage {
            connection: Arc::new(Mutex::new(conn)),
        };

        // 初始化数据库表
        storage.init_db()?;

        info!("SqliteStorage 初始化完成，数据库路径: {}", db_path);
        Ok(storage)
    }

    fn init_db(&self) -> Result<(), String> {
        let conn = self.connection.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS short_links (
                short_code TEXT PRIMARY KEY,
                target_url TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT
            )",
            [],
        )
        .map_err(|e| format!("创建表失败: {}", e))?;

        Ok(())
    }

    fn shortlink_from_row(
        short_code: String,
        target_url: String,
        created_at: String,
        expires_at: Option<String>,
    ) -> Result<ShortLink, String> {
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
            .unwrap_or_else(|_| chrono::Utc::now().into())
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
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        let conn = self.connection.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT short_code, target_url, created_at, expires_at FROM short_links WHERE short_code = ?1"
        ).ok()?;

        let result = stmt.query_row(params![code], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        });

        match result {
            Ok((short_code, target_url, created_at, expires_at)) => {
                match Self::shortlink_from_row(short_code, target_url, created_at, expires_at) {
                    Ok(link) => Some(link),
                    Err(e) => {
                        error!("解析短链接数据失败: {}", e);
                        None
                    }
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => {
                error!("查询短链接失败: {}", e);
                None
            }
        }
    }

    async fn load_all(&self) -> HashMap<String, ShortLink> {
        let conn = self.connection.lock().unwrap();
        let mut links = HashMap::new();

        let mut stmt = match conn
            .prepare("SELECT short_code, target_url, created_at, expires_at FROM short_links")
        {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("准备查询语句失败: {}", e);
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
                error!("查询所有短链接失败: {}", e);
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
                            error!("解析短链接数据失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("读取行数据失败: {}", e);
                }
            }
        }

        info!("已加载 {} 个短链接", links.len());
        links
    }

    async fn set(&self, link: ShortLink) -> Result<(), String> {
        let conn = self.connection.lock().unwrap();

        let created_at = link.created_at.to_rfc3339();
        let expires_at = link.expires_at.map(|dt| dt.to_rfc3339());

        // 先检查记录是否存在
        let exists = {
            let mut stmt = conn.prepare("SELECT 1 FROM short_links WHERE short_code = ?1")
                .map_err(|e| format!("准备查询语句失败: {}", e))?;
            
            stmt.exists(params![link.code])
                .map_err(|e| format!("检查记录存在性失败: {}", e))?
        };

        if exists {
            conn.execute(
                "UPDATE short_links SET target_url = ?2, expires_at = ?3 
                 WHERE short_code = ?1",
                params![link.code, link.target, expires_at],
            )
            .map_err(|e| format!("更新短链接失败: {}", e))?;
            
            info!("短链接已更新: {}", link.code);
        } else {
            conn.execute(
                "INSERT INTO short_links (short_code, target_url, created_at, expires_at) 
                 VALUES (?1, ?2, ?3, ?4)",
                params![link.code, link.target, created_at, expires_at],
            )
            .map_err(|e| format!("插入短链接失败: {}", e))?;
            
            info!("短链接已创建: {}", link.code);
        }

        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<(), String> {
        let conn = self.connection.lock().unwrap();

        let rows_affected = conn
            .execute(
                "DELETE FROM short_links WHERE short_code = ?1",
                params![code],
            )
            .map_err(|e| format!("删除短链接失败: {}", e))?;

        if rows_affected == 0 {
            return Err(format!("短链接不存在: {}", code));
        }

        info!("短链接已删除: {}", code);
        Ok(())
    }

    async fn reload(&self) -> Result<(), String> {
        Ok(())
    }
}
