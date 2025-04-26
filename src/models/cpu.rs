#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub percent: f32,
    pub percent_display: String,
    pub freq: f32,
    pub freq_display: String,
    pub count: u64,
    pub count_display: String,
    pub cpu_temp: f32,
    pub cpu_temp_display: String,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            percent: 0.0,
            percent_display: "0.0%".to_string(),
            freq: 0.0,
            freq_display: "0.0 MHz".to_string(),
            count: 0,
            count_display: "0 cores".to_string(),
            cpu_temp: 0.0,
            cpu_temp_display: "0.0%".to_string(),
        }
    }
}