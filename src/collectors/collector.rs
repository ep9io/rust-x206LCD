use crate::models::{
    cpu::CpuInfo, disk::{DiskInfo, DiskIoInfo},
    memory::{MemoryInfo, SwapMemoryInfo},
    network::NetworkInfo,
    system::{SystemMetrics, TimeInfo},
    AllowedResources,
    SystemInfo,
};
use log::debug;
use indexmap::IndexMap;
use std::sync::Arc;
use sysinfo::{System as SysInfo, Disks as SysInfoDisks};
use systemstat::{Platform, System as SystemStat};
use tokio::{self};

use crate::collectors::{cpu, disk, memory, network, nvidia, system};
use crate::models::nvidia::NvidiaInfo;
use crate::models::system::{SensorInfo, SystemComponent};

pub(crate) async fn collect_system_info(allowed_resources: AllowedResources) -> SystemInfo {
    let allowed_disks: Vec<&str> = allowed_resources.disks.iter().map(|s| s.as_str()).collect();
    let allowed_mount_points: Vec<&str> = allowed_resources
        .mount_points
        .iter()
        .map(|s| s.as_str())
        .collect();
    let allowed_networks: Vec<&str> = allowed_resources
        .networks
        .iter()
        .map(|s| s.as_str())
        .collect();
    let allowed_sensors = allowed_resources.sensors.clone();

    let sys_stat = Arc::new(SystemStat::new());

    let mut sys_info = SysInfo::new_all();
    sys_info.refresh_all();
    let sys_info_arc = Arc::new(sys_info);

    let sys_info_disks_arc = Arc::new(SysInfoDisks::new_with_refreshed_list());



    // Wait for all tasks to complete
    let (
        disk_io,
        blocks,
        net,
        mem,
        swap,
        load,
        uptime,
        cpu_percent,
        cpu_temp,
        cpu_freq,
        cpu_count,
        nvidia,
        sensors,
        top_processes,
        syslog_lines,
    ) = tokio::join!(
        disk::collect_io(&sys_stat, &allowed_disks),
        disk::collect_block_info(&sys_info_disks_arc, &allowed_mount_points),
        network::collect_io(&sys_stat, &allowed_networks),
        memory::collect_ram(&sys_stat),
        memory::collect_swap(&sys_stat),
        system::collect_load(&sys_stat),
        system::collect_uptime(&sys_stat),
        cpu::collect_load_aggregate(&sys_stat),
        cpu::collect_temperature(&sys_stat),
        cpu::collect_frequency(&sys_info_arc),
        cpu::collect_count(&sys_info_arc),
        nvidia::collect(),
        system::collect_sensors(&allowed_sensors),
        system::collect_processes(5),
        system::collect_recent_syslog_lines(5, 75)
    );

    let memory_percent = mem[0].as_u64() as f32 / mem[1].as_u64() as f32;
    let swap_percent = swap[0].as_u64() as f32 / swap[1].as_u64() as f32;
    let block_percent = blocks[0].as_u64() as f32 / blocks[1].as_u64() as f32;
    let cpu_freq = cpu_freq as f32;


    let mut sensor_readings = IndexMap::new();
    let mut nvidia_gpus = Vec::new();
    for (allowed_label_hint, rename_to) in allowed_sensors.iter() {
        let allowed = allowed_label_hint.to_lowercase();
        if allowed.contains("nvidia") {
            // Find the matching NVIDIA GPU from the list
            if let Some(matching_gpu) = nvidia.iter().find(|gpu| gpu.name.to_lowercase().contains(&allowed)) {
                let component_info = SystemComponent {
                    label: rename_to.clone(),
                    temperature: matching_gpu.temperature,
                };
                sensor_readings.insert(rename_to.clone(), component_info);

                let n = NvidiaInfo {
                    name: rename_to.clone(),
                    temperature: matching_gpu.temperature,
                    temperature_display: matching_gpu.temperature_display.to_string(),
                    load: matching_gpu.load,
                    load_display: matching_gpu.load_display.to_string(),
                    memory_used: matching_gpu.memory_used,
                    memory_used_display: matching_gpu.memory_used_display.to_string(),
                    memory_total: matching_gpu.memory_total,
                    memory_total_display: matching_gpu.memory_total_display.to_string(),
                    memory_percent: matching_gpu.memory_percent,
                    memory_percent_display: matching_gpu.memory_percent_display.to_string(),
                };
                nvidia_gpus.push(n);
            }
        } else if let Some(reading) = sensors.readings.get(rename_to) {
            sensor_readings.insert(allowed.clone(), reading.clone());
        }
    }


    // Convert readings into output and display string
    let mut display_parts = Vec::new();
    for component in sensor_readings.values() {
        display_parts.push(format!("{:.0} {}", component.temperature, component.label));
    }
    let sensor_display = display_parts.join(" | ");
    let sensors = SensorInfo {
        readings: sensor_readings,
        display: format!("°C: {}", sensor_display),
    };

    let info = SystemInfo {
        cpu: CpuInfo {
            percent: cpu_percent,
            percent_display: format!("{:.1}%", cpu_percent * 100.0),
            freq: cpu_freq,
            freq_display: format!("{:.1} GHz", cpu_freq / 1000.0),
            count: cpu_count,
            count_display: cpu_count.to_string(),
            cpu_temp,
            cpu_temp_display: format!("{:.1} °C", cpu_temp),
        },

        memory: MemoryInfo {
            percent: memory_percent,
            percent_display: format!("{:.1}%", memory_percent * 100.0),
            used: mem[0].as_u64(),
            used_display: mem[0].to_string(),
            total: mem[1].as_u64(),
            total_display: mem[1].to_string(),
        },

        swap_memory: SwapMemoryInfo {
            percent: swap_percent,
            percent_display: format!("{:.1}%", swap_percent * 100.0),
            used: swap[0].as_u64(),
            used_display: swap[0].to_string(),
            total: swap[1].as_u64(),
            total_display: swap[1].to_string(),
        },

        disk: DiskInfo {
            percent: block_percent,
            percent_display: format!("{:.1}%", block_percent * 100.0),
            used: blocks[0].as_u64(),
            used_display: blocks[0].to_string(),
            total: blocks[1].as_u64(),
            total_display: blocks[1].to_string(),
        },

        disk_io: DiskIoInfo {
            read: disk_io[0].as_u64(),
            read_display: disk_io[0].to_string(),
            write: disk_io[1].as_u64(),
            write_display: disk_io[1].to_string(),
        },

        network: NetworkInfo {
            recv: net[0].as_u64(),
            recv_display: net[0].to_string(),
            sent: net[1].as_u64(),
            sent_display: net[1].to_string(),
        },

        nvidia: nvidia_gpus,

        system: SystemMetrics {
            hostname: system::get_hostname().await,
            sensors,
            uptime: uptime.0,
            uptime_display: uptime.2.join(" "),
            load_avg: (load[0], load[1], load[2]),
            load_avg_display: format!("{:.1} {:.1} {:.1}", load[0], load[1], load[2]),
        },

        time: TimeInfo {
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            time_display: chrono::Local::now().format("%H:%M:%S").to_string(),
        },
        top_cpu_processes: top_processes.0,
        top_memory_processes: top_processes.1,
        syslog_lines,
    };

    debug!("{:?}", info);

    info
}
