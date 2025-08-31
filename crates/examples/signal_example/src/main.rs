use pulse::prelude::*;

mod components;
mod signals;

use chrono::Local;
use std::fs;
use tracing::{debug, error, info, trace};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory if it doesn't exist
    let log_dir = "logs";
    fs::create_dir_all(log_dir)?;

    // Set the default log level if RUST_LOG is not set
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,signal_example=debug"));

    // Format logs with timestamp and thread info
    let format = fmt::format()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_ansi(true)
        .with_timer(fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        ));

    // Clone the format for the second layer
    let file_format = format.clone();

    // Initialize the global subscriber with only file output
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_ansi(false)
                .event_format(file_format)
                .with_writer(std::fs::OpenOptions::new().create(true).append(true).open(
                    format!(
                        "logs/signal_example_{}.log",
                        Local::now().format("%Y%m%d_%H%M%S")
                    ),
                )?),
        )
        .init();

    info!(
        "Logging initialized. Logs will be written to {}/signal_example_*.log",
        log_dir
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging()?;

    // Log application start with some useful information
    info!("Starting signal example application");
    debug!("Debug logging enabled");
    trace!("Trace logging enabled");

    // Log environment information
    #[cfg(debug_assertions)]
    info!("Running in debug mode");

    #[cfg(not(debug_assertions))]
    info!("Running in release mode");

    // Log current working directory
    let current_dir = std::env::current_dir()?;
    info!("Current working directory: {}", current_dir.display());

    // Log environment variables
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        info!("RUST_LOG set to: {}", rust_log);
    } else {
        info!("RUST_LOG not set, using default log level");
    }

    // Create and run the app
    info!("Initializing application...");
    if let Err(e) = render(components::App::new) {
        error!(error = %e, "Application error");
        return Err(e);
    }

    info!("Application shutdown complete");
    Ok(())
}
