#[macro_export]
macro_rules! declare_l1_plugin {
    ($name:expr, $ty:ty) => {
        #[ctor::ctor]
        fn __register_l1_plugin() {
            use std::sync::Arc;
            use $crate::cache::register::register_l1_plugin;
            register_l1_plugin(
                $name,
                Arc::new(|| {
                    Box::pin(async {
                        let l1 = <$ty>::new(1_000, 0.001);
                        Ok(Box::new(l1) as Box<dyn $crate::cache::traits::L1Cache>)
                    })
                }),
            );
        }
    };
}

#[macro_export]
macro_rules! declare_l2_plugin {
    ($name:expr, $ty:ty) => {
        #[ctor::ctor]
        fn __register_cache_plugin() {
            use std::sync::Arc;
            use $crate::cache::register::register_l2_plugin;

            register_l2_plugin(
                $name,
                Arc::new(|| {
                    Box::pin(async {
                        let cache = <$ty>::new();
                        Ok(Box::new(cache) as Box<dyn $crate::cache::traits::L2Cache>)
                    })
                }),
            );
        }
    };
}
