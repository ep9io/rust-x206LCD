use crate::config::AppConfig;
use crate::models::SystemInfo;
use crate::renderer::colours::Colours;
use crate::renderer::{drawing, fonts};
use chrono::Local;
use image::RgbaImage;

pub fn render_resource_bars(
    config: &AppConfig,
    info: &SystemInfo,
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
) {
    let mut y_pos = y;

    let colours = Colours::default();

    let fc_regular = fonts::regular_font_config();

    let pre_bar_spacing = 28;
    let post_bar_spacing = 13;
    let bar_height = 20;

    // Add CPU text
    let cpu_text = format!(
        "CPU {} | {} | x{}",
        info.cpu.percent_display, info.cpu.freq_display, info.cpu.count_display
    );

    drawing::text(
        image,
        colours.text,
        (x + 5) as i32,
        y_pos as i32, // Position text above the progress bar
        &fc_regular,
        &cpu_text,
    );
    y_pos += pre_bar_spacing;

    drawing::progress_bar(
        image,
        (x + 5) as i32,
        y_pos as i32,
        width - 10,
        bar_height,
        info.cpu.percent,
        colours.cpu,
    );
    y_pos += bar_height + post_bar_spacing;

    // Draw Memory usage
    // Add Memory text
    let mem_text = format!(
        "MEM {} | {}/{}",
        info.memory.percent_display, info.memory.used_display, info.memory.total_display
    );

    drawing::text(
        image,
        colours.text,
        (x + 5) as i32,
        y_pos as i32, // Position text above the progress bar
        &fc_regular,
        &mem_text,
    );
    y_pos += pre_bar_spacing;

    drawing::progress_bar(
        image,
        (x + 5) as i32,
        y_pos as i32,
        width - 10,
        bar_height,
        info.memory.percent,
        colours.mem,
    );
    y_pos += bar_height + post_bar_spacing;

    // Draw Disk usage
    // Add Disk text
    let disk_text = format!(
        "DISK {} | {}/{}",
        info.disk.percent_display, info.disk.used_display, info.disk.total_display
    );

    drawing::text(
        image,
        colours.text,
        (x + 5) as i32,
        y_pos as i32, // Position text above the progress bar
        &fc_regular,
        &disk_text,
    );

    y_pos += pre_bar_spacing;
    drawing::progress_bar(
        image,
        (x + 5) as i32,
        y_pos as i32,
        width - 10,
        bar_height,
        info.disk.percent,
        colours.disk,
    );
    y_pos += bar_height + post_bar_spacing;

    // Draw GPU info if available
    for gpu in &info.nvidia {
        // Add GPU text
        let gpu_text = format!(
            "{} {} | {}/{}",
            gpu.name,
            gpu.load_display,
            gpu.memory_used_display,
            gpu.memory_total_display
        );

        drawing::text(
            image,
            colours.text,
            (x + 5) as i32,
            y_pos as i32, // Position text above the progress bar
            &fc_regular,
            &gpu_text,
        );

        y_pos += pre_bar_spacing;
        drawing::progress_bar(
            image,
            (x + 5) as i32,
            y_pos as i32,
            width - 10,
            bar_height,
            gpu.load,
            colours.gpu,
        );
        y_pos += bar_height + post_bar_spacing;
    }

}

pub fn render_processes(
    config: &AppConfig,
    info: &SystemInfo,
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
) {
    let colours = Colours::default();

    let fc_regular = fonts::regular_font_config();
    let fc_title = fonts::title_font_config();

    let mut y_pos = y;

    let process_gap = 23;

    // Top CPU processes
    drawing::text(
        image,
        colours.process,
        (x + 5) as i32,
        (y_pos as i32),
        &fc_title,
        "TOP CPU PROCESSES",
    );
    y_pos += 20;

    for proc in &info.top_cpu_processes {
        let proc_name = if proc.name.len() > 12 {
            &proc.name[..12]
        } else {
            &proc.name
        };
        let proc_text = format!(
            "{:<12} {:<9} {:>6.1}%",
            proc_name, proc.pid, proc.cpu_percent
        );
        drawing::text(
            image,
            colours.text,
            (x + 20) as i32,
            y_pos as i32,
            &fc_regular,
            &proc_text,
        );
        y_pos += process_gap;
    }

    y_pos += 8; // Space between CPU and memory sections

    // Top Memory processes
    drawing::text(
        image,
        colours.process,
        (x + 5) as i32,
        (y_pos as i32),
        &fc_title,
        "TOP MEMORY PROCESSES",
    );
    y_pos += 20;

    for proc in &info.top_memory_processes {
        let proc_name = if proc.name.len() > 12 {
            &proc.name[..12]
        } else {
            &proc.name
        };
        let proc_text = format!(
            "{:<12} {:<9} {:>6.1}%",
            proc_name, proc.pid, proc.memory_percent
        );
        drawing::text(
            image,
            colours.text,
            (x + 20) as i32,
            y_pos as i32,
            &fc_regular,
            &proc_text,
        );
        y_pos += process_gap;
    }
}

pub fn render_header(
    config: &AppConfig,
    info: &SystemInfo,
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
) {
    let colours = Colours::default();

    let fc_regular = fonts::regular_font_config();

    let y_pos = y;

    // Draw header
    let current_time = Local::now().format("%H:%M:%S").to_string();
    let header_text = format!(
        "{} | Up {} | Load Avg {}",
        info.system.hostname, info.system.uptime_display, info.system.load_avg_display
    );

    drawing::text(
        image,
        colours.header,
        (x + 5) as i32,
        y_pos as i32, // Position text above the progress bar
        &fc_regular,
        &header_text,
    );

    // Current time at top right
    drawing::text(
        image,
        colours.header,
        width as i32 - 100,
        y_pos as i32, // Position text above the progress bar
        &fc_regular,
        &current_time,
    );
}

pub fn render_footer(
    config: &AppConfig,
    info: &SystemInfo,
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
) {
    let colours = Colours::default();

    let fc_regular = fonts::regular_font_config();
    let fc_small = fonts::small_font_config();

    let mut y_pos = y;

    // Temperature section
    let sensors_text = info.system.sensors.display.as_str();

    if !sensors_text.is_empty() {
        // Draw temperature text
        drawing::text(
            image,
            colours.sensor,
            (x + 5) as i32,
            y_pos as i32,
            &fc_regular,
            &sensors_text,
        );
        y_pos += 28;
        drawing::horizonal_line(image, 0, y_pos, width);
        y_pos += 1;
    }

    let net_text = format!(
        "NET: ↓{} | ↑{}",
        info.network.recv_display, info.network.sent_display
    );
    let disk_io_text = format!(
        "DISK IO: ↓{} | ↑{}",
        info.disk_io.write_display,info.disk_io.read_display
    );

    // Draw network text
    drawing::text(
        image,
        colours.io,
        (x + 5) as i32,
        y_pos as i32,
        &fc_regular,
        &net_text,
    );

    //y_pos += 28;

    // Draw disk I/O text
    drawing::text(
        image,
        colours.io,
        (width as f64 / 2.0) as i32,
        y_pos as i32,
        &fc_regular,
        &disk_io_text,
    );

    // Draw line before syslog
    y_pos += 30;
    drawing::horizonal_line(image, 0, y_pos, width);
    y_pos += 1;

    for (i, line) in info.syslog_lines.iter().enumerate() {
        let y_position = y_pos + (i as u32 * 18);
        drawing::text(
            image,
            colours.log,
            (x + 5) as i32,
            y_position as i32,
            &fc_small,
            line,
        );
    }
}
