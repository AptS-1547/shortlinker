use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::{Arc, RwLock};

use super::{ShortLink, Storage};
use async_trait::async_trait;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SerializableShortLink {
    short_code: String,
    target_url: String,
    created_at: String,
    expires_at: Option<String>,
}

pub struct FileStorage {
    file_path: String,
    cache: Arc<RwLock<HashMap<String, ShortLink>>>,
}

impl FileStorage {
    pub fn new() -> Self {
        let file_path = env::var("LINKS_FILE").unwrap_or_else(|_| "links.json".to_string());
        let storage = FileStorage {
            file_path,
            cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // 初始化时加载数据到缓存
        if let Ok(links) = storage.load_from_file() {
            let mut cache_guard = storage.cache.write().unwrap();
            *cache_guard = links;
            info!(
                "FileStorage 初始化完成，已加载 {} 个短链接",
                cache_guard.len()
            );
        }

        storage
    }

    fn load_from_file(&self) -> Result<HashMap<String, ShortLink>, String> {
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
                    info!("已加载 {} 个短链接", map.len());
                    Ok(map)
                }
                Err(e) => {
                    error!("解析链接文件失败: {}", e);
                    Err(format!("解析链接文件失败: {}", e))
                }
            },
            Err(_) => {
                info!("链接文件不存在，创建空的存储");
                if let Err(e) = fs::write(&self.file_path, "[]") {
                    error!("创建链接文件失败: {}", e);
                    return Err(format!("创建链接文件失败: {}", e));
                }
                info!("已创建空的链接文件: {}", self.file_path);
                Ok(HashMap::new())
            }
        }
    }

    fn save_to_file(&self, links: &HashMap<String, ShortLink>) -> Result<(), String> {
        let links_vec: Vec<SerializableShortLink> = links
            .iter()
            .map(|(_, link)| SerializableShortLink {
                short_code: link.code.clone(),
                target_url: link.target.clone(),
                created_at: link.created_at.to_rfc3339(),
                expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            })
            .collect();

        let json =
            serde_json::to_string_pretty(&links_vec).map_err(|e| format!("序列化失败: {}", e))?;

        fs::write(&self.file_path, json).map_err(|e| format!("写入文件失败: {}", e))?;

        Ok(())
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        let cache_guard = self.cache.read().unwrap();
        cache_guard.get(code).cloned()
    }

    async fn load_all(&self) -> HashMap<String, ShortLink> {
        let cache_guard = self.cache.read().unwrap();
        cache_guard.clone()
    }

    async fn set(&self, link: ShortLink) -> Result<(), String> {
        // 更新缓存
        {
            let mut cache_guard = self.cache.write().unwrap();
            cache_guard.insert(link.code.clone(), link);
        }

        // 保存到文件
        let cache_guard = self.cache.read().unwrap();
        self.save_to_file(&cache_guard)?;

        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<(), String> {
        // 从缓存中移除
        let removed = {
            let mut cache_guard = self.cache.write().unwrap();
            cache_guard.remove(code).is_some()
        };

        if !removed {
            return Err(format!("短链接不存在: {}", code));
        }

        // 保存到文件
        let cache_guard = self.cache.read().unwrap();
        self.save_to_file(&cache_guard)?;

        Ok(())
    }

    async fn reload(&self) -> Result<(), String> {
        match self.load_from_file() {
            Ok(new_links) => {
                let mut cache_guard = self.cache.write().unwrap();
                *cache_guard = new_links;
                info!("缓存重载完成");
                Ok(())
            }
            Err(e) => {
                error!("重载失败: {}", e);
                Err(e)
            }
        }
    }
}
