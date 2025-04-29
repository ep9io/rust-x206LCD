use crate::models::sensor::Sensor;
use crate::models::system::{ProcessInfo, SensorInfo, SystemComponent};
use crate::utils;
use crate::utils::file;
use indexmap::IndexMap;
use log::{debug, error};
use regex::Regex;
use std::fs::read_dir;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;
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

pub async fn collect_sensors(allowed: &IndexMap<String, String>) -> SensorInfo {
    let start = Instant::now();
    let mut sensors: Vec<Sensor> = Vec::new();
    if let Ok(dir) = read_dir(Path::new("/sys/class/hwmon/")) {
        for entry in dir.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let entry = entry.path();
            if !file_type.is_file()
                && entry
                    .file_name()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .starts_with("hwmon")
            {
                utils::hwmon::from_hwmon(&mut sensors, &entry);
            }
        }
    }

    let mut readings = IndexMap::<String, SystemComponent>::new();

    for sensor in &sensors {
        for (allowed_label_hint, rename_to) in allowed.iter() {
            let reference = format!(
                "{} {} {} {}",
                sensor.name, sensor.label, sensor.model, sensor.path
            )
            .replace("  ", " ")
            .trim()
            .to_lowercase()
            .to_string();
            debug!("sensor: {}", reference);
            if reference.contains(allowed_label_hint) {
                let component_info = SystemComponent {
                    label: rename_to.clone(),
                    temperature: sensor.temperature,
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
/*
pub async fn collect_processes0(count: usize) -> (Vec<ProcessInfo>, Vec<ProcessInfo>) {
    let mut sys = SysInfo::new_all();
    sys.processes();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_memory()
            .with_cpu()
            .with_exe(UpdateKind::OnlyIfNotSet),
    );
    tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;

    let mut cpu: Vec<ProcessInfo> = Vec::new();

    let total_memory = sys.total_memory();
    let mut mems: HashMap<String,ProcessInfo> = HashMap::new();
    for (pid, process) in sys.processes() {
        let exec = process.exe();
        if exec.is_none() {
            continue;
        }
        let exec_str = exec.map_or_else(String::new, |p| p.display().to_string());
        let cpu_percent = process.cpu_usage();
        let mem_percent = process.memory() as f32 / total_memory as f32 * 100.0;
        let name = exec_str.split('/').last().unwrap_or(process.name().to_str().unwrap_or("unknown")).to_string();
        let p = ProcessInfo {
            pid: pid.as_u32(),
            name: name.clone(),
            memory_percent: mem_percent,
            cpu_percent,
        };

        // Shorten the list that needs sorting
        if cpu_percent >= 0.0 {
            cpu.push(p.clone());
        }

        // Shorten the list that needs sorting
        if mem_percent > 0.1 {
            mems.insert(format!("{}{}",name,mem_percent), p);
        }
    }

    let mut memory:Vec<ProcessInfo> = mems.values().cloned().collect();
    // Sort by memory (descending)
    memory.sort_by(|a, b| b.memory_percent.total_cmp(&a.memory_percent));
    memory.truncate(count);

    // Sort by CPU (descending)
    cpu.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
    cpu.truncate(count);

    (cpu, memory)
}
pub async fn collect_processes2(count: usize) -> (Vec<ProcessInfo>, Vec<ProcessInfo>) {
    let mut sys = SysInfo::new_all();

    tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_memory()
            .with_cpu()
            .with_exe(UpdateKind::OnlyIfNotSet),
    );

    let total_memory = sys.total_memory();

    let mut memory: Vec<ProcessInfo> = Vec::new();
    let mut cpu: Vec<ProcessInfo> = Vec::new();

    let mut pids: HashSet<u32> = HashSet::new();
    for (pid, process) in sys.processes() {
        let is_memory = process.memory() > 0;
        let is_cpu = process.cpu_usage() > 0.0;
        let exec = process.exe();
        let exec_str = exec.map_or_else(String::new, |p| p.display().to_string());

        if !exec_str.is_empty() && (is_memory || is_cpu) {
            pids.insert(process.parent().unwrap_or(pid.clone()).as_u32());
        }
    }

    pids.iter().for_each(|pid| {
        if let Some(process) = sys.process(Pid::from(pid.clone() as usize)) {
            let exec = process.exe();
            let exec_str = exec.map_or_else(String::new, |p| p.display().to_string());
            let p = ProcessInfo {
                pid: pid.clone(),
                name: exec_str.split('/').last().unwrap_or("unknown").to_string(),
                memory_percent: process.memory() as f32 / total_memory as f32 * 100.0,
                cpu_percent: process.cpu_usage(),
            };

            if process.memory() > 0 {
                memory.push(p.clone());
            }

            if process.cpu_usage() > 0.0 {
                cpu.push(p);
            }
        }
    });

    // Sort by memory (descending)
    memory.sort_by(|a, b| b.memory_percent.total_cmp(&a.memory_percent));
    memory.truncate(count);

    // Sort by CPU (descending)
    cpu.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
    cpu.truncate(count);

    (cpu, memory)
}

*/
pub async fn collect_processes(count: usize) -> (Vec<ProcessInfo>, Vec<ProcessInfo>) {
    let memory = collect_processes_cmd("memory", count).await;
    let cpu = collect_processes_cmd("cpu", count).await;
    (cpu, memory)
}
pub async fn collect_processes_cmd(sort_by: &str, count: usize) -> Vec<ProcessInfo> {
    let start = Instant::now();
    let sort_key = match sort_by {
        "memory" => "pmem",
        "cpu" => "pcpu",
        _ => "pmem", // Default to memory
    };

    let cmd_start = Instant::now();
    let ps_command = Command::new("ps")
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
    match file::simple_tail("/var/log/syslog", num_lines) {
        Ok(lines) => {
            let parse_start = Instant::now();
            let result = lines
                .into_iter()
                .rev()
                .map(|line| {
                    let parts: Vec<&str> = line.splitn(3, ' ').collect();
                    let message = if parts.len() >= 3 {
                        parts[2].trim()
                    } else {
                        line.as_str()
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
        }
        Err(e) => {
            error!("Error reading syslog: {}", e);
            vec!["Error reading syslog".to_string()]
        }
    }
}
/*
pub async fn collect_recent_syslog_lines_tail(num_lines: usize, character_length: usize) -> Vec<String> {
    let start = Instant::now();

    let cmd_start = Instant::now();
    let tail_command = Command::new("tail")
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


 */
pub async fn get_hostname() -> String {
    let start = Instant::now();
    match file::read_to_string("/etc/hostname") {
        Ok(hostname) => {
            debug!("get_hostname took: {} ms", start.elapsed().as_millis());
            hostname.trim().to_string()
        }
        Err(e) => {
            error!("Failed to get hostname: {}", e);
            String::from("unknown")
        }
    }
}
/*
pub async fn get_hostname_cmd() -> String {
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
}

 */
