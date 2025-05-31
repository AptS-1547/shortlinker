use super::super::{CliError, process_manager::ProcessManager};

pub fn start_server() -> Result<(), CliError> {
    ProcessManager::start_server()
}

pub fn stop_server() -> Result<(), CliError> {
    ProcessManager::stop_server()
}

pub fn restart_server() -> Result<(), CliError> {
    ProcessManager::restart_server()
}
