use dashmap::DashMap;
use std::collections::HashMap;
use std::env;
use std::fs;
use tracing::{error, info};

use async_trait::async_trait;

use super::{SerializableShortLink, ShortLink, Storage};
use crate::errors::{Result, ShortlinkerError};

// 注册 file 存储插件
// 这样子可以在应用启动时自动注册 file 存储插件
declare_storage_plugin!("file", FileStorage);

pub struct FileStorage {
    file_path: String,
    cache: DashMap<String, ShortLink>,
}

impl FileStorage {
    pub async fn new_async() -> Result<Self> {
        let file_path = env::var("DB_FILE_NAME").unwrap_or_else(|_| "links.json".to_string());
        let storage = FileStorage {
            file_path,
            cache: DashMap::new(),
        };

        // Load data into cache during initialization
        let links = storage.load_from_file()?;
        for (code, link) in links {
            storage.cache.insert(code, link);
        }
        info!(
            "FileStorage initialized with {} short links",
            storage.cache.len()
        );

        Ok(storage)
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
        self.cache.get(code).map(|entry| entry.value().clone())
    }

    async fn load_all(&self) -> HashMap<String, ShortLink> {
        self.cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    async fn set(&self, link: ShortLink) -> Result<()> {
        // 检查是否已存在，如果存在则保持原始创建时间
        let final_link = if let Some(existing_entry) = self.cache.get(&link.code) {
            ShortLink {
                code: link.code.clone(),
                target: link.target,
                created_at: existing_entry.value().created_at, // 保持原始创建时间
                expires_at: link.expires_at,
            }
        } else {
            link
        };

        // 更新缓存
        self.cache.insert(final_link.code.clone(), final_link);

        // 保存到文件
        let current_links: HashMap<String, ShortLink> = self
            .cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        self.save_to_file(&current_links)?;

        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<()> {
        // 从缓存中移除
        let removed = self.cache.remove(code).is_some();

        if !removed {
            return Err(ShortlinkerError::not_found(format!(
                "短链接不存在: {}",
                code
            )));
        }

        // 保存到文件
        let current_links: HashMap<String, ShortLink> = self
            .cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        self.save_to_file(&current_links)?;

        Ok(())
    }

    async fn reload(&self) -> Result<()> {
        match self.load_from_file() {
            Ok(new_links) => {
                self.cache.clear();
                for (code, link) in new_links {
                    self.cache.insert(code, link);
                }
                info!("Cache reloaded");
                Ok(())
            }
            Err(e) => {
                error!("Reload failed: {}", e);
                Err(e)
            }
        }
    }

    async fn get_backend_name(&self) -> String {
        "file".to_string()
    }

    async fn increment_click(&self, _code: &str) -> Result<()> {
        Ok(())
    }
}
