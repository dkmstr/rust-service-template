// Copyright (c) 2025 Adolfo Gómez García <dkmaster@dkmon.com>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

/*!
Author: Adolfo Gómez, dkmaster at dkmon dot com
*/
use std::{sync::Arc, time::Duration};
use tokio::sync::Notify;

use anyhow::Result;

// Run service is platform dependent
// Will invoke back this "run" function,
#[cfg(target_os = "windows")]
use crate::windows_service::run_service;

pub trait AsyncServiceTrait: Send + Sync + 'static {
    fn run(&self, stop: Arc<Notify>);

    fn get_stop_notify(&self) -> Arc<Notify>;
}

pub struct AsyncService {
    // Add async fn to call as main_async
    main_async: fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
    stop: Arc<Notify>,
}

impl AsyncService {
    pub fn new(
        main_async: fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
        stop: Arc<Notify>,
    ) -> Self {
        Self { main_async, stop }
    }
    #[cfg(target_os = "windows")]
    pub fn run_service(self) -> Result<()> {
        run_service(self)
    }

    #[cfg(not(target_os = "windows"))]
    pub fn run_service(self) -> Result<()> {
        // On other, just run directly
        // Notify is a dummy here
        self.run(self.stop.clone());
        Ok(())
    }

    async fn signals(stop: Arc<Notify>) {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};

            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            let mut sigint = signal(SignalKind::interrupt()).unwrap();

            tokio::select! {
                _ = sigterm.recv() => {
                    shared::log::info!("Received SIGTERM");
                },
                _ = sigint.recv() => {
                    shared::log::info!("Received SIGINT");
                }
                _ = stop.notified() => {
                    shared::log::info!("Stop notified");
                    return;
                }
            }
            // Notify to stop
            stop.notify_waiters();
        }

        #[cfg(windows)]
        {
            // On windows, we don't have signals, just wait forever
            // The service control handler will notify us to stop
            stop.notified().await;
        }
    }
}

impl AsyncServiceTrait for AsyncService {
    fn run(&self, stop: Arc<Notify>) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all() // Enable timers, I/O, etc.
            .build()
            .unwrap();

        rt.block_on(async move {
            let mut main_task = tokio::spawn((self.main_async)());
            let signals_task = tokio::spawn(AsyncService::signals(stop.clone()));
            tokio::select! {
                res = &mut main_task => {
                    match res {
                        Ok(_) => {
                            crate::log::info!("Main async task completed");
                        },
                        Err(e) => {
                            crate::log::error!("Main async task failed: {}", e);
                        }
                    }
                    stop.notify_waiters();
                    signals_task.abort();  // This can be safely aborted
                },
                // Stop from SCM (on windows) or signal (on unix)
                _ = stop.notified() => {
                    crate::log::debug!("Stop received (external)");
                    // Main task may need to do some cleanup, give it some time
                    let grace = Duration::from_secs(16);
                    if tokio::time::timeout(grace, &mut main_task).await.is_err() {
                        crate::log::warn!("Main task did not stop in {grace:?}, aborting");
                        main_task.abort();
                    }
                    // Also abort signals task
                    signals_task.abort();
                }
            }
        });
    }

    fn get_stop_notify(&self) -> Arc<Notify> {
        self.stop.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;
    use tokio::time::timeout;

    async fn async_main() {
        // main logic
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            println!("Working...");
        }
    }

    #[tokio::test]
    async fn test_run_stops_on_notify() {
        let stop = Arc::new(Notify::new());
        let stop_clone = stop.clone();
        let stopped = Arc::new(AtomicBool::new(false));
        let stopped_clone = stopped.clone();

        let launcher = AsyncService::new(|| Box::pin(async_main()), stop.clone());
        let handle = std::thread::spawn(move || {
            launcher.run(stop_clone);
            stopped_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        // Let it run a bit
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(!stopped.load(std::sync::atomic::Ordering::SeqCst));

        // Notify to stop
        stop.notify_one();
        // Wait for thread to join, with timeout
        let res = timeout(Duration::from_secs(5), async {
            handle.join().unwrap();
        })
        .await;
        assert!(res.is_ok(), "Thread did not stop in time");
        assert!(stopped.load(std::sync::atomic::Ordering::SeqCst));
    }
}
