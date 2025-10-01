// Copyright (c) 2025 Adolfo Gómez García <dkmaster@dkmon.com>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

/*!
Author: Adolfo Gómez, dkmaster at dkmon dot com
*/
use std::{pin::Pin, sync::Arc};

mod log;
mod service;

#[cfg(target_os = "windows")]
mod windows_service;

use crate::service::AsyncService;

use tokio::sync::Notify;

fn async_main(stop: Arc<Notify>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        loop {
            tokio::select! {
                _ = stop.notified() => {
                    println!("Stop recibido");
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                    println!("Trabajando...");
                }
            }
        }
    })
}
fn main() {
    // Setup logging
    crate::log::setup_logging("info");

    // Create the async launcher with our main async function
    let service = AsyncService::new(async_main);

    // Run the service (on Windows) or directly (on other OS)
    if let Err(e) = service.run_service() {
        log::error!("Service failed to run: {}", e);
    }
}
