use env_logger::{Builder, WriteStyle};
use log::{error};
use ax206lcd::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration first (without logging)
    let config = AppConfig::new().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {}", e);
        // Fall back to default configuration
        AppConfig::default()
    });

    // Initialise logger with a configured log level
    Builder::new()
        .filter_level(config.get_log_level())
        .write_style(WriteStyle::Always)
        .format_timestamp_secs()
        .init();

    if let Err(e) = ax206lcd::run().await {
        error!("Application error: {}", e);
        return Err(e.into());
    }
    Ok(())
}
