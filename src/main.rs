use std::io;
use std::process::Command;
use std::collections::HashMap;
use tui::widgets::canvas::Rectangle;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, canvas::Canvas},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Clone, PartialEq)]
struct Monitor {
    name: String,
    resolution: (i32, i32),
    available_resolutions: HashMap<(i32, i32), Vec<f32>>,  // Resolutions with vector of framerates
    framerate: f32,
    position: (i32, i32),
    resolution_string: String,
    position_string: String,
    is_primary: bool,
    is_selected: bool,
    left: Option<usize>,
    right: Option<usize>,
    up: Option<usize>,
    down: Option<usize>
}

enum FocusedWindow {
    MonitorList,
    MonitorInfo,
}

#[derive(Clone, PartialEq)]
enum State {
    Main,
    MonitorEdit,
    InfoEdit,
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get monitor information
    let monitors = get_monitor_info()?;

    // Run the main loop
    let res = run_app(&mut terminal, monitors);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn swap_monitors(monitors: &mut Vec<Monitor>, direction: i32, selected_index: usize, current_monitor: usize, current_state: &State) {
    assert!(*current_state == State::MonitorEdit, "Tried to swap monitors when not in monitor edit state");
    let temp_monitor = monitors[selected_index].clone();
    if direction == 1 {
        monitors[selected_index].position = monitors[current_monitor].position;
        monitors[current_monitor].position.0 += temp_monitor.resolution.0 as i32;
    } else if direction == 2 {
        monitors[selected_index].position.0 += monitors[current_monitor].resolution.0 as i32;
        monitors[current_monitor].position = temp_monitor.position;
    } else if direction == 3 {
        monitors[selected_index].position = monitors[current_monitor].position;
        monitors[current_monitor].position.1 += temp_monitor.resolution.1 as i32;
    } else if direction == 4 {
        monitors[selected_index].position.1 += monitors[current_monitor].resolution.1 as i32;
        monitors[current_monitor].position = temp_monitor.position;
    }
    monitors[selected_index].left = monitors[current_monitor].left;
    monitors[selected_index].right = monitors[current_monitor].right;
    monitors[selected_index].up = monitors[current_monitor].up;
    monitors[selected_index].down = monitors[current_monitor].down;
    monitors[current_monitor].left = temp_monitor.left;
    monitors[current_monitor].right = temp_monitor.right;
    monitors[current_monitor].up = temp_monitor.up;
    monitors[current_monitor].down = temp_monitor.down;

    // update order
    monitors.swap(selected_index, current_monitor);
}

fn run_app<B: tui::backend::Backend>(terminal: &mut Terminal<B>, mut monitors: Vec<Monitor>) -> io::Result<()> {
    let mut selected_index = 0;
    let mut current_monitor = 0;
    let mut selected = false;
    let mut focused_window = FocusedWindow::MonitorList;
    let mut current_state = State::Main;

    let mut info_index = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(f.size());

            let monitor_block = Block::default()
                .title("Monitors")
                .borders(Borders::ALL)
                .style(Style::default().fg(if matches!(focused_window, FocusedWindow::MonitorList) {
                    if matches!(current_state, State::MonitorEdit) {
                        Color::LightMagenta
                    } else {
                        Color::Yellow
                    }
                } else {
                    Color::White
                }));

            let monitor_area = monitor_block.inner(chunks[0]);
            f.render_widget(monitor_block, chunks[0]);

            draw_monitors(f, monitor_area, &monitors, selected_index);

            let info = if let Some(monitor) = monitors.get(selected_index) {
                format!(
                    "Name: {}\nResolution: {}x{}\nPosition: ({}, {})\nResolution string: {}\nPosition string: {}\nPrimary: {}\nup: {}\ndown: {}\nleft: {}\n right: {}\nframerate: {}hz\nResolutions: {:?}\n",
                    monitor.name,
                    monitor.resolution.0,
                    monitor.resolution.1,
                    monitor.position.0,
                    monitor.position.1,
                    monitor.resolution_string,
                    monitor.position_string,
                    if monitor.is_primary { "Yes" } else { "No" },
                    if monitor.up != None { monitors[monitor.up.unwrap()].name.to_string() } else { "None".to_string() },
                    if monitor.down != None { monitors[monitor.down.unwrap()].name.to_string() } else { "None".to_string() },
                    if monitor.left != None { monitors[monitor.left.unwrap()].name.to_string() } else { "None".to_string() },
                    if monitor.right != None { monitors[monitor.right.unwrap()].name.to_string() } else { "None".to_string() },
                    monitor.framerate,
                    monitor.available_resolutions,
                )
            } else {
                "No monitor selected".to_string()
            };

            let info_block = Block::default()
                .title("Monitor Info")
                .borders(Borders::ALL)
                .style(Style::default().fg(
                if matches!(current_state, State::InfoEdit) {
                    Color::LightMagenta
                } else if matches!(focused_window, FocusedWindow::MonitorInfo) {
                    Color::Yellow
                } else {
                    Color::White
                }));

            let info_paragraph = Paragraph::new(info)
                .block(info_block)
                .style(Style::default().fg(Color::White))
                .wrap(tui::widgets::Wrap { trim: true });

            f.render_widget(info_paragraph, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                // quit
                KeyCode::Char('q') => return Ok(()),
                // save
                KeyCode::Char('s') => {
                    let mut iterator = monitors.iter_mut();
                    let mut args: Vec<String> = Vec::new();
                    while let Some(element) = iterator.next() {
                        args.push("--output".to_string());
                        args.push(element.name.to_string());
                        if element.is_primary {
                            args.push("--primary".to_string());
                        }
                        args.push("--mode".to_string());
                        let modestr = format!("{}x{}", element.resolution.0, element.resolution.1);
                        args.push(modestr);
                        args.push("--pos".to_string());
                        let posstr = format!("{}x{}", element.position.0, element.position.1);
                        args.push(posstr);

                    };
                    let output = Command::new("xrandr")
                        .args(args)
                        .output()?;
                    println!("{:?}", output);
                }
                // horizontal movement
                KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Left | KeyCode::Right => {
                    if matches!(current_state, State::MonitorEdit) {
                        let mut direction = 0;
                        if (key.code == KeyCode::Char('l')) | (key.code == KeyCode::Right) {
                            if monitors[selected_index].right.is_some() {
                                selected_index = monitors[selected_index].right.unwrap();
                                direction = 1;
                            }
                        } else {
                            if monitors[selected_index].left.is_some() {
                                selected_index = monitors[selected_index].left.unwrap();
                                direction = 2;
                            }
                        }
                        if direction > 0 && selected  {
                            swap_monitors(&mut monitors, direction, selected_index, current_monitor, &current_state);
                            current_monitor = selected_index;
                        }
                    }
                }
                // vertical movement
                KeyCode::Char('j') | KeyCode::Char('k') | KeyCode::Up | KeyCode::Down => {
                    if matches!(current_state, State::Main) {
                        focused_window = match focused_window {
                            FocusedWindow::MonitorList => FocusedWindow::MonitorInfo,
                            FocusedWindow::MonitorInfo => FocusedWindow::MonitorList,
                        };
                    } else if matches!(current_state, State::MonitorEdit) {
                        let mut direction = 0;
                        if (key.code == KeyCode::Char('j')) | (key.code == KeyCode::Down) {
                            if monitors[selected_index].down.is_some() {
                                selected_index = monitors[selected_index].down.unwrap();
                                direction = 3;
                            }
                        } else {
                            if monitors[selected_index].up.is_some() {
                                selected_index = monitors[selected_index].up.unwrap();
                                direction = 4;
                            }
                        }
                        if direction > 0 && selected {
                            swap_monitors(&mut monitors, direction, selected_index, current_monitor, &current_state);
                            current_monitor = selected_index;
                        }
                    }
                }
                // selection
                KeyCode::Enter => {
                    if matches!(current_state, State::Main) {
                        if matches!(focused_window, FocusedWindow::MonitorList) {
                            current_state = State::MonitorEdit;
                        } else {
                            current_state = State::InfoEdit;
                        }
                    } else if matches!(current_state, State::MonitorEdit) {
                        if monitors[current_monitor].is_selected {
                            monitors[current_monitor].is_selected = false;
                            selected = false;
                        } else {
                            current_monitor = selected_index;
                            monitors[selected_index].is_selected = !monitors[selected_index].is_selected;
                            selected = true;
                        }
                    }
                }
                // set primary
                KeyCode::Char('p') => {
                    if matches!(current_state, State::MonitorEdit) {
                        let mut iterator = monitors.iter_mut();
                        while let Some(element) = iterator.next() {
                            element.is_primary = false;
                        }
                        monitors[selected_index].is_primary = true;
                    }
                }
                // Deselect
                KeyCode::Esc => {
                    if matches!(current_state, State::MonitorEdit) || matches!(current_state, State::InfoEdit) {
                        current_state = State::Main;
                    }
                }
                _ => {}
            }
        }
    }
}

// get initial monitor information from xrandr
fn get_monitor_info() -> io::Result<Vec<Monitor>> {
    let output = Command::new("xrandr")
        .arg("--query")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut selected_framerate = 0.0;
    let mut monitors: Vec<Monitor> = Vec::new();
    let mut current_monitor: Option<Monitor> = None;
    let mut current_resolutions: HashMap<(i32, i32), Vec<f32>> = HashMap::new();  // HashMap to store resolutions and their framerates

    for line in stdout.lines() {
        if line.contains(" connected") && line.ends_with("mm") {
            // Push the previous monitor to the list if there was one
            if let Some(monitor) = current_monitor.take() {
                monitors.push(Monitor {
                    name: monitor.name,
                    resolution: monitor.resolution,
                    position: monitor.position,
                    resolution_string: monitor.resolution_string,
                    position_string: monitor.position_string,
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
                resolution: (resolution[0], resolution[1]),
                position: (position[0], position2[0]),
                framerate: selected_framerate,
                resolution_string: resolution_part.to_string(),
                position_string: if !is_primary { parts[2].to_string() } else { parts[3].to_string() },
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
                    resolution: monitor.resolution,
                    position: monitor.position,
                    framerate: selected_framerate,
                    resolution_string: monitor.resolution_string,
                    position_string: monitor.position_string,
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
            resolution: monitor.resolution,
            position: monitor.position,
            framerate: selected_framerate,
            resolution_string: monitor.resolution_string,
            position_string: monitor.position_string,
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
    for i in 0..monitors.len() {
        for j in 0..monitors.len() {
            if i == j {
                continue;
            }

            if monitors[j].position.0 == (monitors[i].position.0 + monitors[j].resolution.0 ) && monitors[j].position.1 == monitors[i].position.1  {
                monitors[i].right = Some(j);
                monitors[j].left = Some(i);
            } else if monitors[j].position.1  == (monitors[i].position.1 + monitors[j].resolution.1 )&& monitors[j].position.0 == monitors[i].position.0  {
                monitors[i].down = Some(j);
                monitors[j].up = Some(i);
            } else if monitors[j].position.0  == (monitors[i].position.0 - monitors[j].resolution.0 ) && monitors[j].position.1 == monitors[i].position.1  {
                monitors[i].left = Some(j);
                monitors[j].right = Some(i);
            } else if monitors[j].position.1  == (monitors[i].position.1 - monitors[j].resolution.1 )  && monitors[j].position.0 == monitors[i].position.0  {
                monitors[i].up = Some(j);
                monitors[j].down = Some(i);
            }
        }
    }
    Ok(monitors)
}


fn draw_monitors<B: tui::backend::Backend>(f: &mut tui::Frame<B>, area: Rect, monitors: &[Monitor], selected_index: usize) {
    let total_width: f64 = monitors.iter().map(|m| m.position.0 + m.resolution.0 as i32).max().unwrap_or(0).into();
    let total_height: f64 = monitors.iter().map(|m| m.position.1 + m.resolution.1 as i32).max().unwrap_or(0).into();

    let scale_x = 0.9;
    let scale_y = 0.5;

    let monitor_data: Vec<_> = monitors.iter().enumerate().map(|(i, m)| {
        (i, m.position, m.resolution, m.is_selected, m.is_primary, m.name.clone())
    }).collect();

    let canvas = Canvas::default()
        .x_bounds([0.0, total_width])
        .y_bounds([0.0, total_height])
        .paint(move |ctx| {
            for (i, position, resolution, is_selected, is_primary, mut name) in monitor_data.iter().cloned() {
                let x = position.0 as f64 * scale_x + total_width * (1.0 - scale_x)/2.0;
                let y = total_height - (position.1 as f64 * scale_y + total_height * (1.0 - scale_y)/2.0);
                let width = resolution.0 as f64 * scale_x;
                let height = resolution.1 as f64 * scale_y * -1.0;

                let color = if is_selected {
                    Color::LightMagenta
                } else if i == selected_index {
                    Color::Yellow
                } else if is_primary {
                    Color::Green
                } else {
                    Color::White
                };

                ctx.draw(&Rectangle {
                    x,
                    y,
                    width,
                    height,
                    color,
                });

                // Draw monitor name
                if is_selected {
                    if is_primary {
                        name = format!("<{}*>", name);
                    } else {
                        name = format!("<{}>", name);
                    }
                } else if is_primary {
                    name = format!("{}*", name);
                }
                ctx.print(
                    x + width/2.0,
                    y + height/2.0,
                    Span::styled(
                        name,
                        Style::default().fg(Color::Black).bg(color)
                    ),
                );
            }
        });

    f.render_widget(canvas, area);
}

