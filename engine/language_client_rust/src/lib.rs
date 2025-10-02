//! BAML Rust Language Client
//!
//! This crate provides a high-level Rust API for calling BAML functions.
//! It wraps the core `baml-runtime` with a convenient, type-safe interface.

pub mod client;
pub mod context;
pub mod errors;
pub mod ffi;
pub mod result;
pub(crate) mod runtime;
pub mod stream;
pub mod types;

// Re-export main types
pub use client::{BamlClient, BamlClientBuilder};
pub use context::BamlContext;
pub use errors::{BamlError, BamlErrorType};
pub use result::{BamlResult, FunctionResult};
pub use stream::{FunctionResultStream, StreamState};
pub use types::RuntimeContextManager;
pub use types::{
    BamlMap, BamlValue, ClientRegistry, Collector, FunctionLog, HttpBody, HttpRequest,
    HttpResponse, LlmCall, LlmCallKind, LlmStreamCall, SseResponse, StreamTiming, Timing,
    TypeBuilder, Usage,
};

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the current version of the BAML Rust client
pub fn version() -> &'static str {
    VERSION
}
