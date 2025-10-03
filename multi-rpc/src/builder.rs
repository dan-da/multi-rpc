use crate::runner::ServerRunner;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// A type alias for a future that can be spawned as a server task.
pub type ServerTask = Pin<Box<dyn Future<Output = ()> + Send>>;

/// A builder for configuring and launching multiple RPC servers from a single service implementation.
///
/// This provides a fluent interface to add different protocol servers that all delegate
/// to the same shared service logic.
pub struct ServerBuilder<S> {
    service: Arc<S>,
    task_factories: Vec<Box<dyn FnOnce(Arc<S>) -> ServerTask + Send>>,
}

impl<S> ServerBuilder<S>
where
    S: Send + Sync + 'static,
{
    /// Creates a new `ServerBuilder`.
    ///
    /// # Arguments
    ///
    /// * `service` - The service implementation that will be shared across all protocols.
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(service),
            task_factories: Vec::new(),
        }
    }

    /// Adds a protocol server to the builder.
    ///
    /// The `#[multi_rpc_impl]` macro generates protocol-specific factory functions
    /// (e.g., `greeter_impls::tarpc_tcp(...)`) that can be passed to this method.
    ///
    /// # Arguments
    ///
    /// * `factory` - A function that takes the shared service instance and returns a future
    ///   representing the running server task.
    pub fn add_protocol<F>(mut self, factory: F) -> Self
    where
        F: FnOnce(Arc<S>) -> ServerTask + Send + 'static,
    {
        self.task_factories.push(Box::new(factory));
        self
    }

    /// Consumes the builder, spawning all configured protocol servers on the Tokio runtime.
    ///
    /// Returns a `ServerRunner` which can be used to keep the application alive
    /// until a shutdown signal is received.
    pub fn build(self) -> Result<ServerRunner, std::io::Error> {
        println!("🚀 Launching servers...");
        let handles = self
            .task_factories
            .into_iter()
            .map(|task_fn| {
                let task = task_fn(self.service.clone());
                tokio::spawn(task)
            })
            .collect();
        Ok(ServerRunner { handles })
    }
}
