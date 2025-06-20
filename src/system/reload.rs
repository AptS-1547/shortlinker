use crate::cache::{traits::BloomConfig, CompositeCacheTrait};
use crate::storages::Storage;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub async fn reload_all(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    storage: Arc<dyn Storage + 'static>,
) -> anyhow::Result<()> {
    tracing::info!("Starting reload process...");

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

    tracing::info!("Reload process completed successfully");
    Ok(())
}

// Unix平台的信号监听
#[cfg(unix)]
pub async fn setup_reload_mechanism(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    storage: Arc<dyn Storage + 'static>,
) {
    use signal_hook::consts::SIGUSR1;
    use signal_hook_tokio::{Signals, SignalsInfo};
    use tokio_stream::StreamExt;

    let signals = Signals::new([SIGUSR1]).expect("Failed to create signal handler");
    let handle = signals.handle();

    let mut signals = signals.map(|signal| SignalsInfo::from(signal));

    tokio::spawn(async move {
        while let Some(signal) = signals.next().await {
            match signal {
                SignalsInfo::Signal(SIGUSR1) => {
                    tracing::info!("Received SIGUSR1, reloading...");

                    if let Err(e) = reload_all(cache.clone(), storage.clone()).await {
                        tracing::error!("Reload failed: {}", e);
                    } else {
                        tracing::info!("Reload successful");
                    }
                }
                _ => {}
            }
        }

        // 清理信号处理器
        handle.close();
    });
}

// Windows平台的文件监听
#[cfg(windows)]
pub async fn setup_reload_mechanism(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    storage: Arc<dyn Storage + 'static>,
) {
    use std::time::SystemTime;
    use tokio::fs;

    let reload_file = "shortlinker.reload";
    let mut last_check = SystemTime::now();
    let check_interval = Duration::from_secs(3);

    tokio::spawn(async move {
        loop {
            // 使用异步睡眠
            sleep(check_interval).await;

            // 使用异步文件操作
            match fs::metadata(reload_file).await {
                Ok(metadata) => {
                    match metadata.modified() {
                        Ok(modified) => {
                            if modified > last_check {
                                tracing::info!("Reload request detected, reloading...");

                                match reload_all(cache.clone(), storage.clone()).await {
                                    Ok(_) => {
                                        tracing::info!("Reload successful");
                                        last_check = SystemTime::now();

                                        // 异步删除文件
                                        if let Err(e) = fs::remove_file(reload_file).await {
                                            tracing::warn!("Failed to remove reload file: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Reload failed: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::debug!("Failed to get file modification time: {}", e);
                        }
                    }
                }
                Err(_) => {
                    // 文件不存在，这是正常情况
                }
            }
        }
    });
}
