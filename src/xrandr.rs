use std::collections::HashMap;
use std::process::Command;
use std::io;

use crate::monitor::*;
use crate::debug::xrandr_debug::XRANDR_OUTPUT;

// get initial monitor information from xrandr
pub fn get_monitor_info(debug: bool) -> io::Result<Monitors> {
    let output = Command::new("xrandr")
        .arg("--query")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut xrandr_lines = stdout.lines();

    if debug {
        xrandr_lines = XRANDR_OUTPUT.lines();
    }

    let mut selected_framerate = 0.0;
    let mut selected_resolution = (0, 0);
    let mut monitors: Monitors = Vec::new();
    let mut current_monitor: Option<Monitor> = None;
    let mut current_resolutions: HashMap<(i32, i32), Vec<f32>> = HashMap::new();  // HashMap to store resolutions and their framerates

    for line in xrandr_lines {
        // include ends with mm to make sure we only get monitors that are connected AND in use
        if line.contains(" connected") && line.ends_with("mm") {
            // Push the previous monitor to the list if there was one
            if let Some(monitor) = current_monitor.take() {
                monitors.push(Monitor {
                    name: monitor.name,
                    resolution: selected_resolution,
                    displayed_resolution: monitor.displayed_resolution,
                    scale: monitor.displayed_resolution.0 as f32/selected_resolution.0 as f32,
                    position: monitor.position,
                    is_primary: monitor.is_primary,
                    framerate: selected_framerate,
                    available_resolutions: current_resolutions,
                    is_selected: monitor.is_selected,
                    left: monitor.left,
                    right: monitor.right,
                    up: monitor.up,
                    down: monitor.down
                });
            }

            // Reset for the next monitor
            current_resolutions = HashMap::new();

            // Parse monitor name and primary status
            let parts: Vec<&str> = line.split_whitespace().collect();
            let name = parts[0].to_string();
            let is_primary = parts.contains(&"primary");
            let res_pos_part: Vec<&str> = if is_primary { parts[3].split("+").collect() } else { parts[2].split("+").collect() };
            let resolution_part = res_pos_part[0];
            let position_part = res_pos_part[1];

            let resolution: Vec<i32> = resolution_part.split('x')
                .map(|s| s.parse().unwrap_or(0))
                .collect();
            let position: Vec<i32> = position_part.split('+')
                .map(|s| s.parse().unwrap_or(0))
                .collect();
            let position2: Vec<i32> = res_pos_part[2].split('+')
                .map(|s| s.parse().unwrap_or(0))
                .collect();

            current_monitor = Some(Monitor {
                name,
                resolution: selected_resolution,
                displayed_resolution: (resolution[0], resolution[1]),
                scale: resolution[0] as f32/selected_resolution.0 as f32,
                position: (position[0], position2[0]),
                framerate: selected_framerate,
                is_primary,
                available_resolutions: HashMap::new(),
                is_selected: false,
                left: None,
                right: None,
                up: None,
                down: None
            });
        } else if line.contains(" disconnected") {
            // Push the previous monitor to the list if there was one
            if let Some(monitor) = current_monitor.take() {
                monitors.push(Monitor {
                    name: monitor.name,
                    resolution: selected_resolution,
                    displayed_resolution: monitor.displayed_resolution,
                    scale: monitor.displayed_resolution.0 as f32 / selected_resolution.0 as f32,
                    position: monitor.position,
                    framerate: selected_framerate,
                    is_primary: monitor.is_primary,
                    available_resolutions: current_resolutions,
                    is_selected: monitor.is_selected,
                    left: monitor.left,
                    right: monitor.right,
                    up: monitor.up,
                    down: monitor.down
                });
            }

            // Reset for the next monitor
            current_resolutions = HashMap::new();

            current_monitor = None;
        } else if let Some(_) = current_monitor.as_mut() {
            // Parse the resolution and framerates
            let parts: Vec<&str> = line.trim().split_whitespace().collect();

            // Check if the first part is in the format of a resolution (e.g., "2560x1600")
            if let Some(res_part) = parts.get(0) {
                if res_part.contains('x') {
                    let res: Vec<i32> = res_part
                        .split('x')
                        .map(|s| s.parse().unwrap_or(0))
                        .collect();

                    if res.len() == 2 {
                        let resolution = (res[0], res[1]);

                        // Parse framerates from subsequent parts
                        let mut framerates: Vec<f32> = Vec::new();
                        for rate in parts.iter().skip(1) {
                            // Remove any trailing '+' and check for '*' to mark it as selected
                            let cleaned_rate = rate.trim_end_matches(['+'].as_ref());
                            let is_selected = cleaned_rate.ends_with('*');
                            let cleaned_rate = rate.trim_end_matches(['+','*'].as_ref());
                            if let Ok(framerate) = cleaned_rate.parse::<f32>() {
                                if is_selected {
                                    selected_framerate = framerate;
                                    selected_resolution = resolution;
                                }
                                framerates.push(framerate);
                            }
                        }

                        // Insert the resolution and framerates into the hashmap
                        current_resolutions.entry(resolution)
                            .or_insert_with(Vec::new)
                            .extend(framerates);
                    }
                }
            }
        }
    }

    // Push the last monitor after the loop
    if let Some(monitor) = current_monitor.take() {
        monitors.push(Monitor {
            name: monitor.name,
            resolution: selected_resolution,
            displayed_resolution: monitor.displayed_resolution,
            scale: selected_resolution.0 as f32 / monitor.displayed_resolution.0 as f32,
            position: monitor.position,
            framerate: selected_framerate,
            is_primary: monitor.is_primary,
            available_resolutions: current_resolutions,
            is_selected: monitor.is_selected,
            left: monitor.left,
            right: monitor.right,
            up: monitor.up,
            down: monitor.down
        });
    }

    // setup proximity sensor. TODO: allow for margin of error
    Ok(monitors)
}

