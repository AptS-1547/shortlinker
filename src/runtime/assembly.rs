use anyhow::{Context, Result};
use aster_forge_runtime::{AsterRuntime, RuntimeComponentKind, shutdown_resource_component_after};
use aster_forge_tasks::background_task_component_from_shutdown;

use crate::runtime::{components, startup, tasks};

pub async fn run_server() -> Result<()> {
    let startup = startup::prepare_server_startup()
        .await
        .context("server startup failed")?;
    let background_resources = tasks::BackgroundTaskResources::from(&startup);

    let builder = AsterRuntime::builder().component(
        aster_forge_runtime::try_runtime_component_with_shutdown(|shutdown_token| {
            components::http_component(&startup, shutdown_token)
        }),
    )?;
    let process_guard = startup.process_guard;
    builder
        .component(background_task_component_from_shutdown(
            move |shutdown_token| {
                tasks::spawn_background_tasks(background_resources, shutdown_token)
            },
        ))
        .component(shutdown_resource_component_after(
            "process_guard",
            RuntimeComponentKind::Core,
            "release_process_guard",
            &[aster_forge_tasks::BACKGROUND_TASKS_COMPONENT, "http"],
            process_guard,
            |guard| async move {
                drop(guard);
                Ok(())
            },
        ))
        .run()
        .await
        .context("runtime failed")??;

    Ok(())
}
