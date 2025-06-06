use crate::cache::Cache;
use crate::storages;
use std::sync::Arc;
use std::thread;

// Unix平台的信号监听
#[cfg(unix)]
pub fn setup_reload_mechanism(
    cache: Arc<dyn Cache + 'static>,
    storage: Arc<dyn storages::Storage>,
) {
    use signal_hook::{consts::SIGUSR1, iterator::Signals};

    thread::spawn(move || {
        let mut signals = Signals::new([SIGUSR1]).unwrap();
        for _ in signals.forever() {
            tracing::info!("Received SIGUSR1, reloading links from storage");
            if let Err(e) = futures::executor::block_on(storage.reload()) {
                tracing::error!("Failed to reload link configuration: {}", e);
                continue;
            }
            tracing::info!("Link configuration reloaded");
        }
    });
}

// Windows平台的文件监听
#[cfg(windows)]
pub fn setup_reload_mechanism(storage: Arc<dyn storages::Storage>) {
    use std::fs;
    use std::time::{Duration, SystemTime};

    thread::spawn(move || {
        let reload_file = "shortlinker.reload";
        let mut last_check = SystemTime::now();

        loop {
            thread::sleep(Duration::from_millis(3000));

            if let Ok(metadata) = fs::metadata(reload_file) {
                if let Ok(modified) = metadata.modified() {
                    if modified > last_check {
                        tracing::info!("Reload request detected, reloading links from storage");
                        if let Err(e) = futures::executor::block_on(storage.reload()) {
                            tracing::error!("Failed to reload link configuration: {}", e);
                            last_check = SystemTime::now();
                            continue;
                        }
                        last_check = SystemTime::now();
                        tracing::info!("Link configuration reloaded");

                        // 删除触发文件
                        let _ = fs::remove_file(reload_file);
                    }
                }
            }
        }
    });
}
