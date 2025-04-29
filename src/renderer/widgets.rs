use crate::config::AppConfig;
use crate::models::SystemInfo;
use crate::renderer::colours::Colours;
use crate::renderer::{drawing, fonts};
use chrono::Local;
use image::RgbaImage;

pub struct RenderContext<'a> {
    pub config: &'a AppConfig, // Placeholder for future customisations of the widgets
    pub info: &'a SystemInfo,
    pub image: &'a mut RgbaImage,
    pub x: u32,
    pub y: u32,
    pub width: u32,
}

pub fn render_resource_bars(ctx: &mut RenderContext) {
    let mut y_pos = ctx.y;

    let colours = Colours::default();

    let fc_regular = fonts::regular_font_config();

    let pre_bar_spacing = 28;
    let post_bar_spacing = 13;
    let bar_height = 20;

    // Add CPU text
    let cpu_text = format!(
        "CPU {} | {} | x{}",
        ctx.info.cpu.percent_display, ctx.info.cpu.freq_display, ctx.info.cpu.count_display
    );

    drawing::text(
        ctx.image,
        colours.text,
        (ctx.x + 5) as i32,
        y_pos as i32, // Position text above the progress bar
        &fc_regular,
        &cpu_text,
    );
    y_pos += pre_bar_spacing;

    drawing::progress_bar(
        ctx.image,
        (ctx.x + 5) as i32,
        y_pos as i32,
        ctx.width - 10,
        bar_height,
        ctx.info.cpu.percent,
        colours.cpu,
    );
    y_pos += bar_height + post_bar_spacing;

    // Draw Memory usage
    // Add Memory text
    let mem_text = format!(
        "MEM {} | {}/{}",
        ctx.info.memory.percent_display, ctx.info.memory.used_display, ctx.info.memory.total_display
    );

    drawing::text(
        ctx.image,
        colours.text,
        (ctx.x + 5) as i32,
        y_pos as i32, // Position text above the progress bar
        &fc_regular,
        &mem_text,
    );
    y_pos += pre_bar_spacing;

    drawing::progress_bar(
        ctx.image,
        (ctx.x + 5) as i32,
        y_pos as i32,
        ctx.width - 10,
        bar_height,
        ctx.info.memory.percent,
        colours.mem,
    );
    y_pos += bar_height + post_bar_spacing;

    // Draw Disk usage
    // Add Disk text
    let disk_text = format!(
        "DISK {} | {}/{}",
        ctx.info.disk.percent_display, ctx.info.disk.used_display, ctx.info.disk.total_display
    );

    drawing::text(
        ctx.image,
        colours.text,
        (ctx.x + 5) as i32,
        y_pos as i32, // Position text above the progress bar
        &fc_regular,
        &disk_text,
    );

    y_pos += pre_bar_spacing;
    drawing::progress_bar(
        ctx.image,
        (ctx.x + 5) as i32,
        y_pos as i32,
        ctx.width - 10,
        bar_height,
        ctx.info.disk.percent,
        colours.disk,
    );
    y_pos += bar_height + post_bar_spacing;

    // Draw GPU info if available
    for gpu in &ctx.info.nvidia {
        // Add GPU text
        let gpu_text = format!(
            "{} {} | {}/{}",
            gpu.name,
            gpu.load_display,
            gpu.memory_used_display,
            gpu.memory_total_display
        );

        drawing::text(
            ctx.image,
            colours.text,
            (ctx.x + 5) as i32,
            y_pos as i32, // Position text above the progress bar
            &fc_regular,
            &gpu_text,
        );

        y_pos += pre_bar_spacing;
        drawing::progress_bar(
            ctx.image,
            (ctx.x + 5) as i32,
            y_pos as i32,
            ctx.width - 10,
            bar_height,
            gpu.load,
            colours.gpu,
        );
        y_pos += bar_height + post_bar_spacing;
    }

}

pub fn render_processes(ctx: &mut RenderContext) {
    let colours = Colours::default();

    let fc_regular = fonts::regular_font_config();
    let fc_title = fonts::title_font_config();

    let mut y_pos = ctx.y;

    let process_gap = 23;

    // Top CPU processes
    drawing::text(
        ctx.image,
        colours.process,
        (ctx.x + 5) as i32,
        y_pos as i32,
        &fc_title,
        "TOP CPU PROCESSES",
    );
    y_pos += 20;

    for proc in &ctx.info.top_cpu_processes {
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
            ctx.image,
            colours.text,
            (ctx.x + 20) as i32,
            y_pos as i32,
            &fc_regular,
            &proc_text,
        );
        y_pos += process_gap;
    }

    y_pos += 8; // Space between CPU and memory sections

    // Top Memory processes
    drawing::text(
        ctx.image,
        colours.process,
        (ctx.x + 5) as i32,
        y_pos as i32,
        &fc_title,
        "TOP MEMORY PROCESSES",
    );
    y_pos += 20;

    for proc in &ctx.info.top_memory_processes {
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
            ctx.image,
            colours.text,
            (ctx.x + 20) as i32,
            y_pos as i32,
            &fc_regular,
            &proc_text,
        );
        y_pos += process_gap;
    }
}

pub fn render_header(ctx: &mut RenderContext) {
    let colours = Colours::default();

    let fc_regular = fonts::regular_font_config();

    let y_pos = ctx.y;

    // Draw header
    let current_time = Local::now().format("%H:%M:%S").to_string();
    let header_text = format!(
        "{} | Up {} | Load Avg {}",
        ctx.info.system.hostname, ctx.info.system.uptime_display, ctx.info.system.load_avg_display
    );

    drawing::text(
        ctx.image,
        colours.header,
        (ctx.x + 5) as i32,
        y_pos as i32, // Position text above the progress bar
        &fc_regular,
        &header_text,
    );

    // Current time at top right
    drawing::text(
        ctx.image,
        colours.header,
        ctx.width as i32 - 100,
        y_pos as i32, // Position text above the progress bar
        &fc_regular,
        &current_time,
    );
}

pub fn render_footer(ctx: &mut RenderContext) {
    let colours = Colours::default();

    let fc_regular = fonts::regular_font_config();
    let fc_small = fonts::small_font_config();

    let mut y_pos = ctx.y;

    // Temperature section
    let sensors_text = ctx.info.system.sensors.display.as_str();

    if !sensors_text.is_empty() {
        // Draw temperature text
        drawing::text(
            ctx.image,
            colours.sensor,
            (ctx.x + 5) as i32,
            y_pos as i32,
            &fc_regular,
            &sensors_text,
        );
        y_pos += 28;
        drawing::horizonal_line(ctx.image, 0, y_pos, ctx.width);
        y_pos += 1;
    }

    let net_text = format!(
        "NET: ↓{} | ↑{}",
        ctx.info.network.recv_display, ctx.info.network.sent_display
    );
    let disk_io_text = format!(
        "DISK IO: ↓{} | ↑{}",
        ctx.info.disk_io.write_display, ctx.info.disk_io.read_display
    );

    // Draw network text
    drawing::text(
        ctx.image,
        colours.io,
        (ctx.x + 5) as i32,
        y_pos as i32,
        &fc_regular,
        &net_text,
    );

    //y_pos += 28;

    // Draw disk I/O text
    drawing::text(
        ctx.image,
        colours.io,
        (ctx.width as f64 / 2.0) as i32,
        y_pos as i32,
        &fc_regular,
        &disk_io_text,
    );

    // Draw line before syslog
    y_pos += 30;
    drawing::horizonal_line(ctx.image, 0, y_pos, ctx.width);
    y_pos += 1;

    for (i, line) in ctx.info.syslog_lines.iter().enumerate() {
        let y_position = y_pos + (i as u32 * 18);
        drawing::text(
            ctx.image,
            colours.log,
            (ctx.x + 5) as i32,
            y_position as i32,
            &fc_small,
            line,
        );
    }
}
