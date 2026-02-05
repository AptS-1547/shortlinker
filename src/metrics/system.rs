//! System metrics collection
//!
//! Collects process-level metrics like memory and CPU time using sysinfo.

use once_cell::sync::Lazy;
use std::sync::Mutex;
use sysinfo::{Pid, ProcessesToUpdate, System};

use super::METRICS;

/// Cached system info for metrics collection
static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));

/// Update system metrics (memory and CPU time)
///
/// This should be called periodically or on-demand (e.g., when metrics are exported).
pub fn update_system_metrics() {
    let pid = Pid::from_u32(std::process::id());

    let mut sys = match SYSTEM.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    // Refresh only the current process
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

    if let Some(process) = sys.process(pid) {
        // Memory metrics
        let memory_bytes = process.memory();
        let virtual_memory_bytes = process.virtual_memory();

        METRICS
            .process_memory_bytes
            .with_label_values(&["rss"])
            .set(memory_bytes as f64);
        METRICS
            .process_memory_bytes
            .with_label_values(&["virtual"])
            .set(virtual_memory_bytes as f64);

        // CPU time (accumulated, in milliseconds -> convert to seconds)
        let cpu_time_ms = process.accumulated_cpu_time();
        let cpu_time_seconds = cpu_time_ms as f64 / 1000.0;
        METRICS.process_cpu_seconds_total.set(cpu_time_seconds);
    }
}
