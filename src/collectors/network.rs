use log::{debug, error};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use systemstat::{ByteSize, Network, Platform, System};
use tokio::time;

pub async fn collect_io(sys: &System, allowed: &Vec<&str>) -> Vec<ByteSize> {
    let start = Instant::now();
    let result = match sys.networks() {
        Ok(initial_networks) => {
            let initial_results = get_aggregated_stats(sys, initial_networks, allowed);
            let initial_rx_bytes = initial_results[0].as_u64();
            let initial_tx_bytes = initial_results[1].as_u64();

            time::sleep(Duration::from_secs(1)).await;

            // Get new statistics after a second wait
            match sys.networks() {
                Ok(final_networks) => {
                    let final_results = get_aggregated_stats(sys, final_networks, allowed);
                    let final_rx_bytes = final_results[0].as_u64();
                    let final_tx_bytes = final_results[1].as_u64();

                    let rx_bytes = final_rx_bytes.saturating_sub(initial_rx_bytes);
                    let tx_bytes = final_tx_bytes.saturating_sub(initial_tx_bytes);

                    vec![ByteSize::b(rx_bytes), ByteSize::b(tx_bytes)]
                }
                Err(x) => {
                    error!("Network statistics error getting final stats: {}", x);
                    vec![ByteSize::b(0), ByteSize::b(0)]
                }
            }
        }
        Err(x) => {
            error!("Network statistics error getting initial stats: {}", x);
            vec![ByteSize::b(0), ByteSize::b(0)]
        }
    };
    debug!("collect_io took: {} ms", start.elapsed().as_millis());
    result
}

fn get_aggregated_stats(
    sys: &System,
    networks: BTreeMap<String, Network>,
    allowed: &Vec<&str>,
) -> Vec<ByteSize> {
    let start = Instant::now();
    let mut rx_bytes = 0u64;
    let mut tx_bytes = 0u64;

    for net in networks.values() {
        if allowed.contains(&net.name.as_str()) {
            if let Ok(stats) = sys.network_stats(&net.name) {
                rx_bytes = rx_bytes.saturating_add(stats.rx_bytes.as_u64());
                tx_bytes = tx_bytes.saturating_add(stats.tx_bytes.as_u64());
            }
        }
    }

    let result = vec![ByteSize::b(rx_bytes), ByteSize::b(tx_bytes)];
    debug!("get_aggregated_stats took: {} ms", start.elapsed().as_millis());
    result
}