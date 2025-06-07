use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct CachePreference {
    pub l1: String,
    pub l2: String,
}

#[derive(Debug, Clone)]
pub struct ShortLink {
    pub code: String,
    pub target: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableShortLink {
    pub short_code: String,
    pub target_url: String,
    pub created_at: String,
    pub expires_at: Option<String>,

    #[serde(default)]
    pub click: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageConfig {
    pub storage_type: String,
    pub support_click: bool,
}
