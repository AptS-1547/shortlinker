use crate::cache::{traits::BloomConfig, Cache};
use crate::storages::Storage;
use std::sync::Arc;
use std::thread;

// Unix平台的信号监听
#[cfg(unix)]
pub fn setup_reload_mechanism(
    cache: Arc<dyn Cache + 'static>,
    storage: Arc<dyn Storage + 'static>,
) {
    use signal_hook::{consts::SIGUSR1, iterator::Signals};

    thread::spawn(move || {
        let mut signals = Signals::new([SIGUSR1]).unwrap();
        for _ in signals.forever() {
            tracing::info!("Received SIGUSR1, reloading...");
            if let Err(e) = futures::executor::block_on(async {
                storage.reload().await?;
                let links = storage.load_all().await;
                cache
                    .reconfigure(BloomConfig {
                        capacity: links.len(),
                        fp_rate: 0.001,
                    })
                    .await;

                cache.load_cache(links).await;
                Ok::<(), anyhow::Error>(())
            }) {
                tracing::error!("Reload failed: {}", e);
            }
        }
    });
}

// Windows平台的文件监听
#[cfg(windows)]
pub fn setup_reload_mechanism(
    cache: Arc<dyn Cache + 'static>,
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
                        if let Err(e) = futures::executor::block_on(async {
                            storage.reload().await?;
                            let links = storage.load_all().await;
                            cache
                                .reconfigure(BloomConfig {
                                    capacity: links.len(),
                                    fp_rate: 0.001,
                                })
                                .await;

                            cache.load_cache(links).await;
                            Ok::<(), anyhow::Error>(())
                        }) {
                            tracing::error!("Reload failed: {}", e);
                        }
                        last_check = SystemTime::now();
                        let _ = fs::remove_file(reload_file);
                    }
                }
            }
        }
    });
}
