use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortLink {
    pub code: String,
    pub target: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub password: Option<String>,

    #[serde(default)]
    pub click: usize,
}

impl ShortLink {
    /// 检查链接是否已过期
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| exp <= chrono::Utc::now())
            .unwrap_or(false)
    }

    /// 计算缓存 TTL（秒），已过期返回 None
    pub fn cache_ttl(&self, default_ttl: u64) -> Option<u64> {
        match self.expires_at {
            Some(exp) => {
                let now = chrono::Utc::now();
                if exp <= now {
                    None // 已过期，不应缓存
                } else {
                    let remaining = (exp - now).num_seconds() as u64;
                    Some(remaining.min(default_ttl))
                }
            }
            None => Some(default_ttl), // 无过期时间，用默认 TTL
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageConfig {
    pub storage_type: String,
    pub support_click: bool,
}

/// 链接统计信息
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LinkStats {
    pub total_links: usize,
    pub total_clicks: usize,
    pub active_links: usize,
}
