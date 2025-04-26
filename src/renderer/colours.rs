use image::Rgba;

pub(crate) struct Colours {
    pub header: Rgba<u8>,
    pub text: Rgba<u8>,
    pub cpu: Rgba<u8>,
    pub mem: Rgba<u8>,
    pub disk: Rgba<u8>,
    pub gpu: Rgba<u8>,
    pub sensor: Rgba<u8>,
    pub io: Rgba<u8>,
    pub process: Rgba<u8>,
    pub log: Rgba<u8>,
}

impl Default for Colours {
    fn default() -> Self {
        Self {
            header: Rgba([114, 159, 207, 255]),   // Steel blue - for headers
            text: Rgba([238, 238, 236, 255]),     // Off-white - for general text
            cpu: Rgba([87, 174, 36, 255]),        // Vibrant green - for CPU metrics
            mem: Rgba([52, 101, 164, 255]),       // Royal blue - for memory usage
            disk: Rgba([204, 0, 0, 255]),         // Crimson - for disk storage
            gpu: Rgba([173, 127, 168, 255]),      // Lavender - for GPU metrics
            sensor: Rgba([245, 121, 0, 255]),     // Burnt orange - for sensor readings
            io: Rgba([0, 188, 212, 255]),         // Cyan - for IO readings
            process: Rgba([237, 212, 0, 255]),    // Golden yellow - for processes
            log: Rgba([186, 189, 182, 255]),      // Silver gray - for logs
        }
    }
}