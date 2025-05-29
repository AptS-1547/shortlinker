use std::thread;
use log::info;

use crate::storages::STORAGE;

// Unix平台的信号监听
#[cfg(unix)]
pub fn setup_reload_mechanism() {
    use signal_hook::{consts::SIGUSR1, iterator::Signals};
    
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGUSR1]).unwrap();
        for _ in signals.forever() {
            info!("收到 SIGUSR1，正在从 Storage 重载链接");
            if let Err(e) = futures::executor::block_on(STORAGE.reload()) {
                log::error!("重载链接配置失败: {}", e);
                continue;
            }
            log::info!("链接配置已重载");
        }
    });
}

// Windows平台的文件监听
#[cfg(windows)]
pub fn setup_reload_mechanism() {
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
                        info!("检测到重新加载请求，正在从 Storage 重载链接");
                        if let Err(e) = futures::executor::block_on(STORAGE.reload()) {
                            log::error!("重载链接配置失败: {}", e);
                            last_check = SystemTime::now();
                            continue;
                        }
                        last_check = SystemTime::now();
                        log::info!("链接配置已重载");
                        
                        // 删除触发文件
                        let _ = fs::remove_file(reload_file);
                    }
                }
            }
        }
    });
}
