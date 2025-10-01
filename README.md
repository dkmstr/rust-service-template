# Rust Service Template

A template for building **cross‑platform services in Rust** using **Tokio**.  
It integrates with the **Service Control Manager (SCM)** on Windows and handles **POSIX signals** on Unix/Linux.  

This project demonstrates how to structure a robust asynchronous service with **graceful shutdown** and a **configurable timeout**.

---

# Architecture diagram

+-----------------------------+
|  Windows SCM / Unix signals |
+-----------------------------+
           | (control)
           v
   tokio::sync::Notify (stop)
           |
           v
   AsyncService::run()
   - spawn main_async(stop)
   - spawn signals(stop)
   - select! {
       main completes  -> notify & abort signals
       stop notified   -> timeout & abort main, abort signals
     }
           |
           v
   async_main(stop) handles its own cleanup and exits

## ✨ Features

- ✅ Works on **Windows** (via the [`windows`](https://crates.io/crates/windows) crate)  
- ✅ Works on **Linux/Unix** (via `tokio::signal`)  
- ✅ Graceful shutdown using [`tokio::sync::Notify`]  
- ✅ Configurable grace period before aborting stuck tasks  
- ✅ Includes a lifecycle test (`cargo test`)  

---

## 🚀 Getting Started

### 1. Clone and build
```bash
git clone https://github.com/dkmstr/rust-service-template.git
cd rust-service-template
cargo build --release
```

### 2. Define your main async logic
```rust
use std::sync::Arc;
use tokio::sync::Notify;

async fn async_main(stop: Arc<Notify>) {
    loop {
        tokio::select! {
            _ = stop.notified() => {
                println!("Stop received, shutting down...");
                break;
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                println!("Working...");
            }
        }
    }
}
```

### 3. Connect it with the executor
```rust
use std::{future::Future, pin::Pin};
use std::sync::Arc;
use tokio::sync::Notify;

fn executor(stop: Arc<Notify>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        async_main(stop).await;
    })
}
```

Examine current `src/main.rs` for a complete example.