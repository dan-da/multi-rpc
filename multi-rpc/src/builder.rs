//! This module provides the `ServerBuilder` for configuring and launching multiple RPC servers
//! from a single service implementation.
//!
//! When you use `multi-rpc`, your single service object is shared across all the different
//! protocol servers you enable (e.g., tarpc, REST, jsonrpsee). To make this safe and efficient,
//! we rely on two key synchronization primitives: `Arc` and `Mutex`.
//!
//! # `Arc` for Shared Ownership
//! The `std::sync::Arc` (Atomically Reference Counted) enables your single service instance to be
//! owned by multiple server tasks simultaneously. When you launch a server for a protocol, it
//! receives a clone of the `Arc`, giving it a reference to the same underlying service. This
//! prevents the need to duplicate your service's state for each server and ensures all requests
//! are handled by the same, consistent logic.
//!
//! # `Mutex` for Thread-Safe Access
//! The `tokio::sync::Mutex` provides exclusive, thread-safe access to your service object.
//! While `Arc` allows multiple threads to own a reference, it doesn't prevent them from
//! trying to modify the data at the same time. The `Mutex` ensures that only one server
//! task can access or modify the service's state at any given moment. This is crucial for
//! methods that take `&mut self` as it prevents data races and keeps your application's
//! state consistent. We use the `tokio` version of `Mutex` because it works seamlessly
//! with asynchronous code, allowing locks to be held across `.await` points without blocking
//! the entire thread.
//!
//! # Using `ServerBuilder`
//! You use the `ServerBuilder` to set up your server:
//! 1. Create a new builder with your service object.
//! 2. Use `add_protocol` to specify the protocols and network addresses for each server you want to run.
//! 3. Call `build()` to create the servers.
//! 4. Finally, call `run()` on the resulting `ServerRunner` to start listening for requests.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::runner::ServerRunner;

pub type ServerTask = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type ServerTaskFactory<S> = Box<dyn FnOnce(Arc<Mutex<S>>) -> ServerTask + Send>;

pub struct ServerBuilder<S> {
    service: Arc<Mutex<S>>,
    task_factories: Vec<ServerTaskFactory<S>>,
}

impl<S> ServerBuilder<S>
where
    S: Send + Sync + 'static,
{
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(Mutex::new(service)),
            task_factories: Vec::new(),
        }
    }

    /// Adds a protocol's server task factory to the builder.
    pub fn add_protocol<F>(mut self, factory: F) -> Self
    where
        F: FnOnce(Arc<Mutex<S>>) -> ServerTask + Send + 'static,
    {
        self.task_factories.push(Box::new(factory));
        self
    }

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
