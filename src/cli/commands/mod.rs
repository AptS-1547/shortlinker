mod help;
mod link_management;

pub use help::*;
pub use link_management::*;

use super::CliError;
use crate::storages::Storage;
use std::sync::Arc;

#[derive(Debug)]
pub enum Command {
    Help,
    List,
    Add {
        short_code: Option<String>,
        target_url: String,
        force_overwrite: bool,
        expire_time: Option<String>,
        password: Option<String>,
    },
    Remove {
        short_code: String,
    },
    Update {
        short_code: String,
        target_url: String,
        expire_time: Option<String>,
        password: Option<String>,
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
            Command::List => list_links(storage).await,
            Command::Add {
                short_code,
                target_url,
                force_overwrite,
                expire_time,
                password,
            } => {
                add_link(
                    storage,
                    short_code,
                    target_url,
                    force_overwrite,
                    expire_time,
                    password,
                )
                .await
            }
            Command::Update {
                short_code,
                target_url,
                expire_time,
                password,
            } => update_link(storage, short_code, target_url, expire_time, password).await,
            Command::Remove { short_code } => remove_link(storage, short_code).await,
            Command::Export { file_path } => export_links(storage, file_path).await,
            Command::Import {
                file_path,
                force_overwrite,
            } => import_links(storage, file_path, force_overwrite).await,
        }
    }
}
