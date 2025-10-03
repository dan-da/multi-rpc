use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

/// A general-purpose error type for RPC service methods.
///
/// This enum is intended to be used within the `Result` returned by your service
/// trait's methods, allowing business logic errors to be serialized and sent to the client.
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum RpcError {
    /// Represents an internal server error or a logic failure.
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for RpcError {
    fn from(err: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        // Convert the error to its string representation and wrap it in our variant.
        RpcError::InternalError(err.to_string())
    }
}
