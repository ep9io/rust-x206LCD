use crate::models::system::{ProcessInfo, SensorInfo, SystemComponent};
use log::{debug, error};
use ordermap::OrderMap;
use regex::Regex;
use std::time::Duration;
use std::time::Instant;
use sysinfo::{ComponentExt, SystemExt};
use systemstat::{Platform, System};
use tokio::process::Command;

pub async fn collect_load(sys: &System) -> Vec<f32> {
    let start = Instant::now();
    let result = match sys.load_average() {
        Ok(loadavg) => vec![loadavg.one, loadavg.five, loadavg.fifteen],
        Err(x) => {
            error!("Load average: error: {}", x);
            vec![0.0, 0.0, 0.0]
        }
    };
    debug!("collect_load took: {} ms", start.elapsed().as_millis());
    result
}

pub async fn collect_uptime(sys: &System) -> (u64, Vec<u64>, Vec<String>) {
    let start = Instant::now();
    let uptime = sys.uptime().unwrap_or_else(|x| {
        error!("Uptime: error: {}", x);
        Duration::from_secs(0)
    });

    let duration = uptime.as_secs();
    let days = duration / (24 * 3600);
    let hours = (duration % (24 * 3600)) / 3600;
    let minutes = (duration % 3600) / 60;
    let seconds = duration % 60;

    let mut uptime_parts = Vec::new();
    let mut uptime_parts_str = Vec::new();
    if days > 0 {
        uptime_parts.push(days);
        uptime_parts_str.push(format!("{}d", days));
    }
    if hours > 0 {
        uptime_parts.push(hours);
        uptime_parts_str.push(format!("{}h", hours));
    }
    if minutes > 0 {
        uptime_parts.push(minutes);
        uptime_parts_str.push(format!("{}m", minutes));
    }
    if seconds > 0 {
        uptime_parts.push(seconds);
        uptime_parts_str.push(format!("{}s", seconds));
    }

    let result = (duration, uptime_parts, uptime_parts_str);
    debug!("collect_uptime took: {} ms", start.elapsed().as_millis());
    result
}

pub async fn collect_sensors(
    sys: &sysinfo::System,
    allowed: &OrderMap<String, String>,
) -> SensorInfo {
    let start = Instant::now();
    let mut readings = OrderMap::<String, SystemComponent>::new();

    for component in sys.components() {
        let label = component.label().to_lowercase();
        for (allowed_label_hint, rename_to) in allowed.iter() {
            if label.contains(allowed_label_hint) {
                let component_info = SystemComponent {
                    label: rename_to.clone(),
                    temperature: component.temperature(),
                };
                readings.insert(rename_to.clone(), component_info);
            }
        }
    }

    let result = SensorInfo {
        readings,
        display: String::new(), // Keep it blank for now, it'll need to be populated later with nvidia data.
    };
    debug!("collect_sensors took: {} ms", start.elapsed().as_millis());
    result
}

pub async fn collect_processes(sort_by: &str, count: usize) -> Vec<ProcessInfo> {
    let start = Instant::now();
    let sort_key = match sort_by {
        "memory" => "pmem",
        "cpu" => "pcpu",
        _ => "pmem", // Default to memory
    };

    let cmd_start = Instant::now();
    let mut ps_command = Command::new("ps")
        .args(&[
            "-eo",
            "pid,comm,%mem,%cpu",
            &format!("--sort=-{}", sort_key),
        ])
        .output()
        .await;
    debug!(
        "ps command execution took: {} ms",
        cmd_start.elapsed().as_millis()
    );

    let result = match ps_command {
        Ok(output) => {
            if output.status.success() {
                let parse_start = Instant::now();
                let output_str = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = output_str.lines().collect();

                let result = if lines.len() > 1 {
                    let data_lines = &lines[1..std::cmp::min(count + 1, lines.len())];
                    let re = Regex::new(r"^\s*(\d+)\s+(.+?)\s+(\d+\.?\d*)\s+(\d+\.?\d*)$").unwrap();

                    data_lines
                        .iter()
                        .filter_map(|line| {
                            re.captures(line).map(|captures| {
                                let pid = captures[1].parse::<u32>().unwrap_or(0);
                                let name = captures[2].trim().to_string();
                                let memory_percent = captures[3].parse::<f32>().unwrap_or(0.0);
                                let cpu_percent = captures[4].parse::<f32>().unwrap_or(0.0);

                                ProcessInfo {
                                    pid,
                                    name,
                                    memory_percent,
                                    cpu_percent,
                                }
                            })
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                debug!(
                    "Process data parsing took: {} ms",
                    parse_start.elapsed().as_millis()
                );
                result
            } else {
                Vec::new()
            }
        }
        Err(e) => {
            error!("Error getting top processes: {}", e);
            Vec::new()
        }
    };
    debug!(
        "collect_processes (total) took: {} ms",
        start.elapsed().as_millis()
    );
    result
}

pub async fn collect_recent_syslog_lines(num_lines: usize, character_length: usize) -> Vec<String> {
    let start = Instant::now();

    let cmd_start = Instant::now();
    let mut tail_command = Command::new("tail")
        .args(&["-n", &num_lines.to_string(), "/var/log/syslog"])
        .output()
        .await;
    debug!(
        "tail command execution took: {} ms",
        cmd_start.elapsed().as_millis()
    );

    let result = match tail_command {
        Ok(output) => {
            if output.status.success() {
                let parse_start = Instant::now();
                let output_str = String::from_utf8_lossy(&output.stdout);
                let result = output_str
                    .lines()
                    .map(|line| {
                        let parts: Vec<&str> = line.splitn(4, ':').collect();
                        let message = if parts.len() >= 4 {
                            parts[3].trim()
                        } else {
                            line
                        };

                        if message.len() > character_length {
                            format!("{}...", &message[..character_length])
                        } else {
                            message.to_string()
                        }
                    })
                    .collect();
                debug!(
                    "Syslog parsing took: {} ms",
                    parse_start.elapsed().as_millis()
                );
                result
            } else {
                vec!["Error reading syslog".to_string()]
            }
        }
        Err(e) => {
            error!("Error getting syslog: {}", e);
            vec!["Error reading syslog".to_string()]
        }
    };
    debug!(
        "collect_recent_syslog_lines (total) took: {} ms",
        start.elapsed().as_millis()
    );
    result
}

pub async fn get_hostname() -> String {
    /*
    let start = Instant::now();
    match Command::new("hostname")
        .output()
        .await {
        Ok(output) => {
            if output.status.success() {
                let hostname = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                debug!("get_hostname took: {} ms", start.elapsed().as_millis());
                hostname
            } else {
                error!("hostname command failed");
                String::from("unknown")
            }
        }
        Err(e) => {
            error!("Failed to execute hostname command: {}", e);
            String::from("unknown")
        }
    }

     */
    String::from("unknown")
}
