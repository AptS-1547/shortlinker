use crate::cache::traits::{L1Cache, L2Cache};
use crate::errors::Result;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};

pub type BoxedL1CacheFuture = Pin<Box<dyn Future<Output = Result<Box<dyn L1Cache>>> + Send>>;
pub type L1CacheConstructor = Arc<dyn Fn() -> BoxedL1CacheFuture + Send + Sync>;

pub type BoxedL2CacheFuture = Pin<Box<dyn Future<Output = Result<Box<dyn L2Cache>>> + Send>>;
pub type L2CacheConstructor = Arc<dyn Fn() -> BoxedL2CacheFuture + Send + Sync>;

static CACHE_L1_REGISTRY: Lazy<RwLock<HashMap<String, L1CacheConstructor>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static CACHE_L2_REGISTRY: Lazy<RwLock<HashMap<String, L2CacheConstructor>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_l1_plugin<S: Into<String>>(name: S, constructor: L1CacheConstructor) {
    let name = name.into();
    let mut registry = CACHE_L1_REGISTRY.write().unwrap();
    registry.insert(name, constructor);
}

pub fn get_l1_plugin(name: &str) -> Option<L1CacheConstructor> {
    CACHE_L1_REGISTRY.read().unwrap().get(name).cloned()
}

pub fn register_l2_plugin<S: Into<String>>(name: S, constructor: L2CacheConstructor) {
    let name = name.into();
    let mut registry = CACHE_L2_REGISTRY.write().unwrap();
    registry.insert(name, constructor);
}

pub fn get_l2_plugin(name: &str) -> Option<L2CacheConstructor> {
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
