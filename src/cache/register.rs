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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // 使用唯一的插件名称避免测试间干扰
    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn unique_name(prefix: &str) -> String {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("{}_{}", prefix, id)
    }

    #[test]
    fn test_register_and_get_filter_plugin() {
        let name = unique_name("test_filter");

        // 注册一个 mock filter 插件
        let constructor: ExistenceFilterConstructor = Arc::new(|| {
            Box::pin(async {
                // 返回一个简单的 mock
                Err(crate::errors::ShortlinkerError::cache_plugin_not_found(
                    "mock filter for test",
                ))
            })
        });

        register_filter_plugin(name.clone(), constructor);

        // 应该能获取到
        let retrieved = get_filter_plugin(&name);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_get_nonexistent_filter_plugin() {
        let name = unique_name("nonexistent_filter");
        let retrieved = get_filter_plugin(&name);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_register_and_get_object_cache_plugin() {
        let name = unique_name("test_object_cache");

        // 注册一个 mock object cache 插件
        let constructor: ObjectCacheConstructor = Arc::new(|| {
            Box::pin(async {
                Err(crate::errors::ShortlinkerError::cache_plugin_not_found(
                    "mock object cache for test",
                ))
            })
        });

        register_object_cache_plugin(name.clone(), constructor);

        // 应该能获取到
        let retrieved = get_object_cache_plugin(&name);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_get_nonexistent_object_cache_plugin() {
        let name = unique_name("nonexistent_object_cache");
        let retrieved = get_object_cache_plugin(&name);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_register_overwrites_existing() {
        let name = unique_name("overwrite_test");
        static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

        // 第一次注册
        let constructor1: ObjectCacheConstructor = Arc::new(|| {
            CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            Box::pin(async {
                Err(crate::errors::ShortlinkerError::cache_plugin_not_found(
                    "first",
                ))
            })
        });
        register_object_cache_plugin(name.clone(), constructor1);

        // 第二次注册（覆盖）
        let constructor2: ObjectCacheConstructor = Arc::new(|| {
            CALL_COUNT.fetch_add(10, Ordering::SeqCst);
            Box::pin(async {
                Err(crate::errors::ShortlinkerError::cache_plugin_not_found(
                    "second",
                ))
            })
        });
        register_object_cache_plugin(name.clone(), constructor2);

        // 获取并调用，应该是第二个构造函数
        let retrieved = get_object_cache_plugin(&name).unwrap();
        let before = CALL_COUNT.load(Ordering::SeqCst);
        drop(retrieved()); // 调用构造函数并显式 drop future
        let after = CALL_COUNT.load(Ordering::SeqCst);

        // 第二个构造函数加 10
        assert_eq!(after - before, 10);
    }

    #[test]
    fn test_debug_cache_registry_does_not_panic() {
        // 只测试不会 panic
        debug_cache_registry();
    }
}
