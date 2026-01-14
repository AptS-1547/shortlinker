#[macro_export]
macro_rules! declare_existence_filter_plugin {
    ($name:expr, $ty:ty) => {
        #[ctor::ctor]
        fn __register_l1_plugin() {
            use std::sync::Arc;
            use $crate::cache::register::register_filter_plugin;

            register_filter_plugin(
                $name,
                Arc::new(|| {
                    Box::pin(async {
                        match <$ty>::new() {
                            Ok(filter) => {
                                Ok(Box::new(filter) as Box<dyn $crate::cache::ExistenceFilter>)
                            }
                            Err(e) => Err($crate::errors::ShortlinkerError::cache_connection(
                                format!("Failed to create existence filter: {}", e),
                            )),
                        }
                    })
                }),
            );
        }
    };
}

#[macro_export]
macro_rules! declare_object_cache_plugin {
    ($name:expr, $ty:ty) => {
        #[ctor::ctor]
        fn __register_cache_plugin() {
            use std::sync::Arc;
            use $crate::cache::register::register_object_cache_plugin;

            register_object_cache_plugin(
                $name,
                Arc::new(|| {
                    Box::pin(async {
                        match <$ty>::new().await {
                            Ok(cache) => Ok(Box::new(cache) as Box<dyn $crate::cache::ObjectCache>),
                            Err(e) => Err($crate::errors::ShortlinkerError::cache_connection(e)),
                        }
                    })
                }),
            );
        }
    };
}
