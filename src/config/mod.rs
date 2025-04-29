use anyhow::{Context, Result};
use config::{Config, File};
use indexmap::IndexMap;
use log::{debug, info, LevelFilter};
use serde::{Deserialize, Deserializer};
use std::fs;
use std::path::{Path};
use std::str::FromStr;

fn default_vid() -> u16 {
    0x1908
}

fn default_pid() -> u16 {
    0x0102
}

fn deserialize_hex_or_int<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    // Try parsing as hexadecimal if it starts with "0x"
    if value.starts_with("0x") || value.starts_with("0X") {
        u16::from_str_radix(&value[2..], 16).map_err(serde::de::Error::custom)
    } else {
        // Otherwise parse as decimal
        u16::from_str(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LcdConfig {
    pub backlight: u8,
    pub width: u16,
    pub height: u16,
    pub file: String,
    pub polling: u64,
    #[serde(default = "default_vid", deserialize_with = "deserialize_hex_or_int")]
    pub vid: u16,
    #[serde(default = "default_pid", deserialize_with = "deserialize_hex_or_int")]
    pub pid: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DashboardConfig {
    pub file: String,
    pub enabled: bool,
    pub save_to_file: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResourcesConfig {
    #[serde(default)]
    pub disks: Vec<String>,
    #[serde(default)]
    pub networks: Vec<String>,
    #[serde(default)]
    pub mount_points: Vec<String>,
    #[serde(default)]
    pub sensors: IndexMap<String, String>,
}

impl Default for ResourcesConfig {
    fn default() -> Self {
        let mut sensors = IndexMap::new();
        sensors.insert("k10temp".to_string(), "CPU".to_string());
        sensors.insert("amdgpu".to_string(), "GPU0".to_string());
        sensors.insert("NVIDIA RTX A2000".to_string(), "GPU1".to_string());
        sensors.insert("r8169".to_string(), "Eth0".to_string());
        sensors.insert("nvme composite".to_string(), "NVMe0".to_string());

        Self {
            disks: vec!["nvme0n1".to_string(), "sda1".to_string()],
            networks: vec!["enp13s0".to_string(), "enx0024278838ca".to_string()],
            mount_points: vec!["/".to_string()],
            sensors,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(rename = "LCD")]
    pub lcd: LcdConfig,
    #[serde(rename = "DASHBOARD")]
    pub dashboard: DashboardConfig,
    #[serde(rename = "RESOURCES", default)]
    pub resources: ResourcesConfig,
    #[serde(rename = "LOGGING", default)]
    pub logging: LoggingConfig,
}

impl Default for LcdConfig {
    fn default() -> Self {
        Self {
            backlight: 2,
            width: 420,
            height: 250,
            file: "current.png".to_string(),
            polling: 3,
            vid: default_vid(),
            pid: default_pid(),
        }
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            file: "dashboard.png".to_string(),
            enabled: false,
            save_to_file: false,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            lcd: LcdConfig::default(),
            dashboard: DashboardConfig::default(),
            resources: ResourcesConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn new() -> Result<Self> {
        Self::from_file("config.ini")
    }

    pub fn get_log_level(&self) -> LevelFilter {
        match self.logging.level.to_lowercase().as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            "off" => LevelFilter::Off,
            _ => LevelFilter::Info, // Default to Info if invalid
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_path = path.as_ref();
        debug!("Loading configuration from {}", config_path.display());

        let config = Config::builder()
            .add_source(File::with_name(config_path.to_str().unwrap_or("")).format(config::FileFormat::Ini))
            .build()
            .context(format!("Failed to load config from {}", config_path.display()))?;

        let app_config: AppConfig = config.try_deserialize()
            .context("Failed to deserialize config")?;

        Ok(app_config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let config_path = path.as_ref();

        // Build the config string
        let mut config_str = String::new();

        // LCD section
        config_str.push_str(&format!(
            "[LCD]\nbacklight = {}\nwidth = {}\nheight = {}\nfile = {}\npolling = {}\nvid = {:#06x}\npid = {:#06x}\n\n",
            self.lcd.backlight,
            self.lcd.width,
            self.lcd.height,
            self.lcd.file,
            self.lcd.polling,
            self.lcd.vid,
            self.lcd.pid
        ));

        // DASHBOARD section
        config_str.push_str(&format!(
            "[DASHBOARD]\nfile = {}\nenabled = {}\nsave_to_file = {}\n\n",
            self.dashboard.file,
            self.dashboard.enabled,
            self.dashboard.save_to_file
        ));

        // LOGGING section
        config_str.push_str(&format!(
            "[LOGGING]\nlevel = {}\n\n",
            self.logging.level
        ));

        // RESOURCES section
        config_str.push_str("[RESOURCES]\n");

        // Disks
        if !self.resources.disks.is_empty() {
            for disk in &self.resources.disks {
                config_str.push_str(&format!("disks = \"{}\"\n", disk));
            }
        }

        // Networks
        if !self.resources.networks.is_empty() {
            for network in &self.resources.networks {
                config_str.push_str(&format!("networks = \"{}\"\n", network));
            }
        }

        // Mount points
        if !self.resources.mount_points.is_empty() {
            for mount_point in &self.resources.mount_points {
                config_str.push_str(&format!("mount_points = \"{}\"\n", mount_point));
            }
        }

        // Sensors
        if !self.resources.sensors.is_empty() {
            config_str.push_str("\n[RESOURCES.sensors]\n");
            for (key, value) in &self.resources.sensors {
                config_str.push_str(&format!("{} = \"{}\"\n", key, value));
            }
        }

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
        assert_eq!(config.dashboard.file, "dashboard.png");
        assert_eq!(config.dashboard.enabled, false);
        assert_eq!(config.dashboard.save_to_file, false);
    }

    #[test]
    fn test_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = "[LCD]\nbacklight = 5\nwidth = 800\nheight = 600\nfile = \"test.png\"\npolling = 10\n\n[DASHBOARD]\nfile = \"test-dashboard.png\"\nenabled = true\nsave_to_file = true\n";

        temp_file.write_all(config_content.as_bytes()).unwrap();
        let config_path = temp_file.path();

        let config = AppConfig::from_file(config_path).unwrap();

        assert_eq!(config.lcd.backlight, 5);
        assert_eq!(config.lcd.width, 800);
        assert_eq!(config.lcd.height, 600);
        assert_eq!(config.lcd.file, "test.png");
        assert_eq!(config.lcd.polling, 10);
        assert_eq!(config.dashboard.file, "test-dashboard.png");
        assert_eq!(config.dashboard.enabled, true);
        assert_eq!(config.dashboard.save_to_file, true);
    }

    #[test]
    fn test_save_config() {
        let mut config = AppConfig::default();
        // Clear arrays to avoid serialization issues in tests
        config.resources.disks.clear();
        config.resources.networks.clear();
        config.resources.mount_points.clear();
        config.resources.sensors.clear();

        config.lcd.backlight = 7;
        config.lcd.width = 1024;
        config.lcd.height = 768;
        config.lcd.file = "saved.png".to_string();
        config.lcd.polling = 5;
        config.dashboard.file = "saved-dashboard.png".to_string();
        config.dashboard.enabled = true;
        config.dashboard.save_to_file = true;

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
        assert_eq!(loaded_config.dashboard.enabled, true);
        assert_eq!(loaded_config.dashboard.save_to_file, true);
    }

    #[test]
    fn test_hex_values() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = "[LCD]\nbacklight = 5\nwidth = 800\nheight = 600\nfile = \"test.png\"\npolling = 10\nvid = 0x1234\npid = 0x0102\n\n[DASHBOARD]\nfile = \"test-dashboard.png\"\nenabled = true\nsave_to_file = true\n";

        temp_file.write_all(config_content.as_bytes()).unwrap();
        let config_path = temp_file.path();

        let config = AppConfig::from_file(config_path).unwrap();

        // Check that hexadecimal values are correctly parsed
        assert_eq!(config.lcd.vid, 0x1234);
        assert_eq!(config.lcd.pid, 0x0102);
    }
}
