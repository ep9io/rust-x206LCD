#[derive(Debug, Clone)]
pub struct NvidiaInfo {
    pub name: String,
    pub temperature: f32,
    pub temperature_display: String,
    pub load: f32,
    pub load_display: String,
    pub memory_used: u64,
    pub memory_used_display: String,
    pub memory_total: u64,
    pub memory_total_display: String,
    pub memory_percent: f32,
    pub memory_percent_display: String,
}

impl Default for NvidiaInfo {
    fn default() -> Self {
        Self {
            name: String::from(""),
            temperature: 0.0,
            temperature_display: String::from("0.0°C"),
            load: 0.0,
            load_display: String::from("0%"),
            memory_used: 0,
            memory_used_display: String::from("0 MB"),
            memory_total: 0,
            memory_total_display: String::from("0 MB"),
            memory_percent: 0.0,
            memory_percent_display: String::from("0%"),
        }
    }
}