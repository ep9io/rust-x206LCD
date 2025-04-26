use env_logger::{Builder, WriteStyle};
use log::{error, LevelFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::new()
        .filter_level(LevelFilter::Info)
        .write_style(WriteStyle::Always)
        .format_timestamp_secs()
        .init();

    if let Err(e) = ax206lcd::run().await {
        error!("Application error: {}", e);
        return Err(e.into());
    }
    Ok(())
}
