use crate::models::nvidia::NvidiaInfo;
use log::{debug, error};
use std::time::Instant;
use systemstat::ByteSize;
use tokio::process::Command;

pub async fn collect() -> Vec<NvidiaInfo> {
    let start = Instant::now();

    let cmd_start = Instant::now();
    let cmd = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=gpu_name,temperature.gpu,utilization.gpu,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .await;
    debug!("nvidia-smi command execution took: {} ms", cmd_start.elapsed().as_millis());

    let result = match cmd {
        Ok(output) => {
            if output.status.success() {
                let parse_start = Instant::now();
                let output_str = String::from_utf8_lossy(&output.stdout);
                // Split by newlines to handle multiple GPUs
                let info: Vec<NvidiaInfo> = output_str
                    .lines()
                    .filter_map(|line| {
                        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

                        if values.len() == 5 {
                            let name = values[0];
                            let temperature = values[1].parse::<f32>().unwrap_or(0.0);
                            let raw_load = values[2].parse::<f32>().unwrap_or(0.0);

                            // Ensure a GPU load is in range 0-100
                            let load = if raw_load > 0.0 && raw_load <= 1.0 {
                                raw_load
                            } else {
                                raw_load / 100.0
                            };

                            let memory_used = ByteSize::mib(values[3].parse::<f32>().unwrap_or(0.0) as u64);
                            let memory_total = ByteSize::mib(values[4].parse::<f32>().unwrap_or(0.0) as u64);

                            let mem_percent = if memory_total.as_u64() > 0 {
                                (memory_used.as_u64() as f32 / memory_total.as_u64() as f32) * 100.0
                            } else {
                                0.0
                            };

                            Some(NvidiaInfo {
                                name: name.to_string(),
                                temperature,
                                temperature_display: format!("{} °C", temperature),
                                load,
                                load_display: format!("{:.1}%", load * 100.0),
                                memory_used: memory_used.as_u64(),
                                memory_used_display: memory_used.to_string(),
                                memory_total: memory_total.as_u64(),
                                memory_total_display: memory_total.to_string(),
                                memory_percent: mem_percent,
                                memory_percent_display: format!("{:.1}%", mem_percent),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                debug!("Nvidia GPU data parsing took: {} ms", parse_start.elapsed().as_millis());
                info
            } else {
                vec![NvidiaInfo::default()]
            }
        }
        Err(e) => {
            error!("Error getting NVIDIA GPU info: {}", e);
            vec![NvidiaInfo::default()]
        }
    };

    debug!("collect (total Nvidia GPU info collection) took: {} ms", start.elapsed().as_millis());
    result
}