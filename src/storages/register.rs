use crate::errors::Result;
use crate::storages::Storage;
use once_cell::sync::Lazy;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, sync::RwLock};

use tracing::debug;

type BoxedStorageFuture = Pin<Box<dyn Future<Output = Result<Box<dyn Storage>>> + Send>>;
type StorageConstructor = Arc<dyn Fn() -> BoxedStorageFuture + Send + Sync>;

static STORAGE_REGISTRY: Lazy<RwLock<HashMap<String, StorageConstructor>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_storage_plugin<S: Into<String>>(name: S, constructor: StorageConstructor) {
    let name = name.into();
    debug!("Registering storage plugin: {}", name);
    let mut registry = STORAGE_REGISTRY.write().unwrap();
    registry.insert(name, constructor);
}

pub fn get_storage_plugin(name: &str) -> Option<StorageConstructor> {
    STORAGE_REGISTRY.read().unwrap().get(name).cloned()
}

pub fn get_storage_plugin_names() -> Vec<String> {
    STORAGE_REGISTRY.read().unwrap().keys().cloned().collect()
}

/// 调试函数：打印当前所有已注册的 Storage backend 名称
pub fn debug_storage_registry() {
    let registry = STORAGE_REGISTRY.read().unwrap();
    if registry.is_empty() {
        debug!("No storage backends registered.");
    } else {
        debug!("Registered storage backends:");
        for key in registry.keys() {
            debug!(" - {}", key);
        }
    }
}
