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

static CACHE_FILTER_REGISTRY: Lazy<RwLock<HashMap<String, ExistenceFilterConstructor>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static OBJECT_CACHE_REGISTRY: Lazy<RwLock<HashMap<String, ObjectCacheConstructor>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_filter_plugin<S: Into<String>>(name: S, constructor: ExistenceFilterConstructor) {
    let name = name.into();
    let mut registry = CACHE_FILTER_REGISTRY
        .write()
        .expect("Filter registry RwLock poisoned - a thread panicked while holding the lock");
    registry.insert(name, constructor);
}

pub fn get_filter_plugin(name: &str) -> Option<ExistenceFilterConstructor> {
    CACHE_FILTER_REGISTRY
        .read()
        .expect("Filter registry RwLock poisoned - a thread panicked while holding the lock")
        .get(name)
        .cloned()
}

pub fn register_object_cache_plugin<S: Into<String>>(name: S, constructor: ObjectCacheConstructor) {
    let name = name.into();
    let mut registry = OBJECT_CACHE_REGISTRY
        .write()
        .expect("Object cache registry RwLock poisoned - a thread panicked while holding the lock");
    registry.insert(name, constructor);
}

pub fn get_object_cache_plugin(name: &str) -> Option<ObjectCacheConstructor> {
    OBJECT_CACHE_REGISTRY
        .read()
        .expect("Object cache registry RwLock poisoned - a thread panicked while holding the lock")
        .get(name)
        .cloned()
}

pub fn debug_cache_registry() {
    let filter_registry = CACHE_FILTER_REGISTRY
        .read()
        .expect("Filter registry RwLock poisoned");
    if filter_registry.is_empty() {
        tracing::debug!("No Filter plugins registered.");
    } else {
        tracing::debug!("Registered Filter plugins:");
        for key in filter_registry.keys() {
            tracing::debug!(" - {}", key);
        }
    }

    let object_cache_registry = OBJECT_CACHE_REGISTRY
        .read()
        .expect("Object cache registry RwLock poisoned");
    if object_cache_registry.is_empty() {
        tracing::debug!("No Object Cache plugins registered.");
    } else {
        tracing::debug!("Registered Object Cache plugins:");
        for key in object_cache_registry.keys() {
            tracing::debug!(" - {}", key);
        }
    }
}
