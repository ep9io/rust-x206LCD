pub mod config;
pub mod dashboard;

use crate::client::ax206lcd::AX206LCD;
use crate::config::AppConfig;
use crate::collectors::collector;
use crate::models::AllowedResources;
use anyhow::Context;
use log::{debug, error, info};
use std::time::Duration;

mod collectors;
mod models;
mod renderer;

mod client;

pub mod utils;

pub async fn run() -> anyhow::Result<()> {
    info!("Starting application");

    match main_loop().await {
        Ok(_) => info!("Application completed successfully"),
        Err(e) => {
            error!("Application error: {e:#}");
            // Print chain of error causes
            let mut source = e.source();
            while let Some(e) = source {
                error!("Caused by: {e}");
                source = e.source();
            }
            return Err(e).context("Application failed to run");
        }
    }

    Ok(())
}

async fn main_loop() -> anyhow::Result<()> {
    debug!("Loading configuration");
    let config = AppConfig::new().context("Failed to load configuration")?;
    let mut interval = tokio::time::interval(Duration::from_secs(config.lcd.polling));
    let mut lcd: Option<AX206LCD> = None;
    loop {
        interval.tick().await; // Wait for the next tick

        // Declare img variable to be used later
        let img;

        if config.dashboard.enabled {
            // Dashboard is enabled, collect system info and create a dashboard image
            let allowed_resources = AllowedResources {
                disks: config.resources.disks.clone(),
                networks: config.resources.networks.clone(),
                mount_points: config.resources.mount_points.clone(),
                sensors: config.resources.sensors.clone(),
            };

            debug!("Collecting system info");
            let info = collector::collect_system_info(allowed_resources).await;

            // Generate image from metrics
            img = dashboard::create_image(&config, &info);

            // Save image to file if configured to do so
            if config.dashboard.save_to_file {
                dashboard::save_image(&config, &img);
            }
        } else {
            // Dashboard is disabled, load image from file
            debug!("Loading image from file: {}", config.lcd.file);
            img = image::open(&config.lcd.file)
                .context(format!("Failed to load image from {}", config.lcd.file))?;
        }

        // Upload image to the device
        if lcd.is_none() {
            match AX206LCD::new(false) {
                Ok(device) => lcd = Some(device),
                Err(e) => {
                    error!("Failed to initialize LCD device: {}", e);
                    tokio::time::sleep(Duration::from_secs(10)).await; // Longer backoff for hardware errors
                    continue;
                }
            }
        }

        // Set device backlight
        if let Some(ref mut device) = lcd {
            if let Err(e) = device.set_backlight(config.lcd.backlight) {
                error!("Failed to set backlight: {}", e);
                lcd = None;
                continue;
            }

            // Draw the image on the device
            if let Err(e) = device.draw(&img) {
                error!("Failed to draw image: {}", e);
                lcd = None;
                continue;
            }
        }
    }
}
