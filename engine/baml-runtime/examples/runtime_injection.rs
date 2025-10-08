use std::sync::Arc;

use anyhow::Result;
use baml_runtime::BamlRuntime;

#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Builder;

fn main() -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let custom_runtime = Arc::new(
            Builder::new_multi_thread()
                .worker_threads(1)
                .thread_name("baml-runtime-example")
                .enable_all()
                .build()?,
        );

        BamlRuntime::set_tokio_runtime(custom_runtime.clone())?;

        let shared_runtime = BamlRuntime::tokio_runtime_handle()?;
        assert!(Arc::ptr_eq(&custom_runtime, &shared_runtime));

        println!("Custom Tokio runtime successfully installed");
    }

    #[cfg(target_arch = "wasm32")]
    {
        println!("Tokio runtime overrides are not supported on wasm32 targets");
    }

    Ok(())
}
