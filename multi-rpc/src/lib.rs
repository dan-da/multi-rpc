//! A procedural macro-based library to define a Rust service trait once and serve it over multiple RPC protocols.

/// The multi-rpc prelude for convenient importing of the most common items.
pub mod prelude;

/// Contains the `ServerBuilder` for configuring and launching servers.
pub mod builder;
/// Contains the error types used by the library.
pub mod error;
/// Contains the `ServerRunner` for managing running server tasks.
pub mod runner;

// --- Public Dependency Re-exports (For Version Safety) ---

// It's common to re-export serde for convenience
#[cfg(feature = "rest-axum")]
pub use axum;
#[cfg(feature = "jsonrpsee")]
pub use jsonrpsee;
// --- Macro Re-exports ---
/// A procedural macro to generate protocol-specific server implementations from a trait impl.
pub use multi_rpc_macros::multi_rpc_impl;
/// A procedural macro to define a service trait compatible with `multi-rpc`.
pub use multi_rpc_macros::multi_rpc_trait;
/// An attribute to expose a trait method as a REST endpoint. Used with the `rest-axum` feature.
pub use multi_rpc_macros::rest;
pub use serde;
#[cfg(feature = "tarpc")]
pub use tarpc;
