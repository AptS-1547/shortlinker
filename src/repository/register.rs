use crate::errors::Result;
use crate::repository::Repository;
use once_cell::sync::Lazy;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, sync::RwLock};

use tracing::debug;

type BoxedRepositoryFuture = Pin<Box<dyn Future<Output = Result<Box<dyn Repository>>> + Send>>;
type RepositoryConstructor = Arc<dyn Fn() -> BoxedRepositoryFuture + Send + Sync>;

static REPOSITORY_REGISTRY: Lazy<RwLock<HashMap<String, RepositoryConstructor>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_repository_plugin<S: Into<String>>(name: S, constructor: RepositoryConstructor) {
    let name = name.into();
    debug!("Registering repository plugin: {}", name);
    let mut registry = REPOSITORY_REGISTRY.write().unwrap();
    registry.insert(name, constructor);
}

pub fn get_repository_plugin(name: &str) -> Option<RepositoryConstructor> {
    REPOSITORY_REGISTRY.read().unwrap().get(name).cloned()
}

pub fn get_repository_plugin_names() -> Vec<String> {
    REPOSITORY_REGISTRY.read().unwrap().keys().cloned().collect()
}

/// 调试函数：打印当前所有已注册的 Repository backend 名称
pub fn debug_repository_registry() {
    let registry = REPOSITORY_REGISTRY.read().unwrap();
    if registry.is_empty() {
        debug!("No repository backends registered.");
    } else {
        debug!("Registered repository backends:");
        for key in registry.keys() {
            debug!(" - {}", key);
        }
    }
}
