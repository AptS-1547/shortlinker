use crate::cache::traits::{ExistenceFilter, ObjectCache};
use crate::errors::Result;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};

pub type BoxedExistenceFilterFuture =
    Pin<Box<dyn Future<Output = Result<Box<dyn ExistenceFilter>>> + Send>>;
pub type ExistenceFilterConstructor = Arc<dyn Fn() -> BoxedExistenceFilterFuture + Send + Sync>;

pub type BoxedObjectCacheFuture =
    Pin<Box<dyn Future<Output = Result<Box<dyn ObjectCache>>> + Send>>;
pub type ObjectCacheConstructor = Arc<dyn Fn() -> BoxedObjectCacheFuture + Send + Sync>;

static CACHE_L1_REGISTRY: Lazy<RwLock<HashMap<String, ExistenceFilterConstructor>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static CACHE_L2_REGISTRY: Lazy<RwLock<HashMap<String, ObjectCacheConstructor>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_l1_plugin<S: Into<String>>(name: S, constructor: ExistenceFilterConstructor) {
    let name = name.into();
    let mut registry = CACHE_L1_REGISTRY.write().unwrap();
    registry.insert(name, constructor);
}

pub fn get_l1_plugin(name: &str) -> Option<ExistenceFilterConstructor> {
    CACHE_L1_REGISTRY.read().unwrap().get(name).cloned()
}

pub fn register_l2_plugin<S: Into<String>>(name: S, constructor: ObjectCacheConstructor) {
    let name = name.into();
    let mut registry = CACHE_L2_REGISTRY.write().unwrap();
    registry.insert(name, constructor);
}

pub fn get_l2_plugin(name: &str) -> Option<ObjectCacheConstructor> {
    CACHE_L2_REGISTRY.read().unwrap().get(name).cloned()
}

pub fn debug_cache_registry() {
    let l1_registry = CACHE_L1_REGISTRY.read().unwrap();
    if l1_registry.is_empty() {
        tracing::debug!("No L1 cache plugins registered.");
    } else {
        tracing::debug!("Registered L1 cache plugins:");
        for key in l1_registry.keys() {
            tracing::debug!(" - {}", key);
        }
    }

    let l2_registry = CACHE_L2_REGISTRY.read().unwrap();
    if l2_registry.is_empty() {
        tracing::debug!("No L2 cache plugins registered.");
    } else {
        tracing::debug!("Registered L2 cache plugins:");
        for key in l2_registry.keys() {
            tracing::debug!(" - {}", key);
        }
    }
}
