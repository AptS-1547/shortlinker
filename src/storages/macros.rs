#[macro_export]
macro_rules! declare_storage_plugin {
    ($name:expr, $ty:ty) => {
        #[ctor::ctor]
        fn __register_storage_plugin() {
            use std::sync::Arc;
            use $crate::storages::{Storage, register::register_storage_plugin};

            register_storage_plugin(
                $name,
                Arc::new(|| {
                    Box::pin(async {
                        let storage = <$ty>::new_async().await?;
                        Ok(Box::new(storage) as Box<dyn Storage>)
                    })
                }),
            );
        }
    };
}
