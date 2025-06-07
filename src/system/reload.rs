use crate::cache::{traits::BloomConfig, CompositeCacheTrait};
use crate::storages::Storage;
use std::sync::Arc;
use std::thread;

pub async fn reload_all(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    storage: Arc<dyn Storage + 'static>,
) -> anyhow::Result<()> {
    // 重新加载存储
    storage.reload().await?;
    let links = storage.load_all().await;

    // 重新配置缓存
    cache
        .reconfigure(BloomConfig {
            capacity: links.len(),
            fp_rate: 0.001,
        })
        .await;

    // 加载缓存
    cache.load_cache(links).await;

    Ok(())
}

// Unix平台的信号监听
#[cfg(unix)]
pub fn setup_reload_mechanism(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    storage: Arc<dyn Storage + 'static>,
) {
    use signal_hook::{consts::SIGUSR1, iterator::Signals};

    thread::spawn(move || {
        let mut signals = Signals::new([SIGUSR1]).unwrap();
        for _ in signals.forever() {
            tracing::info!("Received SIGUSR1, reloading...");

            if let Err(e) = futures::executor::block_on(reload_all(cache.clone(), storage.clone()))
            {
                tracing::error!("Reload failed: {}", e);
            } else {
                tracing::info!("Reload successful");
            }
        }
    });
}

// Windows平台的文件监听
#[cfg(windows)]
pub fn setup_reload_mechanism(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    storage: Arc<dyn Storage + 'static>,
) {
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
                        tracing::info!("Reload request detected, reloading...");

                        if let Err(e) =
                            futures::executor::block_on(reload_all(cache.clone(), storage.clone()))
                        {
                            tracing::error!("Reload failed: {}", e);
                        } else {
                            tracing::info!("Reload successful");
                        }

                        last_check = SystemTime::now();
                        let _ = fs::remove_file(reload_file);
                    }
                }
            }
        }
    });
}
