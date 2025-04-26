extern crate systemstat;
pub mod config;

use crate::client::ax206lcd::AX206LCD;
use crate::config::AppConfig;
use crate::models::AllowedResources;
use anyhow::Context;
use env_logger::{Builder, WriteStyle};
use log::{debug, error, info, LevelFilter};
use ordermap::OrderMap;
use std::collections::HashMap;
use std::time::Duration;

mod collector;
mod collectors;
mod dashboard;
mod models;
mod renderer;

mod client;

pub async fn run() -> anyhow::Result<()> {
    info!("Starting application");

    tokio::select! {
        result = main_loop() => {
            match result {
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
        }
    }

    Ok(())
}

async fn main_loop() -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let mut lcd: Option<AX206LCD> = None;
    debug!("Loading configuration");
    let config = AppConfig::new().context("Failed to load configuration")?;
    loop {
        interval.tick().await; // Wait for the next tick

        let allowed_resources = AllowedResources {
            disks: vec!["nvme0n1".into(), "sda1".into()],
            networks: vec!["enp13s0".into(), "enx0024278838ca".into()],
            mount_points: vec!["/".into()],
            sensors: OrderMap::from([
                ("k10temp".to_string(), "CPU".to_string()),
                ("amdgpu".to_string(), "GPU0".to_string()),
                ("NVIDIA RTX A2000".to_string(), "GPU1".to_string()),
                ("r8169".to_string(), "Eth0".to_string()),
                ("nvme composite".to_string(), "NVMe0".to_string()),
            ]),
        };

        debug!("Collecting system info");
        let info = collector::collect_system_info(allowed_resources).await;

        // Generate image from metrics
        let img = dashboard::create_image(&config, &info);
        //dashboard::save_image(&config, &img);

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

        if let Some(ref mut device) = lcd {
            if let Err(e) = device.set_backlight(config.lcd.backlight) {
                error!("Failed to set backlight: {}", e);
                lcd = None;
                continue;
            }

            if let Err(e) = device.draw(&img) {
                error!("Failed to draw image: {}", e);
                lcd = None;
                continue;
            }
        }
    }
}
