use std::collections::HashMap;
use std::fs;
use log::{info, error};
use serde::{Deserialize, Serialize};

// 短链接数据结构
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShortLink {
    pub short_code: String,
    pub target_url: String,
}

// 从文件加载短链接
pub fn load_links(file_path: &str) -> HashMap<String, String> {
    match fs::read_to_string(file_path) {
        Ok(content) => {
            match serde_json::from_str::<Vec<ShortLink>>(&content) {
                Ok(links) => {
                    let mut map = HashMap::new();
                    for link in links {
                        map.insert(link.short_code, link.target_url);
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

// 保存短链接到文件
pub fn save_links(links: &HashMap<String, String>, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let links_vec: Vec<ShortLink> = links.iter()
        .map(|(short_code, target_url)| ShortLink {
            short_code: short_code.clone(),
            target_url: target_url.clone(),
        })
        .collect();
    
    let json = serde_json::to_string_pretty(&links_vec)?;
    fs::write(file_path, json)?;
    Ok(())
}
