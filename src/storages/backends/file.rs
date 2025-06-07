use std::collections::HashMap;
use std::env;
use std::fs;
use tracing::{error, info};

use async_trait::async_trait;

use super::{CachePreference, SerializableShortLink, ShortLink, Storage};
use crate::errors::{Result, ShortlinkerError};
use crate::storages::models::StorageConfig;

// 注册 file 存储插件
// 这样子可以在应用启动时自动注册 file 存储插件
declare_storage_plugin!("file", FileStorage);

pub struct FileStorage {
    file_path: String,
}

impl FileStorage {
    pub async fn new_async() -> Result<Self> {
        let file_path = env::var("DB_FILE_NAME").unwrap_or_else(|_| "links.json".to_string());

        // 如果不存在就初始化
        if fs::read_to_string(&file_path).is_err() {
            fs::write(&file_path, "[]")?;
        }

        Ok(FileStorage { file_path })
    }

    fn load_from_file(&self) -> Result<HashMap<String, ShortLink>> {
        match fs::read_to_string(&self.file_path) {
            Ok(content) => match serde_json::from_str::<Vec<SerializableShortLink>>(&content) {
                Ok(links) => {
                    let mut map = HashMap::new();
                    for link in links {
                        let created_at = chrono::DateTime::parse_from_rfc3339(&link.created_at)
                            .unwrap_or_else(|_| chrono::Utc::now().into())
                            .with_timezone(&chrono::Utc);

                        let expires_at = link.expires_at.and_then(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .map(|dt| dt.with_timezone(&chrono::Utc))
                                .ok()
                        });

                        map.insert(
                            link.short_code.clone(),
                            ShortLink {
                                code: link.short_code,
                                target: link.target_url,
                                created_at,
                                expires_at,
                            },
                        );
                    }
                    info!("Loaded {} short links", map.len());
                    Ok(map)
                }
                Err(e) => {
                    error!("Failed to parse link file: {}", e);
                    Err(ShortlinkerError::serialization(format!(
                        "Failed to parse link file: {}",
                        e
                    )))
                }
            },
            Err(_) => {
                info!("Link file not found, creating empty storage");
                if let Err(e) = fs::write(&self.file_path, "[]") {
                    error!("Failed to create link file: {}", e);
                    return Err(ShortlinkerError::file_operation(format!(
                        "Failed to create link file: {}",
                        e
                    )));
                }
                info!("Created empty link file: {}", self.file_path);
                Ok(HashMap::new())
            }
        }
    }

    fn save_to_file(&self, links: &HashMap<String, ShortLink>) -> Result<()> {
        let links_vec: Vec<SerializableShortLink> = links
            .iter()
            .map(|(_, link)| SerializableShortLink {
                short_code: link.code.clone(),
                target_url: link.target.clone(),
                created_at: link.created_at.to_rfc3339(),
                expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
                click: 0, // 点击量暂时不存储
            })
            .collect();

        let json = serde_json::to_string_pretty(&links_vec)?;
        fs::write(&self.file_path, json)?;
        Ok(())
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        match self.load_from_file() {
            Ok(links) => links.get(code).cloned(),
            Err(e) => {
                error!("Failed to load links from file: {}", e);
                None
            }
        }
    }

    async fn load_all(&self) -> HashMap<String, ShortLink> {
        // 从文件加载所有链接
        match self.load_from_file() {
            Ok(links) => links,
            Err(e) => {
                error!("Failed to load links from file: {}", e);
                HashMap::new()
            }
        }
    }

    async fn set(&self, link: ShortLink) -> Result<()> {
        // 检查是否已存在，如果存在则保持原始创建时间
        match self.load_from_file() {
            Ok(mut links) => {
                if let Some(existing_link) = links.get(&link.code) {
                    // 如果已存在，保持原始创建时间
                    let created_at = existing_link.created_at;
                    links.insert(
                        link.code.clone(),
                        ShortLink {
                            code: link.code,
                            target: link.target,
                            created_at,
                            expires_at: link.expires_at,
                        },
                    );
                } else {
                    // 如果不存在，使用当前时间作为创建时间
                    links.insert(link.code.clone(), link);
                }

                // 保存到文件
                self.save_to_file(&links)?;
            }
            Err(e) => {
                error!("Failed to load links from file: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<()> {
        // 从缓存中移除
        match self.load_from_file() {
            Ok(mut links) => {
                if links.remove(code).is_some() {
                    // 如果成功移除，保存到文件
                    self.save_to_file(&links)?;
                    info!("Removed link with code: {}", code);
                    Ok(())
                } else {
                    error!("Link with code {} not found", code);
                    Err(ShortlinkerError::not_found(format!(
                        "Link with code {} not found",
                        code
                    )))
                }
            }
            Err(e) => {
                error!("Failed to load links from file: {}", e);
                Err(e)
            }
        }
    }

    async fn reload(&self) -> Result<()> {
        // 重新加载所有链接到内存
        match self.load_from_file() {
            Ok(links) => {
                info!("Reloaded {} short links from file", links.len());
                Ok(())
            }
            Err(e) => {
                error!("Failed to reload links from file: {}", e);
                Err(e)
            }
        }
    }

    async fn get_backend_config(&self) -> StorageConfig {
        StorageConfig {
            storage_type: "file".into(),
            support_click: true,
        }
    }

    fn increment_click(&self, _code: &str) -> Result<()> {
        Ok(())
    }

    fn preferred_cache(&self) -> CachePreference {
        CachePreference {
            l1: "null".into(),
            l2: "memory".into(),
        }
    }
}
