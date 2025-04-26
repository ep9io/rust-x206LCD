use ordermap::OrderMap;

pub(crate) mod cpu;
pub(crate) mod disk;
pub(crate) mod memory;
pub(crate) mod network;
pub(crate) mod nvidia;
pub(crate) mod system;


#[derive(Debug, Clone)]
pub struct AllowedResources {
    pub disks: Vec<String>,
    pub networks: Vec<String>,
    pub mount_points: Vec<String>,
    pub sensors: OrderMap<String, String>,
}



#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub cpu: cpu::CpuInfo,
    pub memory: memory::MemoryInfo,
    pub swap_memory: memory::SwapMemoryInfo,
    pub disk: disk::DiskInfo,
    pub disk_io: disk::DiskIoInfo,
    pub network: network::NetworkInfo,
    pub nvidia: Vec<nvidia::NvidiaInfo>,
    pub system: system::SystemMetrics,
    pub time: system::TimeInfo,
    pub syslog_lines: Vec<String>,
    pub top_cpu_processes: Vec<system::ProcessInfo>,
    pub top_memory_processes: Vec<system::ProcessInfo>,
}
