use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use log::info;

use crate::storages::{STORAGE, ShortLink};

// Unix平台的信号监听
#[cfg(unix)]
pub fn setup_reload_mechanism(cache: Arc<RwLock<HashMap<String, ShortLink>>>) {
    use signal_hook::{consts::SIGUSR1, iterator::Signals};
    
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGUSR1]).unwrap();
        for _ in signals.forever() {
            info!("收到 SIGUSR1，正在从 Storage 重载链接");
            let new_links = futures::executor::block_on(STORAGE.load_all());
            let mut cache_guard = cache.write().unwrap();
            *cache_guard = new_links;
            log::info!("链接配置已重载");
        }
    });
}

// Windows平台的文件监听
#[cfg(windows)]
pub fn setup_reload_mechanism(links: Arc<RwLock<HashMap<String, String>>>) {
    use std::time::{Duration, SystemTime};
    use std::fs;
    
    thread::spawn(move || {
        let reload_file = "shortlinker.reload";
        let mut last_check = SystemTime::now();
        
        loop {
            thread::sleep(Duration::from_millis(3000));
            
            if let Ok(metadata) = fs::metadata(reload_file) {
                if let Ok(modified) = metadata.modified() {
                    if modified > last_check {
                        info!("检测到重新加载请求，重新加载链接配置");
                        let new_links = load_links(&links_file);
                        let mut links_map = links.write().unwrap();
                        *links_map = new_links;
                        last_check = SystemTime::now();
                        
                        // 删除触发文件
                        let _ = fs::remove_file(reload_file);
                    }
                }
            }
        }
    });
}
