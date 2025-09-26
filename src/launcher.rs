use std::sync::Arc;
use tokio::sync::Notify;

pub fn run(stop: Arc<Notify>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all() // Enable timers, I/O, etc.
        .build()
        .unwrap();

    rt.block_on(async move {
        tokio::select! {
            _ = async_main() => {},
            _ = stop.notified() => {
                // shutdown logic
                println!("Stop received...");
            }
        }
    });
}

async fn async_main() {
    // main logic
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("Working...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_run_stops_on_notify() {
        let stop = Arc::new(Notify::new());
        let stop_clone = stop.clone();
        let stopped = Arc::new(AtomicBool::new(false));
        let stopped_clone = stopped.clone();

        let _handle = std::thread::spawn(move || {
            run(stop_clone);
            stopped_clone.store(true, Ordering::SeqCst);
        });

        // Wait a bit to make sure the service is running
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(
            !stopped.load(Ordering::SeqCst),
            "Service stopped too early"
        );

        // Notify to stop the service
        stop.notify_one();

        // Wait for the service to stop
        let result = timeout(Duration::from_secs(5), async {
            while !stopped.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await;
        assert!(result.is_ok(), "El servicio no se detuvo a tiempo");
    }
}
