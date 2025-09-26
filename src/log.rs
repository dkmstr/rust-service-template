// Copyright (c) 2025 Adolfo Gómez García <dkmaster@dkmon.com>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

/*!
Author: Adolfo Gómez, dkmaster at dkmon dot com
*/
use std::{fs::OpenOptions, sync::OnceLock};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

// Reexport to avoid using crate names for tracing
#[allow(unused_imports)]
pub use tracing::{debug, error, info, trace, warn};

static LOGGER_INIT: OnceLock<()> = OnceLock::new();

#[allow(dead_code)]
pub fn setup_logging(level: &str) {
    let level = std::env::var("SERVICE_LOG_LEVEL").unwrap_or_else(|_| level.to_string());
    let log_path = std::env::var("SERVICE_LOG_PATH")
        .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into());

    // Bridge log crate logs to tracing
    LOGGER_INIT.get_or_init(|| {
        // Main log file
        let main_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(std::path::Path::new(&log_path).join("service.log"))
            .expect("Failed to open main log file");

        let main_layer = fmt::layer()
            .with_writer(main_file)
            .with_ansi(false)
            .with_target(true)
            .with_level(true);

        // In debug mode, add a console layer
        #[cfg(debug_assertions)]
        use tracing_subscriber::Layer;
        #[cfg(debug_assertions)]
        let main_layer = main_layer.and_then(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
                .with_target(true)
                .with_level(true),
        );

        tracing_subscriber::registry()
            .with(main_layer)
            .try_init()
            .ok();

        info!("Logging initialized with level: {}", level);
        info!("log_path resolved to: {}", log_path);
        #[cfg(debug_assertions)]
        {
            use std::env;
            info!("--- ENVIRONMENT VARIABLES ---");
            for (key, value) in env::vars() {
                info!("{} = {}", key, value);
            }
            info!("--- END ENVIRONMENT ---");
        }
    });
}

#[macro_export]
macro_rules! debug_dev {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            tracing::info!($($arg)*);
        }
    };
}
