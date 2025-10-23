#[macro_export]
macro_rules! declare_repository_plugin {
    ($name:expr, $ty:ty) => {
        #[ctor::ctor]
        fn __register_repository_plugin() {
            use std::sync::Arc;
            use $crate::repository::{Repository, register::register_repository_plugin};

            register_repository_plugin(
                $name,
                Arc::new(|| {
                    Box::pin(async {
                        let repository = <$ty>::new_async().await?;
                        Ok(Box::new(repository) as Box<dyn Repository>)
                    })
                }),
            );
        }
    };
}
