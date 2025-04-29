#[derive(Debug, Clone)]
pub struct Sensor {
    pub id: u32,
    pub path: String,
    pub name: String,
    pub label: String,
    pub model: String,
    pub temperature: f32,
}

impl Default for Sensor {
    fn default() -> Self {
        Self {
            id: 0,
            path: String::new(),
            name: String::new(),
            label: String::new(),
            model: String::new(),
            temperature: 0.0,
        }
    }
}