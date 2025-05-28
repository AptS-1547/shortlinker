use std::collections::HashMap;
use std::env;
use std::fs;
use log::{error, info};
use serde::{Deserialize, Serialize};

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
}

impl FileStorage {
    pub fn new() -> Self {
        let file_path = env::var("LINKS_FILE").unwrap_or_else(|_| "links.json".to_string());
        FileStorage { file_path }
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        let links = self.load_all().await;
        links.get(code).cloned()
    }

    async fn load_all(&self) -> HashMap<String, ShortLink> {
        match fs::read_to_string(&self.file_path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<SerializableShortLink>>(&content) {
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

                            map.insert(link.short_code.clone(), ShortLink {
                                code: link.short_code,
                                target: link.target_url,
                                created_at,
                                expires_at,
                            });
                        }
                        info!("加载了 {} 个短链接", map.len());
                        map
                    }
                    Err(e) => {
                        error!("解析链接文件失败: {}", e);
                        HashMap::new()
                    }
                }
            }
            Err(_) => {
                info!("链接文件不存在，创建空的存储");
                HashMap::new()
            }
        }
    }

    async fn set(&self, link: ShortLink) -> Result<(), String> {
        let mut links = self.load_all().await;
        links.insert(link.code.clone(), link);
        
        let links_vec: Vec<SerializableShortLink> = links.iter()
            .map(|(_, link)| SerializableShortLink {
                short_code: link.code.clone(),
                target_url: link.target.clone(),
                created_at: link.created_at.to_rfc3339(),
                expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            })
            .collect();
        
        let json = serde_json::to_string_pretty(&links_vec)
            .map_err(|e| format!("序列化失败: {}", e))?;
        
        fs::write(&self.file_path, json)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        
        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<(), String> {
        let mut links = self.load_all().await;
        
        if links.remove(code).is_none() {
            return Err(format!("短链接不存在: {}", code));
        }
        
        let links_vec: Vec<SerializableShortLink> = links.iter()
            .map(|(_, link)| SerializableShortLink {
                short_code: link.code.clone(),
                target_url: link.target.clone(),
                created_at: link.created_at.to_rfc3339(),
                expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            })
            .collect();
        
        let json = serde_json::to_string_pretty(&links_vec)
            .map_err(|e| format!("序列化失败: {}", e))?;
        
        fs::write(&self.file_path, json)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        
        Ok(())
    }
}
