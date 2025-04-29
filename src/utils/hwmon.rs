use std::fs::{read_dir, File};
use std::io::Read;
use std::path::Path;
use crate::models::sensor::Sensor;

/// Read out `hwmon` info (hardware monitor) from `folder`
/// to get values' path to be used on refresh as well as files containing `max`,
/// `critical value` and `label`. Then we store everything into `components`.
///
/// Note that a thermal [Component] must have a way to read its temperature.
/// If not, it will be ignored and not added into `components`.
///
/// ## What is read:
///
/// - Mandatory: `name` the name of the `hwmon`.
/// - Mandatory: `tempN_input` Drop [Component] if missing
/// - Optional: sensor `label`, in the general case content of `tempN_label`
///   see below for special cases
/// - Optional: `label`
/// - Optional: `/device/model`
/// - Optional: highest historic value in `tempN_highest`.
/// - Optional: max threshold value defined in `tempN_max`
/// - Optional: critical threshold value defined in `tempN_crit`
///
/// Where `N` is a `u32` associated to a sensor like `temp1_max`, `temp1_input`.
///
/// ## Doc to Linux kernel API.
///
/// Kernel hwmon API: https://www.kernel.org/doc/html/latest/hwmon/hwmon-kernel-api.html
/// DriveTemp kernel API: https://docs.kernel.org/gpu/amdgpu/thermal.html#hwmon-interfaces
/// Amdgpu hwmon interface: https://www.kernel.org/doc/html/latest/hwmon/drivetemp.html
pub fn from_hwmon(sensors: &mut Vec<Sensor>, folder: &Path) -> Option<()> {
    let dir = read_dir(folder).ok()?;
    for entry in dir.flatten() {
        if !entry.file_type().is_ok_and(|file_type| !file_type.is_dir()) {
            continue;
        }

        let entry = entry.path();
        let filename = entry.file_name().and_then(|x| x.to_str()).unwrap_or("");
        let Some((id, item)) = filename
            .strip_prefix("temp")
            .and_then(|f| f.split_once('_'))
            .and_then(|(id, item)| Some((id.parse::<u32>().ok()?, item)))
        else {
            continue;
        };

        if item != "input" {
            continue;       
        }

        let name = get_file_line(&folder.join("name"), 16).unwrap_or("".into());
        let model = get_file_line(&folder.join("device/model"), 16).unwrap_or("".into());
        let label = get_file_line(&folder.join(filename.replace("_input", "_label")), 16).unwrap_or("".into());
        let temperature = get_temperature_from_file(&folder.join(filename)).unwrap_or(0.0);
        let sensor = Sensor {
            id,
            path: entry.to_str().unwrap_or("").into(),
            name,
            label,
            model,
            temperature,
        };
        sensors.push(sensor);
    }

    Some(())
}

// Read arbitrary string data.
pub fn get_file_line(file: &Path, capacity: usize) -> Option<String> {
    let mut reader = String::with_capacity(capacity);
    let mut f = File::open(file).ok()?;
    f.read_to_string(&mut reader).ok()?;
    reader.truncate(reader.trim_end().len());
    Some(reader)
}


/// Designed at first for reading an `i32` or `u32` aka `c_long`
/// from a `/sys/class/hwmon` sysfs file.
fn read_number_from_file<N>(file: &Path) -> Option<N>
where
    N: std::str::FromStr,
{
    let mut reader = [0u8; 32];
    let mut f = File::open(file).ok()?;
    let n = f.read(&mut reader).ok()?;
    // parse and trim would complain about `\0`.
    let number = &reader[..n];
    let number = std::str::from_utf8(number).ok()?;
    let number = number.trim();
    // Assert that we cleaned a little bit that string.
    if cfg!(feature = "debug") {
        assert!(!number.contains('\n') && !number.contains('\0'));
    }
    number.parse().ok()
}

// Read a temperature from a `tempN_item` sensor form the sysfs.
// number returned will be in mili-celsius.
//
// Don't call it on `label`, `name` or `type` file.
#[inline]
fn get_temperature_from_file(file: &Path) -> Option<f32> {
    let temp = read_number_from_file(file);
    convert_temp_celsius(temp)
}

/// Takes a raw temperature in mili-celsius and convert it to celsius.
#[inline]
fn convert_temp_celsius(temp: Option<i32>) -> Option<f32> {
    temp.map(|n| (n as f32) / 1000f32)
}
