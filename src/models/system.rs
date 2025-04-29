use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub hostname: String,
    pub uptime: u64,
    pub sensors: SensorInfo,
    pub uptime_display: String,
    pub load_avg: (f32, f32, f32),
    pub load_avg_display: String,
}
impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            hostname: String::new(),
            uptime: 0,
            uptime_display: String::new(),
            sensors: SensorInfo::default(),
            load_avg: (0.0, 0.0, 0.0),
            load_avg_display: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeInfo {
    pub time: u64,
    pub time_display: String,
}
impl Default for TimeInfo {
    fn default() -> Self {
        Self {
            time: 0,
            time_display: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_percent: f32,
    pub cpu_percent: f32,
}
impl Default for ProcessInfo {
    fn default() -> Self {
        Self {
            pid: 0,
            name: String::new(),
            memory_percent: 0.0,
            cpu_percent: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemComponent {
    pub label: String,
    pub temperature: f32,
}

#[derive(Debug, Clone)]
pub struct SensorInfo {
    pub readings: IndexMap<String, SystemComponent>,
    pub display: String,
}
impl Default for SensorInfo {
    fn default() -> Self {
        Self {
            readings: IndexMap::new(),
            display: String::new(),
        }
    }
}
