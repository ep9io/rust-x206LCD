#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub percent: f32,
    pub percent_display: String,
    pub used: u64,
    pub used_display: String,
    pub total: u64,
    pub total_display: String,
}

#[derive(Debug, Clone)]
pub struct SwapMemoryInfo {
    pub percent: f32,
    pub percent_display: String,
    pub used: u64,
    pub used_display: String,
    pub total: u64,
    pub total_display: String,
}

impl Default for MemoryInfo {
    fn default() -> Self {
        Self {
            percent: 0.0,
            percent_display: String::from("0%"),
            used: 0,
            used_display: String::from("0 B"),
            total: 0,
            total_display: String::from("0 B"),
        }
    }
}
impl Default for SwapMemoryInfo {
    fn default() -> Self {
        Self {
            percent: 0.0,
            percent_display: String::from("0%"),
            used: 0,
            used_display: String::from("0 B"),
            total: 0,
            total_display: String::from("0 B"),
        }
    }
}