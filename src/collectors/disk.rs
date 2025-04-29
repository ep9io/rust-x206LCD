use log::{debug, error};
use std::time::{Duration, Instant};
use systemstat::{ByteSize, Platform, System};
use tokio::time;

pub async fn collect_io(sys: &System, allowed: &Vec<&str>) -> Vec<ByteSize> {
    let start = Instant::now();
    const SECTOR_SIZE: u64 = 512;

    let result = match sys.block_device_statistics() {
        Ok(initial_stats) => {
            let mut start_read_sectors = 0u64;
            let mut start_write_sectors = 0u64;

            for block in initial_stats.values() {
                if allowed.contains(&block.name.as_str()) {
                    start_read_sectors =
                        start_read_sectors.saturating_add(block.read_sectors as u64);
                    start_write_sectors =
                        start_write_sectors.saturating_add(block.write_sectors as u64);
                }
            }

            // Wait for the specified duration
            time::sleep(Duration::from_secs(1)).await;

            // Get new statistics after the wait
            match sys.block_device_statistics() {
                Ok(final_stats) => {
                    let mut end_read_sectors = 0u64;
                    let mut end_write_sectors = 0u64;

                    for block in final_stats.values() {
                        if allowed.contains(&block.name.as_str()) {
                            end_read_sectors =
                                end_read_sectors.saturating_add(block.read_sectors as u64);
                            end_write_sectors =
                                end_write_sectors.saturating_add(block.write_sectors as u64);
                        }
                    }

                    let read_bytes = end_read_sectors
                        .saturating_sub(start_read_sectors)
                        .saturating_mul(SECTOR_SIZE);
                    let write_bytes = end_write_sectors
                        .saturating_sub(start_write_sectors)
                        .saturating_mul(SECTOR_SIZE);

                    vec![ByteSize::b(read_bytes), ByteSize::b(write_bytes)]
                }
                Err(x) => {
                    error!("Block statistics error getting final stats: {}", x);
                    vec![ByteSize::b(0), ByteSize::b(0)]
                }
            }
        }
        Err(x) => {
            error!("Block statistics error getting initial stats: {}", x);
            vec![ByteSize::b(0), ByteSize::b(0)]
        }
    };
    debug!("collect_io took: {} ms", start.elapsed().as_millis());
    result
}

pub async fn collect_block_info(disks: &sysinfo::Disks, allowed: &Vec<&str>) -> Vec<ByteSize> {
    let start = Instant::now();
    let mut disk_total = 0;
    let mut disk_used = 0;

    for disk in disks {
        if let Some(mount_str) = disk.mount_point().to_str() {
            if !disk.is_removable() && allowed.contains(&mount_str) {
                disk_total += disk.total_space();
                disk_used += disk.total_space() - disk.available_space();
            }
        }
    }

    let result = vec![ByteSize::b(disk_used), ByteSize::b(disk_total)];
    debug!(
        "collect_block_info took: {} ms",
        start.elapsed().as_millis()
    );
    result
}
