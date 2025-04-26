#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub sent: u64,
    pub sent_display: String,
    pub recv: u64,
    pub recv_display: String,
}

impl Default for NetworkInfo {
    fn default() -> Self {
        Self {
            sent: 0,
            sent_display: String::new(),
            recv: 0,
            recv_display: String::new(),
        }
    }
}