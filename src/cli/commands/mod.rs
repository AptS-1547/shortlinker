mod help;
mod link_management;
mod server;

pub use help::*;
pub use link_management::*;
pub use server::*;

use super::CliError;
use crate::storages::Storage;
use std::sync::Arc;

#[derive(Debug)]
pub enum Command {
    Help,
    Start,
    Stop,
    Restart,
    List,
    Add {
        short_code: Option<String>,
        target_url: String,
        force_overwrite: bool,
        expire_time: Option<String>,
    },
    Remove {
        short_code: String,
    },
    Update {
        short_code: String,
        target_url: String,
        expire_time: Option<String>,
    },
    Export {
        file_path: Option<String>,
    },
    Import {
        file_path: String,
        force_overwrite: bool,
    },
}

impl Command {
    pub async fn execute(self, storage: Arc<dyn Storage>) -> Result<(), CliError> {
        match self {
            Command::Help => {
                show_help();
                Ok(())
            }
            Command::Start => start_server(),
            Command::Stop => stop_server(),
            Command::Restart => restart_server(),
            Command::List => list_links(storage).await,
            Command::Add {
                short_code,
                target_url,
                force_overwrite,
                expire_time,
            } => {
                add_link(
                    storage,
                    short_code,
                    target_url,
                    force_overwrite,
                    expire_time,
                )
                .await
            }
            Command::Update {
                short_code,
                target_url,
                expire_time,
            } => update_link(storage, short_code, target_url, expire_time).await,
            Command::Remove { short_code } => remove_link(storage, short_code).await,
            Command::Export { file_path } => export_links(storage, file_path).await,
            Command::Import {
                file_path,
                force_overwrite,
            } => import_links(storage, file_path, force_overwrite).await,
        }
    }
}
