use std::io;
use tokio::task::JoinHandle;

/// Manages the spawned server tasks, waiting for a shutdown signal to terminate them.
pub struct ServerRunner {
    pub(crate) handles: Vec<JoinHandle<()>>,
}

impl ServerRunner {
    /// Runs all configured servers and blocks the current task until a shutdown
    /// signal (Ctrl+C) is received.
    ///
    /// Upon receiving the signal, it aborts all spawned server tasks.
    pub async fn run(self) -> io::Result<()> {
        println!("✅ Servers running. Press Ctrl+C to shut down.");
        tokio::signal::ctrl_c().await?;

        println!("\nShutdown signal received. Aborting server tasks...");
        for handle in self.handles {
            handle.abort();
        }

        Ok(())
    }
}