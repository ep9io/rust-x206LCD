use anyhow::{Context, Result};
use config::{Config, File};
use log::{debug, info};
use serde::Deserialize;
use std::fs;
use std::path::{Path};

#[derive(Debug, Deserialize, Clone)]
pub struct LcdConfig {
    pub backlight: u8,
    pub width: u16,
    pub height: u16,
    pub file: String,
    pub polling: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DashboardConfig {
    pub file: String,
    pub poll: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(rename = "LCD")]
    pub lcd: LcdConfig,
    #[serde(rename = "DASHBOARD")]
    pub dashboard: DashboardConfig,
}

impl Default for LcdConfig {
    fn default() -> Self {
        Self {
            backlight: 2,
            width: 420,
            height: 250,
            file: "current.png".to_string(),
            polling: 3,
        }
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            file: "system-dashboard.png".to_string(),
            poll: false,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            lcd: LcdConfig::default(),
            dashboard: DashboardConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn new() -> Result<Self> {
        Self::from_file("config.ini")
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_path = path.as_ref();
        debug!("Loading configuration from {}", config_path.display());

        let config = Config::builder()
            .add_source(File::from(config_path))
            .build()
            .context(format!("Failed to load config from {}", config_path.display()))?;

        let app_config: AppConfig = config.try_deserialize()
            .context("Failed to deserialize config")?;

        Ok(app_config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let config_path = path.as_ref();
        let config_str = format!(
            "[LCD]\nbacklight = {}\nwidth = {}\nheight = {}\nfile = {}\npolling = {}\n\n[DASHBOARD]\nfile = {}\npoll = {}\n",
            self.lcd.backlight,
            self.lcd.width,
            self.lcd.height,
            self.lcd.file,
            self.lcd.polling,
            self.dashboard.file,
            self.dashboard.poll
        );

        fs::write(config_path, config_str)
            .context(format!("Failed to save config to {}", config_path.display()))?;

        info!("Configuration saved to {}", config_path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.lcd.backlight, 2);
        assert_eq!(config.lcd.width, 420);
        assert_eq!(config.lcd.height, 250);
        assert_eq!(config.lcd.file, "current.png");
        assert_eq!(config.lcd.polling, 3);
        assert_eq!(config.dashboard.file, "system-dashboard.png");
        assert_eq!(config.dashboard.poll, false);
    }

    #[test]
    fn test_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = "[LCD]\nbacklight = 5\nwidth = 800\nheight = 600\nfile = \"test.png\"\npolling = 10\n\n[DASHBOARD]\nfile = \"test-dashboard.png\"\npoll = true\n";

        temp_file.write_all(config_content.as_bytes()).unwrap();
        let config_path = temp_file.path();

        let config = AppConfig::from_file(config_path).unwrap();

        assert_eq!(config.lcd.backlight, 5);
        assert_eq!(config.lcd.width, 800);
        assert_eq!(config.lcd.height, 600);
        assert_eq!(config.lcd.file, "test.png");
        assert_eq!(config.lcd.polling, 10);
        assert_eq!(config.dashboard.file, "test-dashboard.png");
        assert_eq!(config.dashboard.poll, true);
    }

    #[test]
    fn test_save_config() {
        let mut config = AppConfig::default();
        config.lcd.backlight = 7;
        config.lcd.width = 1024;
        config.lcd.height = 768;
        config.lcd.file = "saved.png".to_string();
        config.lcd.polling = 5;
        config.dashboard.file = "saved-dashboard.png".to_string();
        config.dashboard.poll = true;

        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path();

        config.save(config_path).unwrap();

        let loaded_config = AppConfig::from_file(config_path).unwrap();

        assert_eq!(loaded_config.lcd.backlight, 7);
        assert_eq!(loaded_config.lcd.width, 1024);
        assert_eq!(loaded_config.lcd.height, 768);
        assert_eq!(loaded_config.lcd.file, "saved.png");
        assert_eq!(loaded_config.lcd.polling, 5);
        assert_eq!(loaded_config.dashboard.file, "saved-dashboard.png");
        assert_eq!(loaded_config.dashboard.poll, true);
    }
}
