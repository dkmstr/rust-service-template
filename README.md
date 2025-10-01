# Rust Service Template

A template for building **cross‑platform services in Rust** using **Tokio**.  
It integrates with the **Service Control Manager (SCM)** on Windows and handles **POSIX signals** on Unix/Linux.  

This project demonstrates how to structure a robust asynchronous service with **graceful shutdown** and a **configurable timeout**.

---

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
