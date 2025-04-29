use log::{debug, error};
use std::time::{Instant};
use sysinfo::{System as SysInfo};
use systemstat::{Platform, System};

pub async fn collect_load_aggregate(sys: &System) -> f32 {
    let start = Instant::now();
    let result = match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            // Use tokio's sleep instead of thread::sleep
            //time::sleep(Duration::from_secs(1)).await;
            tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
            match cpu.done() {
                Ok(cpu) => 1.0 - cpu.idle,
                Err(x) => {
                    error!("CPU load measurement error: {}", x);
                    0.0
                }
            }
        }
        Err(x) => {
            error!("CPU load: error: {}", x);
            0.0
        }
    };
    debug!(
        "collect_load_aggregate took: {} ms",
        start.elapsed().as_millis()
    );
    result
}

pub async fn collect_temperature(sys: &System) -> f32 {
    let start = Instant::now();
    let result = sys.cpu_temp().unwrap_or_else(|x| {
        error!("CPU temp: error: {}", x);
        0.0
    });
    debug!(
        "collect_temperature took: {} ms",
        start.elapsed().as_millis()
    );
    result
}

pub async fn collect_frequency(sys: &SysInfo) -> u64 {
    let start = Instant::now();
    //let result = sys.global_cpu_info().frequency();
    let result = sys.cpus().first().unwrap().frequency();
    debug!("collect_frequency took: {} ms", start.elapsed().as_millis());
    result
}

pub async fn collect_count(sys: &SysInfo) -> u64 {
    let start = Instant::now();
    let result = sys.cpus().len() as u64;
    debug!("collect_count took: {} ms", start.elapsed().as_millis());
    result
}
