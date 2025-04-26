#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub percent: f32,
    pub percent_display: String,
    pub used: u64,
    pub used_display: String,
    pub total: u64,
    pub total_display: String,
}

#[derive(Debug, Clone)]
pub struct DiskIoInfo {
    pub read: u64,
    pub read_display: String,
    pub write: u64,
    pub write_display: String,
}

impl Default for DiskInfo {
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

impl Default for DiskIoInfo {
    fn default() -> Self {
        Self {
            read: 0,
            read_display: String::from("0 B"),
            write: 0,
            write_display: String::from("0 B"),
        }
    }
}