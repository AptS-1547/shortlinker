use std::sync::Arc;
use std::time::Duration;

use aster_forge_tasks::BackgroundTasks;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

use crate::analytics::{ClickManager, DataRetentionTask, RawClickEvent};
use crate::runtime::startup::{StartupContext, process_raw_click_event};
use crate::services::LinkCache;

pub struct BackgroundTaskResources {
    metrics: Arc<dyn crate::metrics::MetricsRecorder>,
    database: sea_orm::DatabaseConnection,
    cache: Arc<dyn LinkCache>,
    click_manager: Option<Arc<ClickManager>>,
    raw_event_receiver: Option<crossbeam_channel::Receiver<RawClickEvent>>,
    retention_task: Option<Arc<DataRetentionTask>>,
}

impl From<&StartupContext> for BackgroundTaskResources {
    fn from(startup: &StartupContext) -> Self {
        Self {
            metrics: startup.metrics.clone(),
            database: startup.storage.get_db().clone(),
            cache: startup.cache.clone(),
            click_manager: startup.click_manager.clone(),
            raw_event_receiver: startup.raw_event_receiver.clone(),
            retention_task: startup.retention_task.clone(),
        }
    }
}

pub fn spawn_background_tasks(
    resources: BackgroundTaskResources,
    shutdown_token: CancellationToken,
) -> BackgroundTasks {
    let mut tasks = BackgroundTasks::with_shutdown_token(shutdown_token.clone());

    if let Some(task) = resources
        .metrics
        .forge_recorder()
        .system_metrics_updater_task(shutdown_token.clone())
    {
        tasks.push(task);
    }

    tasks.push(run_user_agent_flush(
        resources.database.clone(),
        shutdown_token.clone(),
    ));
    tasks.push(run_bloom_rebuild(resources.cache, shutdown_token.clone()));
    tasks.push(crate::system::ipc::server::run_ipc_server(
        shutdown_token.clone(),
    ));

    if let Some(retention_task) = resources.retention_task {
        tasks.push(run_retention(retention_task, shutdown_token.clone()));
    }
    if let Some(click_manager) = resources.click_manager {
        tasks.push(run_click_manager(
            click_manager,
            resources.raw_event_receiver,
            shutdown_token,
        ));
    }

    tasks
}

async fn run_click_manager(
    manager: Arc<ClickManager>,
    raw_event_receiver: Option<crossbeam_channel::Receiver<RawClickEvent>>,
    shutdown_token: CancellationToken,
) {
    let mut workers = tokio::task::JoinSet::new();
    let background_manager = manager.clone();
    workers.spawn(async move {
        background_manager.start_background_task().await;
    });
    if let Some(receiver) = raw_event_receiver {
        let event_manager = manager.clone();
        workers.spawn(async move {
            event_manager
                .start_event_processor(receiver, process_raw_click_event)
                .await;
        });
    }

    shutdown_token.cancelled().await;
    manager.cancel();
    while workers.join_next().await.is_some() {}
    manager.flush().await;
}

async fn run_user_agent_flush(
    database: sea_orm::DatabaseConnection,
    shutdown_token: CancellationToken,
) {
    loop {
        tokio::select! {
            _ = shutdown_token.cancelled() => break,
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                if let Some(store) = crate::services::get_user_agent_store()
                    && let Err(error) = store.flush_pending(&database).await
                {
                    warn!(%error, "periodic UserAgent flush failed");
                }
            }
        }
    }
}

async fn run_bloom_rebuild(cache: Arc<dyn LinkCache>, shutdown_token: CancellationToken) {
    loop {
        let interval = crate::config::get_runtime_config()
            .get_u64_or(crate::config::keys::CACHE_BLOOM_REBUILD_INTERVAL, 0);
        if interval == 0 {
            shutdown_token.cancelled().await;
            return;
        }
        tokio::select! {
            _ = shutdown_token.cancelled() => return,
            _ = tokio::time::sleep(Duration::from_secs(interval)) => {
                if let Err(error) = cache.rebuild_all().await {
                    error!(%error, "periodic Bloom filter rebuild failed");
                }
            }
        }
    }
}

async fn run_retention(task: Arc<DataRetentionTask>, shutdown_token: CancellationToken) {
    tokio::select! {
        _ = shutdown_token.cancelled() => return,
        _ = tokio::time::sleep(Duration::from_secs(300)) => {}
    }
    loop {
        if let Err(error) = task.run_cleanup().await {
            error!(%error, "data retention task failed");
        }
        tokio::select! {
            _ = shutdown_token.cancelled() => return,
            _ = tokio::time::sleep(Duration::from_secs(24 * 60 * 60)) => {}
        }
    }
}
