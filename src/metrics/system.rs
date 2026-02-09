//! System metrics collection
//!
//! Collects process-level metrics like memory and CPU time using sysinfo.
//! A background task periodically refreshes these metrics.

use once_cell::sync::Lazy;
use std::sync::Mutex;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tokio::time::{Duration, interval};

use super::get_metrics;

/// Cached system info for metrics collection
static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));

/// System metrics update interval
const UPDATE_INTERVAL_SECS: u64 = 15;

/// Spawn a background task that periodically updates system metrics.
///
/// Should be called once during server startup.
pub fn spawn_system_metrics_updater() {
    tokio::spawn(async {
        let mut ticker = interval(Duration::from_secs(UPDATE_INTERVAL_SECS));
        loop {
            ticker.tick().await;
            update_system_metrics();
        }
    });
}

/// Update system metrics (memory and CPU time)
fn update_system_metrics() {
    let Some(metrics) = get_metrics() else {
        return;
    };

    let pid = Pid::from_u32(std::process::id());

    let mut sys = match SYSTEM.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            tracing::warn!("System metrics mutex was poisoned, recovering");
            poisoned.into_inner()
        }
    };

    // Refresh only the current process
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

    if let Some(process) = sys.process(pid) {
        // Memory metrics
        let memory_bytes = process.memory();
        let virtual_memory_bytes = process.virtual_memory();

        metrics
            .process_memory_bytes
            .with_label_values(&["rss"])
            .set(memory_bytes as f64);
        metrics
            .process_memory_bytes
            .with_label_values(&["virtual"])
            .set(virtual_memory_bytes as f64);

        // CPU time (accumulated, in milliseconds -> convert to seconds)
        let cpu_time_ms = process.accumulated_cpu_time();
        let cpu_time_seconds = cpu_time_ms as f64 / 1000.0;
        metrics.process_cpu_seconds.set(cpu_time_seconds);
    }
}
