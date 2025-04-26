use log::{debug, error};
use std::time::Instant;
use systemstat::{ByteSize, Platform, System};

pub async fn collect_ram(sys: &System) -> Vec<ByteSize> {
    let start = Instant::now();
    let result = match sys.memory() {
        Ok(mem) => {
            let used_bytes = mem.total.as_u64().saturating_sub(mem.free.as_u64());
            let total_bytes = mem.total.as_u64();
            vec![ByteSize::b(used_bytes), ByteSize::b(total_bytes)]
        }
        Err(x) => {
            error!("Memory statistics error getting stats: {}", x);
            vec![ByteSize::b(0), ByteSize::b(0)]
        }
    };
    debug!("collect_ram took: {} ms", start.elapsed().as_millis());
    result
}

pub async fn collect_swap(sys: &System) -> Vec<ByteSize> {
    let start = Instant::now();
    let result = match sys.swap() {
        Ok(swap) => {
            let used_bytes = swap.total.as_u64().saturating_sub(swap.free.as_u64());
            let total_bytes = swap.total.as_u64();
            vec![ByteSize::b(used_bytes), ByteSize::b(total_bytes)]
        }
        Err(x) => {
            error!("Swap statistics error getting stats: {}", x);
            vec![ByteSize::b(0), ByteSize::b(0)]
        }
    };
    debug!("collect_swap took: {} ms", start.elapsed().as_millis());
    result
}
